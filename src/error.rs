//! Error types for the t interpreter.

#![allow(dead_code)] // Methods reserved for future use

use std::fmt;

/// Position information for error reporting.
#[derive(Debug, Clone, Default)]
pub struct Position {
    /// Byte offset in the source programme.
    pub source_pos: Option<usize>,
    /// Line number in the input (1-based).
    pub input_line: Option<usize>,
}

impl Position {
    /// Create a position with source location.
    pub fn at_source(pos: usize) -> Self {
        Self {
            source_pos: Some(pos),
            input_line: None,
        }
    }

    /// Create a position with input line.
    pub fn at_input(line: usize) -> Self {
        Self {
            source_pos: None,
            input_line: Some(line),
        }
    }
}

/// Errors that can occur during interpretation.
#[derive(Debug)]
pub struct Error {
    /// The error message.
    pub message: String,
    /// Position information for the error.
    pub position: Position,
}

impl Error {
    /// Create a runtime error with just a message.
    pub fn runtime(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: Position::default(),
        }
    }

    /// Create a runtime error at a source position.
    pub fn at_source(message: impl Into<String>, pos: usize) -> Self {
        Self {
            message: message.into(),
            position: Position::at_source(pos),
        }
    }

    /// Create a runtime error at an input line.
    pub fn at_input(message: impl Into<String>, line: usize) -> Self {
        Self {
            message: message.into(),
            position: Position::at_input(line),
        }
    }

    /// Add source position to an existing error.
    pub fn with_source_pos(mut self, pos: usize) -> Self {
        self.position.source_pos = Some(pos);
        self
    }

    /// Add input line to an existing error.
    pub fn with_input_line(mut self, line: usize) -> Self {
        self.position.input_line = Some(line);
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        match (self.position.source_pos, self.position.input_line) {
            (Some(pos), Some(line)) => {
                write!(f, " (at position {}, input line {})", pos, line)
            }
            (Some(pos), None) => write!(f, " (at position {})", pos),
            (None, Some(line)) => write!(f, " (at input line {})", line),
            (None, None) => Ok(()),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for interpreter operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_error_display() {
        let err = Error::runtime("something went wrong");
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn error_with_source_pos() {
        let err = Error::at_source("unexpected token", 42);
        assert_eq!(err.to_string(), "unexpected token (at position 42)");
    }

    #[test]
    fn error_with_input_line() {
        let err = Error::at_input("invalid value", 10);
        assert_eq!(err.to_string(), "invalid value (at input line 10)");
    }

    #[test]
    fn error_with_both_positions() {
        let err = Error::runtime("type mismatch")
            .with_source_pos(5)
            .with_input_line(100);
        assert_eq!(
            err.to_string(),
            "type mismatch (at position 5, input line 100)"
        );
    }
}
