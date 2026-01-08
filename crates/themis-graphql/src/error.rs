//! Error types for GraphQL parsing and validation.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for GraphQL operations.
pub type Result<T> = std::result::Result<T, GraphqlError>;

/// Errors that can occur during GraphQL parsing and validation.
#[derive(Error, Debug)]
pub enum GraphqlError {
    /// Error reading a file.
    #[error("Failed to read GraphQL file from {path}: {source}")]
    ReadError {
        /// Path to the file.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Error parsing GraphQL SDL.
    #[error("Failed to parse GraphQL SDL: {message}")]
    ParseError {
        /// Description of the parse error.
        message: String,
    },

    /// Missing required field or definition.
    #[error("Missing required definition: {field}")]
    MissingField {
        /// Name of the missing field or definition.
        field: String,
    },

    /// Invalid GraphQL syntax.
    #[error("Invalid GraphQL syntax at {location}: {reason}")]
    InvalidSyntax {
        /// Location of the error.
        location: String,
        /// Reason for the error.
        reason: String,
    },

    /// No Query type found.
    #[error("No Query type defined in schema")]
    NoQueryType,

    /// Invalid version string.
    #[error("Invalid version: {version}")]
    InvalidVersion {
        /// The invalid version string.
        version: String,
    },

    /// Unsupported feature.
    #[error("Unsupported GraphQL feature: {feature}")]
    UnsupportedFeature {
        /// Name of the unsupported feature.
        feature: String,
    },

    /// Invalid type reference.
    #[error("Invalid type reference: {type_name}")]
    InvalidTypeRef {
        /// Name of the invalid type.
        type_name: String,
    },

    /// Circular type reference.
    #[error("Circular type reference detected: {type_name}")]
    CircularReference {
        /// Name of the type with circular reference.
        type_name: String,
    },

    /// Directive error.
    #[error("Directive error: {message}")]
    DirectiveError {
        /// Description of the directive error.
        message: String,
    },
}

impl From<graphql_parser::query::ParseError> for GraphqlError {
    fn from(err: graphql_parser::query::ParseError) -> Self {
        Self::ParseError {
            message: err.to_string(),
        }
    }
}

impl From<graphql_parser::schema::ParseError> for GraphqlError {
    fn from(err: graphql_parser::schema::ParseError) -> Self {
        Self::ParseError {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GraphqlError::ParseError {
            message: "unexpected token".to_string(),
        };
        assert!(err.to_string().contains("unexpected token"));
    }

    #[test]
    fn test_no_query_error() {
        let err = GraphqlError::NoQueryType;
        assert!(err.to_string().contains("No Query type"));
    }

    #[test]
    fn test_missing_field_error() {
        let err = GraphqlError::MissingField {
            field: "Query".to_string(),
        };
        assert!(err.to_string().contains("Query"));
    }

    #[test]
    fn test_invalid_syntax_error() {
        let err = GraphqlError::InvalidSyntax {
            location: "line 10".to_string(),
            reason: "unexpected }".to_string(),
        };
        assert!(err.to_string().contains("line 10"));
        assert!(err.to_string().contains("unexpected }"));
    }
}
