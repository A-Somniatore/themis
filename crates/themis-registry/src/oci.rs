//! OCI types and constants.

use serde::{Deserialize, Serialize};

/// Media type for Themis artifacts.
pub const THEMIS_ARTIFACT_MEDIA_TYPE: &str = "application/vnd.themis.artifact.v1+json";

/// Media type for OCI image manifests.
pub const OCI_MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";

/// Media type for OCI image index.
pub const OCI_INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";

/// Media type for OCI image config (empty for Themis).
pub const OCI_CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";

/// Common media types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaType {
    /// Themis artifact JSON.
    ThemisArtifact,
    /// OCI image manifest.
    OciManifest,
    /// OCI image index.
    OciIndex,
    /// OCI image config.
    OciConfig,
    /// Unknown media type.
    Unknown,
}

impl MediaType {
    /// Returns the media type string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ThemisArtifact => THEMIS_ARTIFACT_MEDIA_TYPE,
            Self::OciManifest => OCI_MANIFEST_MEDIA_TYPE,
            Self::OciIndex => OCI_INDEX_MEDIA_TYPE,
            Self::OciConfig => OCI_CONFIG_MEDIA_TYPE,
            Self::Unknown => "application/octet-stream",
        }
    }

    /// Parses a media type string.
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            THEMIS_ARTIFACT_MEDIA_TYPE => Self::ThemisArtifact,
            OCI_MANIFEST_MEDIA_TYPE => Self::OciManifest,
            OCI_INDEX_MEDIA_TYPE => Self::OciIndex,
            OCI_CONFIG_MEDIA_TYPE => Self::OciConfig,
            _ => Self::Unknown,
        }
    }
}

/// OCI content descriptor.
///
/// See [OCI Image Spec](https://github.com/opencontainers/image-spec/blob/main/descriptor.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OciDescriptor {
    /// Media type of the referenced content.
    pub media_type: String,

    /// Content digest (e.g., "sha256:abc...").
    pub digest: String,

    /// Size in bytes.
    pub size: u64,

    /// Optional annotations.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub annotations: std::collections::HashMap<String, String>,
}

impl OciDescriptor {
    /// Creates a new descriptor.
    pub fn new(media_type: impl Into<String>, digest: impl Into<String>, size: u64) -> Self {
        Self {
            media_type: media_type.into(),
            digest: digest.into(),
            size,
            annotations: std::collections::HashMap::new(),
        }
    }

    /// Adds an annotation.
    pub fn with_annotation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.annotations.insert(key.into(), value.into());
        self
    }
}

/// OCI image manifest.
///
/// See [OCI Image Spec](https://github.com/opencontainers/image-spec/blob/main/manifest.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OciManifest {
    /// Schema version (always 2).
    pub schema_version: u32,

    /// Media type of this manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    /// Config descriptor (usually empty for artifacts).
    pub config: OciDescriptor,

    /// Layer descriptors (contains the artifact).
    pub layers: Vec<OciDescriptor>,

    /// Optional annotations.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub annotations: std::collections::HashMap<String, String>,
}

impl OciManifest {
    /// Creates a new manifest for a Themis artifact.
    #[must_use]
    pub fn for_artifact(artifact_descriptor: OciDescriptor, config: OciDescriptor) -> Self {
        Self {
            schema_version: 2,
            media_type: Some(OCI_MANIFEST_MEDIA_TYPE.to_string()),
            config,
            layers: vec![artifact_descriptor],
            annotations: std::collections::HashMap::new(),
        }
    }

    /// Creates an empty config descriptor for artifacts.
    ///
    /// OCI artifacts don't need a config, so we use an empty JSON object.
    #[must_use]
    pub fn empty_config() -> OciDescriptor {
        let empty = "{}";
        let digest = format!("sha256:{}", sha256_hex(empty.as_bytes()));
        OciDescriptor::new(OCI_CONFIG_MEDIA_TYPE, digest, empty.len() as u64)
    }

