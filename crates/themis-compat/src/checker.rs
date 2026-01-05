//! High-level compatibility checking API.
//!
//! Provides the main entry point for comparing contracts and validating version bumps.

use crate::differ::diff_contracts;
use crate::report::{CompatibilityReport, SuggestedBump};
use semver::Version as SemVersion;
use std::fmt;
use themis_core::Contract;

/// Error type for compatibility checking.
#[derive(Debug, Clone)]
pub enum CompatibilityError {
    /// The version bump is insufficient for the detected changes.
    InsufficientVersionBump {
        /// The expected minimum bump type.
        expected: SuggestedBump,
        /// The actual bump detected.
        actual: SuggestedBump,
        /// The old version.
        old_version: String,
        /// The new version.
        new_version: String,
    },

    /// Failed to parse a version string.
    InvalidVersion {
        /// The invalid version string.
        version: String,
        /// The parse error message.
        reason: String,
    },
}

impl std::error::Error for CompatibilityError {}

impl fmt::Display for CompatibilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientVersionBump {
                expected,
                actual,
                old_version,
                new_version,
            } => write!(
                f,
                "Version bump from {old_version} to {new_version} ({actual}) is insufficient; \
                 a {expected} bump is required for the detected changes"
            ),
            Self::InvalidVersion { version, reason } => {
                write!(f, "Invalid version '{version}': {reason}")
            }
        }
    }
}

/// Configuration for compatibility checking.
#[derive(Debug, Clone)]
pub struct CompatibilityConfig {
    /// Whether to validate version bumps against detected changes.
    pub validate_versions: bool,

    /// Whether to treat warnings as errors.
    pub strict: bool,
}

impl Default for CompatibilityConfig {
    fn default() -> Self {
        Self {
            validate_versions: true,
            strict: false,
        }
    }
}

/// Checks compatibility between contract versions.
///
/// # Examples
///
/// ```ignore
/// use themis_compat::CompatibilityChecker;
///
/// let checker = CompatibilityChecker::new();
/// let report = checker.check(&old_contract, &new_contract)?;
///
/// if !report.is_compatible {
///     println!("Breaking changes detected!");
/// }
/// ```
pub struct CompatibilityChecker {
    config: CompatibilityConfig,
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl CompatibilityChecker {
    /// Creates a new compatibility checker with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: CompatibilityConfig::default(),
        }
    }

    /// Creates a new compatibility checker with the given configuration.
    #[must_use]
    pub const fn with_config(config: CompatibilityConfig) -> Self {
        Self { config }
    }

    /// Sets whether to validate version bumps.
    #[must_use]
    pub const fn validate_versions(mut self, validate: bool) -> Self {
        self.config.validate_versions = validate;
        self
    }

    /// Sets whether to use strict mode.
    #[must_use]
    pub const fn strict(mut self, strict: bool) -> Self {
        self.config.strict = strict;
        self
    }

    /// Compares two contracts and produces a compatibility report.
    ///
    /// # Arguments
    ///
    /// * `old` - The old (baseline) contract
    /// * `new` - The new contract to compare
    ///
    /// # Returns
    ///
    /// A `CompatibilityReport` containing all detected changes.
    ///
    /// # Errors
    ///
    /// Returns `CompatibilityError` if version validation is enabled and the
    /// version bump is insufficient for the detected changes.
    pub fn check(
        &self,
        old: &Contract,
        new: &Contract,
    ) -> Result<CompatibilityReport, CompatibilityError> {
        let report = diff_contracts(old, new);

        // Validate version bump if configured
        if self.config.validate_versions {
            if let (Some(old_v), Some(new_v)) = (&report.old_version, &report.new_version) {
                Self::validate_version_bump(old_v, new_v, &report)?;
            }
        }

        Ok(report)
    }

    /// Validates that the version bump matches the detected changes.
    fn validate_version_bump(
        old_version: &str,
        new_version: &str,
        report: &CompatibilityReport,
    ) -> Result<(), CompatibilityError> {
        let old = parse_version(old_version)?;
        let new = parse_version(new_version)?;

        let actual_bump = detect_bump(&old, &new);
        let required_bump = report.suggested_bump;

        // Check if bump is sufficient
        if !is_sufficient_bump(actual_bump, required_bump) {
            return Err(CompatibilityError::InsufficientVersionBump {
                expected: required_bump,
                actual: actual_bump,
                old_version: old_version.to_string(),
                new_version: new_version.to_string(),
            });
        }

        Ok(())
    }
}

