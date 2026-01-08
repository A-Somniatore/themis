//! AsyncAPI-specific lint rules.
//!
//! These rules check AsyncAPI-specific best practices:
//! - Channel names should follow naming conventions (kebab-case)
//! - Messages should have proper schema definitions

use crate::reporter::LintIssue;
use crate::rule::{Rule, RuleConfig};
use themis_core::Contract;

/// Checks that `AsyncAPI` channel names follow naming conventions.
///
/// Channel names should use kebab-case (e.g., `user-created`, `order-processed`)
/// rather than camelCase, `snake_case`, or other styles for consistency.
///
/// # Examples
///
/// Valid: `user-created`, `order-processed`, `payment-completed`
/// Invalid: `userCreated`, `user_created`, `UserCreated`
pub struct AsyncAPIChannelNaming;

impl Rule for AsyncAPIChannelNaming {
    fn id(&self) -> &'static str {
        "asyncapi/channel-naming"
    }

    fn description(&self) -> &'static str {
        "AsyncAPI channel names should use kebab-case naming convention"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        for (op_id, op) in &contract.operations {
            // In AsyncAPI, the operation path often represents the channel name
            if let Some(path) = &op.path {
                if !is_kebab_case(path) {
                    issues.push(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Channel '{}' should use kebab-case naming (e.g., '{}')",
                            path,
                            to_kebab_case(path)
                        ),
                        location: Some(format!("operation:{op_id}")),
                    });
                }
            }

            // Also check the operation ID itself for AsyncAPI conventions
            if !is_kebab_case(op_id) && !is_camel_case(op_id) {
                // AsyncAPI operation IDs can be camelCase or kebab-case
                // Flag only clearly wrong formats (snake_case, etc.)
                if op_id.contains('_') {
                    issues.push(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Operation ID '{}' should use camelCase or kebab-case (e.g., '{}')",
                            op_id,
                            to_camel_case(op_id)
                        ),
                        location: Some(format!("operation:{op_id}")),
                    });
                }
            }
        }

        issues
    }
}

/// Checks that `AsyncAPI` messages have proper schema definitions.
///
/// Messages should have schema definitions for their payloads to enable
/// proper validation and code generation. This rule checks that:
/// - Operations have request bodies (messages) defined
/// - Messages have schema types specified
///
/// # Examples
///
/// Valid: Operation with defined payload schema
/// Invalid: Operation without payload or with untyped payload
pub struct AsyncAPIMessageSchema;

impl Rule for AsyncAPIMessageSchema {
    fn id(&self) -> &'static str {
        "asyncapi/message-schema"
    }

    fn description(&self) -> &'static str {
        "AsyncAPI messages should have proper schema definitions"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        for (op_id, op) in &contract.operations {
            // Check if the operation has a request body (message payload)
            let has_request_body = op.request_body.is_some();
            
            // Check if there are response definitions
            let has_responses = !op.responses.is_empty();

            // For publish operations (sending messages), we expect a request body
            // For subscribe operations (receiving messages), we expect response definitions
            if !has_request_body && !has_responses {
                issues.push(LintIssue {
                    rule: self.id().to_string(),
                    severity: config.severity,
                    message: format!(
                        "Operation '{op_id}' has no message schema defined. Add a payload schema for the message."
                    ),
                    location: Some(format!("operation:{op_id}")),
                });
            }

            // If there's a request body, ensure it has actual content
            if let Some(request_body) = &op.request_body {
                if request_body.content.is_empty() {
                    issues.push(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Operation '{op_id}' has a message but no content schema. Define a schema for the payload."
                        ),
                        location: Some(format!("operation:{op_id}")),
                    });
                }
            }
        }

        issues
    }
}

/// Checks that `AsyncAPI` channel names follow event-driven conventions.
///
/// Channel names in event-driven architectures often follow patterns like:
/// - Past tense events: `user-created`, `order-placed`, `payment-completed`
/// - Command patterns: `create-user`, `process-order`, `send-notification`
/// - Topic patterns: `users`, `orders`, `notifications`
///
/// This rule warns about ambiguous channel names that don't follow these patterns.
pub struct AsyncAPIChannelConvention;

