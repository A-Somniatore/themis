//! Contract artifact functionality.
//!
//! This module provides functions for creating and managing contract artifacts.

use std::path::Path;

use themis_artifact::{Artifact, ArtifactBuilder};
use themis_core::Contract;

use crate::error::{SdkError, SdkResult};
use crate::Language;

/// Artifact output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactFormat {
    /// JSON format (human-readable).
    Json,
    /// YAML format (human-readable).
    Yaml,
}

impl std::fmt::Display for ArtifactFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
        }
    }
}

/// Create an artifact from a contract.
///
/// # Arguments
///
/// * `contract` - The contract to create an artifact from
///
/// # Returns
///
/// The created artifact
///
/// # Errors
///
/// Returns an error if artifact creation fails
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::artifact::create_artifact;
/// use themis_sdk::parse::parse_string;
///
/// let contract = parse_string(yaml)?;
/// let artifact = create_artifact(&contract)?;
/// println!("Created artifact: {} v{}", artifact.service, artifact.version);
/// ```
pub fn create_artifact(contract: &Contract) -> SdkResult<Artifact> {
    ArtifactBuilder::from_contract(contract)
        .build()
        .map_err(|e| SdkError::Artifact {
            message: e.to_string(),
        })
}

/// Create an artifact from a contract with additional metadata.
///
/// # Arguments
///
/// * `contract` - The contract to create an artifact from
/// * `git_commit` - Optional git commit SHA
/// * `git_repository` - Optional git repository URL
///
/// # Returns
///
/// The created artifact
///
/// # Errors
///
/// Returns an error if artifact creation fails
pub fn create_artifact_with_metadata(
    contract: &Contract,
    git_commit: Option<&str>,
    git_repository: Option<&str>,
) -> SdkResult<Artifact> {
    let mut builder = ArtifactBuilder::from_contract(contract);

    if let Some(commit) = git_commit {
        builder = builder.git_commit(commit);
    }

    if let Some(repo) = git_repository {
        builder = builder.git_repository(repo);
    }

    builder.build().map_err(|e| SdkError::Artifact {
        message: e.to_string(),
    })
}

/// Create an artifact from a contract file.
///
/// # Arguments
///
/// * `path` - Path to the contract file
///
/// # Returns
///
/// The created artifact
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The contract cannot be parsed
/// - Artifact creation fails
pub fn create_artifact_from_file<P: AsRef<Path>>(path: P) -> SdkResult<Artifact> {
    let contract = crate::parse::parse_file(path)?;
    create_artifact(&contract)
}

/// Serialize an artifact to JSON.
///
/// # Arguments
///
/// * `artifact` - The artifact to serialize
///
/// # Returns
///
/// The serialized artifact as a JSON string
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn serialize_artifact_json(artifact: &Artifact) -> SdkResult<String> {
    serde_json::to_string_pretty(artifact).map_err(|e| SdkError::Artifact {
        message: format!("JSON serialization failed: {}", e),
    })
}

/// Serialize an artifact to YAML.
///
/// # Arguments
///
/// * `artifact` - The artifact to serialize
///
/// # Returns
///
/// The serialized artifact as a YAML string
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn serialize_artifact_yaml(artifact: &Artifact) -> SdkResult<String> {
    serde_yaml::to_string(artifact).map_err(|e| SdkError::Artifact {
        message: format!("YAML serialization failed: {}", e),
    })
}

/// Serialize an artifact to a specific format.
///
/// # Arguments
///
/// * `artifact` - The artifact to serialize
/// * `format` - The output format
///
/// # Returns
///
/// The serialized artifact as a string
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn serialize_artifact(artifact: &Artifact, format: ArtifactFormat) -> SdkResult<String> {
    match format {
        ArtifactFormat::Json => serialize_artifact_json(artifact),
        ArtifactFormat::Yaml => serialize_artifact_yaml(artifact),
    }
}

/// Save an artifact to a file.
///
/// # Arguments
///
/// * `artifact` - The artifact to save
/// * `path` - Path to save the artifact to
/// * `format` - The output format
///
/// # Returns
///
/// Ok if successful
///
/// # Errors
///
/// Returns an error if:
/// - Serialization fails
/// - The file cannot be written
pub fn save_artifact<P: AsRef<Path>>(
    artifact: &Artifact,
    path: P,
    format: ArtifactFormat,
) -> SdkResult<()> {
    let path = path.as_ref();
    let data = serialize_artifact(artifact, format)?;

    std::fs::write(path, data).map_err(|e| SdkError::FileWrite {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Load an artifact from a JSON file.
///
/// # Arguments
///
/// * `path` - Path to the artifact file
///
/// # Returns
///
/// The loaded artifact
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The artifact cannot be deserialized
pub fn load_artifact<P: AsRef<Path>>(path: P) -> SdkResult<Artifact> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| SdkError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Try JSON first, then YAML
    serde_json::from_str(&content)
        .or_else(|_| serde_yaml::from_str(&content))
        .map_err(|e| SdkError::Artifact {
            message: format!("Failed to deserialize artifact: {}", e),
        })
}

/// Get the list of supported artifact formats.
#[must_use]
pub fn supported_formats() -> Vec<ArtifactFormat> {
    vec![ArtifactFormat::Json, ArtifactFormat::Yaml]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats() {
        let formats = supported_formats();
        assert!(!formats.is_empty());
        assert!(formats.contains(&ArtifactFormat::Json));
        assert!(formats.contains(&ArtifactFormat::Yaml));
    }

    #[test]
    fn test_artifact_format_display() {
        assert_eq!(ArtifactFormat::Json.to_string(), "json");
        assert_eq!(ArtifactFormat::Yaml.to_string(), "yaml");
    }
}
