//! Lint issue reporting.
//!
//! This module provides the core linting infrastructure:
//!
//! - [`LintReport`]: Contains all issues found during linting
//! - [`LintReporter`]: Runs lint rules against contracts
//! - [`LintConfig`]: Configuration for enabling/disabling rules

use std::collections::HashMap;

use crate::rule::{Rule, RuleConfig};
use crate::rules;
use themis_core::Contract;

/// A lint report containing all discovered issues.
#[derive(Debug, Default)]
pub struct LintReport {
    /// List of lint issues
    pub issues: Vec<LintIssue>,
}

impl LintReport {
    /// Returns true if there are no issues.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    /// Returns the number of errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count()
    }

    /// Returns the number of warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count()
    }

    /// Returns the number of info messages.
    #[must_use]
    pub fn info_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Info)
            .count()
    }
}

/// A single lint issue.
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// Rule that triggered this issue
    pub rule: String,
    /// Issue severity
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Location in the contract
    pub location: Option<String>,
}

/// Severity of a lint issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error - must be fixed
    Error,
    /// Warning - should be fixed
    Warning,
    /// Info - informational only
    Info,
}

/// Configuration for the linter.
///
/// Controls which rules are enabled and their severity.
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// Per-rule configuration, keyed by rule ID
    pub rules: HashMap<String, RuleConfig>,
}

impl Default for LintConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();

        // Set defaults for all built-in rules
        for rule in rules::all_rules() {
            rules.insert(rule.id().to_string(), rule.default_config());
        }

        Self { rules }
    }
}

impl LintConfig {
    /// Creates a new empty configuration (all rules use defaults).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a strict configuration where all rules are enabled as errors.
    #[must_use]
    pub fn strict() -> Self {
        let mut rules = HashMap::new();

        for rule in rules::all_rules() {
            rules.insert(rule.id().to_string(), RuleConfig::enabled(Severity::Error));
        }

        Self { rules }
    }

    /// Creates a relaxed configuration where all rules are warnings.
    #[must_use]
    pub fn relaxed() -> Self {
        let mut rules = HashMap::new();

        for rule in rules::all_rules() {
            rules.insert(
                rule.id().to_string(),
                RuleConfig::enabled(Severity::Warning),
            );
        }

        Self { rules }
    }

    /// Sets the configuration for a specific rule.
    pub fn set_rule(&mut self, rule_id: &str, config: RuleConfig) {
        self.rules.insert(rule_id.to_string(), config);
    }

    /// Enables a rule with the given severity.
    pub fn enable(&mut self, rule_id: &str, severity: Severity) {
        self.rules
            .insert(rule_id.to_string(), RuleConfig::enabled(severity));
    }

    /// Disables a rule.
    pub fn disable(&mut self, rule_id: &str) {
        self.rules
            .insert(rule_id.to_string(), RuleConfig::disabled());
    }

    /// Gets the configuration for a rule, using defaults if not set.
    #[must_use]
    pub fn get_rule_config(&self, rule_id: &str) -> RuleConfig {
        self.rules.get(rule_id).cloned().unwrap_or_default()
    }
}

/// Lint reporter for running rules against contracts.
pub struct LintReporter {
    /// Configuration for the linter
    config: LintConfig,
    /// Registered rules
    rules: Vec<Box<dyn Rule>>,
}

impl LintReporter {
    /// Creates a new lint reporter with the given configuration.
    #[must_use]
    pub fn new(config: LintConfig) -> Self {
        Self {
            config,
            rules: rules::all_rules(),
        }
    }

    /// Creates a new lint reporter with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(LintConfig::default())
    }

    /// Returns the current configuration.
    #[must_use]
    pub const fn config(&self) -> &LintConfig {
        &self.config
    }

    /// Returns the list of available rules.
    #[must_use]
    pub fn available_rules(&self) -> Vec<(&str, &str)> {
        self.rules
            .iter()
            .map(|r| (r.id(), r.description()))
            .collect()
    }

    /// Runs all lint rules against a contract.
    #[must_use]
    pub fn lint(&self, contract: &Contract) -> LintReport {
        let mut issues = Vec::new();

        for rule in &self.rules {
            let config = self.config.get_rule_config(rule.id());
            if config.enabled {
                let rule_issues = rule.check(contract, &config);
                issues.extend(rule_issues);
            }
        }

        LintReport { issues }
    }
}

impl Default for LintReporter {
    fn default() -> Self {
        Self::with_defaults()
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
            schemas: IndexMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    #[test]
    fn test_lint_report_default() {
        let report = LintReport::default();
        assert!(report.is_clean());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn test_lint_config_default() {
        let config = LintConfig::default();
        // Should have configuration for all rules
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_lint_config_strict() {
        let config = LintConfig::strict();
        // All rules should be errors
        for rule_config in config.rules.values() {
            assert!(rule_config.enabled);
            assert_eq!(rule_config.severity, Severity::Error);
        }
    }

    #[test]
    fn test_lint_config_enable_disable() {
        let mut config = LintConfig::default();

        config.disable("naming/operation-id");
        assert!(!config.get_rule_config("naming/operation-id").enabled);

        config.enable("naming/operation-id", Severity::Error);
        let rule_config = config.get_rule_config("naming/operation-id");
        assert!(rule_config.enabled);
        assert_eq!(rule_config.severity, Severity::Error);
    }

    #[test]
    fn test_linter_with_defaults() {
        let linter = LintReporter::with_defaults();
        assert!(!linter.available_rules().is_empty());
    }

    #[test]
    fn test_linter_clean_contract() {
        let contract = create_test_contract();
        let linter = LintReporter::with_defaults();
        let report = linter.lint(&contract);

        // Empty contract should be clean
        assert!(report.is_clean());
    }

    #[test]
    fn test_linter_finds_naming_issues() {
        let mut contract = create_test_contract();

        let mut op = Operation::new("get_user");
        op.summary = Some("Get user".to_string());
        contract.operations.insert("get_user".to_string(), op);

        let linter = LintReporter::with_defaults();
        let report = linter.lint(&contract);

        // Should find snake_case operation ID
        assert!(!report.is_clean());
        assert!(report
            .issues
            .iter()
            .any(|i| i.rule == "naming/operation-id"));
    }

    #[test]
    fn test_linter_respects_disabled_rules() {
        let mut contract = create_test_contract();

        let mut op = Operation::new("get_user");
        op.summary = Some("Get user".to_string());
        contract.operations.insert("get_user".to_string(), op);

        let mut config = LintConfig::default();
        config.disable("naming/operation-id");

        let linter = LintReporter::new(config);
        let report = linter.lint(&contract);

        // Should NOT find the naming issue since rule is disabled
        assert!(!report
            .issues
            .iter()
            .any(|i| i.rule == "naming/operation-id"));
    }
}