impl Rule for AsyncAPIChannelConvention {
    fn id(&self) -> &'static str {
        "asyncapi/channel-convention"
    }

    fn description(&self) -> &'static str {
        "AsyncAPI channel names should follow event-driven naming conventions"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Common event suffixes that indicate good naming
        let event_patterns = [
            "-created", "-updated", "-deleted", "-completed", "-failed",
            "-started", "-finished", "-processed", "-received", "-sent",
            "-placed", "-cancelled", "-approved", "-rejected",
        ];

        // Command patterns
        let command_patterns = [
            "create-", "update-", "delete-", "process-", "send-",
            "notify-", "validate-", "execute-", "handle-",
        ];

        for (op_id, op) in &contract.operations {
            if let Some(path) = &op.path {
                let lower_path = path.to_lowercase();
                
                // Skip if the path matches known good patterns
                let has_event_pattern = event_patterns.iter().any(|p| lower_path.ends_with(p));
                let has_command_pattern = command_patterns.iter().any(|p| lower_path.starts_with(p));
                let is_plural_topic = lower_path.ends_with('s') && !lower_path.contains('-');
                
                if !has_event_pattern && !has_command_pattern && !is_plural_topic {
                    // Check if it's a simple single word that's not clearly a topic
                    if !path.contains('-') && !path.ends_with('s') {
                        issues.push(LintIssue {
                            rule: self.id().to_string(),
                            severity: config.severity,
                            message: format!(
                                "Channel '{path}' should follow event-driven naming conventions. \
                                Consider using past tense (e.g., 'user-created'), \
                                command form (e.g., 'create-user'), or plural topic (e.g., 'users')."
                            ),
                            location: Some(format!("operation:{op_id}")),
                        });
                    }
                }
            }
        }

        issues
    }
}

/// Check if a string is kebab-case (all lowercase with hyphens)
fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_uppercase() || c == '_' {
            return false;
        }
        // Don't allow consecutive hyphens
        if c == '-' && chars.peek() == Some(&'-') {
            return false;
        }
    }
    true
}

/// Check if a string is camelCase
fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let mut chars = s.chars();
    // First character should be lowercase
    if let Some(first) = chars.next() {
        if !first.is_lowercase() {
            return false;
        }
    }
    // Rest can be alphanumeric, no underscores or hyphens
    for c in chars {
        if !c.is_alphanumeric() {
            return false;
        }
    }
    true
}

/// Convert a string to kebab-case
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == ' ' {
            result.push('-');
        } else if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a string to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}

