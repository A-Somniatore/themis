//! Error types for artifact operations.

use thiserror::Error;

/// Result type for artifact operations.
pub type ArtifactResult<T> = Result<T, ArtifactError>;

/// Errors that can occur during artifact operations.
#[derive(Debug, Error)]
pub enum ArtifactError {
    /// The artifact checksum does not match.
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Expected checksum value.
        expected: String,
        /// Actual computed checksum.
        actual: String,
    },

    /// Missing required field during build.
    #[error("Missing required field: {field}")]
    MissingField {
        /// The missing field name.
        field: String,
    },

    /// Invalid artifact format.
    #[error("Invalid artifact format: {reason}")]
    InvalidFormat {
        /// Reason the format is invalid.
        reason: String,
    },

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Version conflict.
    #[error("Version {version} already exists for service {service}")]
    VersionConflict {
        /// Service name.
        service: String,
        /// Conflicting version.
        version: String,
    },

    /// Artifact not found.
    #[error("Artifact not found: {service}@{version}")]
    NotFound {
        /// Service name.
        service: String,
        /// Version.
        version: String,
    },
}

impl ArtifactError {
    /// Creates a missing field error.
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Creates an invalid format error.
    pub fn invalid_format(reason: impl Into<String>) -> Self {
        Self::InvalidFormat {
            reason: reason.into(),
        }
    }

    /// Creates a checksum mismatch error.
    pub fn checksum_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::ChecksumMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ArtifactError::missing_field("contract");
        assert_eq!(err.to_string(), "Missing required field: contract");

        let err = ArtifactError::invalid_format("invalid JSON");
        assert_eq!(err.to_string(), "Invalid artifact format: invalid JSON");

        let err = ArtifactError::checksum_mismatch("abc", "def");
        assert_eq!(err.to_string(), "Checksum mismatch: expected abc, got def");
    }

    #[test]
    fn test_version_conflict_error() {
        let err = ArtifactError::VersionConflict {
            service: "users-service".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Version 1.0.0 already exists for service users-service"
        );
    }
}
