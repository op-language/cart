# Cart Build Tool Technical Design

Version 1.0

This document defines the technical design of the `cart` build tool and
package manager. The `cart` tool manages Op projects the same way `cargo`
manages Rust projects. It reads and writes the `Cart.toml` manifest,
resolves dependencies from `~/.carts/`, invokes `opc` to compile projects,
and installs libs from a git-based registry.

This document uses the keywords **must**, **shall**, and **may** as RFC
2119 defines.

## Scope

This document defines the `cart` binary, the manifest format, the lockfile
format, the global config, and the dependency model. It also defines the
registry protocol, the build pipeline, the emulator launch, the test
harness, the documentation generator, and the diagnostic system.

This document does not define the Op language grammar or the per-target
opcode tables. The document `language-specification.md` in the `op`
repository defines those. This document does not define the `opc` compiler
pipeline or the intermediate file formats. The document
`technical-design.md` in the `op` repository defines those.

## Background

The `cart` tool was part of the `op` workspace. The project split `cart`
into a standalone repository so that the build tool and the compiler can
ship on independent release cycles. The standalone `cart` repository
vendored the `TargetTriplet`, `CartManifest`, and `Diagnostic` types from
the `op` workspace. This document specifies the full end-state design. The
current code is a skeleton. The implementation work follows this document.

## Design goals

1. The `cart` tool must manage Op projects the same way `cargo` manages
   Rust projects.
2. The `cart` tool must be a single binary that runs on Linux, macOS, and
   Windows.
3. The `cart` tool must invoke the `opc` compiler to build projects.
4. The `cart` tool must install libs in `~/.carts/` and resolve
   dependencies from that directory.
5. The `cart` tool must read and write the `Cart.toml` manifest.
6. The `cart` tool must write and read a `Cart.lock` lockfile for
   reproducible builds.
7. The `cart` tool must support a git-based registry for lib
   installation.
8. The `cart` tool must read a global config in `~/.cart/config.toml`.
9. The `cart` tool must launch the configured emulator for `cart run` and
   `cart test`.
10. The `cart` tool must report errors with the structured diagnostic
    format that the `op` repository defines.

## Architecture overview

The `cart` binary has a thin entry point and a set of command modules. The
entry point parses the command line and dispatches to the matching command
module. Each command module reads the manifest, resolves dependencies, and
invokes `opc` or the registry as needed.

```
cart binary
  |
  +-- cli          command-line parser (clap)
  +-- manifest     Cart.toml types and TOML read/write
  +-- lockfile     Cart.lock types and TOML read/write
  +-- config       ~/.cart/config.toml types and precedence merge
  +-- triplet      target triplet parser
  +-- resolver     dependency graph resolver
  +-- registry     git clone and pull for lib installation
  +-- opc          opc invocation and diagnostic parsing
  +-- diagnostics  error and warning reporting
  +-- cmd/
       +-- init    project scaffolding
       +-- build   compile and write ROM
       +-- run     build and launch emulator
       +-- test    build and run test ROMs
       +-- check   parse-only
       +-- clean   remove target directory
       +-- add     add a dependency
       +-- doc     generate Markdown documentation
       +-- install install a lib
       +-- update  update all dependencies
```

The data flow for a build:

```
Cart.toml --> resolver --> ~/.carts/<name>/ --> opc --> target/<triplet>/
                 |
                 +-> Cart.lock
```

## Command-line interface

```
cart [OPTIONS] <COMMAND>

Global options:
  --manifest-path <PATH>   Path to Cart.toml. Default: ./Cart.toml.
  --quiet                  Suppress non-error output.
  --verbose                Print extra diagnostic output.
  --color <WHEN>           Color output: auto, always, never.
  --frozen                 Error if Cart.lock is out of date.
```

## Subcommand specifications

### cart init

```
cart init [OPTIONS] <name>
```

