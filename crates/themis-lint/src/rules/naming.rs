//! Naming convention lint rules.
//!
//! These rules check that identifiers follow consistent naming conventions:
//! - Operation IDs should be camelCase
//! - URL paths should be kebab-case
//! - Schema names should be `PascalCase`

use crate::reporter::LintIssue;
use crate::rule::{Rule, RuleConfig};
use themis_core::Contract;

/// Checks that operation IDs use camelCase naming.
///
/// # Examples
///
/// Valid: `getUser`, `createOrder`, `deleteItem`
/// Invalid: `GetUser`, `create_order`, `delete-item`
pub struct OperationIdCamelCase;

impl Rule for OperationIdCamelCase {
    fn id(&self) -> &'static str {
        "naming/operation-id"
    }

    fn description(&self) -> &'static str {
        "Operation IDs should use camelCase naming"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .operations
            .iter()
            .filter_map(|(id, op)| {
                if is_camel_case(id) {
                    None
                } else {
                    Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Operation ID '{id}' should be camelCase (e.g., '{}')",
                            to_camel_case(id)
                        ),
                        location: op.path.clone(),
                    })
                }
            })
            .collect()
    }
}

/// Checks that URL paths use kebab-case for path segments.
///
/// # Examples
///
/// Valid: `/users`, `/user-profiles`, `/api/v1/order-items`
/// Invalid: `/userProfiles`, `/order_items`, `/OrderItems`
pub struct PathKebabCase;

impl Rule for PathKebabCase {
    fn id(&self) -> &'static str {
        "naming/path-format"
    }

    fn description(&self) -> &'static str {
        "URL path segments should use kebab-case naming"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .operations
            .iter()
            .filter_map(|(id, op)| {
                let path = op.path.as_ref()?;
                if is_kebab_case_path(path) {
                    None
                } else {
                    Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Path '{path}' for operation '{id}' should use kebab-case segments (e.g., '{}')",
                            to_kebab_case_path(path)
                        ),
                        location: Some(path.clone()),
                    })
                }
            })
            .collect()
    }
}

/// Checks that schema names use `PascalCase` naming.
///
/// # Examples
///
/// Valid: `User`, `OrderItem`, `ApiResponse`
/// Invalid: `user`, `orderItem`, `api_response`
pub struct SchemaNamePascalCase;

impl Rule for SchemaNamePascalCase {
    fn id(&self) -> &'static str {
        "naming/schema-name"
    }

    fn description(&self) -> &'static str {
        "Schema names should use PascalCase naming"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .schemas
            .keys()
            .filter_map(|name| {
                if is_pascal_case(name) {
                    None
                } else {
                    Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Schema name '{name}' should be PascalCase (e.g., '{}')",
                            to_pascal_case(name)
                        ),
                        location: Some(format!("#/components/schemas/{name}")),
                    })
                }
            })
            .collect()
    }
}

/// Checks if a string is camelCase.
///
/// A string is camelCase if:
/// - It starts with a lowercase letter
/// - It contains only alphanumeric characters
/// - Word boundaries are marked by uppercase letters
fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be lowercase
    let first = match chars.next() {
        Some(c) if c.is_ascii_lowercase() => c,
        _ => return false,
    };

    // Rest must be alphanumeric (no underscores or hyphens)
    if !first.is_ascii_lowercase() {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() {
            return false;
        }
    }

    // No consecutive uppercase letters (would indicate ALLCAPS segment)
    !has_consecutive_uppercase(s)
}

/// Checks for consecutive uppercase letters.
fn has_consecutive_uppercase(s: &str) -> bool {
    let mut prev_upper = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if prev_upper {
                return true;
            }
            prev_upper = true;
        } else {
            prev_upper = false;
        }
    }
    false
}

/// Converts a string to camelCase.
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    let mut first = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if first {
            result.push(c.to_ascii_lowercase());
            first = false;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Checks if a path uses kebab-case for its segments.
///
/// Path parameters (e.g., `{userId}`) are allowed to use camelCase.
fn is_kebab_case_path(path: &str) -> bool {
    for segment in path.split('/') {
        if segment.is_empty() {
            continue;
        }

        // Skip path parameters like {userId}
        if segment.starts_with('{') && segment.ends_with('}') {
            continue;
        }

        // Check that segment is kebab-case
        if !is_kebab_case_segment(segment) {
            return false;
        }
    }
    true
}

/// Checks if a segment is kebab-case (lowercase letters and hyphens only).
fn is_kebab_case_segment(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start and end with lowercase letter or digit
    let first = s.chars().next().unwrap();
    let last = s.chars().next_back().unwrap();

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }

    if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
        return false;
    }

    // Only lowercase letters, digits, and hyphens
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Converts a path to kebab-case.
fn to_kebab_case_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if segment.is_empty() || (segment.starts_with('{') && segment.ends_with('}')) {
                segment.to_string()
            } else {
                to_kebab_case(segment)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Converts a single string to kebab-case.
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == ' ' {
            result.push('-');
        } else if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Checks if a string is `PascalCase`.
///
/// A string is `PascalCase` if:
/// - It starts with an uppercase letter
/// - It contains only alphanumeric characters
/// - Word boundaries are marked by uppercase letters
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }

    // Rest must be alphanumeric (no underscores or hyphens)
    for c in s.chars() {
        if !c.is_ascii_alphanumeric() {
            return false;
        }
    }

    // No consecutive uppercase letters
    !has_consecutive_uppercase(s)
}

