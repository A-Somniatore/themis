//! Main SDK struct and high-level API.
//!
//! This module provides the main `Themis` struct that offers a unified interface
//! to all SDK functionality.

use std::path::Path;

use themis_artifact::Artifact;
use themis_codegen::{GeneratedCode, GeneratorConfig};
use themis_compat::CompatibilityReport;
use themis_core::Contract;
use themis_lint::{LintConfig, LintReport};

use crate::artifact::ArtifactFormat;
use crate::error::SdkResult;
use crate::validate::ValidationResult;
use crate::Language;

/// Main Themis SDK struct.
///
/// This struct provides a unified interface to all Themis functionality,
/// including parsing, validation, linting, compatibility checking,
/// code generation, and artifact creation.
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::Themis;
///
/// let themis = Themis::new();
///
/// // Parse a contract
/// let contract = themis.parse_file("api.yaml")?;
///
/// // Validate and lint
/// let validation = themis.validate(&contract)?;
/// let lint_report = themis.lint(&contract);
///
/// // Generate code
/// let code = themis.generate(&contract, Language::Rust)?;
/// ```
#[derive(Debug, Default)]
pub struct Themis {
    lint_config: Option<LintConfig>,
    codegen_config: Option<GeneratorConfig>,
}

impl Themis {
    /// Create a new Themis SDK instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Themis SDK instance with custom lint configuration.
    #[must_use]
    pub fn with_lint_config(mut self, config: LintConfig) -> Self {
        self.lint_config = Some(config);
        self
    }

    /// Create a new Themis SDK instance with custom codegen configuration.
    #[must_use]
    pub fn with_codegen_config(mut self, config: GeneratorConfig) -> Self {
        self.codegen_config = Some(config);
        self
    }

    // ========================================================================
    // Parsing
    // ========================================================================

    /// Parse a contract from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the contract file
    ///
    /// # Returns
    ///
    /// The parsed contract
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> SdkResult<Contract> {
        crate::parse::parse_file(path)
    }

    /// Parse a contract from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - The contract content as a string
    ///
    /// # Returns
    ///
    /// The parsed contract
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be parsed
    pub fn parse_string(&self, content: &str) -> SdkResult<Contract> {
        crate::parse::parse_string(content)
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate a contract.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to validate
    ///
    /// # Returns
    ///
    /// A validation result with any errors or warnings
    ///
    /// # Errors
    ///
    /// Returns an error if validation cannot be performed
    pub fn validate(&self, contract: &Contract) -> SdkResult<ValidationResult> {
        crate::validate::validate(contract)
    }

    // ========================================================================
    // Linting
    // ========================================================================

    /// Lint a contract.
    ///
    /// Uses the configured lint configuration, or defaults if none set.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to lint
    ///
    /// # Returns
    ///
    /// A lint report with all findings
    #[must_use]
    pub fn lint(&self, contract: &Contract) -> LintReport {
        match &self.lint_config {
            Some(config) => crate::lint::lint_with_config(contract, config),
            None => crate::lint::lint(contract),
        }
    }

    /// Lint a contract with a specific configuration.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to lint
    /// * `config` - The lint configuration to use
    ///
    /// # Returns
    ///
    /// A lint report with all findings
    #[must_use]
    pub fn lint_with_config(&self, contract: &Contract, config: &LintConfig) -> LintReport {
        crate::lint::lint_with_config(contract, config)
    }

    // ========================================================================
    // Compatibility
    // ========================================================================

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
    #[must_use]
    pub fn check_compatibility(
        &self,
        old_contract: &Contract,
        new_contract: &Contract,
    ) -> CompatibilityReport {
        crate::compat::check_compatibility(old_contract, new_contract)
    }

    // ========================================================================
    // Code Generation
    // ========================================================================

    /// Generate code from a contract.
    ///
    /// Uses the configured codegen configuration, or defaults if none set.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to generate code from
    /// * `language` - The target language
    ///
    /// # Returns
    ///
    /// The generated code
    ///
    /// # Errors
    ///
    /// Returns an error if code generation fails
    pub fn generate(&self, contract: &Contract, language: Language) -> SdkResult<GeneratedCode> {
        match &self.codegen_config {
            Some(config) => crate::codegen::generate_with_config(contract, language, config),
            None => crate::codegen::generate(contract, language),
        }
    }

    /// Generate code from a contract with a specific configuration.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to generate code from
    /// * `language` - The target language
    /// * `config` - The codegen configuration to use
    ///
    /// # Returns
    ///
    /// The generated code
    ///
    /// # Errors
    ///
    /// Returns an error if code generation fails
    pub fn generate_with_config(
        &self,
        contract: &Contract,
        language: Language,
        config: &GeneratorConfig,
    ) -> SdkResult<GeneratedCode> {
        crate::codegen::generate_with_config(contract, language, config)
    }

    /// Generate code and write to a directory.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to generate code from
    /// * `language` - The target language
    /// * `output_dir` - Directory to write generated files to
    ///
    /// # Returns
    ///
    /// The list of generated file paths
    ///
    /// # Errors
    ///
    /// Returns an error if code generation or file writing fails
    pub fn generate_to_directory<P: AsRef<Path>>(
        &self,
        contract: &Contract,
        language: Language,
        output_dir: P,
    ) -> SdkResult<Vec<std::path::PathBuf>> {
        let code = self.generate(contract, language)?;
        crate::codegen::write_generated_code(&code, output_dir)
    }

    // ========================================================================
    // Artifacts
    // ========================================================================

    /// Create an artifact from a contract.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to create an artifact from
    ///
    /// # Returns
    ///
    /// The created artifact
    ///
    /// # Errors
    ///
    /// Returns an error if artifact creation fails
    pub fn create_artifact(&self, contract: &Contract) -> SdkResult<Artifact> {
        crate::artifact::create_artifact(contract)
    }

    /// Save an artifact to a file.
    ///
    /// # Arguments
    ///
    /// * `artifact` - The artifact to save
    /// * `path` - Path to save the artifact to
    /// * `format` - The output format
    ///
    /// # Returns
    ///
    /// Ok if successful
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file writing fails
    pub fn save_artifact<P: AsRef<Path>>(
        &self,
        artifact: &Artifact,
        path: P,
        format: ArtifactFormat,
    ) -> SdkResult<()> {
        crate::artifact::save_artifact(artifact, path, format)
    }

    /// Load an artifact from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the artifact file
    ///
    /// # Returns
    ///
    /// The loaded artifact
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or deserialization fails
    pub fn load_artifact<P: AsRef<Path>>(&self, path: P) -> SdkResult<Artifact> {
        crate::artifact::load_artifact(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themis_new() {
        let themis = Themis::new();
        assert!(themis.lint_config.is_none());
        assert!(themis.codegen_config.is_none());
    }

    #[test]
    fn test_themis_with_lint_config() {
        let config = LintConfig::default();
        let themis = Themis::new().with_lint_config(config);
        assert!(themis.lint_config.is_some());
    }

    #[test]
    fn test_themis_with_codegen_config() {
        let config = GeneratorConfig::default();
        let themis = Themis::new().with_codegen_config(config);
        assert!(themis.codegen_config.is_some());
    }

    #[test]
    fn test_themis_debug() {
        let themis = Themis::new();
        let debug = format!("{:?}", themis);
        assert!(debug.contains("Themis"));
    }

    #[test]
    fn test_themis_default() {
        let themis = Themis::default();
        assert!(themis.lint_config.is_none());
    }
}
