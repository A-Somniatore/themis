//! Contract compatibility checking functionality.
//!
//! This module provides functions for checking compatibility between contract versions.

use std::path::Path;

use themis_compat::{check_compatibility as compat_check, CompatibilityReport, SuggestedBump};
use themis_core::Contract;

use crate::error::{SdkError, SdkResult};

/// Check compatibility between two contracts.
///
/// # Arguments
///
/// * `old_contract` - The old/previous version of the contract
/// * `new_contract` - The new/current version of the contract
///
/// # Returns
///
/// A compatibility report detailing all changes
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::compat::check_compatibility;
/// use themis_sdk::parse::parse_file;
///
/// let old = parse_file("api-v1.yaml")?;
/// let new = parse_file("api-v2.yaml")?;
/// let report = check_compatibility(&old, &new);
///
/// if !report.is_compatible {
///     eprintln!("Breaking changes detected!");
///     for change in &report.breaking_changes {
///         eprintln!("  - {}", change);
///     }
/// }
/// ```
#[must_use]
pub fn check_compatibility(
    old_contract: &Contract,
    new_contract: &Contract,
) -> CompatibilityReport {
    compat_check(old_contract, new_contract)
}

/// Check compatibility between two contract files.
///
/// # Arguments
///
/// * `old_path` - Path to the old contract file
/// * `new_path` - Path to the new contract file
///
/// # Returns
///
/// A compatibility report detailing all changes
///
/// # Errors
///
/// Returns an error if:
/// - Either file cannot be read
/// - Either contract cannot be parsed
pub fn check_compatibility_files<P: AsRef<Path>, Q: AsRef<Path>>(
    old_path: P,
    new_path: Q,
) -> SdkResult<CompatibilityReport> {
    let old_contract = crate::parse::parse_file(old_path)?;
    let new_contract = crate::parse::parse_file(new_path)?;
    Ok(check_compatibility(&old_contract, &new_contract))
}

/// Summarize a compatibility report into a simple result.
#[derive(Debug, Clone)]
pub struct CompatibilitySummary {
    /// Whether the contracts are backward compatible.
    pub is_backward_compatible: bool,
    /// Number of breaking changes.
    pub breaking_changes_count: usize,
    /// Number of additions.
    pub additions_count: usize,
    /// Number of modifications.
    pub modifications_count: usize,
    /// Suggested version bump based on changes.
    pub suggested_version_bump: SuggestedBump,
}

/// Get a summary of a compatibility report.
///
/// # Arguments
///
/// * `report` - The compatibility report to summarize
///
/// # Returns
///
/// A summary of the compatibility report
#[must_use]
pub fn summarize(report: &CompatibilityReport) -> CompatibilitySummary {
    CompatibilitySummary {
        is_backward_compatible: report.is_compatible,
        breaking_changes_count: report.breaking_changes.len(),
        additions_count: report.additions.len(),
        modifications_count: report.modifications.len(),
        suggested_version_bump: report.suggested_bump,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggested_bump_display() {
        assert_eq!(SuggestedBump::None.to_string(), "none");
        assert_eq!(SuggestedBump::Patch.to_string(), "patch");
        assert_eq!(SuggestedBump::Minor.to_string(), "minor");
        assert_eq!(SuggestedBump::Major.to_string(), "major");
    }

    #[test]
    fn test_suggested_bump_equality() {
        assert_eq!(SuggestedBump::Major, SuggestedBump::Major);
        assert_ne!(SuggestedBump::Major, SuggestedBump::Minor);
    }

    #[test]
    fn test_compatibility_summary_debug() {
        let summary = CompatibilitySummary {
            is_backward_compatible: true,
            breaking_changes_count: 0,
            additions_count: 0,
            modifications_count: 0,
            suggested_version_bump: SuggestedBump::None,
        };
        let debug = format!("{:?}", summary);
        assert!(debug.contains("is_backward_compatible"));
    }
}
