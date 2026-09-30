//! Known emulator executables per operating system per target triplet.
//!
//! This module holds the default emulator matrix and the helpers that
//! detect the operating system and test if a binary is on the PATH.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single entry in the emulator matrix.
///
/// The `os` field is "linux", "macos", or "windows". The `target` field
/// is a target triplet such as "rp2A03-nintendo-nes-ntsc". The
/// `emulator` field is the binary name such as "fceux".
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmulatorEntry {
    pub os: String,
    pub target: String,
    pub emulator: String,
}

/// Wrapper for the `[[emulators]]` section in `~/.cart/config.toml`.
///
/// Uses `transparent` so the entries serialize directly as
/// `[[emulators]]` rather than `[[emulators.emulator]]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct EmulatorsConfig {
    pub emulator: Vec<EmulatorEntry>,
}

/// Detect the operating system.
///
/// Returns "linux", "macos", or "windows". Returns "linux" as a
/// fallback for unknown operating systems.
pub fn current_os() -> &'static str {
    match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    }
}

/// Test if a binary is on the PATH.
///
/// Splits the PATH variable on the correct separator. Checks each
/// directory for the binary. On Windows, appends the `.exe` suffix.
/// Returns true if any directory holds the binary as a file.
pub fn is_in_path(binary: &str) -> bool {
    let path_var = match std::env::var("PATH") {
        Ok(v) => v,
        Err(_) => return false,
    };
    let separator = if current_os() == "windows" { ';' } else { ':' };
    let bin_name = if current_os() == "windows" && !binary.ends_with(".exe") {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    };
    for dir in path_var.split(separator) {
        if dir.is_empty() {
            continue;
        }
        let candidate = std::path::Path::new(dir).join(&bin_name);
        if candidate.is_file() {
            return true;
        }
    }
    false
}

