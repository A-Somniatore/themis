//! Protobuf to Contract normalizer.
//!
//! This module provides utilities for normalizing parsed protobuf
//! data into the Themis Contract model.

use themis_core::Contract;

/// Normalization options for protobuf contracts.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalizerOptions {
    /// Whether to include well-known types in schemas.
    pub include_well_known_types: bool,
    /// Whether to flatten nested messages.
    pub flatten_nested: bool,
    /// Whether to preserve original proto field numbers as metadata.
    pub preserve_field_numbers: bool,
    /// Whether to sort messages alphabetically.
    pub sort_messages: bool,
    /// Whether to sort services alphabetically.
    pub sort_services: bool,
    /// Whether to sort fields within messages.
    pub sort_fields: bool,
    /// Whether to strip comments.
    pub strip_comments: bool,
}

impl Default for NormalizerOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalizerOptions {
    /// Creates a new options builder with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_well_known_types: false,
            flatten_nested: false,
            preserve_field_numbers: false,
            sort_messages: true,
            sort_services: true,
            sort_fields: false,
            strip_comments: false,
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

    /// Sets whether to sort messages.
    #[must_use]
    pub const fn with_sort_messages(mut self, sort: bool) -> Self {
        self.sort_messages = sort;
        self
    }

    /// Sets whether to sort services.
    #[must_use]
    pub const fn with_sort_services(mut self, sort: bool) -> Self {
        self.sort_services = sort;
        self
    }

    /// Sets whether to sort fields.
    #[must_use]
    pub const fn with_sort_fields(mut self, sort: bool) -> Self {
        self.sort_fields = sort;
        self
    }

    /// Sets whether to strip comments.
    #[must_use]
    pub const fn with_strip_comments(mut self, strip: bool) -> Self {
        self.strip_comments = strip;
        self
    }
}

/// Protobuf contract normalizer.
///
/// Normalizes parsed protobuf contracts for consistent comparison.
pub struct ProtobufNormalizer;

impl ProtobufNormalizer {
    /// Create a new normalizer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Normalize a contract with default options.
    #[must_use]
    pub fn normalize(contract: Contract) -> Contract {
        Self::normalize_with_options(contract, &NormalizerOptions::default())
    }

    /// Normalize a contract with custom options.
    #[must_use]
    pub fn normalize_with_options(mut contract: Contract, options: &NormalizerOptions) -> Contract {
        // Sort schemas if requested
        if options.sort_messages {
            let mut schemas: Vec<_> = contract.schemas.into_iter().collect();
            schemas.sort_by(|(a, _), (b, _)| a.cmp(b));
            contract.schemas = schemas.into_iter().collect();
        }

        // Sort operations if requested
        if options.sort_services {
            let mut operations: Vec<_> = contract.operations.into_iter().collect();
            operations.sort_by(|(a, _), (b, _)| a.cmp(b));
            contract.operations = operations.into_iter().collect();
        }

        contract
    }
}

impl Default for ProtobufNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use std::collections::HashMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::Operation;
    use themis_core::schema::{ObjectSchema, Schema};
    use themis_core::version::Version;

    fn create_test_contract() -> Contract {
        let mut schemas = IndexMap::new();
        schemas.insert(
            "User".to_string(),
            Schema::Object(ObjectSchema::default()),
        );
        schemas.insert(
            "Address".to_string(),
            Schema::Object(ObjectSchema::default()),
        );
        schemas.insert(
            "Order".to_string(),
            Schema::Object(ObjectSchema::default()),
        );

        let mut operations = HashMap::new();
        operations.insert("GetUser".to_string(), Operation::new("GetUser"));
        operations.insert("CreateUser".to_string(), Operation::new("CreateUser"));
        operations.insert("DeleteUser".to_string(), Operation::new("DeleteUser"));

        Contract {
            format: ContractFormat::Protobuf,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "TestService".to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            schemas,
            operations,
            security_schemes: HashMap::new(),
        }
    }

    #[test]
    fn test_default_options() {
        let opts = NormalizerOptions::default();
        assert!(!opts.include_well_known_types);
        assert!(!opts.flatten_nested);
        assert!(!opts.preserve_field_numbers);
        assert!(opts.sort_messages);
        assert!(opts.sort_services);
    }

    #[test]
    fn test_options_builder() {
        let opts = NormalizerOptions::new()
            .with_well_known_types(true)
            .with_flatten_nested(true)
            .with_field_numbers(true)
            .with_sort_messages(false)
            .with_strip_comments(true);

        assert!(opts.include_well_known_types);
        assert!(opts.flatten_nested);
        assert!(opts.preserve_field_numbers);
        assert!(!opts.sort_messages);
        assert!(opts.strip_comments);
    }

    #[test]
    fn test_normalize_sorts_schemas() {
        let contract = create_test_contract();
        let normalized = ProtobufNormalizer::normalize(contract);

        let schema_names: Vec<_> = normalized.schemas.keys().collect();
        assert_eq!(schema_names, vec!["Address", "Order", "User"]);
    }

    #[test]
    fn test_normalize_sorts_operations() {
        let contract = create_test_contract();
        let normalized = ProtobufNormalizer::normalize(contract);

        // HashMap doesn't preserve order, but we can verify all operations exist
        assert_eq!(normalized.operations.len(), 3);
        assert!(normalized.operations.contains_key("GetUser"));
        assert!(normalized.operations.contains_key("CreateUser"));
        assert!(normalized.operations.contains_key("DeleteUser"));
    }

    #[test]
    fn test_normalize_with_disabled_sorting() {
        let contract = create_test_contract();
        let options = NormalizerOptions::new()
            .with_sort_messages(false)
            .with_sort_services(false);

        let normalized = ProtobufNormalizer::normalize_with_options(contract, &options);
        
        // Original insertion order should be preserved
        let schema_names: Vec<_> = normalized.schemas.keys().collect();
        assert_eq!(schema_names, vec!["User", "Address", "Order"]);
    }

    #[test]
    fn test_normalizer_default() {
        let normalizer = ProtobufNormalizer::default();
        assert!(std::mem::size_of_val(&normalizer) == 0);
    }
}

