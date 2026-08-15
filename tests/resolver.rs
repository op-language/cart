use cart::manifest::{CartManifest, Dependency};
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

    make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = "0.1"
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

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
nonexistent = "1.0"
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

    let result = resolver::resolve(&manifest, &carts_dir, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("E505"));
}

#[test]
fn resolve_version_mismatch_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let carts_dir = tmp.path().join("carts");
    fs::create_dir_all(&carts_dir).expect("create carts dir");

    make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = "2.0"
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

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
b = "0.1"
"#
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
a = "0.1"
"#
        ),
    )
    .expect("write b");

    let manifest_text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
a = "0.1"
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

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

    make_lib_dir(&carts_dir, "std", "0.1.0");

    let manifest_text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
std = "*"
"#;
    let manifest = CartManifest::from_toml(manifest_text).expect("parse");

    let graph = resolver::resolve(&manifest, &carts_dir, None).expect("resolve");
    assert_eq!(graph.packages.len(), 1);
}
