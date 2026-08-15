# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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