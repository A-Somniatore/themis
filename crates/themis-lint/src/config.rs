//! Configuration file support for themis-lint.
//!
//! This module provides functionality for loading lint configuration from
//! `.themis-lint.yaml` files.
//!
//! # Configuration File Format
//!
//! ```yaml
//! # .themis-lint.yaml
//! extends: strict  # Optional: "default", "strict", or "relaxed"
//!
//! rules:
//!   naming/operation-id:
//!     enabled: true
//!     severity: error
//!   
//!   naming/path-format:
//!     enabled: true
//!     severity: warning
//!   
//!   docs/operation-summary:
//!     enabled: false  # Disable this rule
//! ```

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::reporter::{LintConfig, Severity};
use crate::rule::RuleConfig;

/// Error type for configuration loading.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read configuration file.
    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse configuration file.
    #[error("Failed to parse configuration file: {0}")]
    ParseError(#[from] serde_yaml::Error),

    /// Unknown base configuration.
    #[error("Unknown base configuration: {0}. Valid options: default, strict, relaxed")]
    UnknownBase(String),
}

/// Result type for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// File-based lint configuration.
///
/// This structure represents the configuration loaded from a `.themis-lint.yaml` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintConfigFile {
    /// Base configuration to extend from.
    /// Valid values: "default", "strict", "relaxed"
    #[serde(default)]
    pub extends: Option<String>,

    /// Per-rule configuration overrides.
    #[serde(default)]
    pub rules: HashMap<String, RuleConfigFile>,
}

impl Default for LintConfigFile {
    fn default() -> Self {
        Self {
            extends: None,
            rules: HashMap::new(),
        }
    }
}

/// Per-rule configuration in the file format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleConfigFile {
    /// Whether the rule is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Severity level: "error", "warning", or "info".
    #[serde(default = "default_severity")]
    pub severity: String,
}

fn default_enabled() -> bool {
    true
}

fn default_severity() -> String {
    "warning".to_string()
}

impl Default for RuleConfigFile {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            severity: default_severity(),
        }
    }
}

impl LintConfigFile {
    /// Creates a new empty configuration file.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: &Path) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parses configuration from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML is invalid.
    pub fn from_str(content: &str) -> ConfigResult<Self> {
        let config: LintConfigFile = serde_yaml::from_str(content)?;
        Ok(config)
    }

    /// Converts this file configuration to a `LintConfig`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `extends` value is unknown.
    pub fn to_lint_config(&self) -> ConfigResult<LintConfig> {
        // Start with base configuration
        let mut config = match self.extends.as_deref() {
            None | Some("default") => LintConfig::default(),
            Some("strict") => LintConfig::strict(),
            Some("relaxed") => LintConfig::relaxed(),
            Some(other) => return Err(ConfigError::UnknownBase(other.to_string())),
        };

        // Apply overrides
        for (rule_id, rule_config) in &self.rules {
            let severity = parse_severity(&rule_config.severity);
            if rule_config.enabled {
                config.enable(rule_id, severity);
            } else {
                config.disable(rule_id);
            }
        }

        Ok(config)
    }

    /// Searches for a configuration file in the given directory and parent directories.
    ///
    /// Looks for files named `.themis-lint.yaml` or `.themis-lint.yml`.
    #[must_use]
    pub fn find_config_file(start_dir: &Path) -> Option<std::path::PathBuf> {
        let config_names = [".themis-lint.yaml", ".themis-lint.yml", "themis-lint.yaml"];

        let mut current = Some(start_dir);
        while let Some(dir) = current {
            for name in &config_names {
                let config_path = dir.join(name);
                if config_path.exists() {
                    return Some(config_path);
                }
            }
            current = dir.parent();
        }
        None
    }

    /// Loads configuration from the nearest config file, or returns default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if a config file is found but cannot be parsed.
    pub fn load_or_default(start_dir: &Path) -> ConfigResult<LintConfig> {
        if let Some(config_path) = Self::find_config_file(start_dir) {
            let file_config = Self::from_file(&config_path)?;
            file_config.to_lint_config()
        } else {
            Ok(LintConfig::default())
        }
    }
}

/// Parses a severity string to a `Severity` enum.
fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "error" => Severity::Error,
        "warning" | "warn" => Severity::Warning,
        "info" | "hint" => Severity::Info,
        _ => Severity::Warning,
    }
}