The `cart init` command creates a new Op project. It makes a directory
with the given name. Inside the directory it creates a git repository, a
`Cart.toml` manifest, a `.gitignore` file, a `src/` directory, and a
`tests/` directory.

When the user does not pass `--lib`, the command creates a ROM project.
The entry file is `src/cart.op`. The `Cart.toml` has one `[[rom]]` section.

When the user passes `--lib`, the command creates a lib project. The
entry file is `src/lib.op`. The `Cart.toml` has one `[lib]` section.

Options:

| Flag | Description |
|------|-------------|
| `--lib` | Create a library (lib) project with `src/lib.op`. |
| `--target <triplet>` | Set the default target triplet in `Cart.toml`. |

ROM project `Cart.toml` template:

```toml
[package]
name = "<name>"
version = "0.1.0"
edition = "1"
authors = []
license = ""

[[rom]]
name = "<name>"
path = "src/cart.op"
target = "<triplet>"

[target]
default = "<triplet>"

[dependencies]

[features]
```

ROM project `src/cart.op` template:

```op
//! <name>
//!
//! Project entry point.

noreturn fn main() {
    loop {
    }
}
```

Lib project `Cart.toml` template:

```toml
[package]
name = "<name>"
version = "0.1.0"
edition = "1"
authors = []
license = ""

[lib]
name = "<name>"
path = "src/lib.op"

[dependencies]

[features]
```

Lib project `src/lib.op` template:

```op
//! <name> lib
//!
//! Bank entry point.
```

The `.gitignore` file ignores the `target/` directory.

### cart build

```
cart build [OPTIONS]
```

The `cart build` command reads `Cart.toml`, resolves dependencies, reads
or writes `Cart.lock`, and invokes `opc` to compile each `[[rom]]` target.
It writes the ROM image to `target/<triplet>/<rom-name>.<ext>`.

Options:

| Flag | Description |
|------|-------------|
| `--target <triplet>` | Override the target triplet. |
| `--release` | Build with optimization level 1. |
| `--debug` | Build with optimization level 0. |
| `--feature <name>` | Enable a feature flag. Repeatable. |
| `--format <name>` | Override the output format. |

The `opc` argument contract:

| `opc` flag | Source |
|------------|--------|
| `--target <triplet>` | The resolved target triplet. |
| `--feature <name>` | Each enabled feature flag. |
| `-O <level>` | `0` for `--debug`, `1` for `--release`. Default: `1`. |
| `--format <name>` | The `--format` override if present. |
| `-o <path>` | `target/<triplet>/<rom-name>.<ext>`. |
| `<input>` | The `[[rom]]` `path` field. |

The output extension depends on the target output format. For `ines` the
extension is `.nes`. For `lnx` the extension is `.lnx`. For `raw` the
extension is `.bin`. For `hex` the extension is `.hex`.

If the user passes `--frozen` and `Cart.lock` is out of date, the command
reports error E506 and exits.

### cart run

```
cart run [OPTIONS]
```

The `cart run` command builds the project, selects a run profile, finds the
emulator on `PATH`, and launches the ROM.

Options:

| Flag | Description |
|------|-------------|
| `--profile <name>` | Select a `[[run.profile]]`. Default: `default`. |
| `--target <triplet>` | Override the target triplet. |
| `--release` | Build with optimization level 1. |

The command reads the `[[run.profile]]` sections from `Cart.toml`. The
profile with the name `default` is the default. If the user passes
`--profile`, the command selects the profile with that name. If no profile
matches, the command reports error E501.

The `[[run.profile]]` section has these fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Profile name. |
| `emulator` | string | yes | Emulator executable name. |
| `args` | list of strings | no | Arguments to pass to the emulator. |
| `target` | string | no | Override the target triplet for this profile. |

The command finds the emulator executable on `PATH`. If the emulator is
absent, the command reports error E501.

The command builds the argv from the profile `args` and appends the ROM
path. It spawns the emulator with `std::process::Command`. It inherits the
stdio. It forwards the exit code.

