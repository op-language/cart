//! Diagnostic reporting for the `cart` build tool.
//!
//! A [`Diagnostic`] has a severity, a file path, a line, a column, a
//! message, and an error code. The `cart` tool prints diagnostics to stderr
//! in the structured format.
//!
//! The `EXXX` code is a three-digit number. The first digit names the stage:
//! 5 = cart.

use serde::{Deserialize, Serialize};

/// Error codes for the `cart` build tool.
pub mod codes {
    pub const EMULATOR_NOT_FOUND: u32 = 501;
    pub const MANIFEST_PARSE: u32 = 502;
    pub const TRIPLET_MALFORMED: u32 = 503;
    pub const DEP_RESOLUTION: u32 = 504;
    pub const LIB_NOT_INSTALLED: u32 = 505;
    pub const LOCKFILE_OUT_OF_DATE: u32 = 506;
    pub const BUILD_FAILURE: u32 = 507;
    pub const TEST_FAILURE: u32 = 508;
    pub const CHECKSUM_MISMATCH: u32 = 509;
    pub const GIT_FAILURE: u32 = 510;
}

/// The severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// A diagnostic message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: u32,
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub message: String,
}

impl Diagnostic {
    pub fn error(code: u32, file: &str, line: u32, col: u32, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            file: file.to_string(),
            line,
            col,
            message: message.into(),
        }
    }

    pub fn warning(code: u32, file: &str, line: u32, col: u32, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            file: file.to_string(),
            line,
            col,
            message: message.into(),
        }
    }

    /// Print the diagnostic to stderr in the structured format.
    pub fn print(&self, source: Option<&str>) {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
        eprintln!("{}[E{:03}]: {}", level, self.code, self.message);
        eprintln!("  --> {}:{}:{}", self.file, self.line, self.col);
        eprintln!("   |");
        if let Some(line) = source {
            eprintln!("   | {}", line);
            eprintln!("   |");
        }
    }
}

/// A collection of diagnostics with an error count.
#[derive(Debug, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
    pub error_count: usize,
    pub error_limit: usize,
}

impl Diagnostics {
    pub fn new(error_limit: usize) -> Self {
        Self {
            items: Vec::new(),
            error_count: 0,
            error_limit,
        }
    }

    /// Push a diagnostic. Returns `true` if the error limit was reached.
    pub fn push(&mut self, diag: Diagnostic) -> bool {
        let is_error = diag.severity == Severity::Error;
        self.items.push(diag);
        if is_error {
            self.error_count += 1;
            self.error_count >= self.error_limit
        } else {
            false
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn print_all(&self, sources: &std::collections::HashMap<String, String>) {
        for diag in &self.items {
            let source = sources.get(&diag.file).map(|s| s.as_str());
            diag.print(source);
        }
    }
}
