//! GraphQL to Contract normalizer.
//!
//! This module provides utilities for normalizing parsed GraphQL
//! data into the Themis Contract model.

use themis_core::Contract;

/// Normalization options for GraphQL contracts.
#[derive(Debug, Clone)]
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
    /// Whether to sort types alphabetically.
    pub sort_types: bool,
    /// Whether to sort fields within types.
    pub sort_fields: bool,
    /// Whether to strip descriptions.
    pub strip_descriptions: bool,
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
            include_builtin_scalars: false,
            flatten_interfaces: false,
            expand_unions: false,
            include_directives: false,
            sort_types: true,
            sort_fields: false,
            strip_descriptions: false,
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

    /// Sets whether to sort types.
    #[must_use]
    pub const fn with_sort_types(mut self, sort: bool) -> Self {
        self.sort_types = sort;
        self
    }

    /// Sets whether to sort fields.
    #[must_use]
    pub const fn with_sort_fields(mut self, sort: bool) -> Self {
        self.sort_fields = sort;
        self
    }

    /// Sets whether to strip descriptions.
    #[must_use]
    pub const fn with_strip_descriptions(mut self, strip: bool) -> Self {
        self.strip_descriptions = strip;
        self
    }
}

/// GraphQL contract normalizer.
///
/// Normalizes parsed GraphQL contracts for consistent comparison.
pub struct GraphQLNormalizer;

impl GraphQLNormalizer {
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
        if options.sort_types {
            let mut schemas: Vec<_> = contract.schemas.into_iter().collect();
            schemas.sort_by(|(a, _), (b, _)| a.cmp(b));
            contract.schemas = schemas.into_iter().collect();
        }

        // Sort operations (queries/mutations) by name if requested
        if options.sort_types {
            let mut operations: Vec<_> = contract.operations.into_iter().collect();
            operations.sort_by(|(a, _), (b, _)| a.cmp(b));
            contract.operations = operations.into_iter().collect();
        }

        contract
    }
}

impl Default for GraphQLNormalizer {
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
            "Query".to_string(),
            Schema::Object(ObjectSchema::default()),
        );
        schemas.insert(
            "Mutation".to_string(),
            Schema::Object(ObjectSchema::default()),
        );

        let mut operations = HashMap::new();
        operations.insert("getUser".to_string(), Operation::new("getUser"));
        operations.insert("createUser".to_string(), Operation::new("createUser"));

        Contract {
            format: ContractFormat::GraphQl,
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
        let options = NormalizerOptions::default();
        assert!(!options.include_builtin_scalars);
        assert!(!options.flatten_interfaces);
        assert!(!options.expand_unions);
        assert!(!options.include_directives);
        assert!(options.sort_types);
        assert!(!options.sort_fields);
    }

    #[test]
    fn test_options_builder() {
        let options = NormalizerOptions::new()
            .with_builtin_scalars(true)
            .with_flatten_interfaces(true)
            .with_expand_unions(true)
            .with_directives(true)
            .with_sort_types(false)
            .with_strip_descriptions(true);

        assert!(options.include_builtin_scalars);
        assert!(options.flatten_interfaces);
        assert!(options.expand_unions);
        assert!(options.include_directives);
        assert!(!options.sort_types);
        assert!(options.strip_descriptions);
    }

    #[test]
    fn test_normalize_sorts_types() {
        let contract = create_test_contract();
        let normalized = GraphQLNormalizer::normalize(contract);

        let type_names: Vec<_> = normalized.schemas.keys().collect();
        assert_eq!(type_names, vec!["Mutation", "Query", "User"]);
    }

    #[test]
    fn test_normalize_with_disabled_sorting() {
        let contract = create_test_contract();
        let options = NormalizerOptions::new().with_sort_types(false);

        let normalized = GraphQLNormalizer::normalize_with_options(contract, &options);

        // Original insertion order should be preserved
        let type_names: Vec<_> = normalized.schemas.keys().collect();
        assert_eq!(type_names, vec!["User", "Query", "Mutation"]);
    }

    #[test]
    fn test_normalizer_default() {
        let normalizer = GraphQLNormalizer::default();
        assert!(std::mem::size_of_val(&normalizer) == 0);
    }
}
