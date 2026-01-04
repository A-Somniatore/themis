//! API operation definitions.
//!
//! Operations represent individual API endpoints or RPC methods.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema::Schema;

/// HTTP methods supported by REST APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// HTTP GET
    Get,
    /// HTTP POST
    Post,
    /// HTTP PUT
    Put,
    /// HTTP PATCH
    Patch,
    /// HTTP DELETE
    Delete,
    /// HTTP HEAD
    Head,
    /// HTTP OPTIONS
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Patch => write!(f, "PATCH"),
            Self::Delete => write!(f, "DELETE"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
        }
    }
}

/// An API operation (endpoint or RPC method).
///
/// This represents a single operation that can be performed on the API,
/// normalized from any supported contract format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    /// Unique operation identifier (required by Themis)
    pub operation_id: String,

    /// Human-readable summary
    #[serde(default)]
    pub summary: Option<String>,

    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,

    /// HTTP method (for REST APIs)
    #[serde(default)]
    pub method: Option<HttpMethod>,

    /// Path template (for REST APIs, e.g., "/users/{userId}")
    #[serde(default)]
    pub path: Option<String>,

    /// Request parameters
    #[serde(default)]
    pub parameters: Vec<Parameter>,

    /// Request body schema
    #[serde(default)]
    pub request_body: Option<RequestBody>,

    /// Possible responses, keyed by status code or "default"
    #[serde(default)]
    pub responses: HashMap<String, Response>,

    /// Security requirements for this operation
    #[serde(default)]
    pub security: Vec<SecurityRequirement>,

    /// Whether this operation is deprecated
    #[serde(default)]
    pub deprecated: bool,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Themis-specific metadata
    #[serde(default)]
    pub themis_metadata: Option<ThemisOperationMetadata>,
}

impl Operation {
    /// Creates a new operation with the given ID.
    pub fn new(operation_id: impl Into<String>) -> Self {
        Self {
            operation_id: operation_id.into(),
            summary: None,
            description: None,
            method: None,
            path: None,
            parameters: Vec::new(),
            request_body: None,
            responses: HashMap::new(),
            security: Vec::new(),
            deprecated: false,
            tags: Vec::new(),
            themis_metadata: None,
        }
    }

    /// Returns true if this operation requires authentication.
    #[must_use]
    pub fn requires_auth(&self) -> bool {
        !self.security.is_empty()
    }

    /// Returns true if this operation is idempotent.
    #[must_use]
    pub fn is_idempotent(&self) -> bool {
        // By default, GET, PUT, DELETE, HEAD, OPTIONS are idempotent
        // Can be overridden by Themis metadata
        if let Some(meta) = &self.themis_metadata {
            return meta.idempotent.unwrap_or(false);
        }
        matches!(
            self.method,
            Some(HttpMethod::Get)
                | Some(HttpMethod::Put)
                | Some(HttpMethod::Delete)
                | Some(HttpMethod::Head)
                | Some(HttpMethod::Options)
        )
    }
}

/// Themis-specific operation metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemisOperationMetadata {
    /// Rate limit tier (e.g., "standard", "high", "critical")
    #[serde(default, rename = "rate-limit-tier")]
    pub rate_limit_tier: Option<String>,

    /// Timeout tier (e.g., "fast", "standard", "slow")
    #[serde(default, rename = "timeout-tier")]
    pub timeout_tier: Option<String>,

    /// Whether this operation is idempotent
    #[serde(default)]
    pub idempotent: Option<bool>,
}

/// An operation parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Where the parameter is located
    #[serde(rename = "in")]
    pub location: ParameterLocation,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,

    /// Whether the parameter is deprecated
    #[serde(default)]
    pub deprecated: bool,

    /// Parameter schema
    pub schema: Schema,
}

/// Location of a parameter in the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    /// In the URL path
    Path,
    /// In the query string
    Query,
    /// In request headers
    Header,
    /// In cookies
    Cookie,
}

/// Request body definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestBody {
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the request body is required
    #[serde(default)]
    pub required: bool,

    /// Content by media type
    pub content: HashMap<String, MediaType>,
}

/// Response definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Human-readable description
    pub description: String,

    /// Response content by media type
    #[serde(default)]
    pub content: HashMap<String, MediaType>,

    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, Header>,
}

/// Media type content definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaType {
    /// Schema for this media type
    pub schema: Schema,
}

/// Response header definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the header is required
    #[serde(default)]
    pub required: bool,

    /// Header schema
    pub schema: Schema,
}

/// Security requirement for an operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityRequirement {
    /// Security scheme name
    pub scheme: String,

    /// Required scopes (for OAuth2)
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_new() {
        let op = Operation::new("getUser");
        assert_eq!(op.operation_id, "getUser");
        assert!(!op.requires_auth());
        assert!(!op.deprecated);
    }

    #[test]
    fn test_operation_requires_auth() {
        let mut op = Operation::new("getUser");
        assert!(!op.requires_auth());

        op.security.push(SecurityRequirement {
            scheme: "bearer".to_string(),
            scopes: vec![],
        });
        assert!(op.requires_auth());
    }

    #[test]
    fn test_operation_idempotent() {
        let mut op = Operation::new("getUser");
        op.method = Some(HttpMethod::Get);
        assert!(op.is_idempotent());

        op.method = Some(HttpMethod::Post);
        assert!(!op.is_idempotent());

        // Override with metadata
        op.themis_metadata = Some(ThemisOperationMetadata {
            rate_limit_tier: None,
            timeout_tier: None,
            idempotent: Some(true),
        });
        assert!(op.is_idempotent());
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    #[test]
    fn test_operation_serialization() {
        let op = Operation::new("getUser");
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: Operation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }
}