/// Returns all AsyncAPI-specific lint rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(AsyncAPIChannelNaming),
        Box::new(AsyncAPIMessageSchema),
        Box::new(AsyncAPIChannelConvention),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::{Operation, RequestBody, Response};
    use themis_core::schema::Schema;
    use std::collections::HashMap;

    fn create_test_contract(service_name: &str) -> Contract {
        Contract {
            format: ContractFormat::AsyncApi,
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

    fn create_operation_with_path(path: &str) -> Operation {
        let mut op = Operation::new("test-op");
        op.path = Some(path.to_string());
        op
    }

    fn create_operation_with_request_body() -> Operation {
        let mut content = HashMap::new();
        content.insert(
            "application/json".to_string(),
            themis_core::operation::MediaType {
                schema: Schema::String(themis_core::schema::StringSchema::default()),
            },
        );
        
        let mut op = Operation::new("test-op");
        op.request_body = Some(RequestBody {
            description: None,
            required: false,
            content,
        });
        op
    }

    fn create_operation_with_response() -> Operation {
        let mut responses = HashMap::new();
        responses.insert(
            "200".to_string(),
            Response {
                description: "Success".to_string(),
                content: HashMap::new(),
                headers: HashMap::new(),
            },
        );
        
        let mut op = Operation::new("test-op");
        op.responses = responses;
        op
    }

    // Channel naming tests
    #[test]
    fn test_channel_naming_valid_kebab_case() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_path("user-created");
        contract.operations.insert("userCreated".to_string(), op);

        let rule = AsyncAPIChannelNaming;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert!(issues.is_empty(), "kebab-case channel should pass");
    }

    #[test]
    fn test_channel_naming_invalid_camel_case() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_path("userCreated");
        contract.operations.insert("userCreated".to_string(), op);

        let rule = AsyncAPIChannelNaming;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("kebab-case"));
    }

    #[test]
    fn test_channel_naming_invalid_snake_case_op_id() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_path("user-created");
        contract.operations.insert("user_created".to_string(), op);

        let rule = AsyncAPIChannelNaming;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("camelCase or kebab-case"));
    }

    // Message schema tests
    #[test]
    fn test_message_schema_valid_with_request_body() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_request_body();
        contract.operations.insert("sendMessage".to_string(), op);

        let rule = AsyncAPIMessageSchema;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert!(issues.is_empty(), "operation with request body should pass");
    }

    #[test]
    fn test_message_schema_valid_with_response() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_response();
        contract.operations.insert("receiveMessage".to_string(), op);

        let rule = AsyncAPIMessageSchema;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert!(issues.is_empty(), "operation with response should pass");
    }

    #[test]
    fn test_message_schema_missing_schema() {
        let mut contract = create_test_contract("events");
        let op = Operation::new("emptyOp");
        contract.operations.insert("emptyOp".to_string(), op);

        let rule = AsyncAPIMessageSchema;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("no message schema"));
    }

    #[test]
    fn test_message_schema_empty_content() {
        let mut contract = create_test_contract("events");
        let mut op = Operation::new("emptyContent");
        op.request_body = Some(RequestBody {
            description: None,
            required: false,
            content: HashMap::new(), // Empty content
        });
        contract.operations.insert("emptyContent".to_string(), op);

        let rule = AsyncAPIMessageSchema;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        // Should have 2 issues: no response AND empty content
        assert!(issues.len() >= 1);
        assert!(issues.iter().any(|i| i.message.contains("no content schema")));
    }

    // Channel convention tests
    #[test]
    fn test_channel_convention_valid_event() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_path("user-created");
        contract.operations.insert("userCreated".to_string(), op);

        let rule = AsyncAPIChannelConvention;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert!(issues.is_empty(), "event-pattern channel should pass");
    }

    #[test]
    fn test_channel_convention_valid_command() {
        let mut contract = create_test_contract("commands");
        let op = create_operation_with_path("create-user");
        contract.operations.insert("createUser".to_string(), op);

        let rule = AsyncAPIChannelConvention;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert!(issues.is_empty(), "command-pattern channel should pass");
    }

    #[test]
    fn test_channel_convention_valid_topic() {
        let mut contract = create_test_contract("topics");
        let op = create_operation_with_path("users");
        contract.operations.insert("usersChannel".to_string(), op);

        let rule = AsyncAPIChannelConvention;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert!(issues.is_empty(), "plural topic should pass");
    }

    #[test]
    fn test_channel_convention_ambiguous_name() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_path("notification");
        contract.operations.insert("notification".to_string(), op);

        let rule = AsyncAPIChannelConvention;
        let issues = rule.check(&contract, &RuleConfig::default());
        
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("event-driven naming"));
    }

    // Helper function tests
    #[test]
    fn test_is_kebab_case() {
        assert!(is_kebab_case("user-created"));
        assert!(is_kebab_case("payment-completed"));
        assert!(is_kebab_case("simple"));
        assert!(!is_kebab_case("userCreated"));
        assert!(!is_kebab_case("user_created"));
        assert!(!is_kebab_case("UserCreated"));
        assert!(!is_kebab_case("user--created")); // double hyphen
    }

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("userCreated"));
        assert!(is_camel_case("paymentCompleted"));
        assert!(is_camel_case("simple"));
        assert!(!is_camel_case("UserCreated")); // PascalCase
        assert!(!is_camel_case("user_created"));
        assert!(!is_camel_case("user-created"));
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("userCreated"), "user-created");
        assert_eq!(to_kebab_case("user_created"), "user-created");
        assert_eq!(to_kebab_case("UserCreated"), "user-created");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("user_created"), "userCreated");
        assert_eq!(to_camel_case("user-created"), "userCreated");
        assert_eq!(to_camel_case("USER_CREATED"), "userCreated");
    }

    #[test]
    fn test_all_asyncapi_rules() {
        let rules = all_rules();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_rule_disabled() {
        let mut contract = create_test_contract("events");
        let op = create_operation_with_path("BadName");
        contract.operations.insert("bad_op".to_string(), op);

        let mut config = RuleConfig::default();
        config.enabled = false;

        let rule = AsyncAPIChannelNaming;
        let issues = rule.check(&contract, &config);
        
        assert!(issues.is_empty(), "disabled rule should return no issues");
    }
}
