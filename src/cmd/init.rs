//! `cart init` — create a new Op project.

use crate::manifest::{CartManifest, Features, Lib, Package, Rom, TargetSection};
use anyhow::Result;
use std::fs;
use std::path::Path;

const GITIGNORE: &str = "/target\n";

const ROM_ENTRY: &str =
    "//! {name}\n//!\n//! Project entry point.\n\nnoreturn fn main() {\n    loop {\n    }\n}\n";

const LIB_ENTRY: &str = "//! {name} lib\n//!\n//! Bank entry point.\n";

pub fn init(name: &str, lib: bool, target: Option<String>) -> Result<()> {
    validate_name(name)?;

    let project_dir = std::path::PathBuf::from(name);
    if project_dir.exists() {
        return Err(anyhow::anyhow!("E502: directory '{}' already exists", name));
    }

    let default_target = target.unwrap_or_else(|| "mos6502-nintendo-nes-ntsc".to_string());

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
            }],
            dependencies: Default::default(),
            dev_dependencies: Default::default(),
            target: Some(TargetSection {
                default: default_target,
            }),
            features: Some(Features::default()),
            run: None,
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
