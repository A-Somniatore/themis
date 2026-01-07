//! Performance tests for Themis with large contracts.
//!
//! Tests that validate Themis performs well with realistic workloads.

use std::fmt::Write as _;
use std::time::Instant;
use themis_artifact::ArtifactBuilder;
use themis_core::schema::{IntegerSchema, ObjectSchema, StringSchema};
use themis_core::Schema;
use indexmap::IndexMap;

/// Builds a large contract from YAML string to test parsing performance.
fn build_large_openapi_yaml(num_operations: usize) -> String {
    let mut yaml = String::from(r#"openapi: "3.1.0"
info:
  title: Large Performance Test API
  version: "1.0.0"
paths:
"#);
    
    for i in 0..num_operations {
        let method = match i % 4 {
            0 => "get",
            1 => "post",
            2 => "put",
            _ => "delete",
        };
        
        // Using write! instead of push_str(&format!(...)) to avoid extra allocation
        let _ = write!(yaml, r#"  /resources/{i}/{{resourceId}}:
    {method}:
      operationId: {method}Resource{i}
      summary: {method} resource {i}
      parameters:
        - name: resourceId
          in: path
          required: true
          schema:
            type: string
        - name: filter
          in: query
          required: false
          schema:
            type: string
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                type: object
                properties:
                  id:
                    type: integer
                  name:
                    type: string
                  created_at:
                    type: string
                required:
                  - id
                  - name
"#);
    }
    
    yaml
}

#[test]
fn test_parsing_performance_50_operations() {
    let yaml = build_large_openapi_yaml(50);
    
    let start = Instant::now();
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    let elapsed = start.elapsed();
    
    assert_eq!(contract.operations.len(), 50);
    println!("Parsing 50 operations: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "Parsing too slow: {:?}", elapsed);
}

#[test]
fn test_parsing_performance_100_operations() {
    let yaml = build_large_openapi_yaml(100);
    
    let start = Instant::now();
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    let elapsed = start.elapsed();
    
    assert_eq!(contract.operations.len(), 100);
    println!("Parsing 100 operations: {:?}", elapsed);
    assert!(elapsed.as_millis() < 1000, "Parsing too slow: {:?}", elapsed);
}

#[test]
fn test_artifact_creation_performance() {
    let yaml = build_large_openapi_yaml(100);
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    
    let start = Instant::now();
    let artifact = ArtifactBuilder::from_contract(&contract)
        .service("performance-test-service")
        .version("1.0.0")
        .build()
        .expect("Should build artifact");
    let elapsed = start.elapsed();
    
    // Verify artifact was created correctly
    assert_eq!(artifact.service, "performance-test-service");
    assert_eq!(artifact.version, "1.0.0");
    assert_eq!(artifact.operations.len(), 100);
    
    // Performance check
    println!("Artifact creation for 100 operations: {:?}", elapsed);
    assert!(
        elapsed.as_millis() < 1000,
        "Artifact creation took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_artifact_serialization_performance() {
    let yaml = build_large_openapi_yaml(50);
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    let artifact = ArtifactBuilder::from_contract(&contract)
        .service("serialization-test")
        .version("1.0.0")
        .build()
        .expect("Should build artifact");
    
    // Test JSON serialization
    let start = Instant::now();
    let json = serde_json::to_string(&artifact).expect("Should serialize");
    let json_elapsed = start.elapsed();
    
    // Test JSON deserialization
    let start = Instant::now();
    let _: themis_artifact::Artifact = serde_json::from_str(&json).expect("Should deserialize");
    let deser_elapsed = start.elapsed();
    
    println!("JSON serialization (50 ops): {:?}", json_elapsed);
    println!("JSON deserialization (50 ops): {:?}", deser_elapsed);
    println!("Serialized JSON size: {} bytes", json.len());
    
    // Performance checks
    assert!(json_elapsed.as_millis() < 500, "Serialization too slow");
    assert!(deser_elapsed.as_millis() < 500, "Deserialization too slow");
}

#[test]
fn test_schema_validation_performance() {
    use crate::archimedes_adapter::ArchimedesAdapter;
    
    // Create deeply nested schema
    fn create_nested_schema(depth: usize) -> Schema {
        if depth == 0 {
            Schema::String(StringSchema::default())
        } else {
            Schema::Object(ObjectSchema {
                properties: IndexMap::from([
                    ("nested".to_string(), create_nested_schema(depth - 1)),
                ]),
                required: vec!["nested".to_string()],
                ..Default::default()
            })
        }
    }
    
    // Test validation of moderately nested schema
    let nested_schema = create_nested_schema(8);
    
    let start = Instant::now();
    let result = ArchimedesAdapter::validate_schema(&nested_schema);
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    println!("Schema validation (depth 8): {:?}", elapsed);
    assert!(elapsed.as_micros() < 1000, "Schema validation too slow");
    
    // Test that excessive nesting is rejected
    let too_deep = create_nested_schema(15);
    let result = ArchimedesAdapter::validate_schema(&too_deep);
    assert!(result.is_err(), "Should reject excessively nested schema");
}

#[test]
fn test_operation_routing_performance() {
    use crate::archimedes_adapter::ArchimedesAdapter;
    
    let yaml = build_large_openapi_yaml(100);
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    let artifact = ArtifactBuilder::from_contract(&contract)
        .service("routing-test")
        .version("1.0.0")
        .build()
        .expect("Should build artifact");
    
    let adapted = ArchimedesAdapter::adapt_artifact(&artifact);
    
    // Test routing lookup performance
    let start = Instant::now();
    let mut found = 0;
    for i in 0..100 {
        let method = match i % 4 {
            0 => "GET",
            1 => "POST",
            2 => "PUT",
            _ => "DELETE",
        };
        let path = format!("/resources/{i}/test_value");
        if adapted.find_operation_by_route(method, &path).is_some() {
            found += 1;
        }
    }
    let elapsed = start.elapsed();
    
    println!("100 route lookups: {:?}, found: {}", elapsed, found);
    assert!(elapsed.as_millis() < 100, "Route lookup too slow");
}

#[test]
fn test_checksum_determinism_large_contract() {
    let yaml = build_large_openapi_yaml(50);
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    
    // Create artifacts multiple times
    let artifact1 = ArtifactBuilder::from_contract(&contract)
        .service("checksum-test")
        .version("1.0.0")
        .build()
        .expect("Should build artifact 1");
    
    let artifact2 = ArtifactBuilder::from_contract(&contract)
        .service("checksum-test")
        .version("1.0.0")
        .build()
        .expect("Should build artifact 2");
    
    // Checksums should be identical for identical inputs
    assert_eq!(
        artifact1.checksum.value, artifact2.checksum.value,
        "Checksums should be deterministic"
    );
    
    // Verify we're testing non-trivial content
    assert!(!artifact1.checksum.value.is_empty(), "Checksum should be non-empty");
    println!("Checksum: {}", artifact1.checksum.value);
}

#[test]
fn test_lint_performance_large_contract() {
    use themis_lint::{LintReporter, LintConfig};
    
    let yaml = build_large_openapi_yaml(100);
    let contract = themis_openapi::parse_openapi(&yaml).expect("Should parse");
    
    let config = LintConfig::default();
    let linter = LintReporter::new(config);
    
    let start = Instant::now();
    let _report = linter.lint(&contract);
    let elapsed = start.elapsed();
    
    println!("Linting 100 operations: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "Linting too slow: {:?}", elapsed);
}

#[test]
fn test_compat_diff_performance() {
    use themis_compat::check_compatibility;
    
    let yaml_v1 = build_large_openapi_yaml(50);
    let yaml_v2 = build_large_openapi_yaml(55); // Slightly larger
    
    let contract_v1 = themis_openapi::parse_openapi(&yaml_v1).expect("Should parse v1");
    let contract_v2 = themis_openapi::parse_openapi(&yaml_v2).expect("Should parse v2");
    
    let start = Instant::now();
    let _report = check_compatibility(&contract_v1, &contract_v2);
    let elapsed = start.elapsed();
    
    println!("Compat check (50 vs 55 ops): {:?}", elapsed);
    assert!(elapsed.as_millis() < 1000, "Compat check too slow: {:?}", elapsed);
}
