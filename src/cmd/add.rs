//! `cart add` — add a bank to the Cart.toml dependencies.

use crate::config::GlobalConfig;
use crate::manifest::{CartManifest, Dependency, DetailedDependency};
use crate::registry::{self, GitSource};
use anyhow::Result;
use std::path::Path;

pub fn add(
    manifest_path: &Path,
    bank: &str,
    git: Option<String>,
    path: Option<String>,
    version: Option<String>,
) -> Result<()> {
    let mut manifest = CartManifest::load(manifest_path)?;

    let dep = if let Some(ref git_url) = git {
        Dependency::Detailed(DetailedDependency {
            version: version.clone(),
            git: Some(git_url.clone()),
            branch: None,
            tag: None,
            rev: None,
            path: None,
            features: Vec::new(),
            optional: false,
            default_features: true,
        })
    } else if let Some(p) = path {
        Dependency::Detailed(DetailedDependency {
            version: None,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: Some(p),
            features: Vec::new(),
            optional: false,
            default_features: true,
        })
    } else if let Some(v) = version {
        Dependency::Simple(v)
    } else {
        Dependency::Simple("*".to_string())
    };

    manifest.dependencies.insert(bank.to_string(), dep.clone());

    manifest.save(manifest_path)?;
    eprintln!("Added '{}' to Cart.toml dependencies.", bank);

    if let Some(git_url) = git {
        let carts_dir = GlobalConfig::carts_dir();
        let source = GitSource {
            url: git_url,
            branch: None,
            tag: None,
            rev: None,
        };
        eprintln!("Installing bank '{}' into {}...", bank, carts_dir.display());
        registry::install(bank, &source, &carts_dir)?;
        eprintln!(
            "Installed '{}' in {}.",
            bank,
            carts_dir.join(bank).display()
        );
    }

    Ok(())
}