/// Build the default emulator matrix.
///
/// The matrix covers all supported target triplets across Linux, macOS,
/// and Windows. The data comes from `op/docs/supported-emulators.md`.
pub fn default_emulator_matrix() -> Vec<EmulatorEntry> {
    let mut entries = Vec::new();

    // Helper macro: add an emulator for a triplet on all three OSes.
    macro_rules! add {
        ($triplet:expr, $linux:expr, $macos:expr, $windows:expr) => {{
            let triplet = $triplet;
            let linux: &[&str] = $linux;
            let macos: &[&str] = $macos;
            let windows: &[&str] = $windows;
            for emu in linux {
                entries.push(EmulatorEntry {
                    os: "linux".to_string(),
                    target: triplet.to_string(),
                    emulator: emu.to_string(),
                });
            }
            for emu in macos {
                entries.push(EmulatorEntry {
                    os: "macos".to_string(),
                    target: triplet.to_string(),
                    emulator: emu.to_string(),
                });
            }
            for emu in windows {
                entries.push(EmulatorEntry {
                    os: "windows".to_string(),
                    target: triplet.to_string(),
                    emulator: emu.to_string(),
                });
            }
        }};
    }

    // NES (NTSC and PAL): FCEUX, Mesen2, Mednafen
    add!(
        "rp2A03-nintendo-nes-ntsc",
        &["fceux", "mesen2", "mednafen"],
        &["mesen2", "fceux"],
        &["mesen2", "fceux"]
    );
    add!(
        "rp2A07-nintendo-nes-pal",
        &["fceux", "mesen2", "mednafen"],
        &["mesen2", "fceux"],
        &["mesen2", "fceux"]
    );

    // Atari Lynx: Mednafen
    add!(
        "vl65nc02-atari-lynx",
        &["mednafen"],
        &["mednafen"],
        &["mednafen"]
    );

    // Apple II family: MAME
    add!("mos6502-apple-ii", &["mame"], &["mame"], &["mame"]);
    add!("mos6502-apple-iic", &["mame"], &["mame"], &["mame"]);
    add!("mos6502-apple-iie", &["mame"], &["mame"], &["mame"]);
    add!(
        "mos6502-apple-iie-enhanced",
        &["mame"],
        &["mame"],
        &["mame"]
    );

    // Apple IIgs: MAME
    add!("wdc65c816-apple-iigs", &["mame"], &["mame"], &["mame"]);

    // Atari 800 (NTSC and PAL): Atari800, MAME
    add!(
        "mos6502-atari-800-ntsc",
        &["atari800", "mame"],
        &["mame"],
        &["mame"]
    );
    add!(
        "mos6502-atari-800-pal",
        &["atari800", "mame"],
        &["mame"],
        &["mame"]
    );

    // Atari 2600: Stella
    add!("mos6502-atari-2600", &["stella"], &["stella"], &["stella"]);

    // Atari 5200: Atari800, MAME
    add!(
        "mos6502-atari-5200",
        &["atari800", "mame"],
        &["mame"],
        &["mame"]
    );

    // Atari 7800: MAME
    add!("mos6502-atari-7800", &["mame"], &["mame"], &["mame"]);

    // Commodore 64: VICE
    add!(
        "mos6502-commodore-64",
        &["x64sc", "vice"],
        &["x64sc", "vice"],
        &["x64sc", "vice"]
    );

    // NEC PC Engine: Mednafen
    add!(
        "mos6502-nec-pcengine",
        &["mednafen"],
        &["mednafen"],
        &["mednafen"]
    );

    // Neo Geo AES (68000): MAME
    add!("m68000-neogeo-aes", &["mame"], &["mame"], &["mame"]);

    // Sega Genesis (68000): BlastEm, Mednafen
    add!(
        "m68000-sega-genesis",
        &["blastem", "mednafen"],
        &["blastem", "mednafen"],
        &["blastem", "mednafen"]
    );

    // SNES: bsnes, Snes9x, Mesen2
    add!(
        "wdc65c816-nintendo-snes",
        &["bsnes", "snes9x", "mesen2"],
        &["bsnes", "snes9x", "mesen2"],
        &["bsnes", "snes9x", "mesen2"]
    );

    // Neo Geo AES (Z80 sound CPU): MAME
    add!("z80-neogeo-aes", &["mame"], &["mame"], &["mame"]);

    // Game Boy: SameBoy, mGBA
    add!(
        "sm83-nintendo-gameboy",
        &["sameboy", "mgba-sdl"],
        &["sameboy", "mgba-sdl"],
        &["sameboy", "mgba-sdl"]
    );

    // Game Boy Color: SameBoy, mGBA
    add!(
        "sm83-nintendo-gameboy-color",
        &["sameboy", "mgba-sdl"],
        &["sameboy", "mgba-sdl"],
        &["sameboy", "mgba-sdl"]
    );

    // Sega Game Gear: Mednafen, MAME
    add!(
        "z80-sega-gamegear",
        &["mednafen", "mame"],
        &["mednafen", "mame"],
        &["mednafen", "mame"]
    );

    // Sega Genesis (Z80): BlastEm, Mednafen
    add!(
        "z80-sega-genesis",
        &["blastem", "mednafen"],
        &["blastem", "mednafen"],
        &["blastem", "mednafen"]
    );

    // Sega Master System: Mednafen, MAME
    add!(
        "z80-sega-mastersystem",
        &["mednafen", "mame"],
        &["mednafen", "mame"],
        &["mednafen", "mame"]
    );

    // Sega SG-1000: Mednafen, MAME
    add!(
        "z80-sega-sg1000",
        &["mednafen", "mame"],
        &["mednafen", "mame"],
        &["mednafen", "mame"]
    );

    // Sinclair ZX80: SZ81, MAME
    add!(
        "z80-sinclair-zx80",
        &["sz81", "mame"],
        &["sz81", "mame"],
        &["sz81", "mame"]
    );

    // Sinclair ZX81: SZ81, MAME
    add!(
        "z80-sinclair-zx81",
        &["sz81", "mame"],
        &["sz81", "mame"],
        &["sz81", "mame"]
    );

    // Sinclair Spectrum: Fuse, MAME
    add!(
        "z80-sinclair-spectrum",
        &["fuse", "mame"],
        &["fuse", "mame"],
        &["fuse", "mame"]
    );

    // Texas Instruments TI-85: TilEm2
    add!("z80-ti-85", &["tilem"], &["tilem"], &["tilem"]);

    // Commander X16: x16emu
    add!(
        "w65c02-commander-x16",
        &["x16emu"],
        &["x16emu"],
        &["x16emu"]
    );

    entries
}

/// Return the emulator-specific command-line arguments that select the
/// console model for the given target triplet and emulator binary name.
///
/// Some emulators require an explicit model flag to run a ROM in the
/// correct console mode. SameBoy, for example, uses `--model dmg` for
/// Game Boy (DMG) ROMs and `--model cgb` for Game Boy Color ROMs.
///
/// Returns an empty vector when the given emulator does not need a model
/// flag for the given target triplet.
pub fn emulator_model_args(triplet: &str, emulator: &str) -> Vec<String> {
    // Compare against the binary name (last path component) so that
    // absolute or relative paths to the emulator also match.
    let emu = emulator.rsplit(['/', '\\']).next().unwrap_or(emulator);
    match (triplet, emu) {
        ("sm83-nintendo-gameboy", "sameboy") => {
            vec!["--model".to_string(), "dmg".to_string()]
        }
        ("sm83-nintendo-gameboy-color", "sameboy") => {
            vec!["--model".to_string(), "cgb".to_string()]
        }
        _ => Vec::new(),
    }
}

