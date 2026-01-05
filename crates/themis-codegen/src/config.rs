//! Configuration for code generators.

use serde::{Deserialize, Serialize};

/// Configuration options for code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Whether to add validation derives/decorators.
    pub include_validation: bool,

    /// Whether to generate serde derives (Rust) or equivalent.
    pub include_serialization: bool,

    /// Whether to generate doc comments from descriptions.
    pub include_docs: bool,

    /// Whether to generate builder patterns for request types.
    pub generate_builders: bool,

    /// The naming convention for generated types.
    pub type_naming: NamingConvention,

    /// The naming convention for generated fields.
    pub field_naming: NamingConvention,

    /// Module or package name for generated code.
    pub module_name: Option<String>,

    /// Whether to flatten single-field wrapper types.
    pub flatten_wrappers: bool,

    /// Prefix to add to all generated type names.
    pub type_prefix: Option<String>,

    /// Suffix to add to all generated type names.
    pub type_suffix: Option<String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            include_validation: true,
            include_serialization: true,
            include_docs: true,
            generate_builders: false,
            type_naming: NamingConvention::PascalCase,
            field_naming: NamingConvention::SnakeCase,
            module_name: None,
            flatten_wrappers: false,
            type_prefix: None,
            type_suffix: None,
        }
    }
}

impl GeneratorConfig {
    /// Creates a new configuration with default values.
    pub const fn new() -> Self {
        Self {
            include_validation: true,
            include_serialization: true,
            include_docs: true,
            generate_builders: false,
            type_naming: NamingConvention::PascalCase,
            field_naming: NamingConvention::SnakeCase,
            module_name: None,
            flatten_wrappers: false,
            type_prefix: None,
            type_suffix: None,
        }
    }

    /// Sets the module name.
    pub fn with_module_name(mut self, name: impl Into<String>) -> Self {
        self.module_name = Some(name.into());
        self
    }

    /// Enables validation derives.
    pub const fn with_validation(mut self, enabled: bool) -> Self {
        self.include_validation = enabled;
        self
    }

    /// Enables builder pattern generation.
    pub const fn with_builders(mut self, enabled: bool) -> Self {
        self.generate_builders = enabled;
        self
    }

    /// Sets the type naming convention.
    pub const fn with_type_naming(mut self, convention: NamingConvention) -> Self {
        self.type_naming = convention;
        self
    }

    /// Sets the field naming convention.
    pub const fn with_field_naming(mut self, convention: NamingConvention) -> Self {
        self.field_naming = convention;
        self
    }

    /// Sets a type prefix.
    pub fn with_type_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.type_prefix = Some(prefix.into());
        self
    }
}

/// Naming conventions for generated code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NamingConvention {
    /// PascalCase (e.g., UserProfile)
    #[default]
    PascalCase,
    /// camelCase (e.g., userProfile)
    CamelCase,
    /// snake_case (e.g., user_profile)
    SnakeCase,
    /// SCREAMING_SNAKE_CASE (e.g., USER_PROFILE)
    ScreamingSnakeCase,
    /// kebab-case (e.g., user-profile)
    KebabCase,
}

impl NamingConvention {
    /// Converts a string to this naming convention.
    pub fn convert(&self, s: &str) -> String {
        use heck::{
            ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase,
        };

        match self {
            Self::PascalCase => s.to_upper_camel_case(),
            Self::CamelCase => s.to_lower_camel_case(),
            Self::SnakeCase => s.to_snake_case(),
            Self::ScreamingSnakeCase => s.to_shouty_snake_case(),
            Self::KebabCase => s.to_kebab_case(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GeneratorConfig::default();
        assert!(config.include_validation);
        assert!(config.include_serialization);
        assert!(config.include_docs);
        assert!(!config.generate_builders);
        assert_eq!(config.type_naming, NamingConvention::PascalCase);
        assert_eq!(config.field_naming, NamingConvention::SnakeCase);
    }

    #[test]
    fn test_config_builder() {
        let config = GeneratorConfig::new()
            .with_module_name("my_service")
            .with_validation(false)
            .with_builders(true)
            .with_type_prefix("Api");

        assert_eq!(config.module_name, Some("my_service".to_string()));
        assert!(!config.include_validation);
        assert!(config.generate_builders);
        assert_eq!(config.type_prefix, Some("Api".to_string()));
    }

    #[test]
    fn test_naming_convention_convert() {
        assert_eq!(
            NamingConvention::PascalCase.convert("user_profile"),
            "UserProfile"
        );
        assert_eq!(
            NamingConvention::CamelCase.convert("user_profile"),
            "userProfile"
        );
        assert_eq!(
            NamingConvention::SnakeCase.convert("UserProfile"),
            "user_profile"
        );
        assert_eq!(
            NamingConvention::ScreamingSnakeCase.convert("userProfile"),
            "USER_PROFILE"
        );
        assert_eq!(
            NamingConvention::KebabCase.convert("UserProfile"),
            "user-profile"
        );
    }
}
