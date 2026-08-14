//! `cart clean` — remove the build output directory.

use anyhow::Result;
use std::path::Path;

pub fn clean(manifest_path: &Path) -> Result<()> {
    let target_dir = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("target");

    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)
            .map_err(|e| anyhow::anyhow!("failed to remove {}: {e}", target_dir.display()))?;
        eprintln!("Removed {}", target_dir.display());
    } else {
        eprintln!("Nothing to clean. target/ does not exist.");
    }

    Ok(())
}
