//! `cart update` — update all dependencies to the latest version.

use crate::config::GlobalConfig;
use crate::lockfile::CartLock;
use crate::manifest::CartManifest;
use crate::registry::{self};
use crate::resolver;
use anyhow::Result;
use std::path::Path;

pub fn update(manifest_path: &Path) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;
    let config = GlobalConfig::load();
    let carts_dir = GlobalConfig::carts_dir();

    for (name, dep) in &manifest.dependencies {
        if dep.path().is_some() {
            eprintln!("Skipping path dependency: {name}");
            continue;
        }

        let source = registry::resolve_source(name, dep, config.default_git_base())?;
        eprintln!("Updating bank '{}'...", name);
        match registry::update(name, &source, &carts_dir) {
            Ok(result) => {
                eprintln!(
                    "  Updated to sha: {}",
                    &result.sha[..8.min(result.sha.len())]
                );
            }
            Err(e) => {
                eprintln!("  Failed to update {name}: {e}");
            }
        }
    }

    eprintln!("Re-resolving dependency graph...");
    let graph = resolver::resolve(&manifest, &carts_dir, config.default_git_base())?;

    let lock_path = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("Cart.lock");
    let mut lock = CartLock::load(&lock_path)?.unwrap_or_else(CartLock::new);
    lock.update_from_graph(&graph);
    lock.save(&lock_path)?;

    eprintln!("Updated Cart.lock.");

    Ok(())
}
