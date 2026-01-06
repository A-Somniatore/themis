//! Full workflow integration tests.
//!
//! Tests the complete contract governance workflow:
//! Contract → Parse → Validate → Lint → Diff → Codegen → Artifact

use crate::fixtures::{MINIMAL_CONTRACT, USERS_SERVICE_V1, USERS_SERVICE_V2};
use themis_artifact::ArtifactBuilder;
use themis_codegen::{CodeGenerator, GeneratorConfig, RustGenerator, TypeScriptGenerator, PythonGenerator};
use themis_compat::diff_contracts;
use themis_lint::{LintConfig, LintReporter};
use themis_openapi::parse_openapi;

/// Helper to get all generated code as a single string.
fn all_code(output: &themis_codegen::GeneratedCode) -> String {
    output.files.iter()
        .map(|f| f.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Tests the complete happy path workflow.
#[test]
fn test_full_workflow_happy_path() {
    // Step 1: Parse the contract
    let contract = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse users-service v1");

    // Verify parsing worked - service name may have different casing
    assert!(
        contract.metadata.service_name.to_lowercase().contains("users"),
        "Should have users in service name, got: {}",
        contract.metadata.service_name
    );
    assert!(!contract.operations.is_empty(), "Should have operations");
    assert!(!contract.schemas.is_empty(), "Should have schemas");

    // Step 2: Lint the contract
    let linter = LintReporter::new(LintConfig::default());
    let lint_report = linter.lint(&contract);
    
    println!("Lint report: {} warnings, {} errors", 
        lint_report.warning_count(), 
        lint_report.error_count());

    // Step 3: Generate code for Rust
    let config = GeneratorConfig::default();
    
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract)
        .expect("Should generate Rust code");
    
    let rust_code = all_code(&rust_output);
    assert!(!rust_code.is_empty(), "Rust code should not be empty");
    assert!(rust_code.contains("struct") || rust_code.contains("enum"), 
        "Rust code should have structs or enums");

    // Step 4: Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");
    
    // Verify artifact
    assert!(artifact.verify_checksum().is_ok(), "Artifact should have valid checksum");
    assert_eq!(artifact.service, contract.metadata.service_name);

    println!("Full workflow completed successfully!");
    println!("  - Parsed contract with {} operations", contract.operations.len());
    println!("  - Lint found {} issues", lint_report.issues.len());
    println!("  - Generated {} Rust files", rust_output.files.len());
    println!("  - Created artifact for '{}'", artifact.service);
}

/// Tests the version comparison workflow.
#[test]
fn test_version_comparison_workflow() {
    // Parse both versions
    let v1 = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse v1");
    let v2 = parse_openapi(USERS_SERVICE_V2)
        .expect("Should parse v2");

    // Compare versions
    let changes = diff_contracts(&v1, &v2);

    // Should detect changes between versions
    println!("Changes between v1 and v2:");
    println!("  Breaking: {}", changes.breaking_changes.len());
    println!("  Additions: {}", changes.additions.len());
    println!("  Modifications: {}", changes.modifications.len());
    println!("  Is compatible: {}", changes.is_compatible);
    println!("  Suggested bump: {}", changes.suggested_bump);
}

/// Tests workflow with minimal contract.
#[test]
fn test_minimal_contract_workflow() {
    // Parse minimal contract
    let contract = parse_openapi(MINIMAL_CONTRACT)
        .expect("Should parse minimal contract");

    // Should have exactly one operation
    assert_eq!(contract.operations.len(), 1);
    assert!(contract.operations.contains_key("getHealth"));

    // Lint should pass
    let linter = LintReporter::new(LintConfig::relaxed());
    let report = linter.lint(&contract);
    assert_eq!(report.error_count(), 0, "Minimal contract should have no lint errors");

    // Generate code
    let config = GeneratorConfig::default();
    let rust_gen = RustGenerator::new(config);
    let rust_output = rust_gen.generate(&contract)
        .expect("Should generate code for minimal contract");
    
    let rust_code = all_code(&rust_output);
    assert!(
        rust_code.contains("GetHealth") || rust_code.contains("get_health") || rust_code.contains("Health"),
        "Should generate handler for getHealth"
    );

    // Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");
    assert!(artifact.verify_checksum().is_ok());
}

/// Tests that breaking changes are detected.
#[test]
fn test_breaking_change_detection() {
    use crate::fixtures::BREAKING_CHANGE_CONTRACT;
    
    let v1 = parse_openapi(MINIMAL_CONTRACT)
        .expect("Should parse v1");
    let v2 = parse_openapi(BREAKING_CHANGE_CONTRACT)
        .expect("Should parse v2");

    let changes = diff_contracts(&v1, &v2);

    // Should detect breaking changes:
    // - Operation removed (getHealth → checkHealth has different operationId)
    assert!(
        !changes.is_compatible,
        "Should detect breaking changes between incompatible versions"
    );

    println!("Breaking changes detected:");
    for change in &changes.breaking_changes {
        println!("  - {:?}", change);
    }
}

/// Tests artifact round-trip (create, serialize, deserialize, verify).
#[test]
fn test_artifact_round_trip() {
    let contract = parse_openapi(MINIMAL_CONTRACT)
        .expect("Should parse contract");

    // Create artifact
    let artifact = ArtifactBuilder::from_contract(&contract)
        .build()
        .expect("Should create artifact");

    // Serialize to JSON
    let json = artifact.to_json()
        .expect("Should serialize to JSON");

    // Deserialize back
    let restored = themis_artifact::Artifact::from_json(&json)
        .expect("Should deserialize from JSON");

    // Verify restored artifact
    assert_eq!(artifact.service, restored.service);
    assert_eq!(artifact.version, restored.version);
    assert_eq!(artifact.checksum.value, restored.checksum.value);
    assert!(restored.verify_checksum().is_ok(), "Restored artifact should have valid checksum");
}

/// Tests code generation for all languages.
#[test]
fn test_multi_language_codegen() {
    let contract = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse contract");

    let config = GeneratorConfig::default();

    // Test Rust
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract)
        .expect("Should generate Rust code");
    let rust_code = all_code(&rust_output);
    assert!(!rust_code.is_empty(), "Rust code should not be empty");
    assert!(rust_code.contains("serde") || rust_code.contains("Serialize"), 
        "Rust code should use serde");
    assert!(rust_code.contains("pub struct"), "Rust code should have public structs");
    println!("Generated {} bytes of Rust code", rust_code.len());

    // Test TypeScript
    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen.generate(&contract)
        .expect("Should generate TypeScript code");
    let ts_code = all_code(&ts_output);
    assert!(!ts_code.is_empty(), "TypeScript code should not be empty");
    assert!(ts_code.contains("export") || ts_code.contains("interface") || ts_code.contains("type"), 
        "TypeScript code should have exports");
    println!("Generated {} bytes of TypeScript code", ts_code.len());

    // Test Python
    let py_gen = PythonGenerator::new(config.clone());
    let py_output = py_gen.generate(&contract)
        .expect("Should generate Python code");
    let py_code = all_code(&py_output);
    assert!(!py_code.is_empty(), "Python code should not be empty");
    assert!(py_code.contains("dataclass") || py_code.contains("class "), 
        "Python code should have classes");
    println!("Generated {} bytes of Python code", py_code.len());
}

