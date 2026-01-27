use crate::Span;
use thiserror::Error;

/// Result type alias for IntentScript operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type for IntentScript
#[derive(Debug, Error, Clone, PartialEq)]
pub enum Error {
    #[error("Lexical error at {span:?}: {message}")]
    Lexical { span: Span, message: String },

    #[error("Parse error at {span:?}: {message}")]
    Parse { span: Span, message: String },

    #[error("Semantic error at {span:?}: {message}")]
    Semantic { span: Span, message: String },

    #[error("Type error at {span:?}: expected {expected}, found {found}")]
    Type {
        span: Span,
        expected: String,
        found: String,
    },

    #[error("Constraint error at {span:?}: {message}")]
    Constraint { span: Span, message: String },

    #[error("Policy violation at {span:?}: {message}")]
    PolicyViolation { span: Span, message: String },

    #[error("Lowering error: {message}")]
    Lowering { message: String },

    #[error("Runtime error: {message}")]
    Runtime { message: String },

    #[error("Capability violation: {message}")]
    CapabilityViolation { message: String },

    #[error("Host error: {message}")]
    Host { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Resource limit exceeded: {message}")]
    ResourceLimit { message: String },

    #[error("IO error: {0}")]
    Io(String),

    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn lexical(span: Span, message: impl Into<String>) -> Self {
        Self::Lexical {
            span,
            message: message.into(),
        }
    }

    pub fn parse(span: Span, message: impl Into<String>) -> Self {
        Self::Parse {
            span,
            message: message.into(),
        }
    }

    pub fn semantic(span: Span, message: impl Into<String>) -> Self {
        Self::Semantic {
            span,
            message: message.into(),
        }
    }

    pub fn type_error(span: Span, expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::Type {
            span,
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn constraint(span: Span, message: impl Into<String>) -> Self {
        Self::Constraint {
            span,
            message: message.into(),
        }
    }

    pub fn policy_violation(span: Span, message: impl Into<String>) -> Self {
        Self::PolicyViolation {
            span,
            message: message.into(),
        }
    }

    pub fn lowering(message: impl Into<String>) -> Self {
        Self::Lowering {
            message: message.into(),
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }

    pub fn capability_violation(message: impl Into<String>) -> Self {
        Self::CapabilityViolation {
            message: message.into(),
        }
    }

    pub fn host(message: impl Into<String>) -> Self {
        Self::Host {
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self::ResourceLimit {
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(format!("JSON error: {}", err))
    }
}
