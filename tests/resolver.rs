use cart::manifest::CartManifest;
use cart::resolver;
use std::fs;
use std::path::PathBuf;

fn make_lib_manifest(name: &str, version: &str) -> String {
    format!(
        r#"
[package]
name = "{name}"
version = "{version}"
edition = "1"

[lib]
name = "{name}"
path = "src/lib.op"
"#
    )
}

fn make_lib_dir(carts_dir: &std::path::Path, name: &str, version: &str) -> PathBuf {
    let dir = carts_dir.join(name);
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("create lib dir");
    fs::write(dir.join("Cart.toml"), make_lib_manifest(name, version)).expect("write manifest");
    fs::write(src.join("lib.op"), "//! lib\n").expect("write source");
    dir
}

#[test]
fn resolve_single_path_dependency() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let lib_dir = make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = {{ version = "0.1", path = "{}" }}
"#,
        lib_dir.display()
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let graph = resolver::resolve(&manifest, &carts_dir, None).expect("resolve");
    assert_eq!(graph.packages.len(), 1);
    assert_eq!(graph.packages[0].name, "std");
    assert_eq!(graph.packages[0].version, "0.1.0");
}

#[test]
fn resolve_missing_lib_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let manifest_text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
nonexistent = { version = "1.0", git = "https://example.com/nonexistent" }
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

    let result = resolver::resolve(&manifest, &carts_dir, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("E510"));
}

#[test]
fn resolve_version_mismatch_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let lib_dir = make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = {{ version = "2.0", path = "{}" }}
"#,
        lib_dir.display()
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let result = resolver::resolve(&manifest, &carts_dir, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("E504"));
}

#[test]
fn resolve_path_dependency() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let lib_dir = tmp.path().join("local-lib");
    let src = lib_dir.join("src");
    fs::create_dir_all(&src).expect("create lib dir");
    fs::write(
        lib_dir.join("Cart.toml"),
        make_lib_manifest("local", "0.1.0"),
    )
    .expect("write");
    fs::write(src.join("lib.op"), "//! lib\n").expect("write");

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
local = {{ path = "{}" }}
"#,
        lib_dir.display()
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let graph = resolver::resolve(&manifest, &carts_dir, None).expect("resolve");
    assert_eq!(graph.packages.len(), 1);
    assert_eq!(graph.packages[0].name, "local");
}

#[test]
fn resolve_detects_cycle() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let a_dir = make_lib_dir(&carts_dir, "a", "0.1.0");
    let b_dir = make_lib_dir(&carts_dir, "b", "0.1.0");

    fs::write(
        a_dir.join("Cart.toml"),
        format!(
            r#"
[package]
name = "a"
version = "0.1.0"
edition = "1"

[lib]
name = "a"
path = "src/lib.op"

[dependencies]
b = {{ version = "0.1", path = "{}" }}
"#,
            b_dir.display()
        ),
    )
    .expect("write a");
    fs::write(
        b_dir.join("Cart.toml"),
        format!(
            r#"
[package]
name = "b"
version = "0.1.0"
edition = "1"

[lib]
name = "b"
path = "src/lib.op"

[dependencies]
a = {{ version = "0.1", path = "{}" }}
"#,
            a_dir.display()
        ),
    )
    .expect("write b");

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
a = {{ version = "0.1", path = "{}" }}
"#,
        a_dir.display()
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let result = resolver::resolve(&manifest, &carts_dir, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("E504") && err.contains("cycle"));
}

#[test]
fn resolve_any_version() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let lib_dir = make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = {{ version = "*", path = "{}" }}
"#,
        lib_dir.display()
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let graph = resolver::resolve(&manifest, &carts_dir, None).expect("resolve");
    assert_eq!(graph.packages.len(), 1);
}

#[test]
fn resolve_rejects_version_only_dep() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = "1.0"
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

    let result = resolver::resolve(&manifest, &carts_dir, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("E504"));
    assert!(err.contains("git") || err.contains("path"));
}

fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn make_bare_git_lib(dir: &std::path::Path, name: &str, version: &str) -> PathBuf {
    let work = dir.join(format!("{name}-work"));
    let src = work.join("src");
    fs::create_dir_all(&src).expect("create work dir");
    fs::write(work.join("Cart.toml"), make_lib_manifest(name, version)).expect("write manifest");
    fs::write(src.join("lib.op"), "//! lib\n").expect("write source");

    let bare = dir.join(format!("{name}.git"));
    std::process::Command::new("git")
        .args(["init", "--bare", "--quiet"])
        .arg(&bare)
        .status()
        .expect("git init bare");

    std::process::Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&work)
        .status()
        .expect("git init work");
    std::process::Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(&work)
        .status()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test"])
        .current_dir(&work)
        .status()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(&work)
        .status()
        .expect("git config gpgsign");
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&work)
        .status()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "init", "--quiet"])
        .current_dir(&work)
        .status()
        .expect("git commit");
    std::process::Command::new("git")
        .args(["remote", "add", "origin"])
        .arg(&bare)
        .current_dir(&work)
        .status()
        .expect("git remote");
    std::process::Command::new("git")
        .args(["push", "-q", "origin", "HEAD:master"])
        .current_dir(&work)
        .status()
        .expect("git push");

    bare
}

#[test]
fn resolve_clones_missing_git_dep() {
    if !git_available() {
        eprintln!("skipping: git not on PATH");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let bare = make_bare_git_lib(tmp.path(), "testlib", "0.1.0");
    let bare_url = format!("file://{}", bare.display());

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
testlib = {{ version = "0.1", git = "{}" }}
"#,
        bare_url
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let graph = resolver::resolve(&manifest, &carts_dir, None).expect("resolve");
    assert_eq!(graph.packages.len(), 1);
    assert_eq!(graph.packages[0].name, "testlib");
    assert_eq!(graph.packages[0].version, "0.1.0");
    assert!(carts_dir.join("testlib").join("Cart.toml").exists());
}

#[test]
fn resolve_uses_existing_clone() {
    if !git_available() {
        eprintln!("skipping: git not on PATH");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    let bare = make_bare_git_lib(tmp.path(), "existinglib", "0.1.0");
    let bare_url = format!("file://{}", bare.display());

    let manifest_text = format!(
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
existinglib = {{ version = "0.1", git = "{}" }}
"#,
        bare_url
    );
    let manifest = CartManifest::from_toml(&manifest_text).expect("parse");

    let graph1 = resolver::resolve(&manifest, &carts_dir, None).expect("first resolve");
    assert_eq!(graph1.packages.len(), 1);

    let checksum1 = graph1.packages[0].checksum.clone();

    let graph2 = resolver::resolve(&manifest, &carts_dir, None).expect("second resolve");
    assert_eq!(graph2.packages.len(), 1);
    assert_eq!(graph2.packages[0].checksum, checksum1);
}
