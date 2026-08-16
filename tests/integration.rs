use cart::cmd;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static INT_LOCK: Mutex<()> = Mutex::new(());

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

#[test]
fn init_then_build_rom() {
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "demogame";
    let triplet = "mos6502-nintendo-nes-ntsc";

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();

    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, false, Some(triplet.to_string())).expect("init");
    let project = tmp.path().join(project_name);

    make_fake_opc(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_current_dir(&project).expect("cd project");

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
    let output = project
        .join("target")
        .join(triplet)
        .join(format!("{project_name}.bin"));
    assert!(
        output.exists(),
        "expected rom output at {}",
        output.display()
    );
    assert!(
        project.join("Cart.lock").exists(),
        "expected Cart.lock written"
    );
}

#[test]
fn init_then_build_lib() {
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "demolib";
    let triplet = "mos6502-nintendo-nes-ntsc";

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();

    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, true, Some(triplet.to_string())).expect("init");
    let project = tmp.path().join(project_name);

    make_fake_opc(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_current_dir(&project).expect("cd project");

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
    let output = project
        .join("target")
        .join(triplet)
        .join(format!("{project_name}.opb"));
    assert!(
        output.exists(),
        "expected lib output at {}",
        output.display()
    );
    assert!(
        project.join("Cart.lock").exists(),
        "expected Cart.lock written"
    );
}

#[test]
fn init_then_build_with_git_dep() {
    if !git_available() {
        eprintln!("skipping: git not on PATH");
        return;
    }
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "depdemo";
    let triplet = "mos6502-nintendo-nes-ntsc";

    let bare = make_bare_git_lib(tmp.path(), "std", "0.1.0");
    let bare_url = format!("file://{}", bare.display());

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_home = std::env::var("HOME").unwrap_or_default();

    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, false, Some(triplet.to_string())).expect("init");
    let project = tmp.path().join(project_name);

    let manifest_text = fs::read_to_string(project.join("Cart.toml")).expect("read Cart.toml");
    let dep_section =
        format!("\n[dependencies]\nstd = {{ version = \"0.1\", git = \"{bare_url}\" }}\n");
    let manifest_with_dep = if manifest_text.contains("[dependencies]") {
        manifest_text.replace("[dependencies]\n", &dep_section)
    } else {
        format!("{manifest_text}{dep_section}")
    };
    fs::write(project.join("Cart.toml"), &manifest_with_dep).expect("write Cart.toml");

    make_fake_opc(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());
    std::env::set_current_dir(&project).expect("cd project");

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
    std::env::set_var("HOME", &old_home);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("build should succeed");

    let carts_std = tmp.path().join(".carts").join("std");
    assert!(
        carts_std.join("Cart.toml").exists(),
        "expected std cloned into carts dir at {}",
        carts_std.display()
    );

    let lock_text = fs::read_to_string(project.join("Cart.lock")).expect("read Cart.lock");
    assert!(
        lock_text.contains("std"),
        "expected Cart.lock to record std package, got:\n{lock_text}"
    );

    let output = project
        .join("target")
        .join(triplet)
        .join(format!("{project_name}.bin"));
    assert!(
        output.exists(),
        "expected rom output at {}",
        output.display()
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
