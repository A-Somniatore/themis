//! Lint issue reporting.

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
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    /// Returns the number of warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
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

/// Lint reporter for running rules against contracts.
pub struct LintReporter {
    // TODO: Add configuration
}

impl LintReporter {
    /// Creates a new lint reporter.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Runs all lint rules against a contract.
    pub fn lint(&self, contract: &Contract) -> LintReport {
        // TODO: Implement linting in Week 5
        let _ = contract;
        LintReport::default()
    }
}

impl Default for LintReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_report_default() {
        let report = LintReport::default();
        assert!(report.is_clean());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
    }
}