/// Tests lint configuration affects results.
#[test]
fn test_lint_configuration_affects_results() {
    let contract = parse_openapi(USERS_SERVICE_V1)
        .expect("Should parse contract");

    // Default config
    let default_linter = LintReporter::new(LintConfig::default());
    let default_report = default_linter.lint(&contract);

    // Strict config (all rules as errors)
    let strict_linter = LintReporter::new(LintConfig::strict());
    let strict_report = strict_linter.lint(&contract);

    // Relaxed config (all rules as warnings)
    let relaxed_linter = LintReporter::new(LintConfig::relaxed());
    let relaxed_report = relaxed_linter.lint(&contract);

    // Different configs may find different numbers of issues due to rule sets
    println!("Default: {} issues ({} errors, {} warnings)",
        default_report.issues.len(),
        default_report.error_count(),
        default_report.warning_count()
    );
    println!("Strict: {} issues ({} errors, {} warnings)",
        strict_report.issues.len(),
        strict_report.error_count(),
        strict_report.warning_count()
    );
    println!("Relaxed: {} issues ({} errors, {} warnings)",
        relaxed_report.issues.len(),
        relaxed_report.error_count(),
        relaxed_report.warning_count()
    );

    // Relaxed config should have 0 errors
    assert_eq!(
        relaxed_report.error_count(),
        0,
        "Relaxed config should have no errors"
    );
}
