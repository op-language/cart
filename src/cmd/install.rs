//! `cart install` — install a lib in ~/.carts/.

use crate::config::GlobalConfig;
use crate::registry::{self, GitSource};
use anyhow::Result;

pub fn install(name: &str, git: Option<String>) -> Result<()> {
    let carts_dir = GlobalConfig::carts_dir();
    let config = GlobalConfig::load();

    std::fs::create_dir_all(&carts_dir)?;

    let source = if let Some(url) = git {
        GitSource {
            url,
            branch: None,
            tag: None,
            rev: None,
        }
    } else {
        let base = config.default_git_base().ok_or_else(|| {
            anyhow::anyhow!("E510: no --git URL and no default-git-base in ~/.cart/config.toml")
        })?;
        GitSource {
            url: format!("{base}/{name}"),
            branch: None,
            tag: None,
            rev: None,
        }
    };

    eprintln!("Installing lib '{name}' into {}...", carts_dir.display());
    let result = registry::install(name, &source, &carts_dir)?;
    eprintln!(
        "Installed '{name}' at {} (sha: {})",
        result.path.display(),
        &result.sha[..8.min(result.sha.len())]
    );

    Ok(())
}