    /// Adds an annotation.
    pub fn with_annotation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.annotations.insert(key.into(), value.into());
        self
    }

    /// Gets the first layer (artifact) descriptor.
    #[must_use]
    pub fn artifact_layer(&self) -> Option<&OciDescriptor> {
        self.layers.first()
    }
}

/// OCI tags list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsList {
    /// Repository name.
    pub name: String,

    /// List of tags.
    pub tags: Vec<String>,
}

/// OCI error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciError {
    /// Error code.
    pub code: String,

    /// Error message.
    pub message: String,

    /// Optional detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

/// OCI errors response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciErrors {
    /// List of errors.
    pub errors: Vec<OciError>,
}

/// Computes SHA256 hex digest of data.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Creates a content descriptor from data.
pub fn descriptor_from_data(media_type: &str, data: &[u8]) -> OciDescriptor {
    let digest = format!("sha256:{}", sha256_hex(data));
    OciDescriptor::new(media_type, digest, data.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_conversion() {
        assert_eq!(
            MediaType::ThemisArtifact.as_str(),
            THEMIS_ARTIFACT_MEDIA_TYPE
        );
        assert_eq!(
            MediaType::from_str(THEMIS_ARTIFACT_MEDIA_TYPE),
            MediaType::ThemisArtifact
        );
        assert_eq!(MediaType::from_str("unknown"), MediaType::Unknown);
    }

    #[test]
    fn test_descriptor_creation() {
        let desc = OciDescriptor::new("application/json", "sha256:abc", 100)
            .with_annotation("org.opencontainers.image.title", "test");

        assert_eq!(desc.media_type, "application/json");
        assert_eq!(desc.digest, "sha256:abc");
        assert_eq!(desc.size, 100);
        assert_eq!(
            desc.annotations.get("org.opencontainers.image.title"),
            Some(&"test".to_string())
        );
    }

    #[test]
    fn test_manifest_creation() {
        let layer = OciDescriptor::new(THEMIS_ARTIFACT_MEDIA_TYPE, "sha256:layer", 500);
        let config = OciManifest::empty_config();
        let manifest = OciManifest::for_artifact(layer, config);

        assert_eq!(manifest.schema_version, 2);
        assert_eq!(manifest.layers.len(), 1);
        assert_eq!(manifest.layers[0].media_type, THEMIS_ARTIFACT_MEDIA_TYPE);
    }

    #[test]
    fn test_empty_config() {
        let config = OciManifest::empty_config();
        assert_eq!(config.media_type, OCI_CONFIG_MEDIA_TYPE);
        assert_eq!(config.size, 2); // "{}" is 2 bytes
        assert!(config.digest.starts_with("sha256:"));
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex(b"hello");
        assert_eq!(hash.len(), 64); // SHA256 = 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_descriptor_from_data() {
        let data = b"test data";
        let desc = descriptor_from_data("application/json", data);
        assert_eq!(desc.size, 9);
        assert!(desc.digest.starts_with("sha256:"));
    }

    #[test]
    fn test_manifest_serialization() {
        let layer = OciDescriptor::new(THEMIS_ARTIFACT_MEDIA_TYPE, "sha256:layer", 500);
        let config = OciManifest::empty_config();
        let manifest = OciManifest::for_artifact(layer, config);

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: OciManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.schema_version, 2);
        assert_eq!(parsed.layers.len(), 1);
    }

    #[test]
    fn test_tags_list_deserialization() {
        let json = r#"{"name": "my-org/users-api", "tags": ["1.0.0", "1.1.0", "latest"]}"#;
        let tags: TagsList = serde_json::from_str(json).unwrap();
        assert_eq!(tags.name, "my-org/users-api");
        assert_eq!(tags.tags.len(), 3);
    }

    #[test]
    fn test_oci_error_deserialization() {
        let json = r#"{"errors": [{"code": "NAME_UNKNOWN", "message": "repository not found"}]}"#;
        let errors: OciErrors = serde_json::from_str(json).unwrap();
        assert_eq!(errors.errors.len(), 1);
        assert_eq!(errors.errors[0].code, "NAME_UNKNOWN");
    }
}