### cart test

```
cart test [OPTIONS]
```

The `cart test` command runs the project test suite. Tests are Op source
files in the `tests/` directory.

Options:

| Flag | Description |
|------|-------------|
| `--target <triplet>` | Override the target triplet. |

For each file in `tests/*.op`, the command builds a test ROM. It passes
`--cfg test` to `opc` so that the `#[cfg(feature = "test")]` predicate
selects test code. It links a sentinel stub from the std lib or from the
`[test]` config. It runs the ROM in the emulator that the test profile
names.

The test ROM writes a sentinel byte to a fixed memory address when the
test passes. The `[test]` table in `Cart.toml` maps each machine name to a
sentinel address and a pass value. The command reads the memory dump from
the emulator and checks the sentinel byte.

`[test]` table fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `profile` | string | no | Run profile name for the test emulator. Default: `test`. |
| `sentinel` | table | yes | Map from machine name to sentinel config. |

`[test.sentinel.<machine>]` fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | integer | yes | Memory address of the sentinel byte. |
| `pass_value` | integer | yes | Byte value that signals a pass. |

Example:

```toml
[test]
profile = "test"

[test.sentinel.nes]
address = 0x6000
pass_value = 0xFF

[test.sentinel.lynx]
address = 0xFC00
pass_value = 0x01
```

The command reads the memory dump from the emulator. The emulator writes a
flat binary file to a path that the command passes as a `--dump <path>`
argument. The command reads the byte at the sentinel `address`. It compares
the byte to the `pass_value`. If they match, the test passes. If they do
not match, the test fails.

The command reports pass or fail for each test file. It prints a summary at
the end. If any test fails, the command exits with status code 2.

### cart check

```
cart check [OPTIONS]
```

The `cart check` command invokes `opc --parse` on the project source. It
runs the lexer and the parser. It does not generate code. It is faster
than `cart build`.

Options:

| Flag | Description |
|------|-------------|
| `--target <triplet>` | Override the target triplet. |

The command resolves dependencies and invokes `opc` with the `--parse`
flag. It reports all lexer and parser errors. It does not write any output
files.

### cart clean

```
cart clean
```

The `cart clean` command removes the `target/` directory. It does not
remove `Cart.lock` or any files in `~/.carts/`.

### cart add

```
cart add [OPTIONS] <name>
```

The `cart add` command adds a lib to the `Cart.toml` `[dependencies]`
section. It fetches and installs the lib into `~/.carts/`.

Options:

| Flag | Description |
|------|-------------|
| `--git <URL>` | Git URL for the lib. |
| `--path <PATH>` | Local path for the lib. |
| `--version <REQ>` | Version requirement string. |

The command modifies `Cart.toml` in place. It adds the lib entry to the
`[dependencies]` section. It then fetches and installs the lib.

### cart doc

```
cart doc
```

The `cart doc` command generates Markdown documentation from the doc
comments in the project source files.

The command walks the module tree from the root file. For each `///` doc
comment, it writes a Markdown heading and the doc text. For each `//!`
module doc comment, it writes a module preamble. It links `use` and `::`
references to the target module files.

The output tree is `target/doc/<project>/index.md` plus one `module/*.md`
file per module. The `index.md` file lists all modules with links. Each
module file has a heading with the module path and the doc text.

### cart install

```
cart install <name>
```

The `cart install` command fetches a lib from the registry and installs
it in `~/.carts/<name>/`. It does not modify `Cart.toml`.

The command clones the lib repository into `~/.carts/<name>/`. If the
directory already exists, it pulls the latest changes instead.

### cart update

```
cart update
```

The `cart update` command updates all dependencies listed in `Cart.toml`
to the latest version from the registry. For each dependency, it runs a git
pull in `~/.carts/<name>/`. It then updates `Cart.lock` with the new
resolved SHA for each package.

## Cart.toml manifest

