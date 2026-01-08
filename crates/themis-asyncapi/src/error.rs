//! Error types for `AsyncAPI` parsing and validation.

use std::fmt;

/// Errors that can occur during `AsyncAPI` parsing and validation.
#[derive(Debug, Clone)]
pub enum AsyncApiError {
    /// YAML parsing error
    YamlParse(String),
    /// JSON parsing error
    JsonParse(String),
    /// Missing required field
    MissingField(String),
    /// Invalid `AsyncAPI` version
    InvalidVersion(String),
    /// Invalid channel definition
    InvalidChannel(String),
    /// Invalid message definition
    InvalidMessage(String),
    /// Invalid operation definition
    InvalidOperation(String),
    /// Invalid schema definition
    InvalidSchema(String),
    /// Validation error
    Validation(String),
    /// Reference error
    InvalidReference(String),
}

impl std::error::Error for AsyncApiError {}

impl fmt::Display for AsyncApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::YamlParse(msg) => write!(f, "YAML parse error: {msg}"),
            Self::JsonParse(msg) => write!(f, "JSON parse error: {msg}"),
            Self::MissingField(field) => write!(f, "Missing required field: {field}"),
            Self::InvalidVersion(version) => {
                write!(f, "Invalid `AsyncAPI` version: {version}, expected 3.x.x")
            }
            Self::InvalidChannel(channel) => write!(f, "Invalid channel: {channel}"),
            Self::InvalidMessage(message) => write!(f, "Invalid message: {message}"),
            Self::InvalidOperation(operation) => write!(f, "Invalid operation: {operation}"),
            Self::InvalidSchema(schema) => write!(f, "Invalid schema: {schema}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::InvalidReference(reference) => write!(f, "Invalid reference: {reference}"),
        }
    }
}

impl From<serde_yaml::Error> for AsyncApiError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YamlParse(err.to_string())
    }
}

impl From<serde_json::Error> for AsyncApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParse(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_parse_error_display() {
        let err = AsyncApiError::YamlParse("unexpected token".to_string());
        assert!(err.to_string().contains("YAML parse error"));
    }

    #[test]
    fn test_missing_field_error_display() {
        let err = AsyncApiError::MissingField("asyncapi".to_string());
        assert!(err.to_string().contains("Missing required field"));
    }

    #[test]
    fn test_invalid_version_error_display() {
        let err = AsyncApiError::InvalidVersion("1.0.0".to_string());
        assert!(err.to_string().contains("Invalid `AsyncAPI` version"));
    }

    #[test]
    fn test_invalid_channel_error_display() {
        let err = AsyncApiError::InvalidChannel("bad-channel".to_string());
        assert!(err.to_string().contains("Invalid channel"));
    }
}
