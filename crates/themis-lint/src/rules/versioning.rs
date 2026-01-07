//! Versioning lint rules for API contracts.
//!
//! These rules enforce best practices for API versioning and
//! help maintain clear version semantics:
//! - THEMIS014: Version 0.0.0 is invalid
//! - THEMIS015: No pre-release versions in production
//! - THEMIS016: Version should be meaningful
//! - THEMIS017: Major version 0 indicates unstable API

use crate::reporter::LintIssue;
use crate::rule::{Rule, RuleConfig};
use themis_core::Contract;

/// Returns all versioning rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(RequireSemanticVersion),
        Box::new(NoPreReleaseInProduction),
        Box::new(VersionInInfo),
        Box::new(NoZeroMajorVersion),
    ]
}

// =============================================================================
// THEMIS014: Require Semantic Version
// =============================================================================

/// Checks that API version is a valid semantic version (not 0.0.0).
///
/// Version 0.0.0 is not a meaningful semantic version and should
/// never be used for an API contract.
///
/// # Rule ID
///
/// `versioning/require-semantic-version` (THEMIS014)
pub struct RequireSemanticVersion;

impl Rule for RequireSemanticVersion {
    fn id(&self) -> &'static str {
        "versioning/require-semantic-version"
    }

    fn description(&self) -> &'static str {
        "API version should follow semantic versioning format (major.minor.patch)"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let version = &contract.version;

        if version.major == 0 && version.minor == 0 && version.patch == 0 {
            return vec![LintIssue {
                rule: self.id().to_string(),
                severity: config.severity,
                message: "Version 0.0.0 is not a valid semantic version for an API. \
                          Use at least version 0.1.0 for initial development."
                    .to_string(),
                location: Some("info.version".to_string()),
            }];
        }

        Vec::new()
    }
}

// =============================================================================
// THEMIS015: No Pre-Release in Production
// =============================================================================

/// Checks that production APIs don't have pre-release versions.
///
/// Pre-release versions (alpha, beta, rc) indicate unstable APIs that
/// may change without notice. Production consumers need stable versions.
///
/// # Rule ID
///
/// `versioning/no-pre-release-in-production` (THEMIS015)
pub struct NoPreReleaseInProduction;

impl Rule for NoPreReleaseInProduction {
    fn id(&self) -> &'static str {
        "versioning/no-pre-release-in-production"
    }

    fn description(&self) -> &'static str {
        "Production APIs should not use pre-release versions"
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        if let Some(pre_release) = &contract.version.pre_release {
            return vec![LintIssue {
                rule: self.id().to_string(),
                severity: config.severity,
                message: format!(
                    "Pre-release version '{pre_release}' detected - not recommended for production APIs. \
                     Remove pre-release identifier for production deployment."
                ),
                location: Some("info.version".to_string()),
            }];
        }

        Vec::new()
    }
}

// =============================================================================
// THEMIS016: Version in Info
// =============================================================================

/// Placeholder rule for checking meaningful version information.
///
/// This rule is informational and checks that version information
/// is properly specified in the contract.
///
/// # Rule ID
///
/// `versioning/version-in-info` (THEMIS016)
pub struct VersionInInfo;

impl Rule for VersionInInfo {
    fn id(&self) -> &'static str {
        "versioning/version-in-info"
    }

    fn description(&self) -> &'static str {
        "API should have a meaningful version in the info block"
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::enabled(crate::reporter::Severity::Info)
    }

    fn check(&self, _contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        // This rule is more informational - we check for default-like versions
        // The actual parsing happens earlier, so we just verify meaningful content
        Vec::new()
    }
}

// =============================================================================
// THEMIS017: No Zero Major Version in Production
// =============================================================================

/// Checks that production APIs have major version >= 1.
///
/// Version 0.x.x indicates development/unstable API according to semver.
/// Production APIs should be at least v1.0.0 to indicate stability.
///
/// This rule is disabled by default since many APIs legitimately
/// start development at 0.x versions.
///
/// # Rule ID
///
/// `versioning/no-zero-major-version` (THEMIS017)
pub struct NoZeroMajorVersion;

