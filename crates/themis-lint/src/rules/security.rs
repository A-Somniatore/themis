//! Security lint rules.
//!
//! These rules check for common security issues in API contracts:
//! - THEMIS010: API keys should not be in query parameters
//! - THEMIS011: HTTPS should be required for sensitive operations
//! - THEMIS012: Operations should have explicit security requirements
//! - THEMIS013: Avoid exposing internal error details

use crate::reporter::LintIssue;
use crate::rule::{Rule, RuleConfig};
use themis_core::contract::ApiKeyLocation;
use themis_core::Contract;

/// Checks that API keys are not passed in query parameters.
///
/// Passing API keys in query parameters is a security risk because:
/// - Query parameters are logged in server access logs
/// - They appear in browser history
/// - They may be cached by proxies
/// - They're visible in referrer headers
///
/// API keys should be passed in headers instead.
///
/// # Rule ID
///
/// `security/no-api-key-in-query` (THEMIS010)
pub struct NoApiKeyInQuery;

impl Rule for NoApiKeyInQuery {
    fn id(&self) -> &'static str {
        "security/no-api-key-in-query"
    }

    fn description(&self) -> &'static str {
        "API keys should not be passed in query parameters"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .security_schemes
            .iter()
            .filter_map(|(name, scheme)| {
                if let themis_core::contract::SecuritySchemeType::ApiKey { location, .. } =
                    &scheme.scheme_type
                {
                    if *location == ApiKeyLocation::Query {
                        return Some(LintIssue {
                            rule: self.id().to_string(),
                            severity: config.severity,
                            message: format!(
                                "Security scheme '{name}' uses API key in query parameter. \
                                 Use header-based authentication instead for better security."
                            ),
                            location: Some(format!("#/components/securitySchemes/{name}")),
                        });
                    }
                }
                None
            })
            .collect()
    }
}

/// Checks that operations handling sensitive data have security requirements.
///
/// Operations that modify data (POST, PUT, PATCH, DELETE) or handle
/// user data should have explicit security requirements defined.
///
/// # Rule ID
///
/// `security/require-auth-for-mutations` (THEMIS011)
pub struct RequireAuthForMutations;

impl Rule for RequireAuthForMutations {
    fn id(&self) -> &'static str {
        "security/require-auth-for-mutations"
    }

    fn description(&self) -> &'static str {
        "Mutating operations (POST/PUT/PATCH/DELETE) should have security requirements"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        contract
            .operations
            .iter()
            .filter_map(|(id, op)| {
                // Check if this is a mutating operation
                let is_mutating = op.method.as_ref().is_some_and(|m| {
                    matches!(
                        m,
                        themis_core::operation::HttpMethod::Post
                            | themis_core::operation::HttpMethod::Put
                            | themis_core::operation::HttpMethod::Patch
                            | themis_core::operation::HttpMethod::Delete
                    )
                });

                // Skip if not mutating
                if !is_mutating {
                    return None;
                }

                // Check if it has security requirements
                if op.security.is_empty() {
                    return Some(LintIssue {
                        rule: self.id().to_string(),
                        severity: config.severity,
                        message: format!(
                            "Operation '{id}' modifies data but has no security requirements. \
                             Consider adding authentication."
                        ),
                        location: op.path.clone(),
                    });
                }

                None
            })
            .collect()
    }
}

/// Checks that sensitive parameter names don't appear in query strings.
///
/// Parameters containing passwords, tokens, secrets, or keys should
/// not be passed in query parameters as they may be logged or cached.
///
/// # Rule ID
///
/// `security/no-sensitive-params-in-query` (THEMIS012)
pub struct NoSensitiveParamsInQuery;

