//! Contract linting functionality.
//!
//! This module provides functions for linting contracts against configurable rules.

use std::path::Path;

use themis_core::Contract;
use themis_lint::{LintConfig, LintReport, LintReporter, Severity};

use crate::error::{SdkError, SdkResult};

/// Lint a contract using the default configuration.
///
/// # Arguments
///
/// * `contract` - The contract to lint
///
/// # Returns
///
/// A lint report containing all findings
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::lint::lint;
/// use themis_sdk::parse::parse_string;
///
/// let contract = parse_string(yaml)?;
/// let report = lint(&contract);
/// println!("Found {} errors", report.error_count());
/// ```
#[must_use]
pub fn lint(contract: &Contract) -> LintReport {
    lint_with_config(contract, &LintConfig::default())
}

/// Lint a contract using a custom configuration.
///
/// # Arguments
///
/// * `contract` - The contract to lint
/// * `config` - The lint configuration to use
///
/// # Returns
///
/// A lint report containing all findings
#[must_use]
pub fn lint_with_config(contract: &Contract, config: &LintConfig) -> LintReport {
    let reporter = LintReporter::new(config.clone());
    reporter.lint(contract)
}

/// Lint a contract from a file path.
///
/// # Arguments
///
/// * `path` - Path to the contract file
///
/// # Returns
///
/// A lint report containing all findings
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The contract cannot be parsed
pub fn lint_file<P: AsRef<Path>>(path: P) -> SdkResult<LintReport> {
    let contract = crate::parse::parse_file(path)?;
    Ok(lint(&contract))
}

/// Lint a contract from a file path with custom configuration.
///
/// # Arguments
///
/// * `path` - Path to the contract file
/// * `config` - The lint configuration to use
///
/// # Returns
///
/// A lint report containing all findings
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The contract cannot be parsed
pub fn lint_file_with_config<P: AsRef<Path>>(
    path: P,
    config: &LintConfig,
) -> SdkResult<LintReport> {
    let contract = crate::parse::parse_file(path)?;
    Ok(lint_with_config(&contract, config))
}

/// Get all available lint rules.
///
/// # Returns
///
/// A list of all available lint rules with their metadata
#[must_use]
pub fn available_rules() -> Vec<LintRuleInfo> {
    let reporter = LintReporter::with_defaults();
    reporter
        .available_rules()
        .into_iter()
        .map(|(id, description)| LintRuleInfo {
            id: id.to_string(),
            description: description.to_string(),
            default_severity: Severity::Warning,
        })
        .collect()
}

/// Information about a lint rule.
#[derive(Debug, Clone)]
pub struct LintRuleInfo {
    /// Unique identifier for the rule.
    pub id: String,
    /// Description of what the rule checks.
    pub description: String,
    /// Default severity level.
    pub default_severity: Severity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_rules() {
        let rules = available_rules();
        // We should have at least one rule
        assert!(!rules.is_empty());

        // Each rule should have required fields
        for rule in &rules {
            assert!(!rule.id.is_empty());
            assert!(!rule.description.is_empty());
        }
    }

    #[test]
    fn test_lint_rule_info_debug() {
        let info = LintRuleInfo {
            id: "test-rule".to_string(),
            description: "A test rule".to_string(),
            default_severity: Severity::Warning,
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("test-rule"));
    }
}