/// Parses a version string into a semver Version.
fn parse_version(version: &str) -> Result<SemVersion, CompatibilityError> {
    SemVersion::parse(version).map_err(|e| CompatibilityError::InvalidVersion {
        version: version.to_string(),
        reason: e.to_string(),
    })
}

/// Detects what kind of version bump occurred.
const fn detect_bump(old: &SemVersion, new: &SemVersion) -> SuggestedBump {
    if new.major > old.major {
        SuggestedBump::Major
    } else if new.minor > old.minor {
        SuggestedBump::Minor
    } else if new.patch > old.patch {
        SuggestedBump::Patch
    } else {
        SuggestedBump::None
    }
}

/// Checks if the actual bump is sufficient for the required bump.
fn is_sufficient_bump(actual: SuggestedBump, required: SuggestedBump) -> bool {
    match required {
        SuggestedBump::Major => actual == SuggestedBump::Major,
        SuggestedBump::Minor => {
            matches!(actual, SuggestedBump::Major | SuggestedBump::Minor)
        }
        SuggestedBump::Patch => {
            matches!(
                actual,
                SuggestedBump::Major | SuggestedBump::Minor | SuggestedBump::Patch
            )
        }
        SuggestedBump::None => true,
    }
}

/// Convenience function to check compatibility between two contracts.
///
/// # Arguments
///
/// * `old` - The old (baseline) contract
/// * `new` - The new contract to compare
///
/// # Returns
///
/// A `CompatibilityReport` containing all detected changes.
#[must_use]
pub fn check_compatibility(old: &Contract, new: &Contract) -> CompatibilityReport {
    diff_contracts(old, new)
}

