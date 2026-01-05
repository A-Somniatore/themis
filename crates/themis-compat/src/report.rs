//! Compatibility report types.
//!
//! Defines the report structure for compatibility analysis results.

use crate::changes::{Addition, BreakingChange, Modification};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The suggested semver change based on detected differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestedBump {
    /// Major version bump required (breaking changes detected).
    Major,
    /// Minor version bump required (new features/additions).
    Minor,
    /// Patch version bump required (non-functional changes).
    Patch,
    /// No version change needed.
    None,
}

impl fmt::Display for SuggestedBump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Result of comparing two contract versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// Whether the new version is backward compatible.
    pub is_compatible: bool,

    /// Suggested semver version bump.
    pub suggested_bump: SuggestedBump,

    /// Breaking changes detected.
    pub breaking_changes: Vec<BreakingChange>,

    /// Backwards-compatible additions.
    pub additions: Vec<Addition>,

    /// Non-functional modifications.
    pub modifications: Vec<Modification>,

    /// Old version (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_version: Option<String>,

    /// New version (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version: Option<String>,
}

impl Default for CompatibilityReport {
    fn default() -> Self {
        Self::new()
    }
}

impl CompatibilityReport {
    /// Creates a new empty compatibility report.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            is_compatible: true,
            suggested_bump: SuggestedBump::None,
            breaking_changes: Vec::new(),
            additions: Vec::new(),
            modifications: Vec::new(),
            old_version: None,
            new_version: None,
        }
    }

    /// Adds a breaking change to the report.
    pub fn add_breaking_change(&mut self, change: BreakingChange) {
        self.breaking_changes.push(change);
        self.is_compatible = false;
        self.suggested_bump = SuggestedBump::Major;
    }

    /// Adds an addition to the report.
    pub fn add_addition(&mut self, addition: Addition) {
        self.additions.push(addition);
        if self.suggested_bump != SuggestedBump::Major {
            self.suggested_bump = SuggestedBump::Minor;
        }
    }

    /// Adds a modification to the report.
    pub fn add_modification(&mut self, modification: Modification) {
        self.modifications.push(modification);
        if self.suggested_bump == SuggestedBump::None {
            self.suggested_bump = SuggestedBump::Patch;
        }
    }

    /// Sets the old version.
    pub fn set_old_version(&mut self, version: impl Into<String>) {
        self.old_version = Some(version.into());
    }

    /// Sets the new version.
    pub fn set_new_version(&mut self, version: impl Into<String>) {
        self.new_version = Some(version.into());
    }

    /// Returns the total number of changes.
    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.breaking_changes.len() + self.additions.len() + self.modifications.len()
    }

    /// Returns true if there are no changes.
    #[must_use]
    pub fn is_unchanged(&self) -> bool {
        self.total_changes() == 0
    }

    /// Formats the report as a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.breaking_changes.is_empty() {
            parts.push(format!(
                "{} breaking change{}",
                self.breaking_changes.len(),
                if self.breaking_changes.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ));
        }

        if !self.additions.is_empty() {
            parts.push(format!(
                "{} addition{}",
                self.additions.len(),
                if self.additions.len() == 1 { "" } else { "s" }
            ));
        }

        if !self.modifications.is_empty() {
            parts.push(format!(
                "{} modification{}",
                self.modifications.len(),
                if self.modifications.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ));
        }

        if parts.is_empty() {
            "No changes detected".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl fmt::Display for CompatibilityReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Header
        if self.is_compatible {
            writeln!(f, "✓ Contracts are backward compatible")?;
        } else {
            writeln!(f, "✗ Breaking changes detected")?;
        }

        // Version info
        if let (Some(old), Some(new)) = (&self.old_version, &self.new_version) {
            writeln!(f, "  Comparing: {old} → {new}")?;
        }

        // Summary
        writeln!(f, "  {}", self.summary())?;
        writeln!(f, "  Suggested version bump: {}", self.suggested_bump)?;

        // Breaking changes
        if !self.breaking_changes.is_empty() {
            writeln!(f)?;
            writeln!(f, "Breaking Changes:")?;
            for change in &self.breaking_changes {
                writeln!(f, "  • {change}")?;
            }
        }

        // Additions
        if !self.additions.is_empty() {
            writeln!(f)?;
            writeln!(f, "Additions:")?;
            for addition in &self.additions {
                writeln!(f, "  • {addition}")?;
            }
        }

        // Modifications
        if !self.modifications.is_empty() {
            writeln!(f)?;
            writeln!(f, "Modifications:")?;
            for modification in &self.modifications {
                writeln!(f, "  • {modification}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_report_is_compatible() {
        let report = CompatibilityReport::new();
        assert!(report.is_compatible);
        assert_eq!(report.suggested_bump, SuggestedBump::None);
    }

    #[test]
    fn test_breaking_change_makes_incompatible() {
        let mut report = CompatibilityReport::new();
        report.add_breaking_change(BreakingChange::OperationRemoved {
            operation_id: "test".to_string(),
            path: None,
        });

        assert!(!report.is_compatible);
        assert_eq!(report.suggested_bump, SuggestedBump::Major);
    }

    #[test]
    fn test_addition_suggests_minor() {
        let mut report = CompatibilityReport::new();
        report.add_addition(Addition::OperationAdded {
            operation_id: "test".to_string(),
            path: None,
            method: None,
        });

        assert!(report.is_compatible);
        assert_eq!(report.suggested_bump, SuggestedBump::Minor);
    }

    #[test]
    fn test_modification_suggests_patch() {
        let mut report = CompatibilityReport::new();
        report.add_modification(Modification::DescriptionChanged {
            location: "test".to_string(),
            old: None,
            new: None,
        });

        assert!(report.is_compatible);
        assert_eq!(report.suggested_bump, SuggestedBump::Patch);
    }

    #[test]
    fn test_major_trumps_minor() {
        let mut report = CompatibilityReport::new();
        report.add_addition(Addition::OperationAdded {
            operation_id: "test".to_string(),
            path: None,
            method: None,
        });
        report.add_breaking_change(BreakingChange::OperationRemoved {
            operation_id: "old".to_string(),
            path: None,
        });

        assert_eq!(report.suggested_bump, SuggestedBump::Major);
    }

    #[test]
    fn test_summary_formatting() {
        let mut report = CompatibilityReport::new();
        assert_eq!(report.summary(), "No changes detected");

        report.add_breaking_change(BreakingChange::OperationRemoved {
            operation_id: "test".to_string(),
            path: None,
        });
        assert!(report.summary().contains("1 breaking change"));
    }

    #[test]
    fn test_report_serialization() {
        let mut report = CompatibilityReport::new();
        report.set_old_version("1.0.0");
        report.set_new_version("2.0.0");
        report.add_breaking_change(BreakingChange::OperationRemoved {
            operation_id: "test".to_string(),
            path: None,
        });

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("is_compatible"));
        assert!(json.contains("breaking_changes"));
    }
}
