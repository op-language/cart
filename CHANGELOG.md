# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial `cart` standalone project split from the `op` workspace.
- CLI subcommands: `init`, `build`, `run`, `test`, `check`, `clean`, `add`,
  `doc`, `install`, `update`.
- Vendored `TargetTriplet`, `CartManifest`, and `Diagnostic` types from the
  `op` workspace.