//! Lint rule trait and configuration.
//!
//! This module defines the core [`Rule`] trait that all lint rules must implement,
//! along with configuration types for controlling rule behavior.

use crate::reporter::{LintIssue, Severity};
use themis_core::Contract;

/// Configuration for a lint rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleConfig {
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Severity when the rule triggers
    pub severity: Severity,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Warning,
        }
    }
}

impl RuleConfig {
    /// Creates a new rule configuration.
    #[must_use]
    pub const fn new(enabled: bool, severity: Severity) -> Self {
        Self { enabled, severity }
    }

    /// Creates an enabled rule with the given severity.
    #[must_use]
    pub const fn enabled(severity: Severity) -> Self {
        Self {
            enabled: true,
            severity,
        }
    }

    /// Creates a disabled rule configuration.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            severity: Severity::Warning,
        }
    }
}

/// A lint rule that can check contracts for issues.
///
/// Rules are stateless and receive their configuration at check time.
/// Each rule has a unique ID and can produce multiple issues.
///
/// # Example
///
/// ```ignore
/// struct OperationIdCamelCase;
///
/// impl Rule for OperationIdCamelCase {
///     fn id(&self) -> &'static str {
///         "naming/operation-id"
///     }
///
///     fn description(&self) -> &'static str {
///         "Operation IDs should be camelCase"
///     }
///
///     fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
///         // Check logic here
///     }
/// }
/// ```
pub trait Rule: Send + Sync {
    /// Returns the unique rule ID (e.g., "naming/operation-id").
    fn id(&self) -> &'static str;

    /// Returns a human-readable description of what this rule checks.
    fn description(&self) -> &'static str;

    /// Returns the default configuration for this rule.
    fn default_config(&self) -> RuleConfig {
        RuleConfig::default()
    }

    /// Checks a contract and returns any issues found.
    ///
    /// The `config` parameter controls whether the rule is enabled
    /// and what severity to assign to issues.
    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_config_default() {
        let config = RuleConfig::default();
        assert!(config.enabled);
        assert_eq!(config.severity, Severity::Warning);
    }

    #[test]
    fn test_rule_config_enabled() {
        let config = RuleConfig::enabled(Severity::Error);
        assert!(config.enabled);
        assert_eq!(config.severity, Severity::Error);
    }

    #[test]
    fn test_rule_config_disabled() {
        let config = RuleConfig::disabled();
        assert!(!config.enabled);
    }
}
