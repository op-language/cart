//! `cart add` — add a lib to the Cart.toml dependencies.

use crate::config::GlobalConfig;
use crate::manifest::{CartManifest, Dependency, DetailedDependency};
use crate::registry::{self, GitSource};
use anyhow::Result;
use std::path::Path;

pub fn add(
    manifest_path: &Path,
    name: &str,
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

    manifest.dependencies.insert(name.to_string(), dep.clone());

    manifest.save(manifest_path)?;
    eprintln!("Added '{name}' to Cart.toml dependencies.");

    if let Some(git_url) = git {
        let carts_dir = GlobalConfig::carts_dir();
        let source = GitSource {
            url: git_url,
            branch: None,
            tag: None,
            rev: None,
        };
        eprintln!("Installing lib '{name}' into {}...", carts_dir.display());
        registry::install(name, &source, &carts_dir)?;
        eprintln!("Installed '{name}' in {}.", carts_dir.join(name).display());
    }

    Ok(())
}
