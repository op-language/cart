//! `cart test` — build and run test ROMs in an emulator.

use crate::config::GlobalConfig;
use crate::manifest::CartManifest;
use crate::opc::{self, OpcArgs, OpcStage};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn test(manifest_path: &Path, target: Option<String>) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;
    let config = GlobalConfig::load();

    let tests_dir = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("tests");

    if !tests_dir.exists() || !tests_dir.is_dir() {
        eprintln!("No tests/ directory found. Nothing to test.");
        return Ok(());
    }

    let test_files: Vec<_> = std::fs::read_dir(&tests_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "op").unwrap_or(false))
        .collect();

    if test_files.is_empty() {
        eprintln!("No .op test files found in tests/. Nothing to test.");
        return Ok(());
    }

    let test_profile_name = manifest
        .test
        .as_ref()
        .and_then(|t| t.profile.as_deref())
        .or_else(|| config.test.as_ref().and_then(|t| t.profile.as_deref()))
        .unwrap_or("test");

    let test_profile = manifest
        .run_profile(test_profile_name)
        .or_else(|| config.run_profile(test_profile_name))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "E501: test profile '{}' not found in Cart.toml or config",
                test_profile_name
            )
        })?;

    let rom = manifest
        .rom
        .first()
        .ok_or_else(|| anyhow::anyhow!("E502: no ROM targets in Cart.toml for test build"))?;

    let rom_target = target.clone().unwrap_or_else(|| rom.target.clone());

    let mut passed = 0;
    let mut failed = 0;

    for test_file in &test_files {
        let test_name = test_file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        eprintln!("Building test: {test_name}...");

        let output_dir = manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("target")
            .join(&rom_target)
            .join("tests");
        std::fs::create_dir_all(&output_dir)?;

        let output = output_dir.join(format!("{test_name}.bin"));

        let args = OpcArgs {
            input: test_file.clone(),
            target: rom_target.clone(),
            features: vec!["test".to_string()],
            opt_level: 0,
            format: Some("raw".to_string()),
            output: Some(output.clone()),
            stage: OpcStage::Full,
            include: Vec::new(),
        };

        if let Err(e) = opc::invoke(&args) {
            eprintln!("  FAIL: build error: {e}");
            failed += 1;
            continue;
        }

        let dump_path = output_dir.join(format!("{test_name}.dump"));
        let emulator = which(&test_profile.emulator).ok_or_else(|| {
            anyhow::anyhow!(
                "E501: emulator '{}' not found on PATH",
                test_profile.emulator
            )
        })?;

        let mut cmd = Command::new(&emulator);
        for arg in &test_profile.args {
            cmd.arg(arg);
        }
        cmd.arg(&output).arg("--dump").arg(&dump_path);

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("E501: failed to launch emulator: {e}"))?;

        if !status.success() {
            eprintln!("  FAIL: emulator exited with error");
            failed += 1;
            continue;
        }

        let sentinel_config = manifest
            .test
            .as_ref()
            .and_then(|t| t.sentinel.get(rom_target.split('-').nth(2).unwrap_or("")));

        if let Some(sentinel) = sentinel_config {
            if !dump_path.exists() {
                eprintln!("  FAIL: no memory dump at {}", dump_path.display());
                failed += 1;
                continue;
            }

            let dump = std::fs::read(&dump_path)?;
            let addr = sentinel.address as usize;
            if addr >= dump.len() {
                eprintln!("  FAIL: sentinel address {addr:#x} out of range");
                failed += 1;
                continue;
            }

            if dump[addr] as u64 == sentinel.pass_value {
                eprintln!("  PASS: {test_name}");
                passed += 1;
            } else {
                eprintln!(
                    "  FAIL: {test_name} (sentinel: got {:#x}, expected {:#x})",
                    dump[addr], sentinel.pass_value
                );
                failed += 1;
            }
        } else {
            eprintln!("  PASS: {test_name} (no sentinel configured, build-only)");
            passed += 1;
        }
    }

    eprintln!("\nTest results: {passed} passed, {failed} failed");

    if failed > 0 {
        std::process::exit(2);
    }

    Ok(())
}

fn which(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var("PATH").ok()?;
    for dir in path.split(':') {
        let candidate = std::path::PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
