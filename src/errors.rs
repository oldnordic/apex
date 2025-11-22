//! APEX Error Types
//!
//! Unified error handling across parse, validate, and interpret phases.

use std::fmt;

/// Error kind categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApexErrorKind {
    /// Lexer encountered invalid token
    LexError,
    /// Parser encountered unexpected structure
    ParseError,
    /// TASK block missing (required)
    MissingTask,
    /// Multiple TASK blocks found
    MultipleTasks,
    /// Required block is empty
    EmptyRequiredBlock,
    /// Unknown block identifier
    UnknownBlock,
    /// Invalid tool name syntax
    InvalidToolName,
    /// Constraint violation during execution
    ConstraintViolation,
    /// Validation condition failed
    ValidationFailure,
    /// Internal error (should not happen)
    InternalError,
}

impl fmt::Display for ApexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApexErrorKind::LexError => write!(f, "LexError"),
            ApexErrorKind::ParseError => write!(f, "ParseError"),
            ApexErrorKind::MissingTask => write!(f, "MissingTask"),
            ApexErrorKind::MultipleTasks => write!(f, "MultipleTasks"),
            ApexErrorKind::EmptyRequiredBlock => write!(f, "EmptyRequiredBlock"),
            ApexErrorKind::UnknownBlock => write!(f, "UnknownBlock"),
            ApexErrorKind::InvalidToolName => write!(f, "InvalidToolName"),
            ApexErrorKind::ConstraintViolation => write!(f, "ConstraintViolation"),
            ApexErrorKind::ValidationFailure => write!(f, "ValidationFailure"),
            ApexErrorKind::InternalError => write!(f, "InternalError"),
        }
    }
}

/// APEX error with context
#[derive(Debug, Clone)]
pub struct ApexError {
    /// Error category
    pub kind: ApexErrorKind,
    /// Human-readable message
    pub message: String,
    /// Line number where error occurred (1-indexed)
    pub line: Option<usize>,
    /// Column number (1-indexed)
    pub column: Option<usize>,
}

impl ApexError {
    /// Create a new error
    pub fn new(kind: ApexErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            line: None,
            column: None,
        }
    }

    /// Create error with line context
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Create error with column context
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    // --- Convenience constructors ---

    /// Parse error at optional line
    pub fn parse(msg: impl Into<String>, line: Option<usize>) -> Self {
        let mut err = Self::new(ApexErrorKind::ParseError, msg);
        err.line = line;
        err
    }

    /// Lex error at optional line
    pub fn lex(msg: impl Into<String>, line: Option<usize>) -> Self {
        let mut err = Self::new(ApexErrorKind::LexError, msg);
        err.line = line;
        err
    }

    /// Missing TASK block
    pub fn missing_task() -> Self {
        Self::new(ApexErrorKind::MissingTask, "APEX document must contain exactly one TASK block")
    }

    /// Multiple TASK blocks
    pub fn multiple_tasks(line: usize) -> Self {
        Self::new(ApexErrorKind::MultipleTasks, "APEX document contains multiple TASK blocks")
            .with_line(line)
    }

    /// Empty required block
    pub fn empty_block(name: &str, line: Option<usize>) -> Self {
        let mut err = Self::new(
            ApexErrorKind::EmptyRequiredBlock,
            format!("{} block cannot be empty", name),
        );
        err.line = line;
        err
    }

    /// Unknown block identifier
    pub fn unknown_block(name: &str, line: Option<usize>) -> Self {
        let mut err = Self::new(
            ApexErrorKind::UnknownBlock,
            format!("Unknown block identifier: {}", name),
        );
        err.line = line;
        err
    }

    /// Constraint violation
    pub fn constraint_violation(constraint: &str, reason: &str) -> Self {
        Self::new(
            ApexErrorKind::ConstraintViolation,
            format!("Constraint '{}' violated: {}", constraint, reason),
        )
    }

    /// Validation failure
    pub fn validation_failure(condition: &str) -> Self {
        Self::new(
            ApexErrorKind::ValidationFailure,
            format!("Validation failed: {}", condition),
        )
    }
}

impl fmt::Display for ApexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)?;
        if let Some(line) = self.line {
            write!(f, " (line {})", line)?;
        }
        Ok(())
    }
}

impl std::error::Error for ApexError {}

/// Result type alias for APEX operations
pub type ApexResult<T> = Result<T, ApexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ApexError::missing_task();
        assert!(err.to_string().contains("MissingTask"));
        assert!(err.to_string().contains("TASK block"));
    }

    #[test]
    fn test_error_with_line() {
        let err = ApexError::parse("unexpected token", Some(42));
        assert_eq!(err.line, Some(42));
        assert!(err.to_string().contains("line 42"));
    }
}
