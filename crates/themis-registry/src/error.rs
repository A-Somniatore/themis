//! Registry error types.

use std::path::PathBuf;
use thiserror::Error;

/// Registry operation result.
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Registry errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// Failed to create HTTP client.
    #[error("failed to create HTTP client: {0}")]
    HttpClientError(String),

    /// The artifact was not found in the registry.
    #[error("artifact not found: {service}@{version}")]
    NotFound {
        /// Service name.
        service: String,
        /// Version.
        version: String,
    },

    /// The artifact already exists in the registry.
    #[error("artifact already exists: {service}@{version}")]
    AlreadyExists {
        /// Service name.
        service: String,
        /// Version.
        version: String,
    },

    /// Authentication failed.
    #[error("authentication failed: {message}")]
    AuthenticationFailed {
        /// Error message.
        message: String,
    },

    /// Authorization failed (no permission).
    #[error("authorization failed: {message}")]
    AuthorizationFailed {
        /// Error message.
        message: String,
    },

    /// Invalid artifact reference.
    #[error("invalid artifact reference: {reference}")]
    InvalidReference {
        /// The invalid reference.
        reference: String,
        /// Reason for invalidity.
        reason: String,
    },

    /// Registry returned an error.
    #[error("registry error: {status} - {message}")]
    RegistryError {
        /// HTTP status code.
        status: u16,
        /// Error message.
        message: String,
    },

    /// Network error.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Checksum verification failed.
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Expected checksum.
        expected: String,
        /// Actual checksum.
        actual: String,
    },

    /// Invalid manifest.
    #[error("invalid OCI manifest: {0}")]
    InvalidManifest(String),

    /// Cache error.
    #[error("cache error: {0}")]
    CacheError(String),

    /// IO error.
    #[error("IO error at {path}: {message}")]
    IoError {
        /// Path that caused the error.
        path: PathBuf,
        /// Error message.
        message: String,
    },

    /// Serialization error.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// The registry does not support the required capabilities.
    #[error("registry does not support {capability}")]
    UnsupportedCapability {
        /// The unsupported capability.
        capability: String,
    },

    /// Rate limited.
    #[error("rate limited: retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },
}

impl RegistryError {
    /// Creates a `NotFound` error.
    pub fn not_found(service: impl Into<String>, version: impl Into<String>) -> Self {
        Self::NotFound {
            service: service.into(),
            version: version.into(),
        }
    }

    /// Creates an `AlreadyExists` error.
    pub fn already_exists(service: impl Into<String>, version: impl Into<String>) -> Self {
        Self::AlreadyExists {
            service: service.into(),
            version: version.into(),
        }
    }

    /// Creates an `AuthenticationFailed` error.
    pub fn auth_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
        }
    }

    /// Creates an `AuthorizationFailed` error.
    pub fn authorization_failed(message: impl Into<String>) -> Self {
        Self::AuthorizationFailed {
            message: message.into(),
        }
    }

    /// Creates an `InvalidReference` error.
    pub fn invalid_reference(reference: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidReference {
            reference: reference.into(),
            reason: reason.into(),
        }
    }

    /// Creates a `RegistryError`.
    pub fn registry_error(status: u16, message: impl Into<String>) -> Self {
        Self::RegistryError {
            status,
            message: message.into(),
        }
    }

    /// Creates a `ChecksumMismatch` error.
    pub fn checksum_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::ChecksumMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Creates an `IoError`.
    pub fn io_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::IoError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Returns true if this error indicates the artifact was not found.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Returns true if this error indicates the artifact already exists.
    #[must_use]
    pub const fn is_already_exists(&self) -> bool {
        matches!(self, Self::AlreadyExists { .. })
    }

    /// Returns true if this is an authentication error.
    #[must_use]
    pub const fn is_auth_error(&self) -> bool {
        matches!(
            self,
            Self::AuthenticationFailed { .. } | Self::AuthorizationFailed { .. }
        )
    }

    /// Returns true if this is a retryable error.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::NetworkError(_) | Self::RateLimited { .. } => true,
            Self::RegistryError { status, .. } if *status >= 500 => true,
            _ => false,
        }
    }
}

impl From<reqwest::Error> for RegistryError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for RegistryError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> Self {
        Self::CacheError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RegistryError::not_found("users-api", "1.0.0");
        assert!(err.is_not_found());
        assert_eq!(err.to_string(), "artifact not found: users-api@1.0.0");
    }

    #[test]
    fn test_already_exists() {
        let err = RegistryError::already_exists("users-api", "1.0.0");
        assert!(err.is_already_exists());
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_auth_error() {
        let err = RegistryError::auth_failed("token expired");
        assert!(err.is_auth_error());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_retryable_errors() {
        let network_err = RegistryError::NetworkError("timeout".to_string());
        assert!(network_err.is_retryable());

        let rate_limited = RegistryError::RateLimited {
            retry_after_secs: 60,
        };
        assert!(rate_limited.is_retryable());

        let server_err = RegistryError::registry_error(502, "bad gateway");
        assert!(server_err.is_retryable());

        let client_err = RegistryError::registry_error(400, "bad request");
        assert!(!client_err.is_retryable());
    }

    #[test]
    fn test_checksum_mismatch() {
        let err = RegistryError::checksum_mismatch("abc123", "def456");
        assert_eq!(
            err.to_string(),
            "checksum mismatch: expected abc123, got def456"
        );
    }

    #[test]
    fn test_from_reqwest_error() {
        // We can't easily create a reqwest error, so we just ensure the trait is implemented
        fn _check_impl<T: From<reqwest::Error>>() {}
        fn _assert() {
            _check_impl::<RegistryError>();
        }
    }
}
