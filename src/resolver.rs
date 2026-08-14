//! Dependency resolver.
//!
//! The resolver walks the dependency graph. It reads each bank `Cart.toml`,
//! matches version requirements, detects cycles, and builds a resolved
//! graph with checksums.

use crate::lockfile::LockedSource;
use crate::manifest::{CartManifest, Dependency};
use semver::{Version, VersionReq};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// A resolved package in the dependency graph.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub source: LockedSource,
    pub checksum: String,
}

/// The resolved dependency graph.
#[derive(Debug, Clone, Default)]
pub struct ResolvedGraph {
    pub packages: Vec<ResolvedPackage>,
}

/// Resolve all dependencies from a manifest. Walks the dependency graph,
/// reads each bank `Cart.toml`, matches semver requirements, detects
/// cycles, and computes checksums.
pub fn resolve(
    manifest: &CartManifest,
    carts_dir: &Path,
    default_git_base: Option<&str>,
) -> anyhow::Result<ResolvedGraph> {
    let mut graph = ResolvedGraph::default();
    let mut visited: HashSet<String> = HashSet::new();
    let mut seen_names: BTreeSet<String> = BTreeSet::new();

    let deps: BTreeMap<&String, &Dependency> = manifest
        .dependencies
        .iter()
        .chain(manifest.dev_dependencies.iter())
        .collect();

    for (name, dep) in &deps {
        resolve_one(
            name,
            dep,
            carts_dir,
            default_git_base,
            &mut visited,
            &mut seen_names,
            &mut graph,
        )?;
    }

    Ok(graph)
}

fn resolve_one(
    name: &str,
    dep: &Dependency,
    carts_dir: &Path,
    default_git_base: Option<&str>,
    visited: &mut HashSet<String>,
    seen_names: &mut BTreeSet<String>,
    graph: &mut ResolvedGraph,
) -> anyhow::Result<()> {
    if visited.contains(name) {
        return Err(anyhow::anyhow!(
            "E504: dependency cycle detected at bank '{}'",
            name
        ));
    }

    if seen_names.contains(name) {
        return Ok(());
    }

    visited.insert(name.to_string());

    let source = resolve_source_path(name, dep, carts_dir, default_git_base)?;
    let bank_manifest_path = source.join("Cart.toml");

    if !bank_manifest_path.exists() {
        return Err(anyhow::anyhow!(
            "E505: bank '{}' not installed in {}",
            name,
            carts_dir.display()
        ));
    }

    let bank_manifest = CartManifest::load(&bank_manifest_path)?;
    let bank_version = &bank_manifest.package.version;

    if let Some(req_str) = dep.version_req() {
        let req = VersionReq::parse(req_str)
            .map_err(|e| anyhow::anyhow!("E504: invalid version requirement '{}': {e}", req_str))?;
        let version = Version::parse(bank_version).map_err(|e| {
            anyhow::anyhow!(
                "E504: bank '{}' has invalid version '{}': {e}",
                name,
                bank_version
            )
        })?;
        if !req.matches(&version) {
            return Err(anyhow::anyhow!(
                "E504: bank '{}' version '{}' does not satisfy requirement '{}'",
                name,
                bank_version,
                req_str
            ));
        }
    }

    let checksum = compute_checksum(&source)?;
    let locked_source = if dep.path().is_some() {
        LockedSource::Path {
            dir: source.to_string_lossy().to_string(),
        }
    } else {
        LockedSource::Git {
            url: dep.git().unwrap_or("").to_string(),
            sha: String::new(),
        }
    };

    graph.packages.push(ResolvedPackage {
        name: name.to_string(),
        version: bank_version.clone(),
        source: locked_source,
        checksum,
    });

    seen_names.insert(name.to_string());

    for (sub_name, sub_dep) in &bank_manifest.dependencies {
        resolve_one(
            sub_name,
            sub_dep,
            carts_dir,
            default_git_base,
            visited,
            seen_names,
            graph,
        )?;
    }

    visited.remove(name);
    Ok(())
}

fn resolve_source_path(
    name: &str,
    dep: &Dependency,
    carts_dir: &Path,
    default_git_base: Option<&str>,
) -> anyhow::Result<PathBuf> {
    if let Some(path) = dep.path() {
        return Ok(PathBuf::from(path));
    }
    let dest = carts_dir.join(name);
    if dest.exists() {
        return Ok(dest);
    }
    if let Some(base) = default_git_base {
        return Err(anyhow::anyhow!(
            "E505: bank '{}' not found in {}. Run: cart install {} (from {base}/{name})",
            name,
            carts_dir.display(),
            name
        ));
    }
    Err(anyhow::anyhow!(
        "E505: bank '{}' not found in {}. Run: cart install {}",
        name,
        carts_dir.display(),
        name
    ))
}

/// Compute the SHA-256 checksum of a bank source tree. Hashes the file
/// contents of all files in the directory recursively.
fn compute_checksum(dir: &Path) -> anyhow::Result<String> {
    let mut hasher = Sha256::new();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(dir, &mut files)?;
    files.sort();
    for file in &files {
        let rel = file.strip_prefix(dir).unwrap_or(file);
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update(b"\0");
        let data = std::fs::read(file)?;
        hasher.update(&data);
        hasher.update(b"\0");
    }
    let result = hasher.finalize();
    Ok(hex_encode(&result))
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("failed to read dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().map(|n| n == ".git").unwrap_or(false) {
                continue;
            }
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
