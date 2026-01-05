//! Error types for code generation.

use thiserror::Error;

/// Result type for code generation operations.
pub type CodegenResult<T> = Result<T, CodegenError>;

/// Errors that can occur during code generation.
#[derive(Debug, Error)]
pub enum CodegenError {
    /// The contract is invalid or missing required information.
    #[error("Invalid contract: {reason}")]
    InvalidContract {
        /// Reason the contract is invalid.
        reason: String,
    },

    /// An unsupported schema type was encountered.
    #[error("Unsupported schema type: {schema_type}")]
    UnsupportedSchemaType {
        /// The schema type that is not supported.
        schema_type: String,
    },

    /// A circular reference was detected in the schema.
    #[error("Circular reference detected: {path}")]
    CircularReference {
        /// The path of the circular reference.
        path: String,
    },

    /// Failed to resolve a schema reference.
    #[error("Failed to resolve reference: {reference}")]
    UnresolvedReference {
        /// The reference that could not be resolved.
        reference: String,
    },

    /// Failed to write output file.
    #[error("Failed to write output: {path}")]
    WriteError {
        /// The path that could not be written.
        path: String,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// A general code generation error.
    #[error("Code generation error: {message}")]
    GenerationError {
        /// Description of the error.
        message: String,
    },
}

impl CodegenError {
    /// Creates an invalid contract error.
    pub fn invalid_contract(reason: impl Into<String>) -> Self {
        Self::InvalidContract {
            reason: reason.into(),
        }
    }

    /// Creates an unsupported schema type error.
    pub fn unsupported_schema(schema_type: impl Into<String>) -> Self {
        Self::UnsupportedSchemaType {
            schema_type: schema_type.into(),
        }
    }

    /// Creates a circular reference error.
    pub fn circular_reference(path: impl Into<String>) -> Self {
        Self::CircularReference { path: path.into() }
    }

    /// Creates an unresolved reference error.
    pub fn unresolved_reference(reference: impl Into<String>) -> Self {
        Self::UnresolvedReference {
            reference: reference.into(),
        }
    }

    /// Creates a generation error.
    pub fn generation_error(message: impl Into<String>) -> Self {
        Self::GenerationError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CodegenError::invalid_contract("missing operations");
        assert_eq!(err.to_string(), "Invalid contract: missing operations");

        let err = CodegenError::unsupported_schema("anyOf");
        assert_eq!(err.to_string(), "Unsupported schema type: anyOf");

        let err = CodegenError::circular_reference("User -> Address -> User");
        assert_eq!(
            err.to_string(),
            "Circular reference detected: User -> Address -> User"
        );
    }
}
