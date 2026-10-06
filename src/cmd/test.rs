//! `cart test` — build and run test ROMs in an emulator.

use crate::config::GlobalConfig;
use crate::manifest::CartManifest;
use crate::opc::{self, OpcArgs, OpcStage};
use anyhow::Result;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

/// One `cart test` run over the whole `tests/` directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestSummary {
    /// The tests that passed.
    pub passed: usize,
    /// The tests that failed.
    pub failed: usize,
}

pub fn test(manifest_path: &Path, target: Option<String>) -> Result<TestSummary> {
    let manifest = CartManifest::load(manifest_path)?;
    let config = GlobalConfig::load_or_create();

    let tests_dir = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("tests");

    if !tests_dir.exists() || !tests_dir.is_dir() {
        eprintln!("No tests/ directory found. Nothing to test.");
        return Ok(TestSummary {
            passed: 0,
            failed: 0,
        });
    }

    let test_files: Vec<_> = std::fs::read_dir(&tests_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "op").unwrap_or(false))
        .collect();

    if test_files.is_empty() {
        eprintln!("No .op test files found in tests/. Nothing to test.");
        return Ok(TestSummary {
            passed: 0,
            failed: 0,
        });
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

    // The native demu mode activates when the test profile names demu.
    // Compare against the binary name (last path component) so that
    // paths to the emulator match like `emulator_model_args` does.
    let test_emulator_name = test_profile
        .emulator
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(&test_profile.emulator);
    let is_demu = test_emulator_name == "demu";

    // The demu checks file resolves against the manifest directory.
    let demu_checks_path = manifest
        .test
        .as_ref()
        .and_then(|t| t.demu.as_ref())
        .and_then(|d| d.checks.as_deref())
        .map(|checks| {
            manifest_path
                .parent()
                .unwrap_or(Path::new("."))
                .join(checks)
        });

    // The demu run-frame budget: the manifest field, or the default
    // when the section or the field is absent.
    let demu_frames = manifest
        .test
        .as_ref()
        .and_then(|t| t.demu.as_ref())
        .map_or(crate::manifest::DEFAULT_DEMU_TEST_FRAMES, |d| d.frames());

    let machine = rom_target.split('-').nth(2).unwrap_or("");
    let sentinel_config = manifest.test.as_ref().and_then(|t| t.sentinel.get(machine));

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
            // The demu session loads the linked symbol table that opc
            // writes next to the ROM when stage output is on.
            output_stages: is_demu,
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
        for arg in crate::emulators::emulator_model_args(&rom_target, &test_profile.emulator) {
            cmd.arg(arg);
        }
        for arg in &test_profile.args {
            cmd.arg(arg);
        }
        cmd.arg(&output).arg("--dump").arg(&dump_path);

        if is_demu {
            // demu compares the sentinel byte itself at session end and
            // sets the exit code, so the flags take the place of the
            // dump-reading pass check below.
            if let Some(sentinel) = sentinel_config {
                for arg in
                    crate::emulators::demu_sentinel_args(sentinel.address, sentinel.pass_value)
                {
                    cmd.arg(arg);
                }
            }
            cmd.arg("--symbols")
                .arg(super::build::symbol_output_path(&output));
            if let Some(checks) = &demu_checks_path {
                cmd.arg("--checks").arg(checks);
            }
            // demu boots and parks, so the session carries one
            // run-frame command that lets the test ROM run; demu
            // evaluates the sentinel when the session ends. Dropping
            // the stdin handle closes it and ends the session at the
            // REPL's next read.
            cmd.stdin(Stdio::piped());
            let mut child = cmd
                .spawn()
                .map_err(|e| anyhow::anyhow!("E501: failed to launch emulator: {e}"))?;
            if let Some(mut stdin) = child.stdin.take() {
                writeln!(stdin, "run-frame {demu_frames}")?;
                std::mem::drop(stdin);
            }
            let status = child
                .wait()
                .map_err(|e| anyhow::anyhow!("E501: failed to wait for the emulator: {e}"))?;

            if status.success() {
                eprintln!("  PASS: {test_name}");
                passed += 1;
            } else {
                let note = match status.code() {
                    Some(1) => " (sentinel miss: demu exited 1)".to_string(),
                    Some(code) => format!(" (demu exited with {code})"),
                    None => " (demu killed by signal)".to_string(),
                };
                eprintln!("  FAIL: {test_name}{note}");
                failed += 1;
            }
            continue;
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("E501: failed to launch emulator: {e}"))?;

        if !status.success() {
            eprintln!("  FAIL: emulator exited with error");
            failed += 1;
            continue;
        }

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

    Ok(TestSummary { passed, failed })
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
