//! Protobuf contract validator.
//!
//! Validates protobuf contracts for Themis requirements.

use themis_core::Contract;

use crate::error::Result;

/// Validation result for protobuf contracts.
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

    /// Returns true if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// A validation issue found in the contract.
#[derive(Debug)]
pub struct ValidationIssue {
    /// Issue code.
    pub code: String,
    /// Issue message.
    pub message: String,
    /// Location in the proto file (if available).
    pub location: Option<String>,
}

/// Protobuf contract validator.
#[derive(Debug, Default)]
pub struct ProtoValidator {
    /// Whether to require service definitions.
    require_service: bool,
    /// Whether to require operation IDs on all methods.
    require_operation_ids: bool,
}

impl ProtoValidator {
    /// Creates a new validator with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            require_service: true,
            require_operation_ids: true,
        }
    }

    /// Sets whether to require service definitions.
    #[must_use]
    pub const fn require_service(mut self, require: bool) -> Self {
        self.require_service = require;
        self
    }

    /// Validates a protobuf contract.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ProtobufError`] if validation fails with critical errors.
    pub fn validate(&self, contract: &Contract) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();

        // Check for service/operations
        if self.require_service && contract.operations.is_empty() {
            result.errors.push(ValidationIssue {
                code: "PROTO001".to_string(),
                message: "Contract must have at least one operation (service method)".to_string(),
                location: None,
            });
        }

        // Check operation IDs
        if self.require_operation_ids {
            for (id, op) in &contract.operations {
                if id.is_empty() || op.operation_id.is_empty() {
                    result.errors.push(ValidationIssue {
                        code: "PROTO002".to_string(),
                        message: "All operations must have a non-empty operation_id".to_string(),
                        location: Some(format!("operation: {id}")),
                    });
                }
            }
        }

        // Check for empty schemas
        for (name, _schema) in &contract.schemas {
            if name.is_empty() {
                result.warnings.push(ValidationIssue {
                    code: "PROTO003".to_string(),
                    message: "Schema has empty name".to_string(),
                    location: None,
                });
            }
        }

        // Validate service name
        if contract.metadata.service_name.is_empty() {
            result.errors.push(ValidationIssue {
                code: "PROTO004".to_string(),
                message: "Service name must not be empty".to_string(),
                location: None,
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use std::collections::HashMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::Operation;
    use themis_core::Version;

    fn create_test_contract() -> Contract {
        let mut operations = HashMap::new();
        operations.insert("getUser".to_string(), Operation::new("getUser"));

        Contract {
            format: ContractFormat::Protobuf,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "test-service".to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas: IndexMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_contract() {
        let contract = create_test_contract();
        let validator = ProtoValidator::new();
        let result = validator.validate(&contract).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_empty_operations() {
        let mut contract = create_test_contract();
        contract.operations.clear();

        let validator = ProtoValidator::new();
        let result = validator.validate(&contract).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == "PROTO001"));
    }

    #[test]
    fn test_empty_service_name() {
        let mut contract = create_test_contract();
        contract.metadata.service_name = String::new();

        let validator = ProtoValidator::new();
        let result = validator.validate(&contract).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == "PROTO004"));
    }

    #[test]
    fn test_disable_service_requirement() {
        let mut contract = create_test_contract();
        contract.operations.clear();

        let validator = ProtoValidator::new().require_service(false);
        let result = validator.validate(&contract).unwrap();
        assert!(result.is_valid());
    }
}
