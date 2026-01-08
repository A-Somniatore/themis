//! Protobuf to Contract normalizer.
//!
//! This module provides utilities for normalizing parsed protobuf
//! data into the Themis Contract model.

// The normalizer functionality is currently integrated into the parser.
// This module is a placeholder for future normalization utilities.

/// Normalization options for protobuf contracts.
#[derive(Debug, Clone, Default)]
pub struct NormalizerOptions {
    /// Whether to include well-known types in schemas.
    pub include_well_known_types: bool,
    /// Whether to flatten nested messages.
    pub flatten_nested: bool,
    /// Whether to preserve original proto field numbers as metadata.
    pub preserve_field_numbers: bool,
}

impl NormalizerOptions {
    /// Creates a new options builder with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_well_known_types: false,
            flatten_nested: false,
            preserve_field_numbers: false,
        }
    }

    /// Sets whether to include well-known types.
    #[must_use]
    pub const fn with_well_known_types(mut self, include: bool) -> Self {
        self.include_well_known_types = include;
        self
    }

    /// Sets whether to flatten nested messages.
    #[must_use]
    pub const fn with_flatten_nested(mut self, flatten: bool) -> Self {
        self.flatten_nested = flatten;
        self
    }

    /// Sets whether to preserve field numbers.
    #[must_use]
    pub const fn with_field_numbers(mut self, preserve: bool) -> Self {
        self.preserve_field_numbers = preserve;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = NormalizerOptions::default();
        assert!(!opts.include_well_known_types);
        assert!(!opts.flatten_nested);
        assert!(!opts.preserve_field_numbers);
    }

    #[test]
    fn test_options_builder() {
        let opts = NormalizerOptions::new()
            .with_well_known_types(true)
            .with_flatten_nested(true)
            .with_field_numbers(true);

        assert!(opts.include_well_known_types);
        assert!(opts.flatten_nested);
        assert!(opts.preserve_field_numbers);
    }
}
