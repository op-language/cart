//! `Cart.toml` manifest types and serialization.
//!
//! The manifest mirrors the `Cargo.toml` structure. The `cart` tool uses
//! these types to read, write, and modify manifests.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The root `Cart.toml` manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CartManifest {
    pub package: Package,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<Lib>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rom: Vec<Rom>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, Dependency>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<String, Dependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Features>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<RunProfileSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<TestConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<DocConfig>,
}

/// The `[package]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub edition: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// The `[lib]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lib {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A `[[rom]]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rom {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl Rom {
    /// Get the output format, if specified.
    pub fn format(&self) -> Option<&str> {
        self.format.as_deref()
    }
}

/// The `[target]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetSection {
    pub default: String,
}

/// The `[features]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Features {
    #[serde(flatten)]
    pub flags: BTreeMap<String, Vec<String>>,
}

/// Wrapper for the `[[run.profile]]` sections.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunProfileSection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profile: Vec<RunProfile>,
}

/// A `[[run.profile]]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunProfile {
    pub name: String,
    pub emulator: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// The `[test]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sentinel: BTreeMap<String, Sentinel>,
}

/// A sentinel definition for a machine in `[test.sentinel.<machine>]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sentinel {
    pub address: u64,
    pub pass_value: u64,
}

/// The `[doc]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// A dependency entry. Either a simple version requirement string or a
/// detailed table with git/path/features fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DetailedDependency),
}

/// The detailed form of a dependency.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetailedDependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_false")]
    pub default_features: bool,
}

fn default_true() -> bool {
    true
}

fn is_false(b: &bool) -> bool {
    !b
}

impl Dependency {
    /// Get the version requirement string, if any.
    pub fn version_req(&self) -> Option<&str> {
        match self {
            Dependency::Simple(v) => Some(v),
            Dependency::Detailed(d) => d.version.as_deref(),
        }
    }

    /// Get the git URL, if any.
    pub fn git(&self) -> Option<&str> {
        match self {
            Dependency::Simple(_) => None,
            Dependency::Detailed(d) => d.git.as_deref(),
        }
    }

    /// Get the local path, if any.
    pub fn path(&self) -> Option<&str> {
        match self {
            Dependency::Simple(_) => None,
            Dependency::Detailed(d) => d.path.as_deref(),
        }
    }

    /// Get the features list.
    pub fn features(&self) -> &[String] {
        match self {
            Dependency::Simple(_) => &[],
            Dependency::Detailed(d) => &d.features,
        }
    }
}

impl CartManifest {
    /// Parse a manifest from TOML text.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Serialize the manifest to TOML text.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Load a manifest from a file path.
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
        Self::from_toml(&text)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))
    }

    /// Save a manifest to a file path.
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let text = self.to_toml()?;
        std::fs::write(path, text)
            .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;
        Ok(())
    }

    /// Find a run profile by name.
    pub fn run_profile(&self, name: &str) -> Option<&RunProfile> {
        self.run.as_ref()?.profile.iter().find(|p| p.name == name)
    }

    /// Get the default target triplet. Checks `--target` override first,
    /// then `[target] default`, then the first `[[rom]]` target.
    pub fn default_target(&self, override_target: Option<&str>) -> Option<String> {
        if let Some(t) = override_target {
            return Some(t.to_string());
        }
        if let Some(t) = &self.target {
            if !t.default.is_empty() {
                return Some(t.default.clone());
            }
        }
        self.rom.first().map(|r| r.target.clone())
    }
}
