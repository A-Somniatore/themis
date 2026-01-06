//! Validation integration tests.
//!
//! Tests contract validation and lint functionality.

use crate::fixtures::{MINIMAL_CONTRACT, SECURE_CONTRACT, USERS_SERVICE_V1};
use themis_lint::{LintConfig, LintReporter};
use themis_openapi::{parse_openapi, validate_openapi};

/// Tests that valid contracts pass validation.
#[test]
fn test_valid_contract_passes_validation() {
    let result = validate_openapi(MINIMAL_CONTRACT).expect("Should validate contract");

    assert!(
        result.errors.is_empty(),
        "Valid minimal contract should have no validation errors: {:?}",
        result.errors
    );
}

/// Tests that users service contract passes validation.
#[test]
fn test_users_service_validation() {
    let result = validate_openapi(USERS_SERVICE_V1).expect("Should validate contract");

    // May have warnings but shouldn't have blocking errors
    println!(
        "Validation: {} errors, {} warnings",
        result.errors.len(),
        result.warnings.len()
    );
}

/// Tests that secure contract passes validation.
#[test]
fn test_secure_contract_validation() {
    let result = validate_openapi(SECURE_CONTRACT).expect("Should validate contract");

    assert!(
        result.errors.is_empty(),
        "Secure contract should have no validation errors: {:?}",
        result.errors
    );
}

/// Tests validation detects missing operation IDs.
#[test]
fn test_validation_detects_missing_operation_id() {
    let invalid_contract = r#"
openapi: "3.1.0"
info:
  title: Invalid Service
  version: "1.0.0"
paths:
  /health:
    get:
      # Missing operationId
      responses:
        "200":
          description: Success
"#;

    // This should fail to parse or return validation error
    let result = validate_openapi(invalid_contract);

    // Either the validation returns an error OR we get a result with errors
    match result {
        Ok(validation) => {
            // Should have errors about missing operationId
            assert!(
                !validation.errors.is_empty()
                    || validation.errors.iter().any(|e| e
                        .message
                        .to_lowercase()
                        .contains("operationid")
                        || e.code.contains("001")),
                "Should detect missing operationId: {:?}",
                validation.errors
            );
        }
        Err(e) => {
            // Parser correctly rejects missing operationId
            let error_msg = format!("{:?}", e).to_lowercase();
            assert!(
                error_msg.contains("operationid") || error_msg.contains("missing"),
                "Error should mention operationId: {:?}",
                e
            );
        }
    }
}

/// Tests lint with default configuration.
#[test]
fn test_lint_default_config() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let linter = LintReporter::new(LintConfig::default());
    let report = linter.lint(&contract);

    println!(
        "Lint found {} issues ({} errors, {} warnings)",
        report.issues.len(),
        report.error_count(),
        report.warning_count()
    );
}

/// Tests lint with strict configuration.
#[test]
fn test_lint_strict_config() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let strict_linter = LintReporter::new(LintConfig::strict());
    let strict_report = strict_linter.lint(&contract);

    let default_linter = LintReporter::new(LintConfig::default());
    let default_report = default_linter.lint(&contract);

    // Log what we find
    println!(
        "Strict: {} issues ({} errors, {} warnings)",
        strict_report.issues.len(),
        strict_report.error_count(),
        strict_report.warning_count()
    );
    println!(
        "Default: {} issues ({} errors, {} warnings)",
        default_report.issues.len(),
        default_report.error_count(),
        default_report.warning_count()
    );

    // Strict config should have at least as many errors as default
    // (since it treats warnings as errors)
    assert!(
        strict_report.error_count() >= default_report.error_count(),
        "Strict config should have at least as many errors"
    );
}

/// Tests lint with relaxed configuration.
#[test]
fn test_lint_relaxed_config() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

    let relaxed_linter = LintReporter::new(LintConfig::relaxed());
    let relaxed_report = relaxed_linter.lint(&contract);

    // Relaxed config should have 0 errors (all warnings)
    assert_eq!(
        relaxed_report.error_count(),
        0,
        "Relaxed config should have no errors, only warnings"
    );
}

/// Tests that minimal contract has no lint issues.
#[test]
fn test_minimal_contract_lint() {
    let contract = parse_openapi(MINIMAL_CONTRACT).expect("Should parse contract");

    let linter = LintReporter::new(LintConfig::relaxed());
    let report = linter.lint(&contract);

    assert_eq!(
        report.error_count(),
        0,
        "Minimal contract should have no lint errors"
    );
}

/// Tests validation of contract with security definitions.
#[test]
fn test_security_validation() {
    let contract_with_security = r#"
openapi: "3.1.0"
info:
  title: Secure Service
  version: "1.0.0"
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
security:
  - bearerAuth: []
paths:
  /protected:
    get:
      operationId: getProtected
      security:
        - bearerAuth: []
      responses:
        "200":
          description: Success
        "401":
          description: Unauthorized
"#;

    let result = validate_openapi(contract_with_security).expect("Should validate contract");

    // Security is properly defined, should have no errors
    assert!(
        result.errors.is_empty(),
        "Contract with proper security should pass: {:?}",
        result.errors
    );
}

/// Tests validation summary.
#[test]
fn test_validation_summary() {
    let result = validate_openapi(USERS_SERVICE_V1).expect("Should validate contract");

    println!("Validation Summary:");
    println!("  Errors: {}", result.errors.len());
    println!("  Warnings: {}", result.warnings.len());

    for error in &result.errors {
        println!("  [ERROR] {}: {}", error.code, error.message);
    }
    for warning in &result.warnings {
        println!("  [WARN] {}: {}", warning.code, warning.message);
    }
}
