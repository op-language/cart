//! `cart check` — run the lexer and parser without generating code.

use crate::config::GlobalConfig;
use crate::manifest::CartManifest;
use crate::opc::{self, OpcArgs, OpcStage};
use anyhow::Result;
use std::path::Path;

pub fn check(manifest_path: &Path, target: Option<String>) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;
    let config = GlobalConfig::load();
    let carts_dir = GlobalConfig::carts_dir();

    std::fs::create_dir_all(&carts_dir)?;

    let _graph = crate::resolver::resolve(&manifest, &carts_dir, config.default_git_base())?;

    let targets: Vec<String> = if manifest.rom.is_empty() {
        vec![manifest
            .default_target(target.as_deref())
            .unwrap_or_default()]
    } else {
        manifest.rom.iter().map(|r| r.target.clone()).collect()
    };

    for rom_target in &targets {
        let input = if manifest.rom.is_empty() {
            manifest_path.parent().unwrap_or(Path::new(".")).join(
                manifest
                    .lib
                    .as_ref()
                    .and_then(|b| b.path.as_deref())
                    .unwrap_or("src/lib.op"),
            )
        } else {
            manifest_path.parent().unwrap_or(Path::new(".")).join(
                manifest
                    .rom
                    .first()
                    .and_then(|r| r.path.as_deref())
                    .unwrap_or("src/cart.op"),
            )
        };

        eprintln!("Checking {} for {rom_target}...", input.display());

        let args = OpcArgs {
            input,
            target: rom_target.clone(),
            features: Vec::new(),
            opt_level: 0,
            format: None,
            output: None,
            stage: OpcStage::Parse,
            include: Vec::new(),
        };

        match opc::invoke(&args) {
            Ok(_) => eprintln!("  OK: no parse errors"),
            Err(e) => {
                eprintln!("  {e}");
                return Err(anyhow::anyhow!("E507: parse check failed"));
            }
        }
    }

    Ok(())
}
