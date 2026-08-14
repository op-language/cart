//! `opc` compiler invocation.
//!
//! The `cart` tool shells out to the `opc` compiler to compile projects.
//! This module builds the command-line arguments and invokes the process.

use std::path::PathBuf;
use std::process::Command;

/// Arguments passed to `opc`.
pub struct OpcArgs {
    pub input: PathBuf,
    pub target: String,
    pub features: Vec<String>,
    pub opt_level: u32,
    pub format: Option<String>,
    pub output: Option<PathBuf>,
    pub stage: OpcStage,
}

/// The pipeline stage to run.
#[derive(Debug, Clone, Copy)]
pub enum OpcStage {
    /// Run all stages and write the final ROM image.
    Full,
    /// Run the lexer and parser only. Writes a `.opa` file.
    Parse,
}

impl OpcArgs {
    /// Build a `Command` from these args.
    pub fn to_command(&self) -> Command {
        let mut cmd = Command::new("opc");
        cmd.arg("--target").arg(&self.target);
        for f in &self.features {
            cmd.arg("--feature").arg(f);
        }
        cmd.arg("-O").arg(self.opt_level.to_string());
        if let Some(fmt) = &self.format {
            cmd.arg("--format").arg(fmt);
        }
        if let Some(out) = &self.output {
            cmd.arg("-o").arg(out);
        }
        match self.stage {
            OpcStage::Full => {}
            OpcStage::Parse => {
                cmd.arg("--parse");
            }
        }
        cmd.arg(&self.input);
        cmd
    }
}

/// The result of an `opc` invocation.
pub struct OpcResult {
    pub success: bool,
    pub stderr: String,
}

/// Invoke `opc` with the given args. Returns the exit status and stderr
/// output.
pub fn invoke(args: &OpcArgs) -> anyhow::Result<OpcResult> {
    let mut cmd = args.to_command();
    cmd.stderr(std::process::Stdio::piped());
    let output = cmd
        .output()
        .map_err(|e| anyhow::anyhow!("E507: failed to invoke opc: {e}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    if !success {
        return Err(anyhow::anyhow!("E507: opc failed\n{stderr}"));
    }
    Ok(OpcResult { success, stderr })
}

/// Check if `opc` is available on PATH.
pub fn is_available() -> bool {
    std::process::Command::new("opc")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Get the output extension for a target output format.
pub fn output_extension(format: &str) -> &str {
    match format {
        "ines" => "nes",
        "lnx" => "lnx",
        "raw" => "bin",
        "hex" => "hex",
        _ => "bin",
    }
}
