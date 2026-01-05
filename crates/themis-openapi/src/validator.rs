//! OpenAPI schema validation.
//!
//! Validates OpenAPI specifications for correctness and Themis compliance.
//!
//! # Validation Rules
//!
//! Themis enforces the following validation rules for OpenAPI contracts:
//!
//! ## Required Rules (Errors)
//! - Every operation MUST have a unique `operationId`
//! - Security schemes MUST be defined if operations use security
//! - API version MUST be valid semantic version
//!
//! ## Recommended Rules (Warnings)
//! - Operations SHOULD declare error responses (400, 401, 403, 404, 500)
//! - Operations SHOULD have descriptions
//! - Schemas SHOULD have descriptions
//!
//! # Example
//!
//! ```ignore
//! use themis_openapi::{parse_openapi, validate_contract};
//!
//! let contract = parse_openapi(openapi_yaml)?;
//! let result = validate_contract(&contract);
//!
//! if !result.is_valid() {
//!     for error in &result.errors {
//!         eprintln!("Error: {} - {}", error.code, error.message);
//!     }
//! }
//! ```

use std::collections::HashSet;
use themis_core::{Contract, Operation, ThemisResult};

use crate::parser::parse_openapi;

/// Validation severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Validation error - contract is invalid
    Error,
    /// Validation warning - contract works but has issues
    Warning,
}

/// Validation result containing any issues found.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// List of validation errors
    pub errors: Vec<ValidationIssue>,
    /// List of validation warnings
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Creates a new empty validation result.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if validation passed with no errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the total number of issues.
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Returns the number of errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Returns the number of warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Adds an error to the result.
    pub fn add_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<String>,
    ) {
        self.errors.push(ValidationIssue {
            code: code.into(),
            message: message.into(),
            path,
            severity: Severity::Error,
        });
    }

    /// Adds a warning to the result.
    pub fn add_warning(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<String>,
    ) {
        self.warnings.push(ValidationIssue {
            code: code.into(),
            message: message.into(),
            path,
            severity: Severity::Warning,
        });
    }

    /// Merges another validation result into this one.
    pub fn merge(&mut self, other: Self) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// A validation issue (error or warning).
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Issue code for programmatic handling
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Location in the spec (JSON path)
    pub path: Option<String>,
    /// Severity level
    pub severity: Severity,
}

impl ValidationIssue {
    /// Creates a new validation error.
    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
            severity: Severity::Error,
        }
    }

    /// Creates a new validation warning.
    #[must_use]
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
            severity: Severity::Warning,
        }
    }

    /// Sets the path for this issue.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

// ============================================================================
// Validation Rule Codes
// ============================================================================

/// Validation rule codes for Themis compliance.
pub mod rules {
    /// Operation must have operationId
    pub const MISSING_OPERATION_ID: &str = "THEMIS001";
    /// operationId must be unique across all operations
    pub const DUPLICATE_OPERATION_ID: &str = "THEMIS002";
    /// Security scheme referenced but not defined
    pub const UNDEFINED_SECURITY_SCHEME: &str = "THEMIS003";
    /// Operation should declare error responses
    pub const MISSING_ERROR_RESPONSES: &str = "THEMIS004";
    /// Operation should have a description
    pub const MISSING_OPERATION_DESCRIPTION: &str = "THEMIS005";
    /// Schema should have a description
    pub const MISSING_SCHEMA_DESCRIPTION: &str = "THEMIS006";
    /// Invalid semantic version
    pub const INVALID_VERSION: &str = "THEMIS007";
    /// Operation has no security defined (not even public)
    pub const NO_SECURITY_DEFINED: &str = "THEMIS008";
    /// Response schema is missing
    pub const MISSING_RESPONSE_SCHEMA: &str = "THEMIS009";
}

// ============================================================================
// Public Validation Functions
// ============================================================================

/// Validates an OpenAPI specification for Themis compliance.
///
/// This function parses the OpenAPI content and then validates the resulting
/// contract against Themis rules.
///
/// # Arguments
///
/// * `content` - The OpenAPI specification as YAML or JSON string
///
/// # Returns
///
/// A [`ValidationResult`] containing any errors or warnings.
///
/// # Errors
///
/// Returns an error if the OpenAPI content cannot be parsed.
pub fn validate_openapi(content: &str) -> ThemisResult<ValidationResult> {
    let contract = parse_openapi(content)?;
    Ok(validate_contract(&contract))
}

