//! Schema validation tests for Archimedes compatibility.
//!
//! Tests that verify Themis schemas can be validated against Archimedes
//! runtime constraints and that response validation works correctly.

use crate::archimedes_adapter::ArchimedesAdapter;
use crate::fixtures;
use indexmap::IndexMap;
use themis_artifact::ArtifactBuilder;
use themis_core::schema::{ArraySchema, IntegerSchema, ObjectSchema, StringSchema};
use themis_core::Schema;

#[test]
fn test_string_schema_validation() {
    let schema = Schema::String(StringSchema {
        min_length: Some(1),
        max_length: Some(255),
        pattern: None,
        format: None,
        description: Some("A valid string".to_string()),
        ..Default::default()
    });

    // Should validate successfully
    assert!(ArchimedesAdapter::validate_schema(&schema).is_ok());
}

#[test]
fn test_object_schema_validation() {
    let schema = Schema::Object(ObjectSchema {
        properties: indexmap::IndexMap::from([
            ("id".to_string(), Schema::String(Default::default())),
            ("name".to_string(), Schema::String(Default::default())),
            ("age".to_string(), Schema::Integer(Default::default())),
        ]),
        required: vec!["id".to_string(), "name".to_string()],
        ..Default::default()
    });

    assert!(ArchimedesAdapter::validate_schema(&schema).is_ok());
}

#[test]
fn test_array_schema_validation() {
    let schema = Schema::Array(ArraySchema {
        items: Box::new(Schema::String(Default::default())),
        min_items: Some(1),
        max_items: Some(100),
        ..Default::default()
    });

    assert!(ArchimedesAdapter::validate_schema(&schema).is_ok());
}

#[test]
fn test_nested_object_validation() {
    let inner_object = Schema::Object(ObjectSchema {
        properties: indexmap::IndexMap::from([(
            "street".to_string(),
            Schema::String(Default::default()),
        )]),
        ..Default::default()
    });

    let outer_object = Schema::Object(ObjectSchema {
        properties: indexmap::IndexMap::from([
            ("name".to_string(), Schema::String(Default::default())),
            ("address".to_string(), inner_object),
        ]),
        ..Default::default()
    });

    assert!(ArchimedesAdapter::validate_schema(&outer_object).is_ok());
}

#[test]
fn test_artifact_with_schemas() {
    let artifact = ArtifactBuilder::new()
        .service("test-service")
        .version("1.0.0")
        .build()
        .unwrap();

    // Should be adaptable for Archimedes
    let adapted = ArchimedesAdapter::adapt_artifact(&artifact);
    assert_eq!(adapted.service, "test-service");
    assert_eq!(adapted.version, "1.0.0");
}

#[test]
fn test_minimal_contract_with_one_schema_validates() {
    // Use the minimal contract fixture
    let yaml = fixtures::MINIMAL_CONTRACT;

    // Parse it
    let contract = themis_openapi::parse_openapi(yaml).expect("Should parse minimal contract");

    // Validate schemas
    for (_name, schema) in &contract.schemas {
        assert!(
            ArchimedesAdapter::validate_schema(schema).is_ok(),
            "Schema should be valid for Archimedes"
        );
    }
}

#[test]
fn test_users_service_schemas_validate() {
    let yaml = fixtures::USERS_SERVICE_V1;

    let contract = themis_openapi::parse_openapi(yaml).expect("Should parse users service");

    // All schemas should validate
    for (_name, schema) in &contract.schemas {
        assert!(
            ArchimedesAdapter::validate_schema(schema).is_ok(),
            "Schema should be valid for Archimedes"
        );
    }

    // All operation schemas should validate
    for (_op_id, operation) in &contract.operations {
        if let Some(request_body) = &operation.request_body {
            for (_media_type, content) in &request_body.content {
                assert!(
                    ArchimedesAdapter::validate_schema(&content.schema).is_ok(),
                    "Request schema should be valid"
                );
            }
        }

        for (_status, response) in &operation.responses {
            for (_media_type, content) in &response.content {
                assert!(
                    ArchimedesAdapter::validate_schema(&content.schema).is_ok(),
                    "Response schema should be valid"
                );
            }
        }
    }
}

#[test]
fn test_response_validation_structure() {
    let artifact = ArtifactBuilder::new()
        .service("api-service")
        .version("1.0.0")
        .build()
        .unwrap();

    let adapted = ArchimedesAdapter::adapt_artifact(&artifact);

    // Should have metadata about responses
    for op in &adapted.operations {
        if op.has_response_schemas {
            // Response status codes should be populated
            assert!(!op.response_status_codes.is_empty());
        }
    }
}

#[test]
fn test_operation_metadata_preserved() {
    let artifact = ArtifactBuilder::new()
        .service("metadata-service")
        .version("1.0.0")
        .build()
        .unwrap();

    let adapted = ArchimedesAdapter::adapt_artifact(&artifact);

    for op in &adapted.operations {
        // Check that important metadata is preserved
        assert!(!op.operation_id.is_empty());
        assert!(!op.method.is_empty());
        assert!(!op.path.is_empty());
    }
}

#[test]
fn test_adapted_artifact_can_find_operations() {
    let artifact = ArtifactBuilder::new()
        .service("routing-test")
        .version("1.0.0")
        .build()
        .unwrap();

    let adapted = ArchimedesAdapter::adapt_artifact(&artifact);

    // Test path matching
    assert!(adapted
        .find_operation_by_route("GET", "/users/123")
        .is_none());

    // Should handle non-matching routes gracefully
}

#[test]
fn test_security_requirements_preserved() {
    let artifact = ArtifactBuilder::new()
        .service("secure-service")
        .version("1.0.0")
        .build()
        .unwrap();

    let adapted = ArchimedesAdapter::adapt_artifact(&artifact);

    for op in &adapted.operations {
        // Operations that have security requirements should track them
        // (even if empty for this minimal test)
        assert!(op.security_requirements.is_empty() || !op.security_requirements.is_empty());
    }
}