/// Converts a string to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Returns all naming convention rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(OperationIdCamelCase),
        Box::new(PathKebabCase),
        Box::new(SchemaNamePascalCase),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::Operation;
    use themis_core::schema::{ObjectSchema, Schema};
    use themis_core::Version;

    fn create_test_contract() -> Contract {
        Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "test-service".to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: HashMap::new(),
            schemas: HashMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    // ===== camelCase tests =====

    #[test]
    fn test_is_camel_case_valid() {
        assert!(is_camel_case("getUser"));
        assert!(is_camel_case("createOrder"));
        assert!(is_camel_case("deleteOrderItem"));
        assert!(is_camel_case("get"));
        assert!(is_camel_case("listUsers2"));
    }

    #[test]
    fn test_is_camel_case_invalid() {
        assert!(!is_camel_case("GetUser")); // PascalCase
        assert!(!is_camel_case("get_user")); // snake_case
        assert!(!is_camel_case("get-user")); // kebab-case
        assert!(!is_camel_case("GETUSER")); // ALL CAPS
        assert!(!is_camel_case("getUSER")); // has consecutive uppercase
        assert!(!is_camel_case("")); // empty
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("get_user"), "getUser");
        assert_eq!(to_camel_case("GetUser"), "getUser");
        assert_eq!(to_camel_case("get-user"), "getUser");
        assert_eq!(to_camel_case("get user"), "getUser");
    }

    #[test]
    fn test_operation_id_camel_case_rule() {
        let mut contract = create_test_contract();
        contract
            .operations
            .insert("get_user".to_string(), Operation::new("get_user"));
        contract
            .operations
            .insert("createOrder".to_string(), Operation::new("createOrder"));

        let rule = OperationIdCamelCase;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("get_user"));
    }

    #[test]
    fn test_operation_id_camel_case_disabled() {
        let mut contract = create_test_contract();
        contract
            .operations
            .insert("get_user".to_string(), Operation::new("get_user"));

        let rule = OperationIdCamelCase;
        let config = RuleConfig::disabled();
        let issues = rule.check(&contract, &config);

        assert!(issues.is_empty());
    }

    // ===== kebab-case path tests =====

    #[test]
    fn test_is_kebab_case_path_valid() {
        assert!(is_kebab_case_path("/users"));
        assert!(is_kebab_case_path("/user-profiles"));
        assert!(is_kebab_case_path("/api/v1/order-items"));
        assert!(is_kebab_case_path("/users/{userId}"));
        assert!(is_kebab_case_path("/users/{userId}/orders"));
    }

    #[test]
    fn test_is_kebab_case_path_invalid() {
        assert!(!is_kebab_case_path("/userProfiles")); // camelCase
        assert!(!is_kebab_case_path("/order_items")); // snake_case
        assert!(!is_kebab_case_path("/OrderItems")); // PascalCase
        assert!(!is_kebab_case_path("/USERS")); // ALL CAPS
    }

    #[test]
    fn test_to_kebab_case_path() {
        assert_eq!(to_kebab_case_path("/userProfiles"), "/user-profiles");
        assert_eq!(to_kebab_case_path("/order_items"), "/order-items");
        assert_eq!(
            to_kebab_case_path("/users/{userId}/OrderItems"),
            "/users/{userId}/order-items"
        );
    }

    #[test]
    fn test_path_kebab_case_rule() {
        let mut contract = create_test_contract();

        let mut op1 = Operation::new("getUser");
        op1.path = Some("/userProfiles".to_string());
        contract.operations.insert("getUser".to_string(), op1);

        let mut op2 = Operation::new("listOrders");
        op2.path = Some("/orders".to_string());
        contract.operations.insert("listOrders".to_string(), op2);

        let rule = PathKebabCase;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("/userProfiles"));
    }

    // ===== PascalCase tests =====

    #[test]
    fn test_is_pascal_case_valid() {
        assert!(is_pascal_case("User"));
        assert!(is_pascal_case("OrderItem"));
        assert!(is_pascal_case("ApiResponse"));
        assert!(is_pascal_case("User2"));
    }

    #[test]
    fn test_is_pascal_case_invalid() {
        assert!(!is_pascal_case("user")); // camelCase
        assert!(!is_pascal_case("order_item")); // snake_case
        assert!(!is_pascal_case("order-item")); // kebab-case
        assert!(!is_pascal_case("ORDERITEM")); // ALL CAPS
        assert!(!is_pascal_case("OrderITEM")); // consecutive uppercase
        assert!(!is_pascal_case("")); // empty
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user"), "User");
        assert_eq!(to_pascal_case("order_item"), "OrderItem");
        assert_eq!(to_pascal_case("order-item"), "OrderItem");
        assert_eq!(to_pascal_case("order item"), "OrderItem");
    }

    #[test]
    fn test_schema_name_pascal_case_rule() {
        let mut contract = create_test_contract();
        contract.schemas.insert(
            "user_profile".to_string(),
            Schema::Object(ObjectSchema::default()),
        );
        contract
            .schemas
            .insert("Order".to_string(), Schema::Object(ObjectSchema::default()));

        let rule = SchemaNamePascalCase;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("user_profile"));
    }

    #[test]
    fn test_all_rules_returns_all() {
        let rules = all_rules();
        assert_eq!(rules.len(), 3);

        let ids: Vec<_> = rules.iter().map(|r| r.id()).collect();
        assert!(ids.contains(&"naming/operation-id"));
        assert!(ids.contains(&"naming/path-format"));
        assert!(ids.contains(&"naming/schema-name"));
    }
}
