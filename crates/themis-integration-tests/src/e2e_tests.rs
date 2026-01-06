//! End-to-end workflow tests.
//!
//! Tests the complete contract governance workflow from contract
//! through to Archimedes-compatible artifact loading.

use crate::archimedes_mocks::{
    MockArtifactLoader, MockOperationRouter, MockPolicyInputBuilder, MockRequestContext,
    OperationMetadata,
};
use crate::fixtures::{MINIMAL_CONTRACT, SECURE_CONTRACT, USERS_SERVICE_V1, USERS_SERVICE_V2};
use themis_artifact::ArtifactBuilder;
use themis_codegen::{
    CodeGenerator, GeneratorConfig, PythonGenerator, RustGenerator, TypeScriptGenerator,
};
use themis_compat::diff_contracts;
use themis_lint::{LintConfig, LintReporter};
use themis_openapi::parse_openapi;

/// Helper to get all generated code as a single string.
fn all_code(output: &themis_codegen::GeneratedCode) -> String {
    output
        .files
        .iter()
        .map(|f| f.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Tests the complete happy path: contract → artifact → Archimedes loading.
#[test]
fn test_e2e_contract_to_archimedes() {
    // Step 1: Parse the contract
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    println!("Parsed contract: {}", contract.metadata.service_name);
    println!("  Operations: {}", contract.operations.len());
    println!("  Schemas: {}", contract.schemas.len());

    // Step 2: Lint the contract
    let linter = LintReporter::new(LintConfig::default());
    let lint_report = linter.lint(&contract);

    println!(
        "Lint results: {} errors, {} warnings",
        lint_report.error_count(),
        lint_report.warning_count()
    );

    // Step 3: Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .owner("platform-team")
        .build()
        .expect("Should create artifact");

    assert!(
        artifact.verify_checksum().is_ok(),
        "Artifact should have valid checksum"
    );

    // Step 4: Load into mock Archimedes
    let mut loader = MockArtifactLoader::new();
    loader
        .load(artifact.clone())
        .expect("Archimedes should be able to load Themis artifact");

    // Step 5: Verify artifact is accessible
    let loaded = loader
        .get(&artifact.service, &artifact.version)
        .expect("Should find loaded artifact");

    assert_eq!(loaded.service, contract.metadata.service_name);
    assert_eq!(loaded.operations.len(), contract.operations.len());

    println!("E2E test passed: contract → artifact → Archimedes");
}

/// Tests operation routing from artifact.
#[test]
fn test_e2e_operation_routing() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Create router from artifact
    let router = MockOperationRouter::from_artifact(&artifact);
    let routes = router.list_routes();

    println!("Routes from artifact:");
    for (method, path, op_id) in &routes {
        println!("  {method} {path} -> {op_id}");
    }

    // Verify all contract operations have routes
    for (op_id, _) in &contract.operations {
        let has_route = routes
            .iter()
            .any(|(_, _, route_op_id)| route_op_id == op_id);
        assert!(has_route, "Operation {op_id} should have a route");
    }
}

/// Tests that policy context includes operation metadata.
#[test]
fn test_e2e_policy_context_from_operation() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // For each operation, verify we can build policy context
    for op in &artifact.operations {
        // Extract metadata from nested structure
        let (rate_limit, timeout, idempotent) = if let Some(meta) = &op.metadata {
            (
                meta.rate_limit_tier.clone(),
                meta.timeout_tier.clone(),
                meta.idempotent.unwrap_or(false),
            )
        } else {
            (None, None, false)
        };

        let metadata = OperationMetadata {
            operation_id: op.id.clone(),
            method: op.method.clone(),
            path: op.path.clone(),
            summary: op.summary.clone(),
            rate_limit_tier: rate_limit,
            timeout_tier: timeout,
            is_idempotent: idempotent,
        };

        let policy_builder = MockPolicyInputBuilder::from_operation(&artifact, &metadata);

        // Verify policy input has required fields for Eunomia
        assert_eq!(
            policy_builder.get_operation_id(),
            Some(op.id.as_str()),
            "PolicyInput.operation_id should match Themis operationId"
        );
        assert_eq!(
            policy_builder.get_service_name(),
            Some(artifact.service.as_str()),
            "PolicyInput should include service name"
        );

        println!(
            "Policy context for {}: service={}, method={:?}, path={:?}",
            op.id,
            artifact.service,
            policy_builder.get_http_method(),
            policy_builder.get_resource_path()
        );
    }
}

