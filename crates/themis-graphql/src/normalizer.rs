//! GraphQL to Contract normalizer.
//!
//! This module provides utilities for normalizing parsed GraphQL
//! data into the Themis Contract model.

// The normalizer functionality is currently integrated into the parser.
// This module is a placeholder for future normalization utilities.

/// Normalization options for GraphQL contracts.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalizerOptions {
    /// Whether to include built-in scalar types in schemas.
    pub include_builtin_scalars: bool,
    /// Whether to flatten interface implementations.
    pub flatten_interfaces: bool,
    /// Whether to expand union types.
    pub expand_unions: bool,
    /// Whether to include directive definitions.
    pub include_directives: bool,
}

impl NormalizerOptions {
    /// Creates a new options builder with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_builtin_scalars: false,
            flatten_interfaces: false,
            expand_unions: false,
            include_directives: false,
        }
    }

    /// Sets whether to include built-in scalars.
    #[must_use]
    pub const fn with_builtin_scalars(mut self, include: bool) -> Self {
        self.include_builtin_scalars = include;
        self
    }

    /// Sets whether to flatten interfaces.
    #[must_use]
    pub const fn with_flatten_interfaces(mut self, flatten: bool) -> Self {
        self.flatten_interfaces = flatten;
        self
    }

    /// Sets whether to expand union types.
    #[must_use]
    pub const fn with_expand_unions(mut self, expand: bool) -> Self {
        self.expand_unions = expand;
        self
    }

    /// Sets whether to include directive definitions.
    #[must_use]
    pub const fn with_directives(mut self, include: bool) -> Self {
        self.include_directives = include;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = NormalizerOptions::default();
        assert!(!options.include_builtin_scalars);
        assert!(!options.flatten_interfaces);
        assert!(!options.expand_unions);
        assert!(!options.include_directives);
    }

    #[test]
    fn test_options_builder() {
        let options = NormalizerOptions::new()
            .with_builtin_scalars(true)
            .with_flatten_interfaces(true)
            .with_expand_unions(true)
            .with_directives(true);

        assert!(options.include_builtin_scalars);
        assert!(options.flatten_interfaces);
        assert!(options.expand_unions);
        assert!(options.include_directives);
    }
}