impl Rule for NoSensitiveParamsInQuery {
    fn id(&self) -> &'static str {
        "security/no-sensitive-params-in-query"
    }

    fn description(&self) -> &'static str {
        "Sensitive parameters (password, token, secret, key) should not be in query strings"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        // Patterns that indicate sensitive data
        let sensitive_patterns = [
            "password",
            "passwd",
            "secret",
            "token",
            "api_key",
            "apikey",
            "api-key",
            "access_token",
            "auth",
            "credential",
            "private",
        ];

        contract
            .operations
            .iter()
            .flat_map(|(op_id, op)| {
                op.parameters
                    .iter()
                    .filter(|param| {
                        param.location == themis_core::operation::ParameterLocation::Query
                    })
                    .filter_map(|param| {
                        let name_lower = param.name.to_lowercase();
                        let is_sensitive = sensitive_patterns
                            .iter()
                            .any(|pattern| name_lower.contains(pattern));

                        if is_sensitive {
                            Some(LintIssue {
                                rule: self.id().to_string(),
                                severity: config.severity,
                                message: format!(
                                    "Query parameter '{}' in operation '{op_id}' may contain \
                                     sensitive data. Consider using headers or request body instead.",
                                    param.name
                                ),
                                location: op.path.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

/// Checks that error responses don't expose internal details.
///
/// Error responses should not include stack traces, internal paths,
/// or other implementation details that could help attackers.
///
/// # Rule ID
///
/// `security/no-internal-error-exposure` (THEMIS013)
pub struct NoInternalErrorExposure;

impl Rule for NoInternalErrorExposure {
    fn id(&self) -> &'static str {
        "security/no-internal-error-exposure"
    }

    fn description(&self) -> &'static str {
        "Error responses should not expose internal implementation details"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        // Patterns in schema property names that might indicate internal data exposure
        let suspicious_patterns = [
            "stack_trace",
            "stackTrace",
            "stack",
            "trace",
            "internal_error",
            "internalError",
            "exception",
            "debug",
            "file_path",
            "filePath",
            "line_number",
            "lineNumber",
            "sql_query",
            "sqlQuery",
        ];

        let mut issues = Vec::new();

        for (op_id, op) in &contract.operations {
            // Check 4xx and 5xx responses
            for (status, response) in &op.responses {
                if !status.starts_with('4') && !status.starts_with('5') {
                    continue;
                }

                for (media_type, content) in &response.content {
                    let schema_issues =
                        check_schema_for_sensitive_fields(&content.schema, &suspicious_patterns);

                    for field_name in schema_issues {
                        issues.push(LintIssue {
                            rule: self.id().to_string(),
                            severity: config.severity,
                            message: format!(
                                "Error response for '{op_id}' ({status}, {media_type}) \
                                 contains field '{field_name}' which may expose internal details."
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

/// Recursively check a schema for suspicious field names.
fn check_schema_for_sensitive_fields(
    schema: &themis_core::schema::Schema,
    patterns: &[&str],
) -> Vec<String> {
    let mut found = Vec::new();

    if let themis_core::schema::Schema::Object(obj) = schema {
        for (prop_name, prop_schema) in &obj.properties {
            let name_lower = prop_name.to_lowercase();
            if patterns.iter().any(|p| name_lower.contains(p)) {
                found.push(prop_name.clone());
            }
            // Recursively check nested schemas
            found.extend(check_schema_for_sensitive_fields(prop_schema, patterns));
        }
    }

    found
}

/// Returns all security lint rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(NoApiKeyInQuery),
        Box::new(RequireAuthForMutations),
        Box::new(NoSensitiveParamsInQuery),
        Box::new(NoInternalErrorExposure),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::Severity;
    use indexmap::IndexMap;
    use std::collections::HashMap;
    use themis_core::contract::{
        Contract, ContractFormat, ContractMetadata, SecurityScheme, SecuritySchemeType,
    };
    use themis_core::operation::{
        HttpMethod, MediaType, Operation, Parameter, ParameterLocation, Response,
    };
    use themis_core::schema::{ObjectSchema, Schema, StringSchema};
    use themis_core::version::Version;

    fn create_test_contract() -> Contract {
        Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "Test Service".to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: HashMap::new(),
            schemas: IndexMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    #[test]
    fn test_no_api_key_in_query_violation() {
        let mut contract = create_test_contract();
        contract.security_schemes.insert(
            "apiKey".to_string(),
            SecurityScheme {
                scheme_type: SecuritySchemeType::ApiKey {
                    location: ApiKeyLocation::Query,
                    name: "api_key".to_string(),
                },
                description: None,
            },
        );

        let rule = NoApiKeyInQuery;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("query parameter"));
    }

    #[test]
    fn test_no_api_key_in_query_pass() {
        let mut contract = create_test_contract();
        contract.security_schemes.insert(
            "apiKey".to_string(),
            SecurityScheme {
                scheme_type: SecuritySchemeType::ApiKey {
                    location: ApiKeyLocation::Header,
                    name: "X-API-Key".to_string(),
                },
                description: None,
            },
        );

        let rule = NoApiKeyInQuery;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_require_auth_for_mutations_violation() {
        let mut contract = create_test_contract();
        contract.operations.insert(
            "createUser".to_string(),
            Operation {
                operation_id: "createUser".to_string(),
                summary: None,
                description: None,
                method: Some(HttpMethod::Post),
                path: Some("/users".to_string()),
                parameters: vec![],
                request_body: None,
                responses: HashMap::new(),
                security: vec![], // No security!
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        let rule = RequireAuthForMutations;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("no security requirements"));
    }

    #[test]
    fn test_require_auth_for_mutations_pass_get() {
        let mut contract = create_test_contract();
        contract.operations.insert(
            "getUsers".to_string(),
            Operation {
                operation_id: "getUsers".to_string(),
                summary: None,
                description: None,
                method: Some(HttpMethod::Get),
                path: Some("/users".to_string()),
                parameters: vec![],
                request_body: None,
                responses: HashMap::new(),
                security: vec![], // GET without auth is OK
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        let rule = RequireAuthForMutations;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_no_sensitive_params_in_query_violation() {
        let mut contract = create_test_contract();
        contract.operations.insert(
            "login".to_string(),
            Operation {
                operation_id: "login".to_string(),
                summary: None,
                description: None,
                method: Some(HttpMethod::Post),
                path: Some("/login".to_string()),
                parameters: vec![Parameter {
                    name: "password".to_string(),
                    location: ParameterLocation::Query,
                    description: None,
                    required: true,
                    deprecated: false,
                    schema: Schema::String(StringSchema::default()),
                }],
                request_body: None,
                responses: HashMap::new(),
                security: vec![],
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        let rule = NoSensitiveParamsInQuery;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("password"));
    }

    #[test]
    fn test_no_sensitive_params_in_query_pass_header() {
        let mut contract = create_test_contract();
        contract.operations.insert(
            "auth".to_string(),
            Operation {
                operation_id: "auth".to_string(),
                summary: None,
                description: None,
                method: Some(HttpMethod::Post),
                path: Some("/auth".to_string()),
                parameters: vec![Parameter {
                    name: "Authorization".to_string(),
                    location: ParameterLocation::Header, // Header is OK
                    description: None,
                    required: true,
                    deprecated: false,
                    schema: Schema::String(StringSchema::default()),
                }],
                request_body: None,
                responses: HashMap::new(),
                security: vec![],
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        let rule = NoSensitiveParamsInQuery;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_no_internal_error_exposure_violation() {
        let mut contract = create_test_contract();

        let mut error_schema_props = IndexMap::new();
        error_schema_props.insert(
            "stack_trace".to_string(),
            Schema::String(StringSchema::default()),
        );
        error_schema_props.insert(
            "message".to_string(),
            Schema::String(StringSchema::default()),
        );

        let error_schema = Schema::Object(ObjectSchema {
            description: None,
            properties: error_schema_props,
            required: vec!["message".to_string()],
            additional_properties: None,
            nullable: false,
        });

        let mut responses = HashMap::new();
        let mut content = HashMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: error_schema,
            },
        );
        responses.insert(
            "500".to_string(),
            Response {
                description: "Server error".to_string(),
                content,
                headers: HashMap::new(),
            },
        );

        contract.operations.insert(
            "getUser".to_string(),
            Operation {
                operation_id: "getUser".to_string(),
                summary: None,
                description: None,
                method: Some(HttpMethod::Get),
                path: Some("/users/{id}".to_string()),
                parameters: vec![],
                request_body: None,
                responses,
                security: vec![],
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        let rule = NoInternalErrorExposure;
        let config = RuleConfig::default();
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("stack_trace"));
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 4);
    }

    #[test]
    fn test_disabled_rule_returns_no_issues() {
        let mut contract = create_test_contract();
        contract.security_schemes.insert(
            "apiKey".to_string(),
            SecurityScheme {
                scheme_type: SecuritySchemeType::ApiKey {
                    location: ApiKeyLocation::Query,
                    name: "api_key".to_string(),
                },
                description: None,
            },
        );

        let rule = NoApiKeyInQuery;
        let config = RuleConfig::disabled();
        let issues = rule.check(&contract, &config);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_severity_from_config() {
        let mut contract = create_test_contract();
        contract.security_schemes.insert(
            "apiKey".to_string(),
            SecurityScheme {
                scheme_type: SecuritySchemeType::ApiKey {
                    location: ApiKeyLocation::Query,
                    name: "api_key".to_string(),
                },
                description: None,
            },
        );

        let rule = NoApiKeyInQuery;
        let config = RuleConfig::enabled(Severity::Error);
        let issues = rule.check(&contract, &config);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
    }
}
