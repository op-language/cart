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
        "#!/bin/sh\n# fake opc: write the -o output file and exit 0. With\n# --output-stages it also writes the .linked.opl marker next to it.\nout=\"\"\nprev=\"\"\nstages=0\nfor arg in \"$@\"; do\n  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; fi\n  if [ \"$arg\" = \"--output-stages\" ]; then stages=1; fi\n  prev=\"$arg\"\ndone\nif [ -n \"$out\" ]; then\n  mkdir -p \"$(dirname \"$out\")\"\n  echo \"stub\" > \"$out\"\n  if [ \"$stages\" = \"1\" ]; then\n    base=\"${out%.*}\"\n    echo '{\"version\": 1}' > \"$base.linked.opl\"\n  fi\nfi\nexit 0\n".to_string()
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

/// Write a fake `demu` binary that logs each launch argument on one
/// line of the `$FAKE_DEMU_LOG` file and exits with the
/// `$FAKE_DEMU_EXIT` status. The default exit status is 0.
fn make_fake_dem(dir: &std::path::Path) {
    let fake = dir.join("demu");
    let script = if cfg!(windows) {
        "@echo off\n:loop\nif \"%~1\"==\"\" goto done\necho %~1 >> \"%FAKE_DEMU_LOG%\"\nshift\ngoto loop\n:done\nexit /b %FAKE_DEMU_EXIT%\n".to_string()
    } else {
        "#!/bin/sh\n# fake demu: log the argv, capture the optional stdin log, exit with $FAKE_DEMU_EXIT.\nfor arg in \"$@\"; do\n  echo \"$arg\" >> \"$FAKE_DEMU_LOG\"\ndone\nif [ -n \"$FAKE_DEMU_STDIN\" ]; then\n  cat > \"$FAKE_DEMU_STDIN\"\nfi\nexit \"${FAKE_DEMU_EXIT:-0}\"\n".to_string()
    };
    fs::write(&fake, script).expect("write fake demu");
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

/// Create a minimal std lib in the temp HOME so the auto-clone from
/// GitHub does not trigger. The build command checks if
/// `~/.cart/std/` exists and adds `~/.cart/std/src` as an include
/// path. A bare directory with an empty `src/` is enough.
fn setup_fake_std(home: &std::path::Path) {
    let std_dir = home.join(".cart").join("std");
    let std_src = std_dir.join("src");
    fs::create_dir_all(&std_src).expect("mkdir fake std src");
    fs::write(
        std_dir.join("Cart.toml"),
        "[package]\nname = \"std\"\nversion = \"0.1.0\"\nedition = \"1\"\n\n[lib]\nname = \"std\"\npath = \"src/lib.op\"\n",
    )
    .expect("write fake std Cart.toml");
    fs::write(std_src.join("lib.op"), "//! std lib\n").expect("write fake std lib.op");
}

#[test]
fn init_then_build_rom() {
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "demogame";
    let triplet = "rp2A03-nintendo-nes-ntsc";

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();

    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, false, Some(triplet.to_string()), None).expect("init");
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
        false,
    );

    std::env::set_var("PATH", &old_path);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("build should succeed");
    // NES target uses format = "ines" which produces .nes extension.
    let output = project
        .join("target")
        .join(triplet)
        .join(format!("{project_name}.nes"));
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
    let triplet = "rp2A03-nintendo-nes-ntsc";

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();

    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, true, Some(triplet.to_string()), None).expect("init");
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
    let triplet = "rp2A03-nintendo-nes-ntsc";

    let bare = make_bare_git_lib(tmp.path(), "std", "0.1.0");
    let bare_url = format!("file://{}", bare.display());

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_home = std::env::var("HOME").unwrap_or_default();

    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, false, Some(triplet.to_string()), None).expect("init");
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
    setup_fake_std(tmp.path());
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
        false,
    );

    std::env::set_var("PATH", &old_path);
    std::env::set_var("HOME", &old_home);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("build should succeed");

    let carts_std = tmp.path().join(".cart").join("std");
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

    // NES target uses format = "ines" which produces .nes extension.
    let output = project
        .join("target")
        .join(triplet)
        .join(format!("{project_name}.nes"));
    assert!(
        output.exists(),
        "expected rom output at {}",
        output.display()
    );
}

/// A demu-profile project manifest with a NES sentinel. The optional
/// `[test.demu]` section is present with a checks path and a 30-frame
/// budget when `with_checks` is true.
fn demu_manifest(with_checks: bool) -> String {
    let checks = if with_checks {
        "\n[test.demu]\nchecks = \"tests/checks.toml\"\nframes = 30\n"
    } else {
        ""
    };
    format!(
        r#"[package]
name = "demutest"
version = "0.1.0"
edition = "1"

[[rom]]
name = "demutest"
path = "src/cart.op"
target = "rp2A03-nintendo-nes-ntsc"

[[run.profile]]
name = "test"
emulator = "demu"

[test]
profile = "test"

[test.sentinel.nes]
address = 0x6000
pass_value = 0xFF{checks}"#
    )
}

