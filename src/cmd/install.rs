//! `cart install` — install a bank in ~/.carts/.

use crate::config::GlobalConfig;
use crate::registry::{self, GitSource};
use anyhow::Result;

pub fn install(bank: &str, git: Option<String>) -> Result<()> {
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
            url: format!("{base}/{bank}"),
            branch: None,
            tag: None,
            rev: None,
        }
    };

    eprintln!("Installing bank '{}' into {}...", bank, carts_dir.display());
    let result = registry::install(bank, &source, &carts_dir)?;
    eprintln!(
        "Installed '{}' at {} (sha: {})",
        bank,
        result.path.display(),
        &result.sha[..8.min(result.sha.len())]
    );

    Ok(())
}