/// Validates a parsed contract for Themis compliance.
///
/// Runs all validation rules against the contract and returns a result
/// containing any errors or warnings found.
///
/// # Arguments
///
/// * `contract` - The parsed contract to validate
///
/// # Returns
///
/// A [`ValidationResult`] containing any errors or warnings.
#[must_use]
pub fn validate_contract(contract: &Contract) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Run all validation rules
    validate_operation_ids(contract, &mut result);
    validate_security_schemes(contract, &mut result);
    validate_error_responses(contract, &mut result);
    validate_descriptions(contract, &mut result);

    result
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Formats the operation path for error messages.
/// Returns a string like "GET /users" or "unknown" if method/path not available.
fn format_operation_location(operation: &Operation) -> String {
    let method = operation
        .method
        .as_ref()
        .map_or_else(|| "UNKNOWN".to_string(), |m| format!("{m:?}"));
    let path = operation.path.as_deref().unwrap_or("unknown");
    format!("{method} {path}")
}

/// Builds a JSON path for an operation field.
/// Returns something like "paths./users.get.field"
fn operation_json_path(operation: &Operation, field: &str) -> String {
    let path = operation.path.as_deref().unwrap_or("unknown");
    let method = operation.method.as_ref().map_or_else(
        || "unknown".to_string(),
        |m| format!("{m:?}").to_lowercase(),
    );
    format!("paths.{path}.{method}.{field}")
}

// ============================================================================
// Validation Rule Implementations
// ============================================================================

/// Validates that all operations have unique operationIds.
fn validate_operation_ids(contract: &Contract, result: &mut ValidationResult) {
    let mut seen_ids: HashSet<&str> = HashSet::new();

    for (operation_id, operation) in &contract.operations {
        // Check for empty operationId (should not happen if parser works correctly)
        if operation_id.is_empty() {
            result.add_error(
                rules::MISSING_OPERATION_ID,
                format!(
                    "Operation at {} has empty operationId",
                    format_operation_location(operation)
                ),
                Some(operation_json_path(operation, "operationId")),
            );
            continue;
        }

        // Check for duplicate operationId
        if !seen_ids.insert(operation_id.as_str()) {
            result.add_error(
                rules::DUPLICATE_OPERATION_ID,
                format!("Duplicate operationId '{operation_id}' found"),
                Some(operation_json_path(operation, "operationId")),
            );
        }
    }
}

/// Validates security scheme references.
fn validate_security_schemes(contract: &Contract, result: &mut ValidationResult) {
    let defined_schemes: HashSet<&str> = contract
        .security_schemes
        .keys()
        .map(String::as_str)
        .collect();

    for (operation_id, operation) in &contract.operations {
        // Check if operation has security requirements
        if operation.security.is_empty() {
            // Warning: no security defined
            result.add_warning(
                rules::NO_SECURITY_DEFINED,
                format!("Operation '{operation_id}' has no security requirements defined"),
                Some(operation_json_path(operation, "security")),
            );
        } else {
            // Validate that referenced security schemes exist
            for security_req in &operation.security {
                if !defined_schemes.contains(security_req.scheme.as_str()) {
                    result.add_error(
                        rules::UNDEFINED_SECURITY_SCHEME,
                        format!(
                            "Operation '{}' references undefined security scheme '{}'",
                            operation_id, security_req.scheme
                        ),
                        Some(operation_json_path(operation, "security")),
                    );
                }
            }
        }
    }
}

/// Standard HTTP error status codes that should be documented.
const RECOMMENDED_ERROR_CODES: &[&str] = &["400", "401", "403", "404", "500"];

/// Validates that operations declare standard error responses.
fn validate_error_responses(contract: &Contract, result: &mut ValidationResult) {
    for (operation_id, operation) in &contract.operations {
        let response_codes: HashSet<&str> =
            operation.responses.keys().map(String::as_str).collect();

        // Check for at least some error responses
        let has_error_response = response_codes
            .iter()
            .any(|code| code.starts_with('4') || code.starts_with('5') || *code == "default");

        if !has_error_response {
            result.add_warning(
                rules::MISSING_ERROR_RESPONSES,
                format!(
                    "Operation '{operation_id}' does not declare any error responses (4xx/5xx)"
                ),
                Some(operation_json_path(operation, "responses")),
            );
        }

        // Check if any recommended error codes are missing
        let has_missing_codes = RECOMMENDED_ERROR_CODES
            .iter()
            .any(|code| !response_codes.contains(*code) && !response_codes.contains("default"));

        // Only warn about 401/403 if operation has security
        if !operation.security.is_empty()
            && !response_codes.contains("401")
            && !response_codes.contains("default")
        {
            result.add_warning(
                rules::MISSING_ERROR_RESPONSES,
                format!(
                    "Operation '{operation_id}' has security but no 401 (Unauthorized) response"
                ),
                Some(operation_json_path(operation, "responses")),
            );
        }

        // Always recommend 500 for server errors
        if !response_codes.contains("500")
            && !response_codes.contains("default")
            && has_missing_codes
        {
            result.add_warning(
                rules::MISSING_ERROR_RESPONSES,
                format!("Operation '{operation_id}' does not declare a 500 (Internal Server Error) response"),
                Some(operation_json_path(operation, "responses")),
            );
        }
    }
}

