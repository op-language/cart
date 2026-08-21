use cart::cmd;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static BUILD_LOCK: Mutex<()> = Mutex::new(());

fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn make_fake_opc(dir: &std::path::Path) {
    let fake = dir.join("opc");
    let script = if cfg!(windows) {
        format!(
            "@echo off\nif not exist \"%~dp0{}\" mkdir \"%~dp0{}\"\necho stub > \"%~dp0{}\"\nexit /b 0\n",
            "%OUT%", "%OUT%", "%OUT%"
        )
    } else {
        "#!/bin/sh\n# fake opc: write the -o output file and exit 0\nout=\"\"\nprev=\"\"\nfor arg in \"$@\"; do\n  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; fi\n  prev=\"$arg\"\ndone\nif [ -n \"$out\" ]; then\n  mkdir -p \"$(dirname \"$out\")\"\n  echo \"stub\" > \"$out\"\nfi\nexit 0\n".to_string()
    };
    fs::write(&fake, script).expect("write fake opc");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake).expect("stat").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake, perms).expect("chmod");
    }
}

fn path_with_fake_opc(extra: &std::path::Path) -> String {
    let original = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", extra.display(), original)
}

fn write_manifest(dir: &std::path::Path, text: &str) {
    fs::write(dir.join("Cart.toml"), text).expect("write Cart.toml");
}

fn lib_manifest(name: &str, triplet: &str) -> String {
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "1"

[lib]
name = "{name}"
path = "src/lib.op"

[target]
default = "{triplet}"

[features]
"#
    )
}

fn rom_manifest(name: &str, triplet: &str) -> String {
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "1"

[[rom]]
name = "{name}"
path = "src/cart.op"
target = "{triplet}"

[target]
default = "{triplet}"

[features]
"#
    )
}

#[test]
fn build_lib_invokes_opc() {
    let _lock = BUILD_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("mylib");
    let src = project.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.op"), "//! mylib lib\n").expect("write lib.op");

    let triplet = "rp2A03-nintendo-nes-ntsc";
    write_manifest(&project, &lib_manifest("mylib", triplet));

    make_fake_opc(tmp.path());
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_current_dir(&project).expect("cd");

    let result = cmd::build::build(
        &std::path::PathBuf::from("Cart.toml"),
        None,
        false,
        false,
        Vec::new(),
        None,
        false,
    );

    std::env::set_var("PATH", &old_path);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("build should succeed");
    let output = project.join("target").join(triplet).join("mylib.opb");
    assert!(
        output.exists(),
        "expected lib output at {}",
        output.display()
    );
}

#[test]
fn build_rom_invokes_opc() {
    let _lock = BUILD_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("mygame");
    let src = project.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("cart.op"), "//! mygame\n").expect("write cart.op");

    let triplet = "rp2A03-nintendo-nes-ntsc";
    write_manifest(&project, &rom_manifest("mygame", triplet));

    make_fake_opc(tmp.path());
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_current_dir(&project).expect("cd");

    let result = cmd::build::build(
        &std::path::PathBuf::from("Cart.toml"),
        None,
        false,
        false,
        Vec::new(),
        None,
        false,
    );

    std::env::set_var("PATH", &old_path);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("build should succeed");
    let output = project.join("target").join(triplet).join("mygame.bin");
    assert!(
        output.exists(),
        "expected rom output at {}",
        output.display()
    );
}

#[test]
fn build_errors_when_no_target() {
    let _lock = BUILD_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("empty");
    fs::create_dir_all(&project).expect("mkdir");

    let manifest_text = r#"
[package]
name = "empty"
version = "0.1.0"
edition = "1"

[features]
"#;
    write_manifest(&project, manifest_text);

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&project).expect("cd");

    let result = cmd::build::build(
        &std::path::PathBuf::from("Cart.toml"),
        None,
        false,
        false,
        Vec::new(),
        None,
        false,
    );

    let _ = std::env::set_current_dir(&old_dir);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("E502"), "expected E502, got: {err}");
}

#[test]
fn build_lib_with_git_dep() {
    if !git_available() {
        eprintln!("skipping: git not on PATH");
        return;
    }
    let _lock = BUILD_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");

    let bare = make_bare_git_lib(tmp.path(), "dep-lib", "0.1.0");
    let bare_url = format!("file://{}", bare.display());

    let project = tmp.path().join("mylib");
    let src = project.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.op"), "//! mylib lib\n").expect("write lib.op");

    let triplet = "rp2A03-nintendo-nes-ntsc";
    let manifest_text = format!(
        r#"
[package]
name = "mylib"
version = "0.1.0"
edition = "1"

[lib]
name = "mylib"
path = "src/lib.op"

[dependencies]
dep-lib = {{ version = "0.1", git = "{bare_url}" }}

[target]
default = "{triplet}"

[features]
"#
    );
    write_manifest(&project, &manifest_text);

    make_fake_opc(tmp.path());
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_dir = std::env::current_dir().expect("cwd");
    let old_carts = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());
    std::env::set_current_dir(&project).expect("cd");

    let result = cmd::build::build(
        &std::path::PathBuf::from("Cart.toml"),
        None,
        false,
        false,
        Vec::new(),
        None,
        false,
    );

    std::env::set_var("PATH", &old_path);
    std::env::set_var("HOME", &old_carts);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("build should succeed");
    let output = project.join("target").join(triplet).join("mylib.opb");
    assert!(
        output.exists(),
        "expected lib output at {}",
        output.display()
    );
    let carts = tmp.path().join(".carts").join("dep-lib");
    assert!(
        carts.join("Cart.toml").exists(),
        "expected dep-lib cloned into carts dir at {}",
        carts.display()
    );
    assert!(
        project.join("Cart.lock").exists(),
        "expected Cart.lock written"
    );
}

fn make_bare_git_lib(dir: &std::path::Path, name: &str, version: &str) -> std::path::PathBuf {
    let work = dir.join(format!("{name}-work"));
    let wsrc = work.join("src");
    fs::create_dir_all(&wsrc).expect("create work dir");
    fs::write(
        work.join("Cart.toml"),
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
        ),
    )
    .expect("write manifest");
    fs::write(wsrc.join("lib.op"), "//! lib\n").expect("write source");

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
