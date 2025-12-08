//! CLI command implementations

pub mod cat;
pub mod configure;
pub mod cp;
pub mod du;
pub mod head;
pub mod info;
pub mod ls;
pub mod mb;
pub mod mv;
pub mod presign;
pub mod rb;
pub mod rm;
pub mod sync;

use crate::config::Config;
use crate::OutputFormat;

/// Context passed to all commands
pub struct CommandContext {
    pub config: Config,
    pub output_format: OutputFormat,
    pub verbose: bool,
    pub quiet: bool,
}

impl CommandContext {
    /// Check if output should be JSON
    pub fn is_json(&self) -> bool {
        matches!(self.output_format, OutputFormat::Json)
    }

    /// Print info message if not quiet
    pub fn info(&self, msg: &str) {
        if !self.quiet {
            println!("{}", msg);
        }
    }

    /// Print verbose message if verbose mode
    pub fn debug(&self, msg: &str) {
        if self.verbose {
            eprintln!("[DEBUG] {}", msg);
        }
    }

    /// Print error message
    pub fn error(&self, msg: &str) {
        eprintln!("{}", msg);
    }
}
