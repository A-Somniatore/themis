//! Artifact integration tests.
//!
//! Tests artifact creation, serialization, and verification.

use crate::fixtures::{MINIMAL_CONTRACT, USERS_SERVICE_V1};
use themis_artifact::ArtifactBuilder;
use themis_openapi::parse_openapi;

/// Tests creating an artifact from a parsed contract.
#[test]
fn test_artifact_from_contract() {
    let contract = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    assert_eq!(artifact.service, contract.metadata.service_name);
    assert!(artifact.verify_checksum().is_ok());
}

/// Tests artifact round-trip (create, serialize, deserialize, verify).
#[test]
fn test_artifact_round_trip() {
    let contract = parse_openapi(MINIMAL_CONTRACT)
        .expect("Should parse contract");

    let original = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Serialize to JSON
    let json = original.to_json()
        .expect("Should serialize to JSON");

    // Deserialize back
    let restored = themis_artifact::Artifact::from_json(&json)
        .expect("Should deserialize from JSON");

    // Verify restored artifact
    assert_eq!(original.service, restored.service);
    assert_eq!(original.version, restored.version);
    assert_eq!(original.checksum.value, restored.checksum.value);
    assert!(restored.verify_checksum().is_ok(), "Restored artifact should have valid checksum");
}

/// Tests artifact creation with custom metadata.
#[test]
fn test_artifact_with_custom_metadata() {
    let contract = parse_openapi(MINIMAL_CONTRACT)
        .expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .owner("platform-team")
        .git_repository("https://github.com/org/repo")
        .build()
        .expect("Should create artifact");

    assert_eq!(artifact.metadata.owner, Some("platform-team".to_string()));
    assert!(artifact.verify_checksum().is_ok());
}

/// Tests that artifacts have operations from the contract.
#[test]
fn test_artifact_has_operations() {
    let contract = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Artifact should have operations from the contract
    assert!(!artifact.operations.is_empty(), "Artifact should have operations");
    
    // Number of operations should match
    assert_eq!(
        artifact.operations.len(),
        contract.operations.len(),
        "Artifact should have same number of operations as contract"
    );
}

/// Tests building artifact manually.
#[test]
fn test_manual_artifact_build() {
    let artifact = ArtifactBuilder::new()
        .service("test-service")
        .version("1.0.0")
        .format("openapi", "3.1.0")
        .owner("test-team")
        .build()
        .expect("Should create artifact");

    assert_eq!(artifact.service, "test-service");
    assert_eq!(artifact.version, "1.0.0");
    assert_eq!(artifact.metadata.owner, Some("test-team".to_string()));
    assert!(artifact.verify_checksum().is_ok());
}

/// Tests that different contracts produce different checksums.
#[test]
fn test_different_contracts_different_checksums() {
    let contract1 = parse_openapi(MINIMAL_CONTRACT)
        .expect("Should parse contract 1");
    let contract2 = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse contract 2");

    let artifact1 = ArtifactBuilder::from_contract(&contract1)
        .build()
        .expect("Should create artifact 1");
    let artifact2 = ArtifactBuilder::from_contract(&contract2)
        .build()
        .expect("Should create artifact 2");

    assert_ne!(
        artifact1.checksum.value,
        artifact2.checksum.value,
        "Different contracts should have different checksums"
    );
}

/// Tests artifact format information.
#[test]
fn test_artifact_format_info() {
    let contract = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    assert_eq!(artifact.format, "openapi");
    assert_eq!(artifact.format_version, "3.1.0");
}