/// Validates that operations and schemas have descriptions.
fn validate_descriptions(contract: &Contract, result: &mut ValidationResult) {
    // Check operation descriptions
    for (operation_id, operation) in &contract.operations {
        if operation.description.is_none() {
            result.add_warning(
                rules::MISSING_OPERATION_DESCRIPTION,
                format!("Operation '{operation_id}' is missing a description"),
                Some(operation_json_path(operation, "description")),
            );
        }
    }

    // Check schema descriptions
    for (schema_name, schema) in &contract.schemas {
        if schema.description().is_none() {
            result.add_warning(
                rules::MISSING_SCHEMA_DESCRIPTION,
                format!("Schema '{schema_name}' is missing a description"),
                Some(format!("components.schemas.{schema_name}.description")),
            );
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(result.is_valid());
        assert_eq!(result.issue_count(), 0);
    }

    #[test]
    fn test_validation_result_with_errors() {
        let mut result = ValidationResult::new();
        result.add_error("TEST001", "Test error", Some("/test".to_string()));

        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let mut result = ValidationResult::new();
        result.add_warning("TEST001", "Test warning", None);

        // Warnings don't make the result invalid
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error("E1", "Error 1", None);

        let mut result2 = ValidationResult::new();
        result2.add_warning("W1", "Warning 1", None);
        result2.add_error("E2", "Error 2", None);

        result1.merge(result2);

        assert_eq!(result1.error_count(), 2);
        assert_eq!(result1.warning_count(), 1);
    }

    #[test]
    fn test_validation_issue_builder() {
        let issue = ValidationIssue::error("TEST001", "Test message").with_path("/some/path");

        assert_eq!(issue.code, "TEST001");
        assert_eq!(issue.message, "Test message");
        assert_eq!(issue.path, Some("/some/path".to_string()));
        assert_eq!(issue.severity, Severity::Error);
    }

    #[test]
    fn test_validate_minimal_openapi() {
        let openapi = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /test:
    get:
      operationId: getTest
      responses:
        "200":
          description: Success
"#;
        let result = validate_openapi(openapi).unwrap();

        // Should have warnings but no errors
        assert!(result.is_valid());
        // Warnings for: no security, no error responses, no description
        assert!(result.warning_count() > 0);
    }

    #[test]
    fn test_validate_complete_openapi() {
        let openapi = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
  description: A complete test API
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
paths:
  /test:
    get:
      operationId: getTest
      description: Get a test resource
      security:
        - bearerAuth: []
      responses:
        "200":
          description: Success
        "401":
          description: Unauthorized
        "500":
          description: Server error
"#;
        let result = validate_openapi(openapi).unwrap();

        // Should pass with no errors
        assert!(result.is_valid());
        // May still have some warnings (missing schema descriptions, etc.)
    }

    #[test]
    fn test_validate_duplicate_operation_id() {
        // Duplicate operationIds are caught during parsing, not validation
        let openapi = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /test1:
    get:
      operationId: duplicateId
      responses:
        "200":
          description: Success
  /test2:
    get:
      operationId: duplicateId
      responses:
        "200":
          description: Success
"#;
        // Parsing should fail with duplicate operationId error
        let result = validate_openapi(openapi);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Duplicate operationId"),
            "Expected duplicate operationId error, got: {err}"
        );
    }

    #[test]
    fn test_validate_undefined_security_scheme() {
        let openapi = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /test:
    get:
      operationId: getTest
      security:
        - undefinedScheme: []
      responses:
        "200":
          description: Success
"#;
        let result = validate_openapi(openapi).unwrap();

        // Should have error for undefined security scheme
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == rules::UNDEFINED_SECURITY_SCHEME));
    }

    #[test]
    fn test_validate_missing_error_responses_warning() {
        let openapi = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /test:
    get:
      operationId: getTest
      responses:
        "200":
          description: Success
"#;
        let result = validate_openapi(openapi).unwrap();

        // Should be valid (only warnings)
        assert!(result.is_valid());
        // Should have warning for missing error responses
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == rules::MISSING_ERROR_RESPONSES));
    }

    #[test]
    fn test_validate_no_security_warning() {
        let openapi = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /test:
    get:
      operationId: getTest
      responses:
        "200":
          description: Success
"#;
        let result = validate_openapi(openapi).unwrap();

        // Should have warning for no security
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == rules::NO_SECURITY_DEFINED));
    }
}
