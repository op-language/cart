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
        Some("rp2A03-nintendo-nes-ntsc".to_string()),
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
    assert!(manifest_text.contains("rp2A03-nintendo-nes-ntsc"));
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
        Some("rp2A03-nintendo-nes-ntsc".to_string()),
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

#[test]
fn init_rom_manifest_matches_template() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "demo";
    let project_path = tmp.path().join(project_name);
    let triplet = "rp2A03-nintendo-nes-ntsc";

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, false, Some(triplet.to_string())).expect("init");
    let _ = std::env::set_current_dir(&old_dir);

    let manifest_text = fs::read_to_string(project_path.join("Cart.toml")).expect("read Cart.toml");

    assert!(manifest_text.contains("[features]"));
    assert!(manifest_text.contains("[[rom]]"));
    assert!(manifest_text.contains(&format!("name = \"{project_name}\"")));
    assert!(manifest_text.contains("version = \"0.1.0\""));
    assert!(manifest_text.contains("edition = \"1\""));
    assert!(manifest_text.contains("path = \"src/cart.op\""));
    assert!(manifest_text.contains(&format!("target = \"{triplet}\"")));
    assert!(manifest_text.contains("[target]"));
    assert!(manifest_text.contains(&format!("default = \"{triplet}\"")));

    let entry = fs::read_to_string(project_path.join("src").join("cart.op")).expect("read cart.op");
    assert!(entry.contains("//! demo"));
    assert!(entry.contains("//! Project entry point."));
    assert!(entry.contains("noreturn fn main()"));

    let gitignore = fs::read_to_string(project_path.join(".gitignore")).expect("read .gitignore");
    assert!(gitignore.contains("/target"));
}

#[test]
fn init_lib_manifest_matches_template() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "demolib";
    let project_path = tmp.path().join(project_name);
    let triplet = "rp2A03-nintendo-nes-nes";

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(project_name, true, Some(triplet.to_string())).expect("init");
    let _ = std::env::set_current_dir(&old_dir);

    let manifest_text = fs::read_to_string(project_path.join("Cart.toml")).expect("read Cart.toml");

    assert!(manifest_text.contains("[features]"));
    assert!(manifest_text.contains("[lib]"));
    assert!(!manifest_text.contains("[[rom]]"));
    assert!(manifest_text.contains(&format!("name = \"{project_name}\"")));
    assert!(manifest_text.contains("path = \"src/lib.op\""));

    let entry = fs::read_to_string(project_path.join("src").join("lib.op")).expect("read lib.op");
    assert!(entry.contains("//! demolib lib"));
    assert!(entry.contains("//! Bank entry point."));
}

#[test]
fn init_rejects_invalid_name() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");

    for bad_name in &["", "UPPER", "has space", ".", "..", "Caps"] {
        let result = cmd::init::init(bad_name, false, None);
        assert!(result.is_err(), "expected error for name: {bad_name:?}");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("E502"),
            "expected E502 for name: {bad_name:?}, got: {err}"
        );
    }

    let _ = std::env::set_current_dir(&old_dir);
}

#[test]
fn init_creates_git_repo() {
    let _lock = INIT_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let project_name = "gitproj";
    let project_path = tmp.path().join(project_name);

    let old_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("cd");
    cmd::init::init(
        project_name,
        false,
        Some("rp2A03-nintendo-nes-ntsc".to_string()),
    )
    .expect("init");
    let _ = std::env::set_current_dir(&old_dir);

    assert!(project_path.join(".git").exists());
}
