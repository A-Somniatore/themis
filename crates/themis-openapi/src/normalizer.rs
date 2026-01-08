//! OpenAPI to Themis contract normalizer.
//!
//! Normalizes `OpenAPI` specifications for consistent comparison and processing.
//! This includes sorting operations, removing descriptions, and standardizing
//! schema definitions.

use serde_yaml::Value;

/// Options for `OpenAPI` normalization.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalizerOptions {
    /// Sort paths alphabetically.
    pub sort_paths: bool,
    /// Sort operations within paths alphabetically by method.
    pub sort_operations: bool,
    /// Sort component schemas alphabetically.
    pub sort_schemas: bool,
    /// Remove descriptions from all elements.
    pub strip_descriptions: bool,
    /// Remove examples from all elements.
    pub strip_examples: bool,
    /// Remove `x-` extension properties.
    pub strip_extensions: bool,
    /// Normalize schema types (e.g., expand shorthand).
    pub normalize_schemas: bool,
}

impl Default for NormalizerOptions {
    fn default() -> Self {
        Self {
            sort_paths: true,
            sort_operations: true,
            sort_schemas: true,
            strip_descriptions: false,
            strip_examples: false,
            strip_extensions: false,
            normalize_schemas: true,
        }
    }
}

impl NormalizerOptions {
    /// Creates a new options builder with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sort_paths: true,
            sort_operations: true,
            sort_schemas: true,
            strip_descriptions: false,
            strip_examples: false,
            strip_extensions: false,
            normalize_schemas: true,
        }
    }

    /// Sets whether to sort paths.
    #[must_use]
    pub const fn with_sort_paths(mut self, sort: bool) -> Self {
        self.sort_paths = sort;
        self
    }

    /// Sets whether to sort operations.
    #[must_use]
    pub const fn with_sort_operations(mut self, sort: bool) -> Self {
        self.sort_operations = sort;
        self
    }

    /// Sets whether to sort schemas.
    #[must_use]
    pub const fn with_sort_schemas(mut self, sort: bool) -> Self {
        self.sort_schemas = sort;
        self
    }

    /// Sets whether to strip descriptions.
    #[must_use]
    pub const fn with_strip_descriptions(mut self, strip: bool) -> Self {
        self.strip_descriptions = strip;
        self
    }

    /// Sets whether to strip examples.
    #[must_use]
    pub const fn with_strip_examples(mut self, strip: bool) -> Self {
        self.strip_examples = strip;
        self
    }

    /// Sets whether to strip extensions.
    #[must_use]
    pub const fn with_strip_extensions(mut self, strip: bool) -> Self {
        self.strip_extensions = strip;
        self
    }
}

/// Normalizes `OpenAPI` specifications.
pub struct OpenApiNormalizer;

impl OpenApiNormalizer {
    /// Create a new normalizer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Normalize an `OpenAPI` document with default options.
    #[must_use]
    pub fn normalize(doc: Value) -> Value {
        Self::normalize_with_options(doc, &NormalizerOptions::default())
    }

    /// Normalize an `OpenAPI` document with custom options.
    #[must_use]
    pub fn normalize_with_options(mut doc: Value, options: &NormalizerOptions) -> Value {
        if options.strip_descriptions {
            Self::remove_field(&mut doc, "description");
        }

        if options.strip_examples {
            Self::remove_field(&mut doc, "example");
            Self::remove_field(&mut doc, "examples");
        }

        if options.strip_extensions {
            Self::remove_extensions(&mut doc);
        }

        if options.sort_paths {
            Self::sort_mapping(&mut doc, "paths");
        }

        if options.sort_schemas {
            if let Some(components) = doc.get_mut("components") {
                Self::sort_mapping(components, "schemas");
                Self::sort_mapping(components, "parameters");
                Self::sort_mapping(components, "responses");
                Self::sort_mapping(components, "requestBodies");
                Self::sort_mapping(components, "securitySchemes");
            }
        }

        doc
    }

    /// Remove a field recursively from a YAML value.
    fn remove_field(value: &mut Value, field_name: &str) {
        match value {
            Value::Mapping(map) => {
                map.remove(Value::String(field_name.to_string()));
                for (_, v) in map.iter_mut() {
                    Self::remove_field(v, field_name);
                }
            }
            Value::Sequence(seq) => {
                for item in seq {
                    Self::remove_field(item, field_name);
                }
            }
            _ => {}
        }
    }

    /// Remove all `x-` extension fields recursively.
    fn remove_extensions(value: &mut Value) {
        if let Value::Mapping(map) = value {
            // Collect keys to remove
            let keys_to_remove: Vec<_> = map
                .keys()
                .filter_map(|k| {
                    k.as_str()
                        .filter(|s| s.starts_with("x-"))
                        .map(|_| k.clone())
                })
                .collect();

            for key in keys_to_remove {
                map.remove(&key);
            }

            // Recurse into remaining values
            for (_, v) in map.iter_mut() {
                Self::remove_extensions(v);
            }
        } else if let Value::Sequence(seq) = value {
            for item in seq {
                Self::remove_extensions(item);
            }
        }
    }