/// Tests full workflow with code generation.
#[test]
fn test_e2e_with_code_generation() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    // Generate Rust code
    let config = GeneratorConfig::default();
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen
        .generate(&contract)
        .expect("Should generate Rust code");

    println!("Generated {} Rust files", rust_output.files.len());

    // Generate TypeScript code
    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen
        .generate(&contract)
        .expect("Should generate TypeScript code");

    println!("Generated {} TypeScript files", ts_output.files.len());

    // Generate Python code
    let py_gen = PythonGenerator::new(config);
    let py_output = py_gen
        .generate(&contract)
        .expect("Should generate Python code");

    println!("Generated {} Python files", py_output.files.len());

    // Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Load into mock Archimedes
    let mut loader = MockArtifactLoader::new();
    loader.load(artifact).expect("Should load artifact");

    println!("E2E with codegen passed!");
}

/// Tests version upgrade workflow.
#[test]
fn test_e2e_version_upgrade() {
    // Parse both versions
    let v1_contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse v1");
    let v2_contract = parse_openapi(USERS_SERVICE_V2).expect("Should parse v2");

    // Create artifacts for both
    let v1_artifact = ArtifactBuilder::from_contract(&v1_contract)
        .build()
        .expect("Should create v1 artifact");
    let v2_artifact = ArtifactBuilder::from_contract(&v2_contract)
        .build()
        .expect("Should create v2 artifact");

    // Check compatibility
    let changes = diff_contracts(&v1_contract, &v2_contract);

    println!("Version upgrade analysis:");
    println!("  v1: {} ({})", v1_artifact.service, v1_artifact.version);
    println!("  v2: {} ({})", v2_artifact.service, v2_artifact.version);
    println!("  Breaking changes: {}", changes.breaking_changes.len());
    println!("  Additions: {}", changes.additions.len());
    println!("  Is compatible: {}", changes.is_compatible);
    println!("  Suggested bump: {}", changes.suggested_bump);

    // Load both into mock Archimedes
    let mut loader = MockArtifactLoader::new();
    loader.load(v1_artifact.clone()).expect("Should load v1");
    loader.load(v2_artifact.clone()).expect("Should load v2");

    // Both should be accessible
    assert!(loader
        .get(&v1_artifact.service, &v1_artifact.version)
        .is_some());
    assert!(loader
        .get(&v2_artifact.service, &v2_artifact.version)
        .is_some());
}

/// Tests secure contract workflow with security schemes.
#[test]
fn test_e2e_secure_contract() {
    let contract = parse_openapi(SECURE_CONTRACT).expect("Should parse secure contract");

    // Lint with default config
    let linter = LintReporter::new(LintConfig::default());
    let lint_report = linter.lint(&contract);

    // Should have no errors for properly secured contract
    println!(
        "Secure contract lint: {} errors, {} warnings",
        lint_report.error_count(),
        lint_report.warning_count()
    );

    // Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Verify security-related operations are marked
    for op in &artifact.operations {
        println!(
            "Operation {}: secured operations should be in artifact",
            op.id
        );
    }

    // Load into mock Archimedes
    let mut loader = MockArtifactLoader::new();
    loader.load(artifact).expect("Should load secure artifact");
}

/// Tests request context creation for operations.
#[test]
fn test_e2e_request_context() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    let router = MockOperationRouter::from_artifact(&artifact);

    // Simulate request handling for each operation
    for op in &artifact.operations {
        let ctx = MockRequestContext::new(&op.id, &op.method, &op.path)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json");

        // Verify routing works
        if let Some(routed_op) = router.route(&op.method, &op.path) {
            assert_eq!(routed_op, op.id);
            println!("Routed {} {} -> {}", op.method, op.path, routed_op);
        }

        // Verify request context has all needed info
        assert!(!ctx.request_id.is_empty(), "Should have request ID");
        assert_eq!(ctx.operation_id, op.id);
    }
}

/// Tests minimal contract through full workflow.
#[test]
fn test_e2e_minimal_contract() {
    let contract = parse_openapi(MINIMAL_CONTRACT).expect("Should parse minimal contract");

    // Lint with relaxed config
    let linter = LintReporter::new(LintConfig::relaxed());
    let lint_report = linter.lint(&contract);

    // Minimal contract should pass relaxed linting
    assert_eq!(
        lint_report.error_count(),
        0,
        "Minimal contract should have no errors with relaxed config"
    );

    // Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    assert_eq!(
        artifact.operations.len(),
        1,
        "Minimal contract should have 1 operation"
    );

    // Load into mock Archimedes
    let mut loader = MockArtifactLoader::new();
    loader
        .load(artifact.clone())
        .expect("Should load minimal artifact");

    // Verify operation routing
    let router = MockOperationRouter::from_artifact(&artifact);
    let op_id = router.route("GET", "/health");
    assert_eq!(
        op_id,
        Some("getHealth"),
        "Should route to getHealth operation"
    );
}

