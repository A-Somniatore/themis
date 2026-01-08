//! Error types for Protobuf parsing.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for protobuf operations.
pub type Result<T> = std::result::Result<T, ProtobufError>;

/// Errors that can occur during Protobuf parsing and normalization.
#[derive(Debug, Error)]
pub enum ProtobufError {
    /// Failed to read the proto file from disk.
    #[error("Failed to read proto file '{path}': {source}")]
    ReadError {
        /// Path to the file that couldn't be read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse the protobuf syntax.
    #[error("Failed to parse protobuf: {message}")]
    ParseError {
        /// Description of the parse error.
        message: String,
    },

    /// Missing required field in the protobuf definition.
    #[error("Missing required field: {field}")]
    MissingField {
        /// Name of the missing field.
        field: String,
    },

    /// Invalid protobuf syntax.
    #[error("Invalid syntax: {reason}")]
    InvalidSyntax {
        /// Description of the syntax error.
        reason: String,
    },

    /// Service definition is missing.
    #[error("No service definition found in proto file")]
    NoServiceFound,

    /// Invalid version format.
    #[error("Invalid version format: {version}")]
    InvalidVersion {
        /// The invalid version string.
        version: String,
    },

    /// Unsupported protobuf feature.
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature {
        /// Description of the unsupported feature.
        feature: String,
    },

    /// Invalid field type.
    #[error("Invalid field type: {field_type}")]
    InvalidFieldType {
        /// The invalid field type string.
        field_type: String,
    },

    /// Circular reference detected.
    #[error("Circular reference detected: {path}")]
    CircularReference {
        /// The reference path that forms a cycle.
        path: String,
    },

    /// Import resolution failed.
    #[error("Failed to resolve import '{import}': {reason}")]
    ImportError {
        /// The import path that couldn't be resolved.
        import: String,
        /// Reason for the failure.
        reason: String,
    },
}

impl ProtobufError {
    /// Creates a parse error with the given message.
    pub fn parse<S: Into<String>>(message: S) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Creates a missing field error.
    pub fn missing_field<S: Into<String>>(field: S) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Creates an invalid syntax error.
    pub fn invalid_syntax<S: Into<String>>(reason: S) -> Self {
        Self::InvalidSyntax {
            reason: reason.into(),
        }
    }

    /// Creates an unsupported feature error.
    pub fn unsupported<S: Into<String>>(feature: S) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ProtobufError::parse("unexpected token");
        assert_eq!(
            err.to_string(),
            "Failed to parse protobuf: unexpected token"
        );
    }

    #[test]
    fn test_missing_field_error() {
        let err = ProtobufError::missing_field("service_name");
        assert_eq!(err.to_string(), "Missing required field: service_name");
    }

    #[test]
    fn test_invalid_syntax_error() {
        let err = ProtobufError::invalid_syntax("expected semicolon");
        assert_eq!(err.to_string(), "Invalid syntax: expected semicolon");
    }

    #[test]
    fn test_no_service_error() {
        let err = ProtobufError::NoServiceFound;
        assert_eq!(err.to_string(), "No service definition found in proto file");
    }
}