/// Convenience function to check if a version bump is valid.
///
/// # Arguments
///
/// * `old` - The old (baseline) contract
/// * `new` - The new contract to compare
///
/// # Returns
///
/// `Ok(report)` if the version bump is valid, or an error if insufficient.
///
/// # Errors
///
/// Returns `CompatibilityError::InsufficientVersionBump` if the version
/// bump does not match the detected changes.
pub fn validate_version_bump(
    old: &Contract,
    new: &Contract,
) -> Result<CompatibilityReport, CompatibilityError> {
    CompatibilityChecker::new().check(old, new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::HttpMethod;
    use themis_core::{Operation, Version};

    fn create_contract(version: &str) -> Contract {
        let parts: Vec<u32> = version.split('.').map(|s| s.parse().unwrap_or(0)).collect();
        Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(
                *parts.first().unwrap_or(&0),
                *parts.get(1).unwrap_or(&0),
                *parts.get(2).unwrap_or(&0),
            ),
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
    fn test_checker_default_config() {
        let checker = CompatibilityChecker::new();
        assert!(checker.config.validate_versions);
        assert!(!checker.config.strict);
    }

    #[test]
    fn test_checker_builder_pattern() {
        let checker = CompatibilityChecker::new()
            .validate_versions(false)
            .strict(true);
        assert!(!checker.config.validate_versions);
        assert!(checker.config.strict);
    }

    #[test]
    fn test_check_identical_contracts() {
        let old = create_contract("1.0.0");
        let new = create_contract("1.0.0");

        let checker = CompatibilityChecker::new().validate_versions(false);
        let report = checker.check(&old, &new).unwrap();

        assert!(report.is_compatible);
        assert!(report.is_unchanged());
    }

    #[test]
    fn test_check_with_operation_added() {
        let old = create_contract("1.0.0");

        let mut new = create_contract("1.1.0");
        let mut op = Operation::new("getUser");
        op.path = Some("/users/{id}".to_string());
        op.method = Some(HttpMethod::Get);
        new.operations.insert("getUser".to_string(), op);

        let checker = CompatibilityChecker::new();
        let report = checker.check(&old, &new).unwrap();

        assert!(report.is_compatible);
        assert_eq!(report.additions.len(), 1);
    }

    #[test]
    fn test_validate_insufficient_version_bump() {
        let mut old = create_contract("1.0.0");
        let mut op = Operation::new("getUser");
        op.path = Some("/users/{id}".to_string());
        op.method = Some(HttpMethod::Get);
        old.operations.insert("getUser".to_string(), op);

        // Removing an operation is a breaking change, requires major bump
        let new = create_contract("1.0.1"); // Only patch bump

        let checker = CompatibilityChecker::new();
        let result = checker.check(&old, &new);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CompatibilityError::InsufficientVersionBump { .. }
        ));
    }

    #[test]
    fn test_validate_sufficient_major_bump() {
        let mut old = create_contract("1.0.0");
        let mut op = Operation::new("getUser");
        op.path = Some("/users/{id}".to_string());
        op.method = Some(HttpMethod::Get);
        old.operations.insert("getUser".to_string(), op);

        let new = create_contract("2.0.0"); // Major bump

        let checker = CompatibilityChecker::new();
        let result = checker.check(&old, &new);

        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_bump_major() {
        let old = SemVersion::parse("1.2.3").unwrap();
        let new = SemVersion::parse("2.0.0").unwrap();
        assert_eq!(detect_bump(&old, &new), SuggestedBump::Major);
    }

    #[test]
    fn test_detect_bump_minor() {
        let old = SemVersion::parse("1.2.3").unwrap();
        let new = SemVersion::parse("1.3.0").unwrap();
        assert_eq!(detect_bump(&old, &new), SuggestedBump::Minor);
    }

    #[test]
    fn test_detect_bump_patch() {
        let old = SemVersion::parse("1.2.3").unwrap();
        let new = SemVersion::parse("1.2.4").unwrap();
        assert_eq!(detect_bump(&old, &new), SuggestedBump::Patch);
    }

    #[test]
    fn test_detect_bump_none() {
        let old = SemVersion::parse("1.2.3").unwrap();
        let new = SemVersion::parse("1.2.3").unwrap();
        assert_eq!(detect_bump(&old, &new), SuggestedBump::None);
    }

    #[test]
    fn test_is_sufficient_bump() {
        // Major required
        assert!(is_sufficient_bump(
            SuggestedBump::Major,
            SuggestedBump::Major
        ));
        assert!(!is_sufficient_bump(
            SuggestedBump::Minor,
            SuggestedBump::Major
        ));
        assert!(!is_sufficient_bump(
            SuggestedBump::Patch,
            SuggestedBump::Major
        ));

        // Minor required
        assert!(is_sufficient_bump(
            SuggestedBump::Major,
            SuggestedBump::Minor
        ));
        assert!(is_sufficient_bump(
            SuggestedBump::Minor,
            SuggestedBump::Minor
        ));
        assert!(!is_sufficient_bump(
            SuggestedBump::Patch,
            SuggestedBump::Minor
        ));

        // Patch required
        assert!(is_sufficient_bump(
            SuggestedBump::Major,
            SuggestedBump::Patch
        ));
        assert!(is_sufficient_bump(
            SuggestedBump::Minor,
            SuggestedBump::Patch
        ));
        assert!(is_sufficient_bump(
            SuggestedBump::Patch,
            SuggestedBump::Patch
        ));
    }

    #[test]
    fn test_convenience_functions() {
        let old = create_contract("1.0.0");
        let new = create_contract("1.0.0");

        let report = check_compatibility(&old, &new);
        assert!(report.is_compatible);

        let result = validate_version_bump(&old, &new);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let error = CompatibilityError::InsufficientVersionBump {
            expected: SuggestedBump::Major,
            actual: SuggestedBump::Patch,
            old_version: "1.0.0".to_string(),
            new_version: "1.0.1".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("1.0.0"));
        assert!(msg.contains("1.0.1"));
        assert!(msg.contains("major"));
    }
}
