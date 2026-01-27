use intentscript_core::Error as CoreError;
use std::fmt;
use std::io;

/// CLI-specific error type
#[derive(Debug)]
pub enum CliError {
    /// IO error
    Io(io::Error),
    /// Compiler error
    Compiler(Vec<CoreError>),
    /// Runtime error
    Runtime(CoreError),
    /// Invalid input
    InvalidInput(String),
    /// Other error
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "IO error: {}", e),
            CliError::Compiler(errors) => {
                writeln!(f, "Compilation failed with {} error(s):", errors.len())?;
                for error in errors {
                    writeln!(f, "  {}", error)?;
                }
                Ok(())
            }
            CliError::Runtime(e) => write!(f, "Runtime error: {}", e),
            CliError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<CoreError> for CliError {
    fn from(err: CoreError) -> Self {
        CliError::Runtime(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Other(format!("JSON error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
