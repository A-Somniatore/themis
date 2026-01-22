//! SDK error types.
//!
//! This module defines the error types used throughout the Themis SDK.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for SDK operations.
pub type SdkResult<T> = Result<T, SdkError>;

/// Errors that can occur during SDK operations.
#[derive(Debug, Error)]
pub enum SdkError {
    /// Error reading a file.
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        /// The path that failed to read.
        path: PathBuf,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Error writing a file.
    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        /// The path that failed to write.
        path: PathBuf,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Error parsing a contract.
    #[error("Failed to parse contract: {message}")]
    Parse {
        /// Description of the parse error.
        message: String,
    },

    /// Error validating a contract.
    #[error("Validation failed: {message}")]
    Validation {
        /// Description of the validation error.
        message: String,
    },

    /// Error during linting.
    #[error("Linting failed: {message}")]
    Lint {
        /// Description of the lint error.
        message: String,
    },

    /// Error during compatibility checking.
    #[error("Compatibility check failed: {message}")]
    Compatibility {
        /// Description of the compatibility error.
        message: String,
    },

    /// Error during code generation.
    #[error("Code generation failed: {message}")]
    CodeGen {
        /// Description of the code generation error.
        message: String,
    },

    /// Error during artifact creation.
    #[error("Artifact creation failed: {message}")]
    Artifact {
        /// Description of the artifact error.
        message: String,
    },

    /// Unsupported contract format.
    #[error("Unsupported contract format: {format}")]
    UnsupportedFormat {
        /// The unsupported format.
        format: String,
    },

    /// Unsupported language.
    #[error("Unsupported language: {language}")]
    UnsupportedLanguage {
        /// The unsupported language.
        language: String,
    },

    /// Configuration error.
    #[error("Configuration error: {message}")]
    Config {
        /// Description of the configuration error.
        message: String,
    },
}

impl From<themis_core::ThemisError> for SdkError {
    fn from(err: themis_core::ThemisError) -> Self {
        Self::Parse {
            message: err.to_string(),
        }
    }
}

impl From<themis_codegen::CodegenError> for SdkError {
    fn from(err: themis_codegen::CodegenError) -> Self {
        Self::CodeGen {
            message: err.to_string(),
        }
    }
}

impl From<themis_artifact::ArtifactError> for SdkError {
    fn from(err: themis_artifact::ArtifactError) -> Self {
        Self::Artifact {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SdkError::Parse {
            message: "invalid syntax".to_string(),
        };
        assert_eq!(err.to_string(), "Failed to parse contract: invalid syntax");
    }

    #[test]
    fn test_file_read_error() {
        let err = SdkError::FileRead {
            path: PathBuf::from("/tmp/test.yaml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert!(err.to_string().contains("/tmp/test.yaml"));
    }

    #[test]
    fn test_unsupported_format_error() {
        let err = SdkError::UnsupportedFormat {
            format: "raml".to_string(),
        };
        assert_eq!(err.to_string(), "Unsupported contract format: raml");
    }

    #[test]
    fn test_unsupported_language_error() {
        let err = SdkError::UnsupportedLanguage {
            language: "java".to_string(),
        };
        assert_eq!(err.to_string(), "Unsupported language: java");
    }
}
