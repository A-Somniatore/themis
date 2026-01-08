//! GraphQL-specific lint rules.
//!
//! These rules check GraphQL-specific best practices:
//! - Query/mutation operations should have directives for metadata
//! - Input types should follow naming conventions (ending with `Input`)

use crate::reporter::LintIssue;
use crate::rule::{Rule, RuleConfig};
use themis_core::Contract;

/// Checks that GraphQL operations have directive metadata.
///
/// GraphQL operations should include directives (e.g., `@deprecated`, `@auth`, `@cached`)
/// for metadata about their behavior and requirements.
///
/// # Examples
///
/// Valid: `query GetUser @auth(requires: "user") { ... }`
/// Invalid: `query GetUser { ... }`
pub struct GraphQLOperationDirective;

impl Rule for GraphQLOperationDirective {
    fn id(&self) -> &'static str {
        "graphql/operation-directive"
    }

    fn description(&self) -> &'static str {
        "GraphQL operations should have directive metadata (@auth, @cached, etc.)"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // For GraphQL operations, check if operation paths contain directives
        // In GraphQL SDL, operations are defined with directives like:
        // type Query {
        //   getUser(id: ID!): User @auth(requires: "admin")
        // }

        for (op_id, op) in &contract.operations {
            // Check if the operation path contains typical GraphQL directive markers
            let has_directive = op.path.as_deref().is_some_and(|p| p.contains('@'));

            // For operations without explicit directives, suggest adding them
            // Skip if the operation is already marked as deprecated
            if !has_directive && !op.deprecated {
                // Check if it's a write operation (mutation) or public read operation
                if let Some(method) = op.method {
                    use themis_core::operation::HttpMethod;
                    if matches!(method, HttpMethod::Post | HttpMethod::Put | HttpMethod::Delete | HttpMethod::Get) {
                        issues.push(LintIssue {
                            rule: self.id().to_string(),
                            severity: config.severity,
                            message: format!(
                                "Operation '{op_id}' should include directive metadata (e.g., @auth, @cached, @deprecated)"
                            ),
                            location: op.path.clone(),
                        });
                    }
                }
            }
        }

        issues
    }
}

/// Checks that GraphQL input types follow naming conventions.
///
/// Input types should end with `Input` suffix (e.g., `CreateUserInput`, `FilterInput`)
/// This makes it clear in the schema which types are used for input vs output.
///
/// # Examples
///
/// Valid: `CreateUserInput`, `UserFilterInput`, `PaginationInput`
/// Invalid: `CreateUser`, `UserFilter`, `Pagination` (when used as input)
pub struct GraphQLInputNaming;

impl Rule for GraphQLInputNaming {
    fn id(&self) -> &'static str {
        "graphql/input-naming"
    }

    fn description(&self) -> &'static str {
        "GraphQL input types should end with 'Input' suffix"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Check schema names to identify input types
        for (schema_name, schema) in &contract.schemas {
            // If the schema looks like an input type (has certain patterns or descriptions)
            // but doesn't end with "Input", suggest the naming fix
            
            // Common patterns that indicate an input type:
            // 1. Schema is used as request body (we can infer from context)
            // 2. Schema name contains "Create", "Update", "Filter", "Query"
            // 3. Schema description mentions "input"

            let looks_like_input = schema_name.contains("Create")
                || schema_name.contains("Update")
                || schema_name.contains("Filter")
                || schema_name.contains("Query")
                || schema_name.contains("Search")
                || schema.description().is_some_and(|d| d.to_lowercase().contains("input"));

            if looks_like_input && !schema_name.ends_with("Input") {
                issues.push(LintIssue {
                    rule: self.id().to_string(),
                    severity: config.severity,
                    message: format!(
                        "Input type '{schema_name}' should end with 'Input' suffix (e.g., '{schema_name}Input')"
                    ),
                    location: Some(schema_name.clone()),
                });
            }
        }

        issues
    }
}

/// Returns all GraphQL-specific lint rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(GraphQLOperationDirective),
        Box::new(GraphQLInputNaming),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::{Contract, Operation, Schema, contract::{ContractFormat, ContractMetadata}};

    fn create_test_contract(service_name: &str) -> Contract {
        Contract {
            format: ContractFormat::GraphQl,
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
    fn test_directive_rule_with_directive() {
        let mut contract = create_test_contract("UserAPI");
        let mut op = Operation::new("getUser");
        op.path = Some("/user(id: ID!): User @auth".to_string());
        contract.operations.insert("getUser".to_string(), op);

        let rule = GraphQLOperationDirective;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_directive_rule_without_directive() {
        let mut contract = create_test_contract("UserAPI");
        let mut op = Operation::new("getUser");
        op.path = Some("/user(id: ID!): User".to_string());
        use themis_core::operation::HttpMethod;
        op.method = Some(HttpMethod::Get);
        contract.operations.insert("getUser".to_string(), op);

        let rule = GraphQLOperationDirective;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_input_naming_rule_valid() {
        let mut contract = create_test_contract("UserAPI");
        contract.schemas.insert(
            "CreateUserInput".to_string(),
            Schema::String(Default::default()),
        );

        let rule = GraphQLInputNaming;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_input_naming_rule_invalid_create() {
        let mut contract = create_test_contract("UserAPI");
        contract.schemas.insert(
            "CreateUser".to_string(),
            Schema::String(Default::default()),
        );

        let rule = GraphQLInputNaming;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_input_naming_rule_invalid_update() {
        let mut contract = create_test_contract("UserAPI");
        contract.schemas.insert(
            "UpdateUser".to_string(),
            Schema::String(Default::default()),
        );

        let rule = GraphQLInputNaming;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_input_naming_rule_invalid_filter() {
        let mut contract = create_test_contract("UserAPI");
        contract.schemas.insert(
            "UserFilter".to_string(),
            Schema::String(Default::default()),
        );

        let rule = GraphQLInputNaming;
        let config = RuleConfig::default();

        let issues = rule.check(&contract, &config);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_all_graphql_rules() {
        let rules = all_rules();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id(), "graphql/operation-directive");
        assert_eq!(rules[1].id(), "graphql/input-naming");
    }
}
