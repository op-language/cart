use cart::config::GlobalConfig;
use cart::emulators::{current_os, default_emulator_matrix, is_in_path, EmulatorEntry};
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static CONFIG_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_default_matrix_has_nes_linux() {
    let matrix = default_emulator_matrix();
    let nes: Vec<_> = matrix
        .iter()
        .filter(|e| e.os == "linux" && e.target == "rp2A03-nintendo-nes-ntsc")
        .map(|e| e.emulator.as_str())
        .collect();
    assert!(nes.contains(&"fceux"));
    assert!(nes.contains(&"mesen2"));
    assert!(nes.contains(&"mednafen"));
}

#[test]
fn test_default_matrix_has_gameboy_linux() {
    let matrix = default_emulator_matrix();
    let gb: Vec<_> = matrix
        .iter()
        .filter(|e| e.os == "linux" && e.target == "sm83-nintendo-gameboy")
        .map(|e| e.emulator.as_str())
        .collect();
    assert!(gb.contains(&"sameboy"));
    assert!(gb.contains(&"mgba-sdl"));
}

#[test]
fn test_default_matrix_has_gbc_linux() {
    let matrix = default_emulator_matrix();
    let gbc: Vec<_> = matrix
        .iter()
        .filter(|e| e.os == "linux" && e.target == "sm83-nintendo-gameboy-color")
        .map(|e| e.emulator.as_str())
        .collect();
    assert!(gbc.contains(&"sameboy"));
    assert!(gbc.contains(&"mgba-sdl"));
}

#[test]
fn test_default_matrix_has_lynx_linux() {
    let matrix = default_emulator_matrix();
    let lynx: Vec<_> = matrix
        .iter()
        .filter(|e| e.os == "linux" && e.target == "vl65nc02-atari-lynx")
        .map(|e| e.emulator.as_str())
        .collect();
    assert!(lynx.contains(&"mednafen"));
}

#[test]
fn test_default_matrix_covers_all_29_targets() {
    let matrix = default_emulator_matrix();
    let targets: std::collections::HashSet<_> = matrix.iter().map(|e| e.target.as_str()).collect();
    for (triplet, _, _) in cart::targets::SUPPORTED_TARGETS {
        assert!(
            targets.contains(triplet),
            "missing emulator entries for target {triplet}"
        );
    }
}

#[test]
fn test_default_matrix_covers_three_oses() {
    let matrix = default_emulator_matrix();
    let oses: std::collections::HashSet<_> = matrix.iter().map(|e| e.os.as_str()).collect();
    assert!(oses.contains("linux"));
    assert!(oses.contains("macos"));
    assert!(oses.contains("windows"));
}

#[test]
fn test_emulators_for_returns_correct_list() {
    let config = GlobalConfig::default_with_emulators();
    let emus = config.emulators_for("linux", "sm83-nintendo-gameboy");
    assert_eq!(emus, vec!["sameboy", "mgba-sdl"]);
}

#[test]
fn test_emulators_for_nes_linux() {
    let config = GlobalConfig::default_with_emulators();
    let emus = config.emulators_for("linux", "rp2A03-nintendo-nes-ntsc");
    assert!(emus.contains(&"fceux".to_string()));
    assert!(emus.contains(&"mesen2".to_string()));
}

#[test]
fn test_emulators_for_empty_on_unknown_target() {
    let config = GlobalConfig::default_with_emulators();
    let emus = config.emulators_for("linux", "nonexistent-target");
    assert!(emus.is_empty());
}

#[test]
fn test_emulators_for_empty_on_unknown_os() {
    let config = GlobalConfig::default_with_emulators();
    let emus = config.emulators_for("plan9", "rp2A03-nintendo-nes-ntsc");
    assert!(emus.is_empty());
}

#[test]
fn test_is_in_path_finds_cargo() {
    assert!(is_in_path("cargo"));
}

#[test]
fn test_is_in_path_rejects_missing() {
    assert!(!is_in_path("this_binary_does_not_exist_xyz123"));
}

#[test]
fn test_current_os_returns_known_value() {
    let os = current_os();
    assert!(os == "linux" || os == "macos" || os == "windows");
}

#[test]
fn test_load_or_create_writes_config_file() {
    let _lock = CONFIG_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let old_home = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());

    let config_path = tmp.path().join(".cart").join("config.toml");
    assert!(!config_path.exists());

    let config = GlobalConfig::load_or_create();
    assert!(config_path.exists(), "config file should be created");
    assert!(config.emulators.is_some(), "emulators should be populated");

    let text = fs::read_to_string(&config_path).expect("read config");
    assert!(
        text.contains("[[emulators]]"),
        "config should have [[emulators]] section"
    );
    assert!(text.contains("fceux"), "config should contain fceux");

    std::env::set_var("HOME", old_home);
}

#[test]
fn test_load_or_create_preserves_existing_config() {
    let _lock = CONFIG_LOCK.lock().unwrap();
    let tmp = tempdir().expect("tempdir");
    let old_home = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("HOME", tmp.path().to_string_lossy().to_string());

    let cart_dir = tmp.path().join(".cart");
    fs::create_dir_all(&cart_dir).expect("mkdir");
    let config_path = cart_dir.join("config.toml");
    fs::write(
        &config_path,
        "[build]\ntarget = \"rp2A03-nintendo-nes-ntsc\"\n",
    )
    .expect("write config");

    let config = GlobalConfig::load_or_create();
    assert_eq!(
        config.default_target(),
        Some("rp2A03-nintendo-nes-ntsc"),
        "existing config should be loaded"
    );

    let text = fs::read_to_string(&config_path).expect("read config");
    assert!(
        !text.contains("[[emulators]]"),
        "existing config should not be overwritten"
    );

    std::env::set_var("HOME", old_home);
}

#[test]
fn test_save_and_load_roundtrip() {
    let tmp = tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.toml");

    let config = GlobalConfig::default_with_emulators();
    config.save(&config_path).expect("save");
    assert!(config_path.exists());

    let loaded = GlobalConfig::load_from(&config_path).expect("load");
    assert!(loaded.emulators.is_some());
    let emus = loaded.emulators_for("linux", "rp2A03-nintendo-nes-ntsc");
    assert!(emus.contains(&"fceux".to_string()));
}

#[test]
fn test_emulator_entry_serde() {
    let entry = EmulatorEntry {
        os: "linux".to_string(),
        target: "rp2A03-nintendo-nes-ntsc".to_string(),
        emulator: "fceux".to_string(),
    };
    let text = toml::to_string(&entry).expect("serialize");
    assert!(text.contains("os = \"linux\""));
    assert!(text.contains("emulator = \"fceux\""));
}