The `Cart.toml` file is the project manifest. It mirrors the `Cargo.toml`
structure.

### Full example

```toml
[package]
name = "nes-demo"
version = "0.1.0"
edition = "1"
authors = ["Dave Grantham <dwg@linuxprogrammer.org>"]
license = "Apache-2.0"

[lib]
name = "nes-demo-lib"
path = "src/lib.op"

[[rom]]
name = "nes-demo"
path = "src/cart.op"
target = "rp2A03-nintendo-nes-ntsc"

[dependencies]
mos6502 = "1.0"
nes = { version = "1.0", git = "https://github.com/op-language/nes" }
std = { path = "../std" }

[dev-dependencies]
test-utils = { version = "0.1", path = "../test-utils" }

[target]
default = "rp2A03-nintendo-nes-ntsc"

[features]
debug = []
undocumented = []

[[run.profile]]
name = "default"
emulator = "mesen"
args = ["--rom"]

[[run.profile]]
name = "debug"
emulator = "mesen"
args = ["--rom", "--debugger"]

[test]
profile = "test"

[test.sentinel.nes]
address = 0x6000
pass_value = 0xFF
```

### [package]

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Project name. |
| `version` | string | yes | Semantic version. |
| `edition` | string | no | Language edition. Default: `"1"`. |
| `authors` | list of strings | no | Author names. |
| `license` | string | no | SPDX license identifier. |

### [lib]

Defines a library (lib) target. A project may have at most one `[lib]`
section.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Lib name. |
| `path` | string | no | Root source file. Default: `src/lib.op`. |

### [[rom]]

Defines a binary (ROM) target. A project may have multiple `[[rom]]`
sections for different targets.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Binary name. |
| `path` | string | no | Root source file. Default: `src/cart.op`. |
| `target` | string | yes | Target triplet. |

### [dependencies]

Lists the libs that the project depends on. Each entry has one of these
forms:

```toml
[dependencies]
std = "1.0"
nes = { version = "1.0", git = "https://github.com/op-language/nes" }
my-lib = { path = "../my-lib" }
```

The simple form is a version requirement string:

```toml
std = "1.0"
```

The detailed form is a table with these fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | no | Version requirement string. |
| `git` | string | no | Git URL for the lib. |
| `branch` | string | no | Git branch. Default: the default branch. |
| `tag` | string | no | Git tag. |
| `rev` | string | no | Git commit SHA. |
| `path` | string | no | Local filesystem path. |
| `features` | list of strings | no | Feature flags to enable. |
| `optional` | boolean | no | Mark the dependency as optional. Default: `false`. |
| `default-features` | boolean | no | Enable default features. Default: `true`. |

A dependency must specify `version`, `git`, or `path`. If `git` is present
without `version`, the command uses the latest commit on the default
branch. If `path` is present, the command uses the lib at that path
without git operations.

### [dev-dependencies]

Same format as `[dependencies]`. The `cart test` command uses these
dependencies only for test builds.

### [target]

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `default` | string | no | Default target triplet. |

### [features]

Defines feature flags for conditional compilation. Each feature is a name
with a list of sub-features and optional dependencies.

```toml
[features]
debug = []
undocumented = []
audio = ["nes/audio"]
```

### [[run.profile]]

Defines a run profile for `cart run` and `cart test`. A project may have
multiple profiles.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Profile name. |
| `emulator` | string | yes | Emulator executable name. |
| `args` | list of strings | no | Arguments to pass to the emulator. |
| `target` | string | no | Override the target triplet for this profile. |

The profile with the name `default` is the default for `cart run`. The
`--profile` flag selects a profile by name.

### [test]

Configures the test harness for `cart test`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `profile` | string | no | Run profile name for the test emulator. Default: `test`. |
| `sentinel` | table | yes | Map from machine name to sentinel config. |

Each key in the `sentinel` table is a machine name. The value is a table
with `address` and `pass_value` fields.

### [doc]

Optional overrides for the documentation generator.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `output` | string | no | Output directory. Default: `target/doc`. |

## Cart.lock lockfile

The `Cart.lock` file records the exact version and source of each
dependency in the resolved graph. The `cart build` command writes it when
it is absent. The `cart build` command reads it when it is present. The
`cart update` command rewrites it after it pulls new versions.

### Format

The file uses TOML. It has a `version` field and a `[[package]]` array.

```toml
version = 1

[[package]]
name = "std"
version = "0.1.0"
source = { git = "https://github.com/op-language/std", sha = "abc123" }
checksum = "e3b0c44298fc1c149afbf4c8996fb924"

[[package]]
name = "nes"
version = "1.0.0"
source = { path = "../nes" }
checksum = "d41d8cd98f00b204e9800998ecf8427e"
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `version` | integer | Lockfile format version. Current: `1`. |
| `name` | string | Lib name. |
| `version` | string | Resolved lib version. |
| `source` | table | Source of the lib. |
| `checksum` | string | SHA-256 hash of the lib source tree. |

The `source` table has one of these forms:

- `{ git = "<url>", sha = "<sha>" }` for a git source.
- `{ path = "<dir>" }` for a path source.

### Resolution algorithm

1. Read `Cart.toml` dependencies.
2. For each dependency, locate it in `~/.carts/<name>/`. Clone it if it is
   missing. Use the `git` URL from `Cart.toml`.
3. Read the lib `Cart.toml` at `~/.carts/<name>/Cart.toml`. Get the lib
   version from the `[package]` section.
4. Match the version requirement from the project `Cart.toml` against the
   lib version. Use semver range matching.
5. Read the lib dependencies recursively. Resolve each one the same way.
6. Detect cycles. If a cycle exists, report error E504.
7. Build the resolved graph.
8. Compute the SHA-256 checksum for each package. Hash the file contents of
   the lib source tree.
9. Write `Cart.lock`.

### Frozen mode

If the user passes `--frozen`, the command reads `Cart.lock` and checks
that it matches the resolved graph. If the lockfile is out of date, the
command reports error E506 and exits. The command does not update the
lockfile in frozen mode.

## ~/.carts/ layout

Each lib lives in `~/.carts/<name>/`. The directory is a git clone of the
lib repository. It contains the lib `Cart.toml` and the `src/` directory.

```
~/.carts/
  std/
    Cart.toml
    src/
      lib.op
      cpu.op
      machine.op
      ...
  nes/
    Cart.toml
    src/
      lib.op
      ...
```

The `cart` tool never edits the contents of a lib directory. It only
clones and pulls. The `.git` directory stays in each lib directory so
that `cart update` can run git fetch and git checkout.

## ~/.cart/config.toml config

The `~/.cart/config.toml` file stores global configuration. The file is
optional. If it is absent, the `cart` tool uses built-in defaults.

```toml
[registry]
default-git-base = "https://github.com/op-language"

[build]
target = "rp2A03-nintendo-nes-ntsc"
opt-level = 1

[[run.profile]]
name = "default"
emulator = "mesen"
args = ["--rom"]

[[run.profile]]
name = "test"
emulator = "mesen"
args = ["--rom", "--dump"]

[test]
profile = "test"
```

### Sections

| Section | Description |
|---------|-------------|
| `[registry]` | Registry defaults. |
| `[build]` | Default build settings. |
| `[[run.profile]]` | Global run profiles. |
| `[test]` | Global test defaults. |

### [registry]

| Field | Type | Description |
|-------|------|-------------|
| `default-git-base` | string | Base URL for git clones. The command appends `/<name>` to form the full URL. |

### [build]

| Field | Type | Description |
|-------|------|-------------|
| `target` | string | Default target triplet. |
| `opt-level` | integer | Default optimization level. `0` or `1`. |

### Precedence

The `cart` tool applies settings in this order. A higher level overrides a
lower level.

1. Built-in defaults.
2. `~/.cart/config.toml`.
3. `Cart.toml`.
4. CLI flags.

## Dependency resolution

The resolver walks the dependency graph. It reads each lib `Cart.toml`,
matches version requirements, detects cycles, and builds a resolved graph.

### Algorithm

1. Read `Cart.toml` `[dependencies]` and `[dev-dependencies]`.
2. For each dependency entry:
   a. If the entry has `path`, resolve to the local path. Skip git
      operations.
   b. If the entry has `git`, clone the repository into
      `~/.carts/<name>/` if it is absent. If the directory exists, use it
      as-is.
   c. If the entry has only `version`, look for the lib in
      `~/.carts/<name>/`. If it is absent, clone from the registry default
      git base.
3. Read the lib `Cart.toml` at the resolved location. Get the lib
   version from `[package]` `version`.
4. Match the version requirement against the lib version. Use semver
   range matching. If the version does not match, report error E504.
5. Read the lib `[dependencies]` recursively. Resolve each one.
6. Detect cycles. Maintain a visited set during the walk. If the walk
   reaches a lib that is already in the visited set, report error E504.
7. Collect all resolved packages into a graph.
8. Compute the SHA-256 checksum for each package.
9. Write `Cart.lock`.

### Semver range matching

The version requirement string uses semver range syntax. The `semver`
crate parses the requirement and matches it against the lib version.

| Requirement | Matches |
|-------------|--------|
| `"1.0"` | `>=1.0.0, <2.0.0` |
| `"1.0.0"` | `>=1.0.0, <2.0.0` |
| `"=1.0.0"` | `==1.0.0` |
| `">=1.0"` | `>=1.0.0` |
| `"^1.0"` | `>=1.0.0, <2.0.0` |
| `"*"` | Any version. |

## Registry protocol

The registry uses git only. The `cart install <name>` command clones the
lib repository into `~/.carts/<name>/`.

### Install flow

1. Determine the git URL. If the dependency entry in `Cart.toml` has a
   `git` field, use that URL. If not, form the URL from the registry
   `default-git-base` and the lib name: `<base>/<name>`.
2. If `~/.carts/<name>/` exists, pull the latest changes with `git fetch`
   and `git checkout`.
3. If `~/.carts/<name>/` does not exist, clone the repository with `git
   clone <url> ~/.carts/<name>/`.
4. If the entry has `branch`, `tag`, or `rev`, checkout that ref after the
   clone or pull.
5. If the git operation fails, report error E510.

### Update flow

1. Read `Cart.toml` `[dependencies]`.
2. For each dependency with a git source, run `git fetch` and `git pull`
   in `~/.carts/<name>/`.
3. Re-resolve the dependency graph.
4. Rewrite `Cart.lock` with the new resolved SHAs.

A central registry index is deferred to future work. The language
specification lists this as a future item.

## Emulator launch

The `cart run` and `cart test` commands launch an emulator to run a ROM.

### Run flow

1. Build the project to get the ROM image.
2. Select the run profile from `[[run.profile]]` in `Cart.toml`. The
   `--profile` flag selects the profile. The default profile has the name
   `default`.
3. If the profile has a `target` field, override the target triplet.
4. Find the emulator executable on `PATH` with the `which` lookup.
5. If the emulator is absent, report error E501.
6. Build the argv. Start with the profile `args`. Append the ROM path.
7. Spawn the emulator with `std::process::Command`. Inherit the stdio.
8. Wait for the emulator to exit. Forward the exit code.

### Test flow

1. Build a test ROM for each file in `tests/*.op`.
2. Select the test profile. The `[test]` `profile` field names the
   profile. The default name is `test`.
3. Find the emulator on `PATH`.
4. Build the argv. Start with the profile `args`. Append the ROM path.
   Append `--dump <path>` where `<path>` is a temporary file for the
   memory dump.
5. Spawn the emulator. Wait for it to exit.
6. Read the memory dump file. The file is a flat binary.
7. Read the byte at the sentinel `address` for the target machine.
8. Compare the byte to the `pass_value`. If they match, the test passes.
   If they do not match, the test fails.
9. Report the result for each test file. Print a summary.
10. If any test fails, exit with status code 2.

## cart doc Markdown generation

The `cart doc` command walks the module tree and writes Markdown files.

### Module walk

1. Start at the root source file. For a lib project, the root is
   `src/lib.op`. For a ROM project, the root is the `[[rom]]` `path`.
2. Parse the file. Find all `///` and `//!` doc comments.
3. For each `//!` module doc comment, write a module preamble.
4. For each `///` doc comment, find the declaration that follows it. Write
   a Markdown heading with the declaration name and the doc text.
5. Find all `mod <name>;` declarations. For each one, resolve the file
   `<name>.op` in the same directory. Walk that file the same way.
6. Find all `use <path>;` declarations. For each one, add a cross-reference
   link to the target module file.

### Output tree

```
target/doc/<project>/
  index.md
  cpu.md
  machine.md
  machine/
    nes.md
    nes/
      const.md
      types.md
      macros.md
```

The `index.md` file lists all modules with links. Each module file has a
heading with the module path and the doc text from the `//!` comments. Each
declaration in the module has a sub-heading with its name and the doc text
from the `///` comments.

## Diagnostics

The `cart` tool uses the same diagnostic format as the `opc` compiler. A
diagnostic has a severity, a code, a file path, a line, a column, and a
message.

### Output format

```
error[E5xx]: message text
  --> file.op:line:col
   |
   | source line text
   |       ^^^^ hint text
   |
```

### Error codes

The `EXXX` code is a three-digit number. The first digit names the stage.
For `cart`, the first digit is `5`.

| Code | Description |
|------|-------------|
| E501 | Emulator not found. |
| E502 | Manifest parse error. |
| E503 | Triplet malformed. |
| E504 | Dependency resolution failure. |
| E505 | Bank not installed. |
| E506 | Lockfile out of date. |
| E507 | Build failure (opc returned an error). |
| E508 | Test failure (sentinel check failed). |
| E509 | Checksum mismatch. |
| E510 | Git operation failure. |

### Diagnostic reporting

The `cart` tool prints diagnostics to stderr. It uses the `Diagnostic`
and `Diagnostics` types from the `diagnostics` module. The
`Diagnostics` collection has an error limit. The default limit is 20. When
the error count reaches the limit, the tool stops and exits.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success. No errors. |
| 1 | Any error. |
| 2 | Test failure. One or more tests failed. |
| 3 | Lockfile frozen mismatch. The `--frozen` flag was set and `Cart.lock` is out of date. |

## Conformance

A conforming `cart` implementation must:

1. Implement all 10 subcommands: `init`, `build`, `run`, `test`, `check`,
   `clean`, `add`, `doc`, `install`, and `update`.
2. Read and write the `Cart.toml` format as this document defines.
3. Install libs in `~/.carts/` via git clone.
4. Resolve dependencies from `~/.carts/` before invoking `opc`.
5. Write and read the `Cart.lock` lockfile as this document defines.
6. Support the `~/.cart/config.toml` config file.
7. Support `[[run.profile]]` profiles in `Cart.toml`.
8. Run tests in an emulator and check a memory sentinel.
9. Generate Markdown documentation from doc comments.
10. Report errors with the structured diagnostic format.
11. Use the exit codes as this document defines.

## Future work

The following items are deferred. A future revision may define them.

1. A central registry index for libs beyond git URLs.
2. Language server protocol support for editor integration.
3. `cart publish` for publishing libs to a registry.
4. Workspace support for multi-project repositories.
5. Run profile inheritance from global config to project config.
6. A test framework DSL beyond the sentinel mechanism.
7. `cart vendor` for vendoring dependencies offline.
8. `cart fetch` for fetching dependencies without building.