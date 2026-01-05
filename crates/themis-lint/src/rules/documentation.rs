//! Documentation lint rules.
//!
//! These rules check that API contracts are properly documented:
//! - Operations should have summaries
//! - Operations should have descriptions
//! - Schemas should have descriptions

use crate::reporter::{LintIssue, Severity};
use crate::rule::{Rule, RuleConfig};
use themis_core::Contract;

/// Checks that all operations have a summary.
///
/// Summaries are short, one-line descriptions of what an operation does.
/// They are used in API documentation and tooling.
pub struct OperationSummary;

impl Rule for OperationSummary {
    fn id(&self) -> &'static str {
        "docs/operation-summary"
    }

    fn description(&self) -> &'static str {
        "Operations should have a summary"
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::enabled(Severity::Warning)
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .operations
            .iter()
            .filter_map(|(id, op)| {
                let has_summary = op.summary.as_ref().is_some_and(|s| !s.trim().is_empty());
                if has_summary {
                    None
                } else {
                    Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!("Operation '{id}' is missing a summary"),
                        location: op.path.clone(),
                    })
                }
            })
            .collect()
    }
}

/// Checks that all operations have a description.
///
/// Descriptions provide detailed information about what an operation does,
/// its parameters, and any side effects.
pub struct OperationDescription;

impl Rule for OperationDescription {
    fn id(&self) -> &'static str {
        "docs/operation-description"
    }

    fn description(&self) -> &'static str {
        "Operations should have a description"
    }

    fn default_config(&self) -> RuleConfig {
        // Default to off - descriptions are nice but not always required
        RuleConfig::disabled()
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .operations
            .iter()
            .filter_map(|(id, op)| {
                let has_description = op
                    .description
                    .as_ref()
                    .is_some_and(|d| !d.trim().is_empty());
                if has_description {
                    None
                } else {
                    Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!("Operation '{id}' is missing a description"),
                        location: op.path.clone(),
                    })
                }
            })
            .collect()
    }
}

/// Checks that all schemas have a description.
///
/// Schema descriptions help API consumers understand the purpose and
/// constraints of data types.
pub struct SchemaDescription;

impl Rule for SchemaDescription {
    fn id(&self) -> &'static str {
        "docs/schema-description"
    }

    fn description(&self) -> &'static str {
        "Schemas should have a description"
    }

    fn default_config(&self) -> RuleConfig {
        // Default to off - not all schemas need descriptions
        RuleConfig::disabled()
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .schemas
            .iter()
            .filter_map(|(name, schema)| {
                if schema_has_description(schema) {
                    None
                } else {
                    Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!("Schema '{name}' is missing a description"),
                        location: Some(format!("#/components/schemas/{name}")),
                    })
                }
            })
            .collect()
    }
}

/// Checks if a schema has a description.
fn schema_has_description(schema: &themis_core::schema::Schema) -> bool {
    use themis_core::schema::Schema;

    match schema {
        Schema::String(s) => s.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Integer(i) => i.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Number(n) => n.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Boolean(b) => b.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Array(a) => a.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Object(o) => o.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Enum(e) => e.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::OneOf(o) => o.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::AllOf(a) => a.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::AnyOf(a) => a.description.as_ref().is_some_and(|d| !d.is_empty()),
        Schema::Ref(_) | Schema::Null => true, // Refs and nulls don't need descriptions
    }
}

/// Returns all documentation rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(OperationSummary),
        Box::new(OperationDescription),
        Box::new(SchemaDescription),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::Operation;
    use themis_core::schema::{ObjectSchema, Schema, StringSchema};
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

    #[test]
    fn test_operation_summary_missing() {
        let mut contract = create_test_contract();

        let op1 = Operation::new("getUser");
        contract.operations.insert("getUser".to_string(), op1);

        let mut op2 = Operation::new("createUser");
        op2.summary = Some("Create a new user".to_string());
        contract.operations.insert("createUser".to_string(), op2);

        let rule = OperationSummary;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("getUser"));
    }

    #[test]
    fn test_operation_summary_empty() {
        let mut contract = create_test_contract();

        let mut op = Operation::new("getUser");
        op.summary = Some("  ".to_string()); // whitespace only
        contract.operations.insert("getUser".to_string(), op);

        let rule = OperationSummary;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("getUser"));
    }

    #[test]
    fn test_operation_description_disabled_by_default() {
        let mut contract = create_test_contract();
        let op = Operation::new("getUser");
        contract.operations.insert("getUser".to_string(), op);

        let rule = OperationDescription;
        let config = rule.default_config();
        let issues = rule.check(&contract, &config);

        // Should be disabled by default
        assert!(issues.is_empty());
    }

    #[test]
    fn test_operation_description_when_enabled() {
        let mut contract = create_test_contract();
        let op = Operation::new("getUser");
        contract.operations.insert("getUser".to_string(), op);

        let rule = OperationDescription;
        let config = RuleConfig::enabled(Severity::Warning);
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("getUser"));
    }

    #[test]
    fn test_schema_description_missing() {
        let mut contract = create_test_contract();

        // Schema without description
        contract
            .schemas
            .insert("User".to_string(), Schema::Object(ObjectSchema::default()));

        // Schema with description
        contract.schemas.insert(
            "Order".to_string(),
            Schema::String(StringSchema {
                description: Some("An order identifier".to_string()),
                ..Default::default()
            }),
        );

        let rule = SchemaDescription;
        let config = RuleConfig::enabled(Severity::Warning);
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("User"));
    }

    #[test]
    fn test_all_rules_returns_all() {
        let rules = all_rules();
        assert_eq!(rules.len(), 3);

        let ids: Vec<_> = rules.iter().map(|r| r.id()).collect();
        assert!(ids.contains(&"docs/operation-summary"));
        assert!(ids.contains(&"docs/operation-description"));
        assert!(ids.contains(&"docs/schema-description"));
    }
}
