//! Contract validation functionality.
//!
//! This module provides functions for validating parsed contracts.

use themis_core::contract::ContractFormat;
use themis_core::Contract;

use crate::error::SdkResult;

/// Validation result with details about any issues found.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the contract is valid.
    pub is_valid: bool,
    /// List of validation errors.
    pub errors: Vec<ValidationIssue>,
    /// List of validation warnings.
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Create a new successful validation result.
    #[must_use]
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a new validation result with errors.
    #[must_use]
    pub fn with_errors(errors: Vec<ValidationIssue>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add a warning to the result.
    pub fn add_warning(&mut self, warning: ValidationIssue) {
        self.warnings.push(warning);
    }

    /// Check if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// A single validation issue.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// The issue message.
    pub message: String,
    /// The location in the contract (path-like).
    pub location: Option<String>,
    /// The severity of the issue.
    pub severity: ValidationSeverity,
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// An error that must be fixed.
    Error,
    /// A warning that should be addressed.
    Warning,
    /// An informational message.
    Info,
}

/// Validate a contract.
///
/// # Arguments
///
/// * `contract` - The contract to validate
///
/// # Returns
///
/// A validation result indicating success or failure with details
///
/// # Errors
///
/// Returns an error if validation cannot be performed
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::validate::validate;
/// use themis_sdk::parse::parse_string;
///
/// let contract = parse_string(yaml)?;
/// let result = validate(&contract)?;
/// if !result.is_valid {
///     for error in &result.errors {
///         eprintln!("Error: {}", error.message);
///     }
/// }
/// ```
pub fn validate(contract: &Contract) -> SdkResult<ValidationResult> {
    validate_with_format(contract, contract.format.clone())
}

/// Validate a contract with a specific format validator.
///
/// # Arguments
///
/// * `contract` - The contract to validate
/// * `format` - The contract format to use for validation
///
/// # Returns
///
/// A validation result indicating success or failure with details
///
/// # Errors
///
/// Returns an error if validation cannot be performed
pub fn validate_with_format(
    contract: &Contract,
    format: ContractFormat,
) -> SdkResult<ValidationResult> {
    // Validate based on format
    let errors = match format {
        ContractFormat::OpenApi => validate_openapi(contract),
        ContractFormat::Protobuf => validate_protobuf(contract),
        ContractFormat::GraphQl => validate_graphql(contract),
        ContractFormat::AsyncApi => validate_asyncapi(contract),
    };

    Ok(ValidationResult::with_errors(errors))
}

/// Validate OpenAPI-specific rules.
fn validate_openapi(contract: &Contract) -> Vec<ValidationIssue> {
    let mut errors = Vec::new();

    // Basic validation rules
    if contract.metadata.service_name.is_empty() {
        errors.push(ValidationIssue {
            message: "Contract name (info.title) is required".to_string(),
            location: Some("info.title".to_string()),
            severity: ValidationSeverity::Error,
        });
    }

    if contract.version.to_string().is_empty() {
        errors.push(ValidationIssue {
            message: "Contract version (info.version) is required".to_string(),
            location: Some("info.version".to_string()),
            severity: ValidationSeverity::Error,
        });
    }

    errors
}

/// Validate Protobuf-specific rules.
fn validate_protobuf(contract: &Contract) -> Vec<ValidationIssue> {
    let mut errors = Vec::new();

    // Check for package name
    if contract.metadata.service_name.is_empty() {
        errors.push(ValidationIssue {
            message: "Package name is required".to_string(),
            location: Some("package".to_string()),
            severity: ValidationSeverity::Error,
        });
    }

    errors
}

/// Validate GraphQL-specific rules.
fn validate_graphql(contract: &Contract) -> Vec<ValidationIssue> {
    let mut errors = Vec::new();

    // Check for at least one type definition
    if contract.schemas.is_empty() {
        errors.push(ValidationIssue {
            message: "At least one type definition is required".to_string(),
            location: Some("types".to_string()),
            severity: ValidationSeverity::Error,
        });
    }

    errors
}

/// Validate AsyncAPI-specific rules.
fn validate_asyncapi(contract: &Contract) -> Vec<ValidationIssue> {
    let mut errors = Vec::new();

    if contract.metadata.service_name.is_empty() {
        errors.push(ValidationIssue {
            message: "Contract name (info.title) is required".to_string(),
            location: Some("info.title".to_string()),
            severity: ValidationSeverity::Error,
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let errors = vec![ValidationIssue {
            message: "test error".to_string(),
            location: Some("test.path".to_string()),
            severity: ValidationSeverity::Error,
        }];
        let result = ValidationResult::with_errors(errors);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::success();
        result.add_warning(ValidationIssue {
            message: "test warning".to_string(),
            location: None,
            severity: ValidationSeverity::Warning,
        });
        assert!(result.is_valid);
        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_severity() {
        assert_eq!(ValidationSeverity::Error, ValidationSeverity::Error);
        assert_ne!(ValidationSeverity::Error, ValidationSeverity::Warning);
    }
}
