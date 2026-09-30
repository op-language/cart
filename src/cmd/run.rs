//! `cart run` — build the project and launch the ROM in an emulator.

use crate::config::GlobalConfig;
use crate::manifest::CartManifest;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(
    manifest_path: &Path,
    target: Option<String>,
    release: bool,
    profile: Option<String>,
) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;
    let config = GlobalConfig::load_or_create();

    let profile_name = profile.as_deref().unwrap_or("default");

    let run_profile = manifest
        .run_profile(profile_name)
        .or_else(|| config.run_profile(profile_name))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "E501: run profile '{}' not found in Cart.toml or config",
                profile_name
            )
        })?;

    let rom = manifest
        .rom
        .first()
        .ok_or_else(|| anyhow::anyhow!("E502: no ROM targets in Cart.toml"))?;

    let rom_target = target
        .clone()
        .or_else(|| run_profile.target.clone())
        .unwrap_or_else(|| rom.target.clone());

    let rom_path = super::build::rom_output_path(
        &manifest,
        manifest_path,
        &rom_target,
        &rom.name,
        rom.format.as_deref(),
    );

    // Build the ROM if it does not exist.
    if !rom_path.exists() {
        super::build::build(
            manifest_path,
            Some(rom_target.clone()),
            release,
            false,
            Vec::new(),
            None,
            false,
        )?;
    }

    if !rom_path.exists() {
        return Err(anyhow::anyhow!(
            "E507: ROM file not found at {}. Build failed?",
            rom_path.display()
        ));
    }

    let emulator = which(&run_profile.emulator).ok_or_else(|| {
        anyhow::anyhow!(
            "E501: emulator '{}' not found on PATH",
            run_profile.emulator
        )
    })?;

    eprintln!("Running {} in {}...", rom.name, run_profile.emulator);

    let mut cmd = Command::new(&emulator);
    // Auto-determine emulator-specific launch arguments (e.g. SameBoy's
    // `--model dmg` / `--model cgb`) from the target triplet and the
    // emulator name. x16emu takes the ROM path inside its `-prg`/`-run`
    // option pair, so the returned arguments may already embed it.
    let launch_args =
        crate::emulators::emulator_prepend_args(&rom_target, &run_profile.emulator, &rom_path);
    let rom_placed = launch_args
        .iter()
        .any(|arg| Path::new(arg) == rom_path.as_path());
    for arg in launch_args {
        cmd.arg(arg);
    }
    for arg in &run_profile.args {
        cmd.arg(arg);
    }
    if !rom_placed {
        cmd.arg(&rom_path);
    }

    cmd.spawn()
        .map_err(|e| anyhow::anyhow!("E501: failed to launch emulator: {e}"))?;

    Ok(())
}

fn which(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var("PATH").ok()?;
    for dir in path.split(':') {
        let candidate = std::path::PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
