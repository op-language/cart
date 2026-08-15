//! Git-based registry for lib installation.
//!
//! The registry uses git only. The `cart install` command clones a lib
//! repository into `~/.carts/<name>/`. The `cart update` command pulls the
//! latest changes.

use std::path::{Path, PathBuf};

/// The source specification for a lib.
#[derive(Debug, Clone)]
pub struct GitSource {
    pub url: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
}

/// Result of an install or update operation.
#[derive(Debug)]
pub struct InstallResult {
    pub path: PathBuf,
    pub sha: String,
}

/// Install a lib from a git source into `~/.carts/<name>/`. If the
/// directory already exists, pull the latest changes instead.
pub fn install(name: &str, source: &GitSource, carts_dir: &Path) -> anyhow::Result<InstallResult> {
    let dest = carts_dir.join(name);
    if dest.exists() {
        update_repo(&dest, source)
    } else {
        clone_repo(&dest, source)
    }
}

/// Update a lib in `~/.carts/<name>/` by fetching and checking out the
/// resolved ref.
pub fn update(name: &str, source: &GitSource, carts_dir: &Path) -> anyhow::Result<InstallResult> {
    let dest = carts_dir.join(name);
    if !dest.exists() {
        return Err(anyhow::anyhow!(
            "E510: lib '{}' not found in {}",
            name,
            carts_dir.display()
        ));
    }
    update_repo(&dest, source)
}

/// Resolve the git source from a dependency and an optional default git
/// base URL.
pub fn resolve_source(
    name: &str,
    dep: &super::manifest::Dependency,
    default_git_base: Option<&str>,
) -> anyhow::Result<GitSource> {
    match dep {
        super::manifest::Dependency::Simple(_) => {
            let base = default_git_base.ok_or_else(|| {
                anyhow::anyhow!("E510: no git URL for '{}' and no default-git-base", name)
            })?;
            Ok(GitSource {
                url: format!("{base}/{name}"),
                branch: None,
                tag: None,
                rev: None,
            })
        }
        super::manifest::Dependency::Detailed(d) => {
            if let Some(url) = &d.git {
                Ok(GitSource {
                    url: url.clone(),
                    branch: d.branch.clone(),
                    tag: d.tag.clone(),
                    rev: d.rev.clone(),
                })
            } else if let Some(_path) = &d.path {
                Err(anyhow::anyhow!(
                    "E510: path dependencies do not use git clone"
                ))
            } else {
                let base = default_git_base.ok_or_else(|| {
                    anyhow::anyhow!("E510: no git URL for '{}' and no default-git-base", name)
                })?;
                Ok(GitSource {
                    url: format!("{base}/{name}"),
                    branch: None,
                    tag: None,
                    rev: None,
                })
            }
        }
    }
}

fn clone_repo(dest: &Path, source: &GitSource) -> anyhow::Result<InstallResult> {
    let mut builder = git2::build::RepoBuilder::new();
    if let Some(branch) = &source.branch {
        builder.branch(branch);
    }
    builder
        .clone(&source.url, dest)
        .map_err(|e| anyhow::anyhow!("E510: git clone '{}' failed: {e}", source.url))?;
    let repo = git2::Repository::open(dest)
        .map_err(|e| anyhow::anyhow!("E510: failed to open repo: {e}"))?;
    if let Some(tag) = &source.tag {
        checkout_tag(&repo, tag)?;
    }
    if let Some(rev) = &source.rev {
        checkout_rev(&repo, rev)?;
    }
    let sha = head_sha(&repo)?;
    Ok(InstallResult {
        path: dest.to_path_buf(),
        sha,
    })
}

fn update_repo(dest: &Path, source: &GitSource) -> anyhow::Result<InstallResult> {
    let repo = git2::Repository::open(dest)
        .map_err(|e| anyhow::anyhow!("E510: failed to open repo at {}: {e}", dest.display()))?;
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| anyhow::anyhow!("E510: failed to find origin remote: {e}"))?;
    remote
        .fetch(&["refs/heads/*:refs/heads/*"], None, None)
        .map_err(|e| anyhow::anyhow!("E510: git fetch failed: {e}"))?;
    if let Some(branch) = &source.branch {
        checkout_branch(&repo, branch)?;
    } else if let Some(tag) = &source.tag {
        checkout_tag(&repo, tag)?;
    } else if let Some(rev) = &source.rev {
        checkout_rev(&repo, rev)?;
    }
    let sha = head_sha(&repo)?;
    Ok(InstallResult {
        path: dest.to_path_buf(),
        sha,
    })
}

fn checkout_branch(repo: &git2::Repository, branch: &str) -> anyhow::Result<()> {
    let refname = format!("refs/heads/{branch}");
    let (object, reference) = repo
        .revparse_ext(&refname)
        .map_err(|e| anyhow::anyhow!("E510: failed to find branch '{branch}': {e}"))?;
    repo.checkout_tree(&object, None)
        .map_err(|e| anyhow::anyhow!("E510: checkout failed: {e}"))?;
    if let Some(r) = reference {
        repo.set_head(r.name().unwrap_or(&refname))
            .map_err(|e| anyhow::anyhow!("E510: set_head failed: {e}"))?;
    }
    Ok(())
}

fn checkout_tag(repo: &git2::Repository, tag: &str) -> anyhow::Result<()> {
    let (object, _) = repo
        .revparse_ext(tag)
        .map_err(|e| anyhow::anyhow!("E510: failed to find tag '{tag}': {e}"))?;
    repo.checkout_tree(&object, None)
        .map_err(|e| anyhow::anyhow!("E510: checkout failed: {e}"))?;
    Ok(())
}

fn checkout_rev(repo: &git2::Repository, rev: &str) -> anyhow::Result<()> {
    let (object, _) = repo
        .revparse_ext(rev)
        .map_err(|e| anyhow::anyhow!("E510: failed to find rev '{rev}': {e}"))?;
    repo.checkout_tree(&object, None)
        .map_err(|e| anyhow::anyhow!("E510: checkout failed: {e}"))?;
    Ok(())
}

fn head_sha(repo: &git2::Repository) -> anyhow::Result<String> {
    let head = repo
        .head()
        .map_err(|e| anyhow::anyhow!("E510: failed to read HEAD: {e}"))?;
    let target = head
        .target()
        .ok_or_else(|| anyhow::anyhow!("E510: HEAD has no target"))?;
    Ok(target.to_string())
}