/// Return the fixed arguments that `cart run` places before the run
/// profile's own arguments when launching a ROM.
///
/// SameBoy and the other model-flag emulators keep the
/// `emulator_model_args` contract: the returned arguments precede the
/// ROM path, which the caller appends itself. x16emu is the exception:
/// its `-run` flag is an option to `-prg` and must follow the PRG path,
/// so the returned arguments embed the ROM path between the two flags
/// and the caller must not append it again.
pub fn emulator_prepend_args(triplet: &str, emulator: &str, rom_path: &Path) -> Vec<String> {
    // Compare against the binary name (last path component) so that
    // absolute or relative paths to the emulator also match.
    let emu = emulator.rsplit(['/', '\\']).next().unwrap_or(emulator);
    if triplet == "w65c02-commander-x16" && emu == "x16emu" {
        return vec![
            "-prg".to_string(),
            rom_path.to_string_lossy().into_owned(),
            "-run".to_string(),
        ];
    }
    emulator_model_args(triplet, emulator)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_default_matrix_covers_all_supported_targets() {
        let matrix = default_emulator_matrix();
        let targets: std::collections::HashSet<_> =
            matrix.iter().map(|e| e.target.as_str()).collect();
        for (triplet, _, _) in crate::targets::SUPPORTED_TARGETS {
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
    fn test_is_in_path_finds_cargo() {
        // cargo is on the PATH in any reasonable test environment
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
    fn test_model_args_sameboy_dmg() {
        let args = emulator_model_args("sm83-nintendo-gameboy", "sameboy");
        assert_eq!(args, vec!["--model".to_string(), "dmg".to_string()]);
    }

    #[test]
    fn test_model_args_sameboy_cgb() {
        let args = emulator_model_args("sm83-nintendo-gameboy-color", "sameboy");
        assert_eq!(args, vec!["--model".to_string(), "cgb".to_string()]);
    }

    #[test]
    fn test_model_args_sameboy_path() {
        // Absolute/relative paths to the emulator should also match.
        let args = emulator_model_args("sm83-nintendo-gameboy", "/usr/local/bin/sameboy");
        assert_eq!(args, vec!["--model".to_string(), "dmg".to_string()]);
    }

    #[test]
    fn test_model_args_other_emulator_empty() {
        assert!(emulator_model_args("sm83-nintendo-gameboy", "mgba-sdl").is_empty());
    }

    #[test]
    fn test_model_args_other_target_empty() {
        assert!(emulator_model_args("rp2A03-nintendo-nes-ntsc", "sameboy").is_empty());
    }

    #[test]
    fn test_default_matrix_has_x16_all_oses() {
        let matrix = default_emulator_matrix();
        for os in ["linux", "macos", "windows"] {
            let entries: Vec<_> = matrix
                .iter()
                .filter(|e| e.os == os && e.target == "w65c02-commander-x16")
                .map(|e| e.emulator.as_str())
                .collect();
            assert!(entries.contains(&"x16emu"), "no x16emu entry on {os}");
        }
    }

    #[test]
    fn test_prepend_args_x16emu_prg_run() {
        let args = emulator_prepend_args(
            "w65c02-commander-x16",
            "x16emu",
            std::path::Path::new("demo.prg"),
        );
        assert_eq!(
            args,
            vec![
                "-prg".to_string(),
                "demo.prg".to_string(),
                "-run".to_string()
            ]
        );
    }

    #[test]
    fn test_prepend_args_x16emu_path() {
        // Absolute/relative paths to the emulator should also match.
        let args = emulator_prepend_args(
            "w65c02-commander-x16",
            "/opt/x16/x16emu",
            std::path::Path::new("demo.prg"),
        );
        assert_eq!(args[0], "-prg".to_string());
        assert_eq!(args[2], "-run".to_string());
    }

    #[test]
    fn test_prepend_args_defers_to_model_args() {
        // Other pairs keep the emulator_model_args contract: fixed model
        // arguments only, with the ROM path appended by the caller.
        let args = emulator_prepend_args(
            "sm83-nintendo-gameboy",
            "sameboy",
            std::path::Path::new("game.gb"),
        );
        assert_eq!(args, vec!["--model".to_string(), "dmg".to_string()]);
    }

    #[test]
    fn test_prepend_args_other_pair_empty() {
        assert!(emulator_prepend_args(
            "rp2A03-nintendo-nes-ntsc",
            "mesen2",
            std::path::Path::new("game.nes"),
        )
        .is_empty());
    }
}
