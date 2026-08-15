//! `Cart.lock` lockfile types and serialization.
//!
//! The lockfile records the exact version and source of each dependency in
//! the resolved graph. The `cart build` command writes it when it is absent
//! and reads it when it is present.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// The root `Cart.lock` lockfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartLock {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package: Vec<LockedPackage>,
}

/// A single locked package entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: LockedSource,
    pub checksum: String,
}

/// The source of a locked package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LockedSource {
    Git { url: String, sha: String },
    Path { dir: String },
}

impl CartLock {
    /// Create a new empty lockfile with the current format version.
    pub fn new() -> Self {
        Self {
            version: 1,
            package: Vec::new(),
        }
    }

    /// Parse a lockfile from TOML text.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Serialize the lockfile to TOML text.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Load a lockfile from a file path. Returns `None` if the file does
    /// not exist.
    pub fn load(path: &Path) -> anyhow::Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
        let lock = Self::from_toml(&text)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;
        Ok(Some(lock))
    }

    /// Save the lockfile to a file path.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let text = self.to_toml()?;
        std::fs::write(path, text)
            .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;
        Ok(())
    }

    /// Check if the lockfile is fresh relative to a resolved graph. The
    /// lockfile is fresh when every package in the graph has a matching
    /// entry in the lockfile with the same version, source, and checksum.
    pub fn is_fresh(&self, graph: &super::resolver::ResolvedGraph) -> bool {
        let lock_map: BTreeMap<&str, &LockedPackage> =
            self.package.iter().map(|p| (p.name.as_str(), p)).collect();

        for pkg in &graph.packages {
            match lock_map.get(pkg.name.as_str()) {
                Some(lp) => {
                    if lp.version != pkg.version || lp.checksum != pkg.checksum {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    /// Update the lockfile from a resolved graph.
    pub fn update_from_graph(&mut self, graph: &super::resolver::ResolvedGraph) {
        self.package = graph
            .packages
            .iter()
            .map(|pkg| LockedPackage {
                name: pkg.name.clone(),
                version: pkg.version.clone(),
                source: pkg.source.clone(),
                checksum: pkg.checksum.clone(),
            })
            .collect();
    }
}

impl Default for CartLock {
    fn default() -> Self {
        Self::new()
    }
}
