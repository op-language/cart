//! Global config in `~/.cart/config.toml`.
//!
//! The config file stores global settings. The `cart` tool applies settings
//! in this order: built-in defaults, then `~/.cart/config.toml`, then
//! `Cart.toml`, then CLI flags.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// The root global config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emulators: Option<super::emulators::EmulatorsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<super::manifest::RunProfileSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<super::manifest::TestConfig>,
}

/// The `[registry]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_git_base: Option<String>,
}

/// The `[build]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_level: Option<u32>,
}

impl GlobalConfig {
    /// Load the config from `~/.cart/config.toml`. Returns `Default` if the
    /// file does not exist.
    pub fn load() -> Self {
        let path = Self::config_path();
        match Self::load_from(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("warning: failed to load {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Load the config from `~/.cart/config.toml`. If the file does not
    /// exist, write a default config with the emulator matrix and return
    /// it. If the file exists but fails to parse, return `Default`.
    pub fn load_or_create() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default_with_emulators();
            if let Err(e) = config.save(&path) {
                eprintln!("warning: failed to write {}: {e}", path.display());
            }
            return config;
        }
        match Self::load_from(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("warning: failed to load {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Build a default config with the emulator matrix populated.
    pub fn default_with_emulators() -> Self {
        Self {
            registry: None,
            build: None,
            emulators: Some(super::emulators::EmulatorsConfig {
                emulator: super::emulators::default_emulator_matrix(),
            }),
            run: None,
            test: None,
        }
    }

    /// Save the config to a specific path.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("failed to create {}: {e}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize config: {e}"))?;
        std::fs::write(path, text)
            .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;
        Ok(())
    }

    /// Load the config from a specific path.
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
        toml::from_str(&text)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))
    }

    /// Get the path to `~/.cart/config.toml`.
    pub fn config_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home)
            .join(".cart")
            .join("config.toml")
    }

    /// Get the default dependencies directory `~/.cart/`.
    pub fn carts_dir() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".cart")
    }

    /// Get the std lib directory `~/.cart/std/`.
    pub fn std_dir() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".cart").join("std")
    }

    /// Get the default git base URL for the registry.
    pub fn default_git_base(&self) -> Option<&str> {
        self.registry.as_ref()?.default_git_base.as_deref()
    }

    /// Get the default target triplet from the global config.
    pub fn default_target(&self) -> Option<&str> {
        self.build.as_ref()?.target.as_deref()
    }

    /// Get the default optimization level.
    pub fn default_opt_level(&self) -> u32 {
        self.build.as_ref().and_then(|b| b.opt_level).unwrap_or(1)
    }

    /// Find a global run profile by name.
    pub fn run_profile(&self, name: &str) -> Option<&super::manifest::RunProfile> {
        self.run.as_ref()?.profile.iter().find(|p| p.name == name)
    }

    /// Get the list of known emulator binary names for a given operating
    /// system and target triplet.
    pub fn emulators_for(&self, os: &str, target: &str) -> Vec<String> {
        let empty = Vec::new();
        let entries = self
            .emulators
            .as_ref()
            .map(|e| &e.emulator)
            .unwrap_or(&empty);
        entries
            .iter()
            .filter(|e| e.os == os && e.target == target)
            .map(|e| e.emulator.clone())
            .collect()
    }
}