impl Rule for NoZeroMajorVersion {
    fn id(&self) -> &'static str {
        "versioning/no-zero-major-version"
    }

    fn description(&self) -> &'static str {
        "Production APIs should have major version >= 1"
    }

    fn default_config(&self) -> RuleConfig {
        // Disabled by default - many APIs start at 0.x
        RuleConfig::disabled()
    }

    fn check(&self, contract: &Contract, config: &RuleConfig) -> Vec<LintIssue> {
        if !config.enabled {
            return Vec::new();
        }

        let version = &contract.version;

        if version.major == 0 {
            return vec![LintIssue {
                rule: self.id().to_string(),
                severity: config.severity,
                message: format!(
                    "API version {}.{}.{} has major version 0, indicating unstable/development status. \
                     Consider releasing as v1.0.0 when the API is production-ready.",
                    version.major, version.minor, version.patch
                ),
                location: Some("info.version".to_string()),
            }];
        }

        Vec::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::Severity;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::Version;

    fn create_test_contract() -> Contract {
        Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "test-api".to_string(),
                description: Some("Test API".to_string()),
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: Default::default(),
            schemas: Default::default(),
            security_schemes: Default::default(),
        }
    }

    fn enabled_config() -> RuleConfig {
        RuleConfig::enabled(Severity::Warning)
    }

    fn disabled_config() -> RuleConfig {
        RuleConfig::disabled()
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 4);
    }

    // =========================================================================
    // RequireSemanticVersion tests
    // =========================================================================

    #[test]
    fn test_require_semver_valid_version() {
        let contract = create_test_contract();
        let rule = RequireSemanticVersion;

        let issues = rule.check(&contract, &enabled_config());
        assert!(issues.is_empty());
    }

    #[test]
    fn test_require_semver_zero_version() {
        let mut contract = create_test_contract();
        contract.version = Version::new(0, 0, 0);

        let rule = RequireSemanticVersion;
        let issues = rule.check(&contract, &enabled_config());

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "versioning/require-semantic-version");
        assert!(issues[0].message.contains("0.0.0"));
    }

    #[test]
    fn test_require_semver_disabled() {
        let mut contract = create_test_contract();
        contract.version = Version::new(0, 0, 0);

        let rule = RequireSemanticVersion;
        let issues = rule.check(&contract, &disabled_config());

        assert!(issues.is_empty());
    }

    // =========================================================================
    // NoPreReleaseInProduction tests
    // =========================================================================

    #[test]
    fn test_no_pre_release_stable_version() {
        let contract = create_test_contract();
        let rule = NoPreReleaseInProduction;

        let issues = rule.check(&contract, &enabled_config());
        assert!(issues.is_empty());
    }

    #[test]
    fn test_no_pre_release_alpha_version() {
        let mut contract = create_test_contract();
        contract.version = Version::with_pre_release(1, 0, 0, "alpha");

        let rule = NoPreReleaseInProduction;
        let issues = rule.check(&contract, &enabled_config());

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "versioning/no-pre-release-in-production");
        assert!(issues[0].message.contains("alpha"));
    }

    #[test]
    fn test_no_pre_release_beta_version() {
        let mut contract = create_test_contract();
        contract.version = Version::with_pre_release(2, 0, 0, "beta.1");

        let rule = NoPreReleaseInProduction;
        let issues = rule.check(&contract, &enabled_config());

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("beta.1"));
    }

    // =========================================================================
    // VersionInInfo tests
    // =========================================================================

    #[test]
    fn test_version_in_info_rule_exists() {
        let rule = VersionInInfo;
        assert_eq!(rule.id(), "versioning/version-in-info");
    }

    // =========================================================================
    // NoZeroMajorVersion tests
    // =========================================================================

    #[test]
    fn test_no_zero_major_disabled_by_default() {
        let mut contract = create_test_contract();
        contract.version = Version::new(0, 1, 0);

        let rule = NoZeroMajorVersion;

        // Default config should be disabled
        let default_cfg = rule.default_config();
        assert!(!default_cfg.enabled);

        let issues = rule.check(&contract, &default_cfg);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_no_zero_major_when_enabled() {
        let mut contract = create_test_contract();
        contract.version = Version::new(0, 5, 0);

        let rule = NoZeroMajorVersion;
        let issues = rule.check(&contract, &enabled_config());

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "versioning/no-zero-major-version");
        assert!(issues[0].message.contains("major version 0"));
    }

    #[test]
    fn test_no_zero_major_passes_for_v1() {
        let contract = create_test_contract(); // v1.0.0

        let rule = NoZeroMajorVersion;
        let issues = rule.check(&contract, &enabled_config());

        assert!(issues.is_empty());
    }

    #[test]
    fn test_severity_from_config() {
        let mut contract = create_test_contract();
        contract.version = Version::new(0, 0, 0);

        let rule = RequireSemanticVersion;

        let warning_cfg = RuleConfig::enabled(Severity::Warning);
        let issues = rule.check(&contract, &warning_cfg);
        assert_eq!(issues[0].severity, Severity::Warning);

        let error_cfg = RuleConfig::enabled(Severity::Error);
        let issues = rule.check(&contract, &error_cfg);
        assert_eq!(issues[0].severity, Severity::Error);
    }
}