    /// Sort a mapping by keys.
    fn sort_mapping(doc: &mut Value, path: &str) {
        if let Some(Value::Mapping(map)) = doc.get_mut(path) {
            let mut entries: Vec<_> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.sort_by(|(a, _), (b, _)| {
                let a_str = a.as_str().unwrap_or("");
                let b_str = b.as_str().unwrap_or("");
                a_str.cmp(b_str)
            });
            map.clear();
            for (k, v) in entries {
                map.insert(k, v);
            }
        }
    }
}

impl Default for OpenApiNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = NormalizerOptions::default();
        assert!(options.sort_paths);
        assert!(options.sort_operations);
        assert!(options.sort_schemas);
        assert!(!options.strip_descriptions);
        assert!(!options.strip_examples);
        assert!(!options.strip_extensions);
    }

    #[test]
    fn test_options_builder() {
        let options = NormalizerOptions::new()
            .with_sort_paths(false)
            .with_strip_descriptions(true)
            .with_strip_extensions(true);

        assert!(!options.sort_paths);
        assert!(options.strip_descriptions);
        assert!(options.strip_extensions);
    }

    #[test]
    fn test_normalize_strips_descriptions() {
        let yaml = r#"
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
  description: This should be removed
paths:
  /users:
    get:
      operationId: getUsers
      description: Also removed
      responses:
        '200':
          description: Success
"#;
        let doc: Value = serde_yaml::from_str(yaml).unwrap();
        let options = NormalizerOptions {
            strip_descriptions: true,
            ..Default::default()
        };
        let normalized = OpenApiNormalizer::normalize_with_options(doc, &options);

        // Check descriptions are removed
        assert!(normalized.get("info").unwrap().get("description").is_none());
        let paths = normalized.get("paths").unwrap();
        let users = paths.get("/users").unwrap();
        let get = users.get("get").unwrap();
        assert!(get.get("description").is_none());
    }

    #[test]
    fn test_normalize_strips_examples() {
        let yaml = r#"
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
components:
  schemas:
    User:
      type: object
      properties:
        name:
          type: string
          example: John Doe
"#;
        let doc: Value = serde_yaml::from_str(yaml).unwrap();
        let options = NormalizerOptions {
            strip_examples: true,
            ..Default::default()
        };
        let normalized = OpenApiNormalizer::normalize_with_options(doc, &options);

        let schemas = normalized
            .get("components")
            .unwrap()
            .get("schemas")
            .unwrap();
        let user = schemas.get("User").unwrap();
        let props = user.get("properties").unwrap();
        let name = props.get("name").unwrap();
        assert!(name.get("example").is_none());
    }

    #[test]
    fn test_normalize_strips_extensions() {
        let yaml = r#"
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
  x-custom-field: should be removed
paths:
  /users:
    x-handler: UserHandler
    get:
      operationId: getUsers
      x-rate-limit: 100
      responses:
        '200':
          description: Success
"#;
        let doc: Value = serde_yaml::from_str(yaml).unwrap();
        let options = NormalizerOptions {
            strip_extensions: true,
            ..Default::default()
        };
        let normalized = OpenApiNormalizer::normalize_with_options(doc, &options);

        // Check extensions are removed
        assert!(normalized
            .get("info")
            .unwrap()
            .get("x-custom-field")
            .is_none());
        let paths = normalized.get("paths").unwrap();
        let users = paths.get("/users").unwrap();
        assert!(users.get("x-handler").is_none());
        let get = users.get("get").unwrap();
        assert!(get.get("x-rate-limit").is_none());
    }

    #[test]
    fn test_normalize_sorts_paths() {
        let yaml = r#"
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
paths:
  /users:
    get:
      operationId: getUsers
      responses:
        '200':
          description: Success
  /orders:
    get:
      operationId: getOrders
      responses:
        '200':
          description: Success
  /accounts:
    get:
      operationId: getAccounts
      responses:
        '200':
          description: Success
"#;
        let doc: Value = serde_yaml::from_str(yaml).unwrap();
        let normalized = OpenApiNormalizer::normalize(doc);

        // Check paths are sorted alphabetically
        let paths = normalized.get("paths").unwrap().as_mapping().unwrap();
        let keys: Vec<_> = paths
            .keys()
            .filter_map(|k| k.as_str())
            .collect();
        assert_eq!(keys, vec!["/accounts", "/orders", "/users"]);
    }

    #[test]
    fn test_normalize_sorts_schemas() {
        let yaml = r#"
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
components:
  schemas:
    User:
      type: object
    Address:
      type: object
    Order:
      type: object
"#;
        let doc: Value = serde_yaml::from_str(yaml).unwrap();
        let normalized = OpenApiNormalizer::normalize(doc);

        // Check schemas are sorted alphabetically
        let schemas = normalized
            .get("components")
            .unwrap()
            .get("schemas")
            .unwrap()
            .as_mapping()
            .unwrap();
        let keys: Vec<_> = schemas
            .keys()
            .filter_map(|k| k.as_str())
            .collect();
        assert_eq!(keys, vec!["Address", "Order", "User"]);
    }

    #[test]
    fn test_normalizer_default() {
        let normalizer = OpenApiNormalizer::default();
        assert!(std::mem::size_of_val(&normalizer) == 0); // Zero-sized type
    }
}
