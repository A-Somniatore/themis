//! Operation types for artifacts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use themis_core::Schema;

/// An operation in an artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactOperation {
    /// Operation ID (e.g., "getUser").
    pub id: String,

    /// HTTP method (e.g., "GET", "POST").
    pub method: String,

    /// URL path (e.g., "/users/{userId}").
    pub path: String,

    /// Short summary of the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Detailed description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Security requirements (e.g., ["spiffe", "bearer"]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<String>,

    /// Request schema (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_schema: Option<Schema>,

    /// Response schemas by status code.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub response_schemas: HashMap<String, Schema>,

    /// Operation-specific metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OperationMetadata>,

    /// Tags for grouping operations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Whether the operation is deprecated.
    #[serde(default)]
    pub deprecated: bool,
}

/// Operation-specific metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationMetadata {
    /// Rate limit tier (e.g., "standard", "premium").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_tier: Option<String>,

    /// Timeout tier (e.g., "fast", "slow").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_tier: Option<String>,

    /// Whether the operation is idempotent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent: Option<bool>,

    /// Custom metadata.
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

impl ArtifactOperation {
    /// Creates a new artifact operation.
    pub fn new(id: impl Into<String>, method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            path: path.into(),
            summary: None,
            description: None,
            security: Vec::new(),
            request_schema: None,
            response_schemas: HashMap::new(),
            metadata: None,
            tags: Vec::new(),
            deprecated: false,
        }
    }

    /// Sets the summary.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a security requirement.
    pub fn with_security(mut self, security: impl Into<String>) -> Self {
        self.security.push(security.into());
        self
    }

    /// Sets the request schema.
    pub fn with_request_schema(mut self, schema: Schema) -> Self {
        self.request_schema = Some(schema);
        self
    }

    /// Adds a response schema for a status code.
    pub fn with_response_schema(mut self, status: impl Into<String>, schema: Schema) -> Self {
        self.response_schemas.insert(status.into(), schema);
        self
    }

    /// Sets operation metadata.
    pub fn with_metadata(mut self, metadata: OperationMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_operation_builder() {
        let op = ArtifactOperation::new("getUser", "GET", "/users/{userId}")
            .with_summary("Get a user by ID")
            .with_description("Retrieves a user by their unique identifier")
            .with_security("bearer");

        assert_eq!(op.id, "getUser");
        assert_eq!(op.method, "GET");
        assert_eq!(op.path, "/users/{userId}");
        assert_eq!(op.summary.unwrap(), "Get a user by ID");
        assert_eq!(op.security, vec!["bearer"]);
    }

    #[test]
    fn test_artifact_operation_serialization() {
        let op =
            ArtifactOperation::new("getUser", "GET", "/users/{userId}").with_summary("Get a user");

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"id\":\"getUser\""));
        assert!(json.contains("\"method\":\"GET\""));

        let parsed: ArtifactOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, op.id);
    }

    #[test]
    fn test_operation_metadata() {
        let metadata = OperationMetadata {
            rate_limit_tier: Some("premium".to_string()),
            timeout_tier: Some("fast".to_string()),
            idempotent: Some(true),
            custom: HashMap::new(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"rate_limit_tier\":\"premium\""));
        assert!(json.contains("\"idempotent\":true"));
    }
}
