//! GraphQL contract validator.
//!
//! Validates GraphQL contracts for Themis requirements.

use themis_core::Contract;

use crate::error::Result;

/// Validation result for GraphQL contracts.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Errors that prevent the contract from being used.
    pub errors: Vec<ValidationIssue>,
    /// Warnings that don't prevent usage but should be addressed.
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Returns true if validation passed (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the total number of issues.
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// A validation issue (error or warning).
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Rule code (e.g., "GQL001").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Location in the schema.
    pub location: Option<String>,
}

/// GraphQL contract validator.
#[derive(Debug)]
pub struct GraphqlValidator {
    /// Whether a Query type is required.
    require_query_type: bool,
    /// Whether descriptions are required on types.
    require_descriptions: bool,
    /// Whether deprecated fields must have a reason.
    require_deprecation_reason: bool,
}

impl Default for GraphqlValidator {
    fn default() -> Self {
        Self {
            require_query_type: true,
            require_descriptions: false,
            require_deprecation_reason: true,
        }
    }
}

impl GraphqlValidator {
    /// Creates a new validator with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether a Query type is required.
    #[must_use]
    pub const fn require_query_type(mut self, require: bool) -> Self {
        self.require_query_type = require;
        self
    }

    /// Sets whether descriptions are required on types.
    #[must_use]
    pub const fn require_descriptions(mut self, require: bool) -> Self {
        self.require_descriptions = require;
        self
    }

    /// Validates a GraphQL contract.
    ///
    /// # Errors
    ///
    /// Returns an error if validation cannot be performed.
    pub fn validate(&self, contract: &Contract) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();

        // GQL001: Check for operations
        if contract.operations.is_empty() && self.require_query_type {
            result.errors.push(ValidationIssue {
                code: "GQL001".to_string(),
                message: "No operations defined. GraphQL schema requires at least a Query type."
                    .to_string(),
                location: None,
            });
        }

        // GQL002: Check for empty service name
        if contract.metadata.service_name.is_empty() {
            result.errors.push(ValidationIssue {
                code: "GQL002".to_string(),
                message: "Service name cannot be empty".to_string(),
                location: None,
            });
        }

        // GQL003: Check for schema description
        if self.require_descriptions && contract.metadata.description.is_none() {
            result.warnings.push(ValidationIssue {
                code: "GQL003".to_string(),
                message: "Schema should have a description".to_string(),
                location: None,
            });
        }

        // GQL004: Check for deprecated operations without reason
        if self.require_deprecation_reason {
            for (op_id, op) in &contract.operations {
                if op.deprecated && op.description.is_none() {
                    result.warnings.push(ValidationIssue {
                        code: "GQL004".to_string(),
                        message: format!(
                            "Deprecated operation '{op_id}' should have a deprecation reason"
                        ),
                        location: Some(format!("operation:{op_id}")),
                    });
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::Version;

    fn create_test_contract(
        service_name: &str,
        operation_count: usize,
    ) -> Contract {
        let mut operations = HashMap::new();
        for i in 0..operation_count {
            operations.insert(
                format!("query{i}"),
                themis_core::operation::Operation {
                    operation_id: format!("query{i}"),
                    summary: Some(format!("Query {i}")),
                    description: None,
                    method: None,
                    path: None,
                    parameters: Vec::new(),
                    request_body: None,
                    responses: HashMap::new(),
                    security: Vec::new(),
                    deprecated: false,
                    tags: Vec::new(),
                    themis_metadata: None,
                },
            );
        }

        Contract {
            format: ContractFormat::GraphQl,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: service_name.to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas: indexmap::IndexMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_contract() {
        let contract = create_test_contract("test-service", 1);
        let validator = GraphqlValidator::new();
        let result = validator.validate(&contract).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_empty_operations() {
        let contract = create_test_contract("test-service", 0);
        let validator = GraphqlValidator::new();
        let result = validator.validate(&contract).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == "GQL001"));
    }

    #[test]
    fn test_empty_service_name() {
        let contract = create_test_contract("", 1);
        let validator = GraphqlValidator::new();
        let result = validator.validate(&contract).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == "GQL002"));
    }

    #[test]
    fn test_disable_query_requirement() {
        let contract = create_test_contract("test-service", 0);
        let validator = GraphqlValidator::new().require_query_type(false);
        let result = validator.validate(&contract).unwrap();
        assert!(result.is_valid());
    }
}
