//! Protobuf-specific lint rules.
//!
//! These rules check Protobuf-specific best practices:
//! - Package names should use lowercase with dots
//! - Service names should follow naming conventions

use crate::reporter::LintIssue;
use crate::rule::{Rule, RuleConfig};
use themis_core::Contract;

/// Checks that Protobuf package names follow the naming convention.
///
/// Package names should use lowercase with dot separators (e.g., `com.example.api`)
/// This rule is only applied when the contract is in Protobuf format.
///
/// # Examples
///
/// Valid: `com.example.users`, `google.api`, `acme.v1`
/// Invalid: `Com.Example.Users`, `google_api`, `ACME`
pub struct ProtobufPackageName;

impl Rule for ProtobufPackageName {
    fn id(&self) -> &'static str {
        "protobuf/package-name"
    }

    fn description(&self) -> &'static str {
        "Protobuf package names should be lowercase with dots (e.g., com.example.api)"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Extract package name from contract metadata service name
        let service_name = &contract.metadata.service_name;
        if is_valid_package_name(service_name) {
            return issues;
        }

        // If the name doesn't follow conventions, report an issue
        if !service_name.chars().all(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-') {
            issues.push(LintIssue {
                rule: self.id().to_string(),
                severity: config.severity,
                message: format!(
                    "Package name '{service_name}' should be lowercase with dots (e.g., 'com.example.service')"
                ),
                location: Some(format!("package: {service_name}")),
            });
        } else if service_name.contains('_') {
            issues.push(LintIssue {
                rule: self.id().to_string(),
                severity: config.severity,
                message: format!(
                    "Package name '{service_name}' should use dots instead of underscores (e.g., 'com.example.service')"
                ),
                location: Some(format!("package: {service_name}")),
            });
        }

        issues
    }
}

/// Checks that Protobuf service names follow naming conventions.
///
/// Service names should be `PascalCase` and descriptive (e.g., `UserService`, `OrderAPI`)
///
/// # Examples
///
/// Valid: `UserService`, `OrderAPI`, `AuthenticationService`
/// Invalid: `userService`, `user_service`, `USER_SERVICE`
pub struct ProtobufServiceName;

impl Rule for ProtobufServiceName {
    fn id(&self) -> &'static str {
        "protobuf/service-name"
    }

    fn description(&self) -> &'static str {
        "Protobuf service names should be PascalCase"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // For each operation (which represents a service method in Protobuf)
        // we can infer the service name from the operation ID
        // In Protobuf, the service is often encoded as Service.Method format

        for op_id in contract.operations.keys() {
            // Service methods are typically formatted as ServiceName/MethodName
            // Check if the prefix looks like a service name
            let parts: Vec<&str> = op_id.split('/').collect();
            if parts.len() == 2 {
                let service_name = parts[0];
                if !is_pascal_case(service_name) {
                    issues.push(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Service name '{service_name}' should be PascalCase (e.g., 'UserService')"
                        ),
                        location: Some(op_id.clone()),
                    });
                }
            }
        }

        issues
    }
}

/// Returns all Protobuf-specific lint rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ProtobufPackageName),
        Box::new(ProtobufServiceName),
    ]
}

/// Checks if a string is a valid Protobuf package name.
/// Valid: lowercase, digits, dots (e.g., com.example.api.v1)
fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Package name should contain at least one dot
    if !name.contains('.') {
        return false;
    }

    // All characters should be lowercase letters, digits, or dots
    name.chars().all(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')
}

/// Checks if a string is in `PascalCase` (starts with uppercase, no underscores).
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start with uppercase
    if !s.chars().next().unwrap().is_ascii_uppercase() {
        return false;
    }

    // No underscores allowed
    !s.contains('_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::{Contract, Operation, contract::{ContractFormat, ContractMetadata}};

    fn create_test_contract(service_name: &str) -> Contract {
        Contract {
            format: ContractFormat::Protobuf,
            version: "1.0.0".parse().unwrap(),
            metadata: ContractMetadata {
                service_name: service_name.to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: Default::default(),
            schemas: Default::default(),
            security_schemes: Default::default(),
        }
    }

    #[test]
    fn test_valid_package_names() {
        let valid_names = vec![
            "com.example.api",
            "google.api.v1",
            "mycompany.service.users",
        ];

        for name in valid_names {
            assert!(
                is_valid_package_name(name),
                "Expected {} to be valid",
                name
            );
        }
    }

    #[test]
    fn test_invalid_package_names() {
        let invalid_names = vec![
            "ComExample",
            "com_example",
            "Com.Example.Api",
            "api",
        ];

        for name in invalid_names {
            assert!(
                !is_valid_package_name(name),
                "Expected {} to be invalid",
                name
            );
        }
    }

    #[test]
    fn test_package_name_rule_valid() {
        let rule = ProtobufPackageName;
        let contract = create_test_contract("com.example.api");
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_package_name_rule_invalid_uppercase() {
        let rule = ProtobufPackageName;
        let contract = create_test_contract("Com.Example.Api");
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_package_name_rule_invalid_underscore() {
        let rule = ProtobufPackageName;
        let contract = create_test_contract("com_example_api");
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_pascal_case_validation() {
        assert!(is_pascal_case("UserService"));
        assert!(is_pascal_case("AuthAPI"));
        assert!(!is_pascal_case("userService"));
        assert!(!is_pascal_case("user_service"));
        assert!(!is_pascal_case("USER_SERVICE"));
    }

    #[test]
    fn test_service_name_rule() {
        let mut contract = create_test_contract("com.example.api");
        contract.operations.insert(
            "UserService/GetUser".to_string(),
            Operation::new("UserService/GetUser"),
        );

        let rule = ProtobufServiceName;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_service_name_rule_invalid() {
        let mut contract = create_test_contract("com.example.api");
        contract.operations.insert(
            "userService/GetUser".to_string(),
            Operation::new("userService/GetUser"),
        );

        let rule = ProtobufServiceName;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_all_protobuf_rules() {
        let rules = all_rules();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id(), "protobuf/package-name");
        assert_eq!(rules[1].id(), "protobuf/service-name");
    }
}
