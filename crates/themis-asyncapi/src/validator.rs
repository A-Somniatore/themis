//! `AsyncAPI` specification validator.
//!
//! Validates `AsyncAPI` specifications against common rules and best practices.

use crate::error::AsyncApiError;

/// Validates `AsyncAPI` specifications.
pub struct AsyncApiValidator;

/// A validation finding.
#[derive(Debug, Clone)]
pub struct ValidationFinding {
    /// Rule ID (e.g., "ASYNC001")
    pub rule_id: String,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Path to the problematic element
    pub path: String,
}

/// Severity of a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Informational
    Info,
    /// Warning - should be addressed
    Warning,
    /// Error - must be fixed
    Error,
}

impl AsyncApiValidator {
    /// Create a new validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validate an `AsyncAPI` document.
    ///
    /// # Arguments
    ///
    /// * `doc` - The parsed `AsyncAPI` document as a YAML value
    ///
    /// # Returns
    ///
    /// A list of validation findings
    #[must_use]
    pub fn validate(doc: &serde_yaml::Value) -> Vec<ValidationFinding> {
        let mut findings = Vec::new();

        // ASYNC001: Check for asyncapi version
        Self::check_version(doc, &mut findings);

        // ASYNC002: Check for info section
        Self::check_info(doc, &mut findings);

        // ASYNC003: Check channel names
        Self::check_channels(doc, &mut findings);

        // ASYNC004: Check operation IDs
        Self::check_operations(doc, &mut findings);

        findings
    }

    /// Validate and return errors if any critical issues found.
    ///
    /// # Errors
    ///
    /// Returns `AsyncApiError::Validation` if critical validation errors found
    pub fn validate_strict(doc: &serde_yaml::Value) -> Result<(), AsyncApiError> {
        let findings = Self::validate(doc);
        let errors: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            let messages: Vec<_> = errors.iter().map(|e| e.message.clone()).collect();
            Err(AsyncApiError::Validation(messages.join("; ")))
        }
    }

    /// ASYNC001: Check `AsyncAPI` version
    fn check_version(doc: &serde_yaml::Value, findings: &mut Vec<ValidationFinding>) {
        match doc.get("asyncapi") {
            Some(serde_yaml::Value::String(version)) => {
                if !version.starts_with("3.") {
                    findings.push(ValidationFinding {
                        rule_id: "ASYNC001".to_string(),
                        severity: Severity::Error,
                        message: format!("`AsyncAPI` version {version} not supported, expected 3.x"),
                        path: "asyncapi".to_string(),
                    });
                }
            }
            Some(_) => {
                findings.push(ValidationFinding {
                    rule_id: "ASYNC001".to_string(),
                    severity: Severity::Error,
                    message: "asyncapi field must be a string".to_string(),
                    path: "asyncapi".to_string(),
                });
            }
            None => {
                findings.push(ValidationFinding {
                    rule_id: "ASYNC001".to_string(),
                    severity: Severity::Error,
                    message: "Missing required field: asyncapi".to_string(),
                    path: String::new(),
                });
            }
        }
    }

    /// ASYNC002: Check info section
    fn check_info(doc: &serde_yaml::Value, findings: &mut Vec<ValidationFinding>) {
        match doc.get("info") {
            Some(info) => {
                if info.get("title").is_none() {
                    findings.push(ValidationFinding {
                        rule_id: "ASYNC002".to_string(),
                        severity: Severity::Error,
                        message: "Missing required field: info.title".to_string(),
                        path: "info".to_string(),
                    });
                }
                if info.get("version").is_none() {
                    findings.push(ValidationFinding {
                        rule_id: "ASYNC002".to_string(),
                        severity: Severity::Error,
                        message: "Missing required field: info.version".to_string(),
                        path: "info".to_string(),
                    });
                }
            }
            None => {
                findings.push(ValidationFinding {
                    rule_id: "ASYNC002".to_string(),
                    severity: Severity::Error,
                    message: "Missing required field: info".to_string(),
                    path: String::new(),
                });
            }
        }
    }

    /// ASYNC003: Check channel names follow conventions
    fn check_channels(doc: &serde_yaml::Value, findings: &mut Vec<ValidationFinding>) {
        if let Some(serde_yaml::Value::Mapping(channels)) = doc.get("channels") {
            for (name, _) in channels {
                if let serde_yaml::Value::String(channel_name) = name {
                    // Check for spaces in channel names
                    if channel_name.contains(' ') {
                        findings.push(ValidationFinding {
                            rule_id: "ASYNC003".to_string(),
                            severity: Severity::Warning,
                            message: format!(
                                "Channel name '{channel_name}' should not contain spaces"
                            ),
                            path: format!("channels.{channel_name}"),
                        });
                    }
                    // Check for uppercase (convention is lowercase/camelCase)
                    if channel_name.chars().next().is_some_and(char::is_uppercase) {
                        findings.push(ValidationFinding {
                            rule_id: "ASYNC003".to_string(),
                            severity: Severity::Info,
                            message: format!(
                                "Channel name '{channel_name}' starts with uppercase"
                            ),
                            path: format!("channels.{channel_name}"),
                        });
                    }
                }
            }
        }
    }

    /// ASYNC004: Check operation IDs
    fn check_operations(doc: &serde_yaml::Value, findings: &mut Vec<ValidationFinding>) {
        if let Some(serde_yaml::Value::Mapping(operations)) = doc.get("operations") {
            for (name, op) in operations {
                if let serde_yaml::Value::String(op_name) = name {
                    // Check for action field
                    if op.get("action").is_none() {
                        findings.push(ValidationFinding {
                            rule_id: "ASYNC004".to_string(),
                            severity: Severity::Error,
                            message: format!("Operation '{op_name}' missing required field: action"),
                            path: format!("operations.{op_name}"),
                        });
                    }
                    // Check for channel field
                    if op.get("channel").is_none() {
                        findings.push(ValidationFinding {
                            rule_id: "ASYNC004".to_string(),
                            severity: Severity::Error,
                            message: format!(
                                "Operation '{op_name}' missing required field: channel"
                            ),
                            path: format!("operations.{op_name}"),
                        });
                    }
                }
            }
        }
    }
}

impl Default for AsyncApiValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_asyncapi_document() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
channels: {}
operations: {}
"#;
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let findings = AsyncApiValidator::validate(&doc);
        let errors: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_missing_asyncapi_version() {
        let yaml = r#"
info:
  title: Test API
  version: 1.0.0
"#;
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let findings = AsyncApiValidator::validate(&doc);
        assert!(findings.iter().any(|f| f.rule_id == "ASYNC001"));
    }

    #[test]
    fn test_invalid_asyncapi_version() {
        let yaml = r#"
asyncapi: 2.0.0
info:
  title: Test API
  version: 1.0.0
"#;
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let findings = AsyncApiValidator::validate(&doc);
        assert!(findings
            .iter()
            .any(|f| f.rule_id == "ASYNC001" && f.severity == Severity::Error));
    }

    #[test]
    fn test_missing_info_title() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  version: 1.0.0
"#;
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let findings = AsyncApiValidator::validate(&doc);
        assert!(findings
            .iter()
            .any(|f| f.rule_id == "ASYNC002" && f.message.contains("title")));
    }
}
