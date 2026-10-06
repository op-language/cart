use cart::manifest::{CartManifest, Dependency};

#[test]
fn parse_simple_manifest() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"
edition = "1"

[[rom]]
name = "test"
path = "src/cart.op"
target = "rp2A03-nintendo-nes-ntsc"

[dependencies]
std = "1.0"
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    assert_eq!(manifest.package.name, "test");
    assert_eq!(manifest.rom.len(), 1);
    assert_eq!(manifest.rom[0].target, "rp2A03-nintendo-nes-ntsc");
    assert!(matches!(
        &manifest.dependencies["std"],
        Dependency::Simple(s) if s == "1.0"
    ));
}

#[test]
fn parse_detailed_dependency() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
nes = { version = "1.0", git = "https://github.com/op/nes", branch = "main", features = ["audio"], optional = true }
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let dep = &manifest.dependencies["nes"];
    match dep {
        Dependency::Detailed(d) => {
            assert_eq!(d.version.as_deref(), Some("1.0"));
            assert_eq!(d.git.as_deref(), Some("https://github.com/op/nes"));
            assert_eq!(d.branch.as_deref(), Some("main"));
            assert_eq!(d.features, vec!["audio"]);
            assert!(d.optional);
            assert!(d.default_features);
        }
        _ => panic!("expected Detailed dependency"),
    }
}

#[test]
fn parse_path_dependency() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
local = { path = "../local" }
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let dep = &manifest.dependencies["local"];
    assert!(matches!(dep, Dependency::Detailed(_)));
    assert_eq!(dep.path(), Some("../local"));
}

#[test]
fn roundtrip_manifest() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"
edition = "1"
authors = ["Dave"]
license = "Apache-2.0"

[[rom]]
name = "test"
path = "src/cart.op"
target = "rp2A03-nintendo-nes-ntsc"

[dependencies]
std = "1.0"

[target]
default = "rp2A03-nintendo-nes-ntsc"

[features]
debug = []
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let serialized = manifest.to_toml().expect("serialize");
    let reparsed = CartManifest::from_toml(&serialized).expect("reparse");
    assert_eq!(reparsed.package.name, manifest.package.name);
    assert_eq!(reparsed.rom.len(), manifest.rom.len());
    assert_eq!(reparsed.dependencies.len(), manifest.dependencies.len());
}

#[test]
fn parse_run_profiles() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "rp2A03-nintendo-nes-ntsc"

[[run.profile]]
name = "default"
emulator = "mesen"
args = ["--rom"]

[[run.profile]]
name = "debug"
emulator = "mesen"
args = ["--rom", "--debugger"]
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    assert_eq!(manifest.run.as_ref().unwrap().profile.len(), 2);
    let default = manifest.run_profile("default").expect("find default");
    assert_eq!(default.emulator, "mesen");
    assert_eq!(default.args, vec!["--rom"]);
}

#[test]
fn parse_test_config() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "rp2A03-nintendo-nes-ntsc"

[test]
profile = "test"

[test.sentinel.nes]
address = 0x6000
pass_value = 0xFF
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let test = manifest.test.as_ref().expect("test config");
    assert_eq!(test.profile.as_deref(), Some("test"));
    let nes = test.sentinel.get("nes").expect("nes sentinel");
    assert_eq!(nes.address, 0x6000);
    assert_eq!(nes.pass_value, 0xFF);
}

#[test]
fn parse_test_config_without_demu_section() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "rp2A03-nintendo-nes-ntsc"

[test]
profile = "test"

[test.sentinel.nes]
address = 0x6000
pass_value = 0xFF
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let test = manifest.test.as_ref().expect("test config");
    assert!(test.demu.is_none(), "no [test.demu] section");
}

#[test]
fn parse_test_demu_checks_section() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "sm83-nintendo-gameboy"

[test]
profile = "demu"

[test.demu]
checks = "tests/checks.toml"
frames = 30

[test.sentinel.gb]
address = 0xA000
pass_value = 0x64
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let test = manifest.test.as_ref().expect("test config");
    let demu = test.demu.as_ref().expect("[test.demu] section");
    assert_eq!(demu.checks.as_deref(), Some("tests/checks.toml"));
    assert_eq!(demu.frames, Some(30), "the manifest frame budget");
    assert_eq!(demu.frames(), 30);
    assert_eq!(test.profile.as_deref(), Some("demu"));

    // Round trip: the section serializes back.
    let text = manifest.to_toml().expect("serialize");
    let reparsed = CartManifest::from_toml(&text).expect("reparse");
    assert_eq!(
        reparsed.test.as_ref().unwrap().demu,
        test.demu,
        "the [test.demu] section survives the round trip"
    );
}

#[test]
fn test_demu_frames_default() {
    use cart::manifest::DEFAULT_DEMU_TEST_FRAMES;

    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "rp2A03-nintendo-nes-ntsc"

[test]
profile = "demu"

[test.demu]
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    let demu = manifest
        .test
        .as_ref()
        .expect("test config")
        .demu
        .as_ref()
        .expect("[test.demu] section");
    assert_eq!(demu.frames, None, "no frames field in the manifest");
    assert_eq!(
        demu.frames(),
        DEFAULT_DEMU_TEST_FRAMES,
        "the default budget applies"
    );
    assert_eq!(DEFAULT_DEMU_TEST_FRAMES, 120);
}

#[test]
fn default_target_from_rom() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "rp2A03-nintendo-nes-ntsc"
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    assert_eq!(
        manifest.default_target(None),
        Some("rp2A03-nintendo-nes-ntsc".to_string())
    );
}

#[test]
fn default_target_override() {
    let text = r#"
[package]
name = "test"
version = "0.1.0"

[[rom]]
name = "test"
target = "rp2A03-nintendo-nes-ntsc"
"#;
    let manifest = CartManifest::from_toml(text).expect("parse");
    assert_eq!(
        manifest.default_target(Some("z80-nintendo-gameboy")),
        Some("z80-nintendo-gameboy".to_string())
    );
}