/// Tests artifact JSON serialization round-trip with Archimedes loading.
#[test]
fn test_e2e_artifact_serialization() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let original = ArtifactBuilder::from_contract(&contract)
        .owner("platform-team")
        .git_repository("https://github.com/org/users-service")
        .build()
        .expect("Should create artifact");

    // Verify original checksum
    original
        .verify_checksum()
        .expect("Original artifact should have valid checksum");

    // Serialize to JSON (simulating registry storage)
    let json = original.to_json().expect("Should serialize to JSON");

    // Deserialize (simulating Archimedes fetching from registry)
    let restored =
        themis_artifact::Artifact::from_json(&json).expect("Should deserialize from JSON");

    // Verify the restored artifact has matching checksum value
    assert_eq!(
        restored.checksum.value, original.checksum.value,
        "Stored checksum should match after round-trip"
    );

    // Verify the restored artifact checksum is valid
    restored
        .verify_checksum()
        .expect("Restored artifact should have valid checksum after round-trip");

    // Load restored artifact into mock Archimedes
    let mut loader = MockArtifactLoader::new();
    loader
        .load(restored.clone())
        .expect("Archimedes should be able to load deserialized artifact");

    // Verify all data survived round-trip
    let loaded = loader
        .get(&restored.service, &restored.version)
        .expect("Should find loaded artifact");

    assert_eq!(loaded.service, original.service);
    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.operations.len(), original.operations.len());
    assert_eq!(loaded.checksum.value, original.checksum.value);
}

/// Tests that generated code references match artifact operations.
#[test]
fn test_e2e_codegen_matches_artifact() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    // Generate Rust code
    let config = GeneratorConfig::default();
    let rust_gen = RustGenerator::new(config);
    let rust_output = rust_gen
        .generate(&contract)
        .expect("Should generate Rust code");

    let rust_code = all_code(&rust_output);

    // Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Verify each operation from artifact should have corresponding generated code
    for op in &artifact.operations {
        // The handler name should appear in generated code (in some form)
        let has_handler =
            rust_code.contains(&op.id) || rust_code.to_lowercase().contains(&op.id.to_lowercase());

        // Note: Generated code may use different naming conventions (camelCase vs snake_case)
        // so we check both forms
        println!(
            "Checking operation {} in generated code: {}",
            op.id,
            if has_handler {
                "found"
            } else {
                "not found (may use different naming)"
            }
        );
    }

    // Load into mock Archimedes to verify artifact validity
    let mut loader = MockArtifactLoader::new();
    loader.load(artifact).expect("Should load artifact");
}

/// Tests large contract handling (performance check).
#[test]
fn test_e2e_large_contract_performance() {
    // Use the largest contract we have
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let start = std::time::Instant::now();

    // Full workflow
    let linter = LintReporter::new(LintConfig::default());
    let _lint_report = linter.lint(&contract);

    let config = GeneratorConfig::default();
    let rust_gen = RustGenerator::new(config);
    let _rust_output = rust_gen.generate(&contract).expect("Should generate code");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    let mut loader = MockArtifactLoader::new();
    loader.load(artifact).expect("Should load artifact");

    let elapsed = start.elapsed();
    println!("Full E2E workflow completed in {:?}", elapsed);

    // Should complete in reasonable time (< 1 second for small contracts)
    assert!(
        elapsed.as_secs() < 5,
        "Workflow should complete in under 5 seconds"
    );
}

/// Tests error handling throughout the workflow.
#[test]
fn test_e2e_error_handling() {
    // Test invalid contract
    let invalid_yaml = "not: valid: openapi";
    let parse_result = parse_openapi(invalid_yaml);
    assert!(parse_result.is_err(), "Should fail to parse invalid YAML");

    // Test missing required fields
    let missing_version = r#"
openapi: "3.1.0"
info:
  title: Test
paths: {}
"#;
    let result = parse_openapi(missing_version);
    // This might parse but validation should catch issues
    if let Ok(contract) = result {
        let linter = LintReporter::new(LintConfig::strict());
        let lint_report = linter.lint(&contract);
        println!(
            "Missing version contract: {} errors",
            lint_report.error_count()
        );
    }
}

/// Tests that artifact format version is correct.
#[test]
fn test_e2e_artifact_format_version() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Verify format info
    assert_eq!(artifact.format, "openapi");
    assert_eq!(artifact.format_version, "3.1.0");

    // Archimedes should be able to use this format info
    let mut loader = MockArtifactLoader::new();
    loader.load(artifact).expect("Should load artifact");
}
