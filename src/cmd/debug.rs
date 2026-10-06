//! `cart debug` — build the ROM and launch the demu socket server.
//!
//! demu serves the JSON-lines debug protocol on a Unix-domain socket.
//! The command builds the ROM with the opc stage output so that the
//! `.linked.opl` symbol table exists next to it, starts demu with the
//! socket server, prints the socket path, and waits until demu exits
//! (a user interrupt tears both processes down).

use crate::manifest::CartManifest;
use anyhow::Result;
use std::path::Path;

pub fn debug(manifest_path: &Path, target: Option<String>) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;

    let rom = manifest
        .rom
        .first()
        .ok_or_else(|| anyhow::anyhow!("E502: no ROM targets in Cart.toml"))?;

    let rom_target = target.unwrap_or_else(|| rom.target.clone());

    let rom_path = super::build::rom_output_path(
        &manifest,
        manifest_path,
        &rom_target,
        &rom.name,
        rom.format.as_deref(),
    );

    // A debug session needs fresh ROM and symbol files, so the build
    // always runs and always emits the stage files.
    super::build::build(
        manifest_path,
        Some(rom_target.clone()),
        false,
        false,
        Vec::new(),
        None,
        false,
        true,
    )?;
    let symbols_path = super::build::symbol_output_path(&rom_path);

    let socket_dir = manifest_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("target")
        .join(&rom_target);
    std::fs::create_dir_all(&socket_dir)?;
    let socket_path = socket_dir.join("debug.demu-sock");

    // The is_in_path lookup handles Windows PATH separators and the
    // .exe suffix; Command::new resolves the binary on PATH.
    if !crate::emulators::is_in_path("demu") {
        return Err(anyhow::anyhow!("E501: emulator 'demu' not found on PATH"));
    }

    eprintln!("Launching demu for {}...", rom.name);

    // Announce the socket path on stdout before demu starts, because
    // the wait below blocks until demu exits.
    println!("demu listening on {}", socket_path.display());

    let mut cmd = std::process::Command::new("demu");
    // The demu model arguments supply `--config <triplet>`; the debug
    // session loads the symbol table and serves the protocol on the
    // socket. The ROM path stays positional.
    for arg in crate::emulators::emulator_model_args(&rom_target, "demu") {
        cmd.arg(arg);
    }
    cmd.arg("--symbols").arg(&symbols_path);
    cmd.arg("--serve").arg(&socket_path);
    cmd.arg(&rom_path);

    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("E501: failed to launch emulator: {e}"))?;

    // The command waits until demu exits. A nonzero demu status is a failed
    // session; demu's own diagnostics are already on stderr.
    if !status.success() {
        let note = match status.code() {
            Some(code) => format!("demu exited with {code}"),
            None => "demu killed by signal".to_string(),
        };
        return Err(anyhow::anyhow!("E501: debug session ended: {note}"));
    }

    Ok(())
}