/// Converts a `RuleConfig` to `RuleConfigFile` for serialization.
impl From<&RuleConfig> for RuleConfigFile {
    fn from(config: &RuleConfig) -> Self {
        Self {
            enabled: config.enabled,
            severity: match config.severity {
                Severity::Error => "error".to_string(),
                Severity::Warning => "warning".to_string(),
                Severity::Info => "info".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_empty_config() {
        let yaml = "";
        let config = LintConfigFile::from_str(yaml).unwrap();
        assert!(config.extends.is_none());
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_parse_extends_default() {
        let yaml = r#"
extends: default
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        assert_eq!(config.extends, Some("default".to_string()));
    }

    #[test]
    fn test_parse_extends_strict() {
        let yaml = r#"
extends: strict
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        let lint_config = config.to_lint_config().unwrap();

        // All rules should be errors in strict mode
        for rule_config in lint_config.rules.values() {
            if rule_config.enabled {
                assert_eq!(rule_config.severity, Severity::Error);
            }
        }
    }

    #[test]
    fn test_parse_extends_relaxed() {
        let yaml = r#"
extends: relaxed
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        let lint_config = config.to_lint_config().unwrap();

        // All rules should be warnings in relaxed mode
        for rule_config in lint_config.rules.values() {
            if rule_config.enabled {
                assert_eq!(rule_config.severity, Severity::Warning);
            }
        }
    }

    #[test]
    fn test_parse_extends_unknown() {
        let yaml = r#"
extends: unknown
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        let result = config.to_lint_config();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::UnknownBase(_)));
    }

    #[test]
    fn test_parse_rule_enabled() {
        let yaml = r#"
rules:
  naming/operation-id:
    enabled: true
    severity: error
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        let lint_config = config.to_lint_config().unwrap();

        let rule = lint_config.get_rule_config("naming/operation-id");
        assert!(rule.enabled);
        assert_eq!(rule.severity, Severity::Error);
    }

    #[test]
    fn test_parse_rule_disabled() {
        let yaml = r#"
rules:
  naming/operation-id:
    enabled: false
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        let lint_config = config.to_lint_config().unwrap();

        let rule = lint_config.get_rule_config("naming/operation-id");
        assert!(!rule.enabled);
    }

    #[test]
    fn test_parse_severity_variations() {
        assert_eq!(parse_severity("error"), Severity::Error);
        assert_eq!(parse_severity("ERROR"), Severity::Error);
        assert_eq!(parse_severity("warning"), Severity::Warning);
        assert_eq!(parse_severity("warn"), Severity::Warning);
        assert_eq!(parse_severity("info"), Severity::Info);
        assert_eq!(parse_severity("hint"), Severity::Info);
        assert_eq!(parse_severity("unknown"), Severity::Warning); // default
    }

    #[test]
    fn test_parse_full_config() {
        let yaml = r#"
extends: default

rules:
  naming/operation-id:
    enabled: true
    severity: error
  
  naming/path-format:
    enabled: true
    severity: warning
  
  docs/operation-summary:
    enabled: false
"#;
        let config = LintConfigFile::from_str(yaml).unwrap();
        let lint_config = config.to_lint_config().unwrap();

        // Check operation-id
        let op_id = lint_config.get_rule_config("naming/operation-id");
        assert!(op_id.enabled);
        assert_eq!(op_id.severity, Severity::Error);

        // Check path-format
        let path_fmt = lint_config.get_rule_config("naming/path-format");
        assert!(path_fmt.enabled);
        assert_eq!(path_fmt.severity, Severity::Warning);

        // Check operation-summary is disabled
        let op_summary = lint_config.get_rule_config("docs/operation-summary");
        assert!(!op_summary.enabled);
    }

    #[test]
    fn test_find_config_file_in_current_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".themis-lint.yaml");
        std::fs::write(&config_path, "extends: default").unwrap();

        let found = LintConfigFile::find_config_file(temp_dir.path());
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_config_file_in_parent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let config_path = temp_dir.path().join(".themis-lint.yaml");
        std::fs::write(&config_path, "extends: default").unwrap();

        let found = LintConfigFile::find_config_file(&subdir);
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_config_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let found = LintConfigFile::find_config_file(temp_dir.path());
        assert!(found.is_none());
    }

    #[test]
    fn test_load_or_default_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".themis-lint.yaml");

        let yaml = r#"
extends: strict
rules:
  naming/operation-id:
    enabled: false
"#;
        std::fs::write(&config_path, yaml).unwrap();

        let config = LintConfigFile::load_or_default(temp_dir.path()).unwrap();

        // Should have strict base with operation-id disabled
        let op_id = config.get_rule_config("naming/operation-id");
        assert!(!op_id.enabled);
    }

    #[test]
    fn test_load_or_default_without_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = LintConfigFile::load_or_default(temp_dir.path()).unwrap();

        // Should return default configuration
        // Just verify it's valid
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".themis-lint.yaml");

        let yaml = "extends: relaxed";
        let mut file = std::fs::File::create(&config_path).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let config = LintConfigFile::from_file(&config_path).unwrap();
        assert_eq!(config.extends, Some("relaxed".to_string()));
    }

    #[test]
    fn test_rule_config_file_from_rule_config() {
        let config = RuleConfig::enabled(Severity::Error);
        let file_config: RuleConfigFile = (&config).into();

        assert!(file_config.enabled);
        assert_eq!(file_config.severity, "error");
    }

    #[test]
    fn test_invalid_yaml() {
        let yaml = "invalid: yaml: content:";
        let result = LintConfigFile::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_yml_extension() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".themis-lint.yml");
        std::fs::write(&config_path, "extends: default").unwrap();

        let found = LintConfigFile::find_config_file(temp_dir.path());
        assert_eq!(found, Some(config_path));
    }
}