#[test]
fn test_demu_native_mode_passes_on_the_exit_code() {
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("demutest");
    fs::create_dir_all(project.join("src")).expect("mkdir src");
    fs::create_dir_all(project.join("tests")).expect("mkdir tests");
    fs::write(project.join("Cart.toml"), demu_manifest(false)).expect("write Cart.toml");
    fs::write(project.join("src").join("cart.op"), "//! cart\n").expect("write cart.op");
    fs::write(project.join("tests").join("pass.op"), "//! test\n").expect("write test op");

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_home = std::env::var("HOME").unwrap_or_default();
    make_fake_opc(tmp.path());
    make_fake_dem(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());
    std::env::set_current_dir(&project).expect("cd project");

    let log_path = tmp.path().join("demu-argv.log");
    fs::remove_file(&log_path).ok();
    let stdin_path = tmp.path().join("demu-stdin.log");
    std::env::set_var("FAKE_DEMU_LOG", &log_path);
    std::env::set_var("FAKE_DEMU_STDIN", &stdin_path);
    std::env::remove_var("FAKE_DEMU_EXIT");

    let result = cmd::test::test(&project.join("Cart.toml"), None);

    std::env::set_var("PATH", &old_path);
    std::env::set_var("HOME", &old_home);
    let _ = std::env::set_current_dir(&old_dir);
    std::env::remove_var("FAKE_DEMU_STDIN");

    let summary = result.expect("test run succeeds");
    assert_eq!(
        summary,
        cmd::test::TestSummary {
            passed: 1,
            failed: 0
        },
        "the zero exit status is a pass"
    );

    // The session carries the default run-frame budget: one
    // `run-frame <N>` command, then the session ends.
    let stdin_log = fs::read_to_string(&stdin_path).expect("read demu stdin log");
    assert_eq!(stdin_log, "run-frame 120\n", "the default frame budget");

    let triplet = "rp2A03-nintendo-nes-ntsc";
    let tests_dir = project.join("target").join(triplet).join("tests");
    let log = fs::read_to_string(&log_path).expect("read demu argv log");
    let lines: Vec<String> = log.lines().map(String::from).collect();
    match lines.as_slice() {
        [a1, a2, a3, a4, a5, a6, a7, a8, a9] => {
            assert_eq!(a1, "--config");
            assert_eq!(a2, triplet);
            assert_eq!(
                std::path::Path::new(a3),
                tests_dir.join("pass.bin"),
                "the ROM output is the positional"
            );
            assert_eq!(a4, "--dump");
            assert_eq!(std::path::Path::new(a5), tests_dir.join("pass.dump"));
            assert_eq!(a6, "--sentinel");
            assert_eq!(a7, "0x6000:0xff", "the sentinel pair uses the hex form");
            assert_eq!(a8, "--symbols");
            assert_eq!(
                std::path::Path::new(a9),
                tests_dir.join("pass.linked.opl"),
                "the linked symbol table is next to the ROM"
            );
        }
        other => panic!("unexpected demu argv: {other:?}"),
    }

    // The fake opc wrote the symbol-table marker for the stage output.
    assert!(
        tests_dir.join("pass.linked.opl").exists(),
        "opc --output-stages ran in the demu test mode"
    );
}

