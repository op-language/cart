use cart::cmd;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static INIT_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn init_rom_project() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "mygame";
    let project_path = tmp.path().join(project_name);

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");
    let result = cmd::init::init(
        project_name,
        false,
        Some("mos6502-nintendo-nes-ntsc".to_string()),
    );
    let _ = std::env::set_current_dir(&old_dir);
    result.expect("init");

    assert!(project_path.join("Cart.toml").exists());
    assert!(project_path.join("src").join("cart.op").exists());
    assert!(project_path.join("tests").exists());
    assert!(project_path.join(".gitignore").exists());

    let manifest_text = fs::read_to_string(project_path.join("Cart.toml")).expect("read");
    assert!(manifest_text.contains("mygame"));
    assert!(manifest_text.contains("[[rom]]"));
    assert!(manifest_text.contains("mos6502-nintendo-nes-ntsc"));
}

#[test]
fn init_lib_project() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "mylib";
    let project_path = tmp.path().join(project_name);

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");
    let result = cmd::init::init(
        project_name,
        true,
        Some("mos6502-nintendo-nes-ntsc".to_string()),
    );
    let _ = std::env::set_current_dir(&old_dir);
    result.expect("init");

    assert!(project_path.join("Cart.toml").exists());
    assert!(project_path.join("src").join("lib.op").exists());

    let manifest_text = fs::read_to_string(project_path.join("Cart.toml")).expect("read");
    assert!(manifest_text.contains("mylib"));
    assert!(manifest_text.contains("[lib]"));
    assert!(!manifest_text.contains("[[rom]]"));
}

#[test]
fn init_fails_on_existing_dir() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "exists";
    let project_path = tmp.path().join(project_name);
    fs::create_dir_all(&project_path).expect("mkdir");

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");
    let result = cmd::init::init(project_name, false, None);
    let _ = std::env::set_current_dir(&old_dir);

    assert!(result.is_err());
}
