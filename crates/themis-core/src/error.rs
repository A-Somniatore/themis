//! Error types for Themis.
//!
//! Provides a standardized error model used across all Themis components.
//!
//! This module provides two types of errors:
//! - [`ThemisError`] - Internal errors for Themis toolchain operations
//! - [`ThemisErrorEnvelope`] - Standard API error format (re-exported from shared types)
//!
//! ## Usage
//!
//! For CLI and toolchain errors, use `ThemisError`:
//! ```rust
//! use themis_core::error::ThemisError;
//!
//! fn validate_contract() -> Result<(), ThemisError> {
//!     Err(ThemisError::SchemaValidation {
//!         message: "Missing operationId".to_string(),
//!     })
//! }
//! ```
//!
//! For API responses, use the shared `ThemisErrorEnvelope`:
//! ```rust
//! use themis_core::error::ThemisErrorEnvelope;
//!
//! let error = ThemisErrorEnvelope::validation_failed("Invalid request");
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// Re-export shared error types from themis-platform-types
pub use themis_platform_types::{ErrorCode, FieldError, ThemisErrorEnvelope};

/// Result type alias using [`ThemisError`].
pub type ThemisResult<T> = Result<T, ThemisError>;

/// The main error type for Themis operations.
#[derive(Error, Debug)]
pub enum ThemisError {
    /// Failed to read a file.
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// Underlying IO error
        #[source]
        source: std::io::Error,
    },

    /// Failed to write a file.
    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        /// Path to the file that couldn't be written
        path: PathBuf,
        /// Underlying IO error
        #[source]
        source: std::io::Error,
    },

    /// Invalid YAML syntax.
    #[error("Invalid YAML in '{path}': {message}")]
    YamlParse {
        /// Path to the file with invalid YAML
        path: PathBuf,
        /// Error message
        message: String,
    },

    /// Invalid JSON syntax.
    #[error("Invalid JSON in '{path}': {message}")]
    JsonParse {
        /// Path to the file with invalid JSON
        path: PathBuf,
        /// Error message
        message: String,
    },

    /// Invalid contract schema.
    #[error("Invalid contract schema: {message}")]
    SchemaValidation {
        /// Description of the validation error
        message: String,
    },

    /// Invalid version string.
    #[error("Invalid version '{version}': {reason}")]
    InvalidVersion {
        /// The invalid version string
        version: String,
        /// Why it's invalid
        reason: String,
    },

    /// Missing required field in contract.
    #[error("Missing required field '{field}' in {context}")]
    MissingField {
        /// The missing field name
        field: String,
        /// Where the field was expected
        context: String,
    },

    /// Invalid operation definition.
    #[error("Invalid operation '{operation_id}': {message}")]
    InvalidOperation {
        /// The operation ID
        operation_id: String,
        /// Description of the issue
        message: String,
    },

    /// Unresolved reference.
    #[error("Unresolved reference '{reference}' in {context}")]
    UnresolvedReference {
        /// The unresolved reference (e.g., "$ref" value)
        reference: String,
        /// Where the reference was found
        context: String,
    },

    /// Breaking change detected.
    #[error("Breaking change detected: {message}")]
    BreakingChange {
        /// Description of the breaking change
        message: String,
    },

    /// Lint rule violation.
    #[error("Lint error [{rule}]: {message}")]
    LintViolation {
        /// The rule that was violated
        rule: String,
        /// Description of the violation
        message: String,
    },

    /// Code generation error.
    #[error("Code generation failed for {language}: {message}")]
    CodeGeneration {
        /// Target language
        language: String,
        /// Error description
        message: String,
    },

    /// Registry operation failed.
    #[error("Registry error: {message}")]
    Registry {
        /// Error description
        message: String,
    },

    /// Generic internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ThemisError {
    /// Returns the error code for this error variant.
    ///
    /// These codes can be used for programmatic error handling.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileRead { .. } => "FILE_READ_ERROR",
            Self::FileWrite { .. } => "FILE_WRITE_ERROR",
            Self::YamlParse { .. } => "YAML_PARSE_ERROR",
            Self::JsonParse { .. } => "JSON_PARSE_ERROR",
            Self::SchemaValidation { .. } => "SCHEMA_VALIDATION_ERROR",
            Self::InvalidVersion { .. } => "INVALID_VERSION",
            Self::MissingField { .. } => "MISSING_FIELD",
            Self::InvalidOperation { .. } => "INVALID_OPERATION",
            Self::UnresolvedReference { .. } => "UNRESOLVED_REFERENCE",
            Self::BreakingChange { .. } => "BREAKING_CHANGE",
            Self::LintViolation { .. } => "LINT_VIOLATION",
            Self::CodeGeneration { .. } => "CODE_GENERATION_ERROR",
            Self::Registry { .. } => "REGISTRY_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Returns true if this error is recoverable.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::LintViolation { .. } | Self::BreakingChange { .. }
        )
    }
}

/// A serializable error response following Themis conventions.
///
/// This is the standard error format returned by Themis-governed APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Machine-readable error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Request ID for tracing (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Trace ID for distributed tracing (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Creates a new error response.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            request_id: None,
            trace_id: None,
            details: None,
        }
    }

    /// Adds a request ID to the error response.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Adds a trace ID to the error response.
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Adds details to the error response.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl From<&ThemisError> for ErrorResponse {
    fn from(err: &ThemisError) -> Self {
        Self::new(err.code(), err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        let err = ThemisError::InvalidVersion {
            version: "bad".to_string(),
            reason: "test".to_string(),
        };
        assert_eq!(err.code(), "INVALID_VERSION");
    }

    #[test]
    fn test_error_display() {
        let err = ThemisError::MissingField {
            field: "operationId".to_string(),
            context: "GET /users".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Missing required field 'operationId' in GET /users"
        );
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse::new("TEST_ERROR", "Something went wrong")
            .with_request_id("req-123")
            .with_details(serde_json::json!({"field": "value"}));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("TEST_ERROR"));
        assert!(json.contains("req-123"));
    }

    #[test]
    fn test_error_response_from_themis_error() {
        let err = ThemisError::BreakingChange {
            message: "Field removed".to_string(),
        };
        let response = ErrorResponse::from(&err);
        assert_eq!(response.code, "BREAKING_CHANGE");
    }

    #[test]
    fn test_error_is_recoverable() {
        let lint_err = ThemisError::LintViolation {
            rule: "naming".to_string(),
            message: "test".to_string(),
        };
        assert!(lint_err.is_recoverable());

        let read_err = ThemisError::Internal("test".to_string());
        assert!(!read_err.is_recoverable());
    }
}