#[test]
fn test_demu_native_mode_forwards_checks_and_fails_on_the_sentinel_miss() {
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("demutest");
    fs::create_dir_all(project.join("src")).expect("mkdir src");
    fs::create_dir_all(project.join("tests")).expect("mkdir tests");
    fs::write(project.join("Cart.toml"), demu_manifest(true)).expect("write Cart.toml");
    fs::write(project.join("src").join("cart.op"), "//! cart\n").expect("write cart.op");
    fs::write(project.join("tests").join("pass.op"), "//! test\n").expect("write test op");
    fs::write(project.join("tests").join("checks.toml"), "[checks]\n").expect("write checks");
    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_home = std::env::var("HOME").unwrap_or_default();
    make_fake_opc(tmp.path());
    make_fake_dem(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());
    std::env::set_current_dir(&project).expect("cd project");

    let log_path = tmp.path().join("demu-argv.log");
    fs::remove_file(&log_path).ok();
    let stdin_path = tmp.path().join("demu-stdin.log");
    std::env::set_var("FAKE_DEMU_LOG", &log_path);
    std::env::set_var("FAKE_DEMU_STDIN", &stdin_path);
    std::env::set_var("FAKE_DEMU_EXIT", "1");

    let result = cmd::test::test(&project.join("Cart.toml"), None);

    std::env::set_var("PATH", &old_path);
    std::env::set_var("HOME", &old_home);
    let _ = std::env::set_current_dir(&old_dir);
    std::env::remove_var("FAKE_DEMU_EXIT");
    std::env::remove_var("FAKE_DEMU_STDIN");

    let summary = result.expect("the run collects failures");
    assert_eq!(
        summary,
        cmd::test::TestSummary {
            passed: 0,
            failed: 1
        },
        "the demu exit status 1 is the sentinel miss"
    );

    // The manifest's frames field sets the run-frame budget.
    let stdin_log = fs::read_to_string(&stdin_path).expect("read demu stdin log");
    assert_eq!(stdin_log, "run-frame 30\n", "the manifest frame budget");

    // The checks flag forwards the [test.demu] checks path after the
    // symbol flag.
    let log = fs::read_to_string(&log_path).expect("read demu argv log");
    let lines: Vec<String> = log.lines().map(String::from).collect();
    let checks_pos = lines
        .iter()
        .position(|a| a == "--checks")
        .expect("checks flag");
    assert_eq!(
        lines.get(checks_pos + 1).map(String::as_str),
        project.join("tests").join("checks.toml").to_str(),
        "the checks path resolves against the manifest"
    );
}

#[test]
fn test_debug_launches_demu_with_the_socket_server() {
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("demutest");
    fs::create_dir_all(project.join("src")).expect("mkdir src");
    fs::write(project.join("Cart.toml"), demu_manifest(false)).expect("write Cart.toml");
    fs::write(project.join("src").join("cart.op"), "//! cart\n").expect("write cart.op");

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let old_home = std::env::var("HOME").unwrap_or_default();
    make_fake_opc(tmp.path());
    make_fake_dem(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());
    std::env::set_current_dir(&project).expect("cd project");

    let log_path = tmp.path().join("demu-argv.log");
    fs::remove_file(&log_path).ok();
    std::env::set_var("FAKE_DEMU_LOG", &log_path);
    std::env::remove_var("FAKE_DEMU_EXIT");

    let result = cmd::debug::debug(&project.join("Cart.toml"), None);

    std::env::set_var("PATH", &old_path);
    std::env::set_var("HOME", &old_home);
    let _ = std::env::set_current_dir(&old_dir);

    result.expect("the debug launch succeeds");

    let triplet = "rp2A03-nintendo-nes-ntsc";
    let target_dir = project.join("target").join(triplet);
    let log = fs::read_to_string(&log_path).expect("read demu argv log");
    let lines: Vec<String> = log.lines().map(String::from).collect();
    match lines.as_slice() {
        [a1, a2, a3, a4, a5, a6, a7] => {
            assert_eq!(a1, "--config");
            assert_eq!(a2, triplet);
            assert_eq!(a3, "--symbols");
            assert_eq!(
                std::path::Path::new(a4),
                target_dir.join("demutest.linked.opl"),
                "the symbol table is next to the ROM"
            );
            assert_eq!(a5, "--serve");
            assert_eq!(
                std::path::Path::new(a6),
                target_dir.join("debug.demu-sock"),
                "the socket path sits under the target directory"
            );
            assert_eq!(std::path::Path::new(a7), target_dir.join("demutest.bin"));
        }
        other => panic!("unexpected demu argv: {other:?}"),
    }
}

/// A non-demu-profile project manifest with a NES sentinel.
fn testemu_manifest() -> String {
    r#"[package]
name = "emutest"
version = "0.1.0"
edition = "1"

[[rom]]
name = "emutest"
path = "src/cart.op"
target = "rp2A03-nintendo-nes-ntsc"

[[run.profile]]
name = "test"
emulator = "testemu"

[test]
profile = "test"

[test.sentinel.nes]
address = 0x6000
pass_value = 0xFF"#
        .to_string()
}

/// Write a fake generic emulator that logs each launch argument on one
/// line of `$FAKE_EMU_LOG` and writes a 64-KiB memory dump of the byte
/// value in `$FAKE_EMU_DUMP_VALUE` (0xFF by default) at the `--dump`
/// path. POSIX only.
fn make_fake_emu(dir: &std::path::Path) {
    let fake = dir.join("testemu");
    let script = "#!/bin/sh\n# fake emulator: log the argv and write a flat dump.\nprev=\"\"\nfor arg in \"$@\"; do\n  echo \"$arg\" >> \"$FAKE_EMU_LOG\"\n  if [ \"$prev\" = \"--dump\" ]; then dump=\"$arg\"; fi\n  prev=\"$arg\"\ndone\nmkdir -p \"$(dirname \"$dump\")\"\nif [ \"${FAKE_EMU_DUMP_VALUE:-255}\" = \"255\" ]; then\n  head -c 65536 /dev/zero | tr '\\0' '\\377' > \"$dump\"\nelse\n  head -c 65536 /dev/zero > \"$dump\"\nfi\nexit 0\n"
        .to_string();
    fs::write(&fake, script).expect("write fake emulator");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake).expect("stat").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake, perms).expect("chmod");
    }
}

