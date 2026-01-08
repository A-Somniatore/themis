//! `AsyncAPI` specification normalizer.
//!
//! Normalizes `AsyncAPI` specifications for consistent comparison and processing.

/// Options for `AsyncAPI` normalization.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalizerOptions {
    /// Sort channels alphabetically
    pub sort_channels: bool,
    /// Sort operations alphabetically
    pub sort_operations: bool,
    /// Remove descriptions
    pub strip_descriptions: bool,
    /// Remove examples
    pub strip_examples: bool,
    /// Inline all $ref references
    pub inline_refs: bool,
    /// Normalize schema types
    pub normalize_schemas: bool,
}

impl Default for NormalizerOptions {
    fn default() -> Self {
        Self {
            sort_channels: true,
            sort_operations: true,
            strip_descriptions: false,
            strip_examples: false,
            inline_refs: false,
            normalize_schemas: true,
        }
    }
}

/// Normalizes `AsyncAPI` specifications.
pub struct AsyncApiNormalizer;

impl AsyncApiNormalizer {
    /// Create a new normalizer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Normalize an `AsyncAPI` document with default options.
    #[must_use]
    pub fn normalize(doc: serde_yaml::Value) -> serde_yaml::Value {
        Self::normalize_with_options(doc, &NormalizerOptions::default())
    }

    /// Normalize an `AsyncAPI` document with custom options.
    #[must_use]
    pub fn normalize_with_options(
        mut doc: serde_yaml::Value,
        options: &NormalizerOptions,
    ) -> serde_yaml::Value {
        if options.strip_descriptions {
            Self::remove_field(&mut doc, "description");
        }

        if options.strip_examples {
            Self::remove_field(&mut doc, "examples");
        }

        if options.sort_channels {
            Self::sort_mapping(&mut doc, "channels");
        }

        if options.sort_operations {
            Self::sort_mapping(&mut doc, "operations");
        }

        doc
    }

    /// Remove a field recursively from a YAML value.
    fn remove_field(value: &mut serde_yaml::Value, field_name: &str) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                map.remove(serde_yaml::Value::String(field_name.to_string()));
                for (_, v) in map.iter_mut() {
                    Self::remove_field(v, field_name);
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    Self::remove_field(item, field_name);
                }
            }
            _ => {}
        }
    }

    /// Sort a mapping by keys.
    fn sort_mapping(doc: &mut serde_yaml::Value, path: &str) {
        if let Some(serde_yaml::Value::Mapping(map)) = doc.get_mut(path) {
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

impl Default for AsyncApiNormalizer {
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
        assert!(options.sort_channels);
        assert!(options.sort_operations);
        assert!(!options.strip_descriptions);
    }

    #[test]
    fn test_normalize_strips_descriptions() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
  description: This should be removed
channels:
  userCreated:
    description: Also removed
"#;
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let options = NormalizerOptions {
            strip_descriptions: true,
            ..Default::default()
        };
        let normalized = AsyncApiNormalizer::normalize_with_options(doc, &options);

        // Check descriptions are removed
        assert!(normalized.get("info").unwrap().get("description").is_none());
    }
}
