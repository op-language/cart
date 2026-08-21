//! `cart build` — compile the project and write the ROM image.

use crate::config::GlobalConfig;
use crate::lockfile::{CartLock, LockedSource};
use crate::manifest::CartManifest;
use crate::opc::{self, OpcArgs, OpcStage};
use crate::resolver;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn build(
    manifest_path: &Path,
    target: Option<String>,
    release: bool,
    debug: bool,
    features: Vec<String>,
    format: Option<String>,
    frozen: bool,
) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;
    let config = GlobalConfig::load();
    let carts_dir = GlobalConfig::carts_dir();
    let std_dir = GlobalConfig::std_dir();

    std::fs::create_dir_all(&carts_dir)?;

    // Auto-checkout the std lib to ~/.cart/std/ if not present.
    if !std_dir.exists() {
        eprintln!("Checking out std lib to {}...", std_dir.display());
        if let Some(parent) = std_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        git2::build::RepoBuilder::new()
            .clone("https://github.com/op-language/std", &std_dir)
            .map_err(|e| anyhow::anyhow!("E510: failed to clone std lib: {e}"))?;
    }

    let graph = resolver::resolve(&manifest, &carts_dir, config.default_git_base())?;

    let lock_path = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("Cart.lock");
    let existing_lock = CartLock::load(&lock_path)?;

    if frozen {
        if let Some(lock) = &existing_lock {
            if !lock.is_fresh(&graph) {
                return Err(anyhow::anyhow!(
                    "E506: Cart.lock is out of date. Run `cart build` without --frozen to update it."
                ));
            }
        }
    }

    let mut lock = existing_lock.unwrap_or_else(CartLock::new);
    lock.update_from_graph(&graph);
    lock.save(&lock_path)?;

    // Collect include paths from resolved dependencies.
    let mut include_paths: Vec<String> = graph
        .packages
        .iter()
        .map(|pkg| match &pkg.source {
            LockedSource::Path { dir } => format!("{dir}/src"),
            LockedSource::Git { .. } => {
                // Git deps are installed in ~/.carts/<name>/
                format!("{}/{}/src", carts_dir.display(), pkg.name)
            }
        })
        .collect();

    // Always add ~/.cart/std/src as a default include path. The opc
    // compiler requires the std lib for all projects.
    include_paths.push(std_dir.join("src").to_string_lossy().to_string());

    let opt_level = if debug {
        0
    } else if release {
        1
    } else {
        config.default_opt_level()
    };

    if manifest.rom.is_empty() {
        if let Some(lib) = &manifest.lib {
            let lib_target = target
                .clone()
                .or_else(|| {
                    manifest
                        .target
                        .as_ref()
                        .map(|t| t.default.clone())
                        .filter(|d| !d.is_empty())
                })
                .or_else(|| config.default_target().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("E503: no target triplet for lib build"))?;

            let input = manifest_path
                .parent()
                .unwrap_or(Path::new("."))
                .join(lib.path.as_deref().unwrap_or("src/lib.op"));

            let output_dir = manifest_path
                .parent()
                .unwrap_or(Path::new("."))
                .join("target")
                .join(&lib_target);
            std::fs::create_dir_all(&output_dir)?;

            let output = output_dir.join(format!("{}.opb", lib.name));

            let all_features = [
                manifest
                    .features
                    .as_ref()
                    .map(|f| f.flags.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default(),
                features.clone(),
            ]
            .concat();

            eprintln!("Building lib {} for {}...", lib.name, lib_target);

            let args = OpcArgs {
                input,
                target: lib_target.clone(),
                features: all_features,
                opt_level,
                format: Some("raw".to_string()),
                output: Some(output.clone()),
                stage: OpcStage::Full,
                include: include_paths.clone(),
            };

            opc::invoke(&args)?;

            eprintln!("Lib written to {}", output.display());
            return Ok(());
        }

        return Err(anyhow::anyhow!("E502: no lib or rom target in Cart.toml"));
    }

    for rom in &manifest.rom {
        let rom_target = target.clone().unwrap_or_else(|| rom.target.clone());
        let input = manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(rom.path.as_deref().unwrap_or("src/cart.op"));

        let output_dir = manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("target")
            .join(&rom_target);
        std::fs::create_dir_all(&output_dir)?;

        let ext = format
            .as_deref()
            .map(opc::output_extension)
            .unwrap_or("bin");
        let output = output_dir.join(format!("{}.{}", rom.name, ext));

        let all_features = [
            manifest
                .features
                .as_ref()
                .map(|f| f.flags.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default(),
            features.clone(),
        ]
        .concat();

        eprintln!("Building {} for {}...", rom.name, rom_target);

        let args = OpcArgs {
            input,
            target: rom_target,
            features: all_features,
            opt_level,
            format: format.clone(),
            output: Some(output.clone()),
            stage: OpcStage::Full,
            include: include_paths.clone(),
        };

        opc::invoke(&args)?;

        eprintln!("ROM written to {}", output.display());
    }

    Ok(())
}

/// Get the output path for a ROM target. Used by `cart run`.
pub fn rom_output_path(
    _manifest: &CartManifest,
    manifest_path: &Path,
    target: &str,
    rom_name: &str,
    format: Option<&str>,
) -> PathBuf {
    let ext = format.map(opc::output_extension).unwrap_or("bin");
    manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("target")
        .join(target)
        .join(format!("{rom_name}.{ext}"))
}