#[test]
fn test_nondemu_mode_passes_on_the_dump_sentinel() {
    if cfg!(windows) {
        eprintln!("skipping: the fake emulator is POSIX-only");
        return;
    }
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("emutest");
    fs::create_dir_all(project.join("src")).expect("mkdir src");
    fs::create_dir_all(project.join("tests")).expect("mkdir tests");
    fs::write(project.join("Cart.toml"), testemu_manifest()).expect("write Cart.toml");
    fs::write(project.join("src").join("cart.op"), "//! cart\n").expect("write cart.op");
    fs::write(project.join("tests").join("pass.op"), "//! test\n").expect("write test op");

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    make_fake_opc(tmp.path());
    make_fake_emu(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_current_dir(&project).expect("cd project");

    let log_path = tmp.path().join("emu-argv.log");
    fs::remove_file(&log_path).ok();
    std::env::set_var("FAKE_EMU_LOG", &log_path);
    std::env::remove_var("FAKE_EMU_DUMP_VALUE");

    let result = cmd::test::test(&project.join("Cart.toml"), None);

    std::env::set_var("PATH", &old_path);
    let _ = std::env::set_current_dir(&old_dir);

    let summary = result.expect("test run succeeds");
    assert_eq!(
        summary,
        cmd::test::TestSummary {
            passed: 1,
            failed: 0
        },
        "the sentinel byte matches the dump"
    );

    // The generic launch keeps the spawn-and-read contract: the ROM
    // output, the dump flag, and the dump path. No demu flags and no
    // opc stage output.
    let triplet = "rp2A03-nintendo-nes-ntsc";
    let tests_dir = project.join("target").join(triplet).join("tests");
    let log = fs::read_to_string(&log_path).expect("read emulator argv log");
    let lines: Vec<String> = log.lines().map(String::from).collect();
    match lines.as_slice() {
        [a1, a2, a3] => {
            assert_eq!(std::path::Path::new(a1), tests_dir.join("pass.bin"));
            assert_eq!(a2, "--dump");
            assert_eq!(std::path::Path::new(a3), tests_dir.join("pass.dump"));
        }
        other => panic!("unexpected argv shape: {other:?}"),
    }
    assert!(
        !lines.iter().any(|a| a == "--sentinel" || a == "--symbols"),
        "the non-demu launch forwards no demu flag"
    );
}

#[test]
fn test_nondemu_mode_fails_on_the_sentinel_mismatch() {
    if cfg!(windows) {
        eprintln!("skipping: the fake emulator is POSIX-only");
        return;
    }
    let _lock = INT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project = tmp.path().join("emutest");
    fs::create_dir_all(project.join("src")).expect("mkdir src");
    fs::create_dir_all(project.join("tests")).expect("mkdir tests");
    fs::write(project.join("Cart.toml"), testemu_manifest()).expect("write Cart.toml");
    fs::write(project.join("src").join("cart.op"), "//! cart\n").expect("write cart.op");
    fs::write(project.join("tests").join("pass.op"), "//! test\n").expect("write test op");

    let old_dir = std::env::current_dir().expect("cwd");
    let old_path = std::env::var("PATH").unwrap_or_default();
    make_fake_opc(tmp.path());
    make_fake_emu(tmp.path());
    std::env::set_var("PATH", path_with_fake_opc(tmp.path()));
    std::env::set_current_dir(&project).expect("cd project");

    let log_path = tmp.path().join("emu-argv.log");
    fs::remove_file(&log_path).ok();
    std::env::set_var("FAKE_EMU_LOG", &log_path);
    std::env::set_var("FAKE_EMU_DUMP_VALUE", "0");

    let result = cmd::test::test(&project.join("Cart.toml"), None);

    std::env::set_var("PATH", &old_path);
    let _ = std::env::set_current_dir(&old_dir);
    std::env::remove_var("FAKE_EMU_DUMP_VALUE");

    let summary = result.expect("the run collects failures");
    assert_eq!(
        summary,
        cmd::test::TestSummary {
            passed: 0,
            failed: 1
        },
        "the sentinel byte mismatches the dump"
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
