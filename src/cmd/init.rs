//! `cart init` — create a new Op project.

use crate::config::GlobalConfig;
use crate::emulators::{current_os, is_in_path};
use crate::manifest::{
    CartManifest, Features, Lib, Package, Rom, RunProfile, RunProfileSection, TargetSection,
};
use crate::targets::SUPPORTED_TARGETS;
use anyhow::Result;
use std::fs;
use std::path::Path;

const GITIGNORE: &str = "/target\n";

const ROM_ENTRY: &str =
    "//! {name}\n//!\n//! Project entry point.\n\nnoreturn fn main() {\n    loop {\n    }\n}\n";

const LIB_ENTRY: &str = "//! {name} lib\n//!\n//! Bank entry point.\n";

pub fn init(
    name: &str,
    lib: bool,
    target: Option<String>,
    add_run_profile: Option<String>,
) -> Result<()> {
    validate_name(name)?;

    let project_dir = std::path::PathBuf::from(name);
    if project_dir.exists() {
        return Err(anyhow::anyhow!("E502: directory '{}' already exists", name));
    }

    let default_target = match target {
        Some(t) => t,
        None => {
            use dialoguer::Select;
            if !can_prompt() {
                return Err(anyhow::anyhow!(
                    "E502: no target specified and interactive input is not available. \
                     Use --target <triplet>."
                ));
            }
            let items: Vec<String> = SUPPORTED_TARGETS
                .iter()
                .map(|(trip, cpu, plat)| format!("{trip}  ({cpu}, {plat})"))
                .collect();
            let selection = Select::new()
                .with_prompt("Select the default target triplet")
                .items(&items)
                .default(0)
                .interact()?;
            SUPPORTED_TARGETS[selection].0.to_string()
        }
    };

    // Determine the default run profile.
    let run_profile = determine_run_profile(&default_target, add_run_profile)?;

    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("tests"))?;

    let manifest = if lib {
        let entry = LIB_ENTRY.replace("{name}", name);
        fs::write(project_dir.join("src").join("lib.op"), entry)?;

        CartManifest {
            package: Package {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                edition: "1".to_string(),
                authors: Vec::new(),
                license: None,
            },
            lib: Some(Lib {
                name: name.to_string(),
                path: Some("src/lib.op".to_string()),
            }),
            rom: Vec::new(),
            dependencies: Default::default(),
            dev_dependencies: Default::default(),
            target: Some(TargetSection {
                default: default_target.clone(),
            }),
            features: Some(Features::default()),
            run: None,
            test: None,
            doc: None,
        }
    } else {
        let entry = ROM_ENTRY.replace("{name}", name);
        fs::write(project_dir.join("src").join("cart.op"), entry)?;

        CartManifest {
            package: Package {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                edition: "1".to_string(),
                authors: Vec::new(),
                license: None,
            },
            lib: None,
            rom: vec![Rom {
                name: name.to_string(),
                path: Some("src/cart.op".to_string()),
                target: default_target.clone(),
                format: default_format_for(&default_target),
            }],
            dependencies: Default::default(),
            dev_dependencies: Default::default(),
            target: Some(TargetSection {
                default: default_target,
            }),
            features: Some(Features::default()),
            run: run_profile.map(|rp| RunProfileSection { profile: vec![rp] }),
            test: None,
            doc: None,
        }
    };

    manifest.save(&project_dir.join("Cart.toml"))?;
    fs::write(project_dir.join(".gitignore"), GITIGNORE)?;

    init_git(&project_dir)?;

    eprintln!("Created {name} project in {}", project_dir.display());
    Ok(())
}

/// Check if interactive prompts are available.
///
/// Returns false if the `CART_NON_INTERACTIVE` environment variable is set,
/// or if stdin or stdout is not a terminal.
fn can_prompt() -> bool {
    use std::io::IsTerminal;
    if std::env::var("CART_NON_INTERACTIVE").is_ok() {
        return false;
    }
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Determine the default run profile for the given target.
///
/// If `add_run_profile` is set, use the given emulator directly. Otherwise
/// prompt the user. If the user selects "Yes", detect the operating system,
/// find installed emulators for the target, and let the user select one.
/// If stdin is not a terminal, skip the prompt and return None.
fn determine_run_profile(
    target: &str,
    add_run_profile: Option<String>,
) -> Result<Option<RunProfile>> {
    use dialoguer::Select;

    // If --add-run-profile was given, use it directly.
    if let Some(emu) = add_run_profile {
        if emu.is_empty() {
            return Err(anyhow::anyhow!(
                "E502: --add-run-profile value must not be empty"
            ));
        }
        return Ok(Some(RunProfile {
            name: "default".to_string(),
            emulator: emu,
            args: Vec::new(),
            target: None,
        }));
    }

    // If stdin is not a terminal, skip the interactive prompt.
    if !can_prompt() {
        return Ok(None);
    }

    // Ask the user if they want a default run profile.
    let selection = Select::new()
        .with_prompt("Add a default run profile?")
        .items(["Yes", "No"])
        .default(0)
        .interact()?;

    if selection == 1 {
        return Ok(None);
    }

    // Detect the OS and find installed emulators for the target.
    let config = GlobalConfig::load_or_create();
    let os = current_os();
    let candidates = config.emulators_for(os, target);

    // Filter to only installed emulators.
    let installed: Vec<String> = candidates
        .into_iter()
        .filter(|name| is_in_path(name))
        .collect();

    if installed.is_empty() {
        eprintln!(
            "warning: no known emulators for target '{}' found in PATH. \
             Skipping default run profile. You can add one manually in Cart.toml.",
            target
        );
        return Ok(None);
    }

    // Let the user select from the installed emulators.
    let selection = Select::new()
        .with_prompt("Select an emulator for the default run profile")
        .items(&installed)
        .default(0)
        .interact()?;

    Ok(Some(RunProfile {
        name: "default".to_string(),
        emulator: installed[selection].clone(),
        args: Vec::new(),
        target: None,
    }))
}

fn init_git(dir: &Path) -> Result<()> {
    let result = std::process::Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(dir)
        .status();
    if let Err(e) = result {
        eprintln!("warning: git init failed: {e}");
    }
    Ok(())
}

fn default_format_for(target: &str) -> Option<String> {
    if target.contains("nes") {
        Some("ines".to_string())
    } else if target.contains("lynx") {
        Some("lnx".to_string())
    } else if target.contains("gameboy") {
        // The gameboy-color triplet also contains "gameboy"; both use the
        // gb format.
        Some("gb".to_string())
    } else if target.contains("snes") {
        Some("snes".to_string())
    } else if target.contains("genesis") {
        Some("sega".to_string())
    } else if target.contains("mastersystem")
        || target.contains("gamegear")
        || target.contains("sg1000")
    {
        Some("sms".to_string())
    } else if target.contains("atari-7800") {
        Some("a78".to_string())
    } else if target.contains("commander-x16") {
        Some("prg".to_string())
    } else {
        None
    }
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("E502: project name must not be empty"));
    }
    if name == "." || name == ".." {
        return Err(anyhow::anyhow!(
            "E502: project name must not be '.' or '..'"
        ));
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !valid {
        return Err(anyhow::anyhow!(
            "E502: project name '{}' must contain only lowercase letters, digits, hyphens, and underscores",
            name
        ));
    }
    Ok(())
}
