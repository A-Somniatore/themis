//! Artifact type definition.

use crate::error::{ArtifactError, ArtifactResult};
use crate::operation::ArtifactOperation;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use themis_core::Schema;

/// Schema version for artifact format.
pub const ARTIFACT_SCHEMA_VERSION: &str = "https://themis.somniatore.com/schemas/artifact.v1.json";

/// A published contract artifact.
///
/// Artifacts are immutable, content-addressed representations of contracts
/// that can be published to a registry and loaded at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// JSON Schema reference for the artifact format.
    #[serde(rename = "$schema")]
    pub schema: String,

    /// Contract version (semantic version).
    pub version: String,

    /// Service name.
    pub service: String,

    /// Contract format (e.g., "openapi", "protobuf").
    pub format: String,

    /// Format version (e.g., "3.1.0" for OpenAPI).
    pub format_version: String,

    /// Artifact metadata.
    pub metadata: ArtifactMetadata,

    /// Checksum for integrity verification.
    pub checksum: Checksum,

    /// All operations defined in the contract.
    pub operations: Vec<ArtifactOperation>,

    /// Named schemas referenced by operations (uses IndexMap for deterministic ordering).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub schemas: IndexMap<String, Schema>,

    /// Base64-encoded original contract source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_contract: Option<String>,
}

/// Artifact metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// When the artifact was created.
    pub created_at: DateTime<Utc>,

    /// Git commit SHA (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,

    /// Git repository URL (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_repository: Option<String>,

    /// Team or user that owns this contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// Custom metadata.
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Checksum for integrity verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    /// Hashing algorithm (e.g., "sha256").
    pub algorithm: String,

    /// Hex-encoded hash value.
    pub value: String,
}

impl Artifact {
    /// Verifies the artifact checksum.
    ///
    /// Returns `Ok(())` if the checksum matches, or an error if it doesn't.
    pub fn verify_checksum(&self) -> ArtifactResult<()> {
        let computed = self.compute_checksum();
        if computed != self.checksum.value {
            return Err(ArtifactError::checksum_mismatch(
                &self.checksum.value,
                &computed,
            ));
        }
        Ok(())
    }

    /// Computes the checksum of the artifact content.
    ///
    /// The checksum is computed over a deterministic JSON representation
    /// of the artifact, excluding the checksum field itself.
    pub fn compute_checksum(&self) -> String {
        // Create a copy without the checksum for hashing
        let hashable = HashableArtifact {
            version: &self.version,
            service: &self.service,
            format: &self.format,
            format_version: &self.format_version,
            operations: &self.operations,
            schemas: &self.schemas,
            raw_contract: self.raw_contract.as_deref(),
        };

        let json = serde_json::to_string(&hashable).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Returns the artifact identifier (service@version).
    pub fn id(&self) -> String {
        format!("{}@{}", self.service, self.version)
    }

    /// Gets an operation by ID.
    pub fn get_operation(&self, id: &str) -> Option<&ArtifactOperation> {
        self.operations.iter().find(|op| op.id == id)
    }

    /// Gets a schema by name.
    pub fn get_schema(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }

    /// Returns the number of operations.
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Serializes the artifact to JSON.
    pub fn to_json(&self) -> ArtifactResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserializes an artifact from JSON.
    pub fn from_json(json: &str) -> ArtifactResult<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Loads an artifact from a file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> ArtifactResult<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    /// Saves the artifact to a file.
    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> ArtifactResult<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for ArtifactMetadata {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            git_commit: None,
            git_repository: None,
            owner: None,
            custom: HashMap::new(),
        }
    }
}

impl Checksum {
    /// Creates a new SHA-256 checksum.
    pub fn sha256(value: impl Into<String>) -> Self {
        Self {
            algorithm: "sha256".to_string(),
            value: value.into(),
        }
    }
}

/// Helper struct for computing checksum (excludes checksum and metadata.created_at).
#[derive(Serialize)]
struct HashableArtifact<'a> {
    version: &'a str,
    service: &'a str,
    format: &'a str,
    format_version: &'a str,
    operations: &'a [ArtifactOperation],
    schemas: &'a IndexMap<String, Schema>,
    raw_contract: Option<&'a str>,
}

// Hex encoding helper (we don't want to add another dependency)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_artifact() -> Artifact {
        Artifact {
            schema: ARTIFACT_SCHEMA_VERSION.to_string(),
            version: "1.0.0".to_string(),
            service: "users-service".to_string(),
            format: "openapi".to_string(),
            format_version: "3.1.0".to_string(),
            metadata: ArtifactMetadata::default(),
            checksum: Checksum::sha256("placeholder"),
            operations: vec![ArtifactOperation::new("getUser", "GET", "/users/{userId}")],
            schemas: IndexMap::new(),
            raw_contract: None,
        }
    }

    #[test]
    fn test_artifact_id() {
        let artifact = create_test_artifact();
        assert_eq!(artifact.id(), "users-service@1.0.0");
    }

    #[test]
    fn test_artifact_get_operation() {
        let artifact = create_test_artifact();
        let op = artifact.get_operation("getUser");
        assert!(op.is_some());
        assert_eq!(op.unwrap().method, "GET");

        assert!(artifact.get_operation("unknown").is_none());
    }

    #[test]
    fn test_artifact_checksum_computation() {
        let artifact = create_test_artifact();
        let checksum1 = artifact.compute_checksum();
        let checksum2 = artifact.compute_checksum();
        assert_eq!(checksum1, checksum2);

        // Verify checksum is a hex string
        assert_eq!(checksum1.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_artifact_checksum_verification() {
        let mut artifact = create_test_artifact();
        artifact.checksum = Checksum::sha256(artifact.compute_checksum());

        // Should pass verification
        assert!(artifact.verify_checksum().is_ok());

        // Modify and verify fails
        artifact.version = "2.0.0".to_string();
        assert!(artifact.verify_checksum().is_err());
    }

    #[test]
    fn test_artifact_serialization() {
        let artifact = create_test_artifact();
        let json = artifact.to_json().unwrap();

        // Pretty-printed JSON has whitespace around colons
        assert!(json.contains("\"$schema\":"));
        assert!(json.contains("\"service\": \"users-service\""));
        assert!(json.contains("\"version\": \"1.0.0\""));

        let parsed = Artifact::from_json(&json).unwrap();
        assert_eq!(parsed.service, artifact.service);
        assert_eq!(parsed.version, artifact.version);
    }

    #[test]
    fn test_artifact_operation_count() {
        let artifact = create_test_artifact();
        assert_eq!(artifact.operation_count(), 1);
    }

    #[test]
    fn test_checksum_sha256() {
        let checksum = Checksum::sha256("abc123");
        assert_eq!(checksum.algorithm, "sha256");
        assert_eq!(checksum.value, "abc123");
    }
}
