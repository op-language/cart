# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No changes.

## [0.6.0]

### Added
- `format` field in the `[[rom]]` section of `Cart.toml`. The build
  uses this field to determine the output file extension. For the NES,
  `format = "ines"` produces a `.nes` file. `cart init` sets the
  format automatically for NES and Lynx targets.
- `--include` path passing to the `opc` compiler. The build collects
  include paths from resolved dependencies and passes them with `-I`
  flags.
- Auto-checkout of the std lib to `~/.cart/std/`. If the std lib is
  not present, `cart build` clones it from
  `https://github.com/op-language/std`.
- `~/.cart/std/src` is always added as a default include path.

### Changed
- `cart build` now uses the manifest `rom.format` field to determine
  the output format, falling back to the `--format` CLI flag, then
  `"bin"` as the default.
- `rom_output_path` now checks the manifest for the ROM format field
  to determine the correct file extension.
- `cart run` now builds the ROM only if it does not already exist.
  Previously, `cart run` always called `cart build` unconditionally.
- `cart run` now passes the manifest `rom.format` field to
  `rom_output_path` to determine the correct ROM file path.

## [0.5.0]

### Changed
- Updated `SUPPORTED_TARGETS` in `src/targets.rs`. Replaced
  `mos65sc02-atari-lynx` with `vl65nc02-atari-lynx`. Replaced
  `z80-nintendo-gameboy` with `sm83-nintendo-gameboy`. Replaced
  `z80-nintendo-gameboy-color` with `sm83-nintendo-gameboy-color`.
  Updated CPU names to `VLSI VL65NC02` and `Sharp SM83`.

## [0.4.0]

### Added
- `dialoguer` dependency for the interactive target selection.
- `SUPPORTED_TARGETS` registry in `src/targets.rs`. It lists every
  canonical triplet, CPU name, and platform name.
- `cart init` shows an interactive select list when the user does
  not supply the `--target` flag. The selected triplet becomes the
  `default` target in `Cart.toml`. The `--target` flag overrides the
  prompt.

### Changed
- All tests use `rp2A03-nintendo-nes-ntsc` as the fixture target.
- The technical-design doc references
  `rp2A03-nintendo-nes-ntsc`.

## [0.3.0]

### Added
- `cart build` now clones a missing git dependency into `~/.carts/`
  during resolution instead of erroring with E505.
- `cart build` now compiles lib projects with `opc` and writes the
  output to `target/<triplet>/<libname>.opb`.
- `cart init` now emits the `[features]` table in the generated
  `Cart.toml`.
- `cart init` now validates the project name. Names must contain only
  lowercase letters, digits, hyphens, and underscores.

### Changed
- A dependency must now specify either `git` or `path`. A version-only
  dependency (e.g. `std = "1.0"`) is rejected with E504.
- `cart add` now requires `--git` or `--path`. The version-only
  fallback is removed.
- `cart init` lib template doc comment changed from "Lib entry point."
  to "Bank entry point." to match the design document.

## [0.2.0]

### Changed
- Renamed "bank" to "lib" throughout: `--bank` CLI flag is now `--lib`,
  `[bank]` TOML section is now `[lib]`, `bank.op` is now `lib.op`, `Bank`
  struct is now `Lib`, `BANK_ENTRY` constant is now `LIB_ENTRY`,
  `BANK_NOT_INSTALLED` constant is now `LIB_NOT_INSTALLED`.
- Dependency package names drop the `-bank` suffix: `mos6502-bank` is now
  `mos6502`, `nes-bank` is now `nes`, etc.
- `cart add` and `cart install` positional argument renamed from `<bank>`
  to `<name>`.
- All error messages, log messages, and doc comments updated from "bank"
  to "lib".
- Hardware banking terms (`#[rom(bank = 0)]`, `#[chr(bank = 0)]`,
  `CART_BANK_0`, `cpu::dbr`, `pub banks: u32`, `SectionKind::Bank`) are
  kept as "bank" because they refer to retro hardware memory banking.

## [0.1.0]

### Added
- Initial `cart` standalone project split from the `op` workspace.
- CLI subcommands: `init`, `build`, `run`, `test`, `check`, `clean`, `add`,
  `doc`, `install`, `update`.
- Vendored `TargetTriplet`, `CartManifest`, and `Diagnostic` types from the
  `op` workspace.