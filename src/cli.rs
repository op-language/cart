//! Command-line interface for `cart`.
//!
//! Implements the subcommands: `init`, `build`, `run`, `test`, `check`,
//! `clean`, `add`, `doc`, `install`, and `update`.

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

use crate::cmd;

/// The Op build tool and package manager.
#[derive(Debug, Parser)]
#[command(name = "cart", version, about, long_about = None)]
pub struct CartArgs {
    /// Path to Cart.toml.
    #[arg(long, global = true)]
    pub manifest_path: Option<PathBuf>,

    /// Suppress non-error output.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Print extra diagnostic output.
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Color output: auto, always, never.
    #[arg(long, global = true)]
    pub color: Option<String>,

    /// Error if Cart.lock is out of date.
    #[arg(long, global = true)]
    pub frozen: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new Op project.
    Init {
        name: String,
        /// Create a library (bank) project with src/bank.op.
        #[arg(long)]
        bank: bool,
        /// Set the default target triplet in Cart.toml.
        #[arg(long)]
        target: Option<String>,
    },
    /// Build the project.
    Build {
        /// Override the target triplet.
        #[arg(long)]
        target: Option<String>,
        /// Build with optimization level 1.
        #[arg(long)]
        release: bool,
        /// Build with optimization level 0.
        #[arg(long)]
        debug: bool,
        /// Enable a feature flag.
        #[arg(long = "feature", action = ArgAction::Append)]
        features: Vec<String>,
        /// Override the output format.
        #[arg(long)]
        format: Option<String>,
    },
    /// Build the project and launch the ROM in the configured emulator.
    Run {
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        release: bool,
        /// Select a run profile by name.
        #[arg(long)]
        profile: Option<String>,
    },
    /// Run the project's test suite.
    Test {
        #[arg(long)]
        target: Option<String>,
    },
    /// Run the lexer and parser without generating code.
    Check {
        #[arg(long)]
        target: Option<String>,
    },
    /// Remove the build output directory.
    Clean,
    /// Add a bank to the Cart.toml dependencies.
    Add {
        bank: String,
        /// Git URL for the bank.
        #[arg(long)]
        git: Option<String>,
        /// Local path for the bank.
        #[arg(long)]
        path: Option<String>,
        /// Version requirement string.
        #[arg(long)]
        version: Option<String>,
    },
    /// Generate documentation from doc comments.
    Doc,
    /// Install a bank in ~/.carts/.
    Install {
        bank: String,
        /// Git URL for the bank.
        #[arg(long)]
        git: Option<String>,
    },
    /// Update all dependencies to the latest version.
    Update,
}

/// Entry point for the `cart` CLI.
pub fn run() -> Result<()> {
    let args = CartArgs::parse();
    let manifest_path = args
        .manifest_path
        .unwrap_or_else(|| std::path::PathBuf::from("Cart.toml"));

    match args.command {
        Command::Init { name, bank, target } => cmd::init::init(&name, bank, target),
        Command::Build {
            target,
            release,
            debug,
            features,
            format,
        } => cmd::build::build(
            &manifest_path,
            target,
            release,
            debug,
            features,
            format,
            args.frozen,
        ),
        Command::Run {
            target,
            release,
            profile,
        } => cmd::run::run(&manifest_path, target, release, profile),
        Command::Test { target } => cmd::test::test(&manifest_path, target),
        Command::Check { target } => cmd::check::check(&manifest_path, target),
        Command::Clean => cmd::clean::clean(&manifest_path),
        Command::Add {
            bank,
            git,
            path,
            version,
        } => cmd::add::add(&manifest_path, &bank, git, path, version),
        Command::Doc => cmd::doc::doc(&manifest_path),
        Command::Install { bank, git } => cmd::install::install(&bank, git),
        Command::Update => cmd::update::update(&manifest_path),
    }
}
