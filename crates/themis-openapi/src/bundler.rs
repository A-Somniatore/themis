//! External reference bundler for OpenAPI specifications.
//!
//! Resolves and inlines external `$ref` references in OpenAPI documents.
//! Supports:
//! - Relative file paths (e.g., `./schemas/user.yaml`)
//! - Parent directory paths (e.g., `../common/errors.yaml`)
//! - Fragment references (e.g., `./schemas.yaml#/components/schemas/User`)
//!
//! # Example
//!
//! ```rust,ignore
//! use themis_openapi::bundler::{BundleOptions, OpenApiBundler};
//! use std::path::Path;
//!
//! let bundler = OpenApiBundler::new(BundleOptions::default());
//! let bundled = bundler.bundle_file(Path::new("api.yaml"))?;
//! ```

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use themis_core::{ThemisError, ThemisResult};

/// Options for the bundler.
#[derive(Debug, Clone)]
pub struct BundleOptions {
    /// Whether to follow external URL references (disabled by default for security).
    pub follow_urls: bool,
    /// Maximum depth of reference resolution to prevent infinite loops.
    pub max_depth: usize,
    /// Whether to preserve the original `$ref` structure or fully inline.
    pub preserve_refs: bool,
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            follow_urls: false,
            max_depth: 100,
            preserve_refs: true,
        }
    }
}

impl BundleOptions {
    /// Create bundler options that follow URL references.
    #[must_use]
    pub const fn with_url_refs(mut self) -> Self {
        self.follow_urls = true;
        self
    }

    /// Set maximum resolution depth.
    #[must_use]
    pub const fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set whether to preserve ref structure.
    #[must_use]
    pub const fn with_preserve_refs(mut self, preserve: bool) -> Self {
        self.preserve_refs = preserve;
        self
    }
}

/// Bundles OpenAPI documents by resolving external `$ref` references.
pub struct OpenApiBundler {
    options: BundleOptions,
    /// Cache of loaded documents to avoid re-reading files.
    document_cache: HashMap<PathBuf, Value>,
    /// Track visited refs to detect circular references.
    visited_refs: HashSet<String>,
}

impl OpenApiBundler {
    /// Creates a new bundler with the given options.
    #[must_use]
    pub fn new(options: BundleOptions) -> Self {
        Self {
            options,
            document_cache: HashMap::new(),
            visited_refs: HashSet::new(),
        }
    }

    /// Bundles an OpenAPI specification from a file, resolving all external refs.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the root OpenAPI specification file
    ///
    /// # Returns
    ///
    /// The bundled OpenAPI document as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File cannot be read
    /// - YAML/JSON parsing fails
    /// - External reference cannot be resolved
    /// - Circular reference detected
    pub fn bundle_file(&mut self, path: &Path) -> ThemisResult<Value> {
        let content = std::fs::read_to_string(path).map_err(|e| ThemisError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        let base_path = path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

        self.bundle_content(&content, &base_path)
    }

    /// Bundles OpenAPI content from a string with a base path for resolving relative refs.
    ///
    /// # Arguments
    ///
    /// * `content` - The OpenAPI specification as YAML or JSON string
    /// * `base_path` - Base directory for resolving relative file references
    ///
    /// # Returns
    ///
    /// The bundled OpenAPI document as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - YAML/JSON parsing fails
    /// - External reference cannot be resolved
    /// - Circular references are detected
    pub fn bundle_content(&mut self, content: &str, base_path: &Path) -> ThemisResult<Value> {
        let mut doc: Value = serde_yaml::from_str(content).or_else(|yaml_err| {
            serde_json::from_str(content).map_err(|json_err| ThemisError::YamlParse {
                path: base_path.to_path_buf(),
                message: format!("Failed to parse as YAML ({yaml_err}) or JSON ({json_err})"),
            })
        })?;

        self.visited_refs.clear();
        self.resolve_refs(&mut doc, base_path, 0)?;

        Ok(doc)
    }

    /// Recursively resolves `$ref` references in a JSON value.
    fn resolve_refs(&mut self, value: &mut Value, base_path: &Path, depth: usize) -> ThemisResult<()> {
        if depth > self.options.max_depth {
            return Err(ThemisError::SchemaValidation {
                message: format!(
                    "Maximum reference depth ({}) exceeded. Possible circular reference.",
                    self.options.max_depth
                ),
            });
        }

        match value {
            Value::Object(map) => {
                // Check if this is a $ref object
                if let Some(ref_value) = map.get("$ref").cloned() {
                    if let Some(ref_str) = ref_value.as_str() {
                        // Only resolve external refs (not starting with #)
                        if !ref_str.starts_with('#') {
                            let resolved = self.resolve_external_ref(ref_str, base_path, depth)?;
                            // Inline the resolved content regardless of preserve_refs option
                            // (preserve_refs is for future enhancement to keep refs in components)
                            *value = resolved;
                            return Ok(());
                        }
                    }
                }

                // Recursively process all values
                for (_, v) in map.iter_mut() {
                    self.resolve_refs(v, base_path, depth)?;
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.resolve_refs(item, base_path, depth)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Resolves an external reference.
    fn resolve_external_ref(
        &mut self,
        ref_str: &str,
        base_path: &Path,
        depth: usize,
    ) -> ThemisResult<Value> {
        // Check for circular references
        let ref_key = format!("{}:{}", base_path.display(), ref_str);
        if self.visited_refs.contains(&ref_key) {
            return Err(ThemisError::SchemaValidation {
                message: format!("Circular reference detected: {ref_str}"),
            });
        }
        self.visited_refs.insert(ref_key);

        // Parse the reference: file_path#json_pointer
        let (file_path, fragment) = parse_reference(ref_str)?;

        // Check if this is a URL reference
        if file_path.starts_with("http://") || file_path.starts_with("https://") {
            if !self.options.follow_urls {
                return Err(ThemisError::SchemaValidation {
                    message: format!(
                        "URL references are disabled. Enable with BundleOptions::with_url_refs(). Reference: {file_path}"
                    ),
                });
            }
            return Err(ThemisError::SchemaValidation {
                message: format!("URL reference resolution not yet implemented: {file_path}"),
            });
        }

        // Resolve relative path
        let resolved_path = resolve_file_path(base_path, &file_path);

        // Load the external document
        let mut external_doc = self.load_document(&resolved_path)?;

        // Get the new base path for nested refs
        let new_base_path = resolved_path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

        // Resolve refs in the external document
        self.resolve_refs(&mut external_doc, &new_base_path, depth + 1)?;

        // Extract the fragment if present
        if let Some(json_pointer) = fragment {
            extract_json_pointer(&external_doc, &json_pointer)
        } else {
            Ok(external_doc)
        }
    }

    /// Loads a document from file, using cache if available.
    fn load_document(&mut self, path: &Path) -> ThemisResult<Value> {
        // Canonicalize path for consistent caching
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if let Some(cached) = self.document_cache.get(&canonical_path) {
            return Ok(cached.clone());
        }

        let content = std::fs::read_to_string(path).map_err(|e| ThemisError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        let doc: Value = serde_yaml::from_str(&content).or_else(|yaml_err| {
            serde_json::from_str(&content).map_err(|json_err| ThemisError::YamlParse {
                path: path.to_path_buf(),
                message: format!("Failed to parse as YAML ({yaml_err}) or JSON ({json_err})"),
            })
        })?;

        self.document_cache.insert(canonical_path, doc.clone());
        Ok(doc)
    }
}

/// Parses a reference string into file path and optional fragment.
///
/// Examples:
/// - `./schemas/user.yaml` -> ("./schemas/user.yaml", None)
/// - `./schemas.yaml#/components/schemas/User` -> ("./schemas.yaml", Some("/components/schemas/User"))
fn parse_reference(ref_str: &str) -> ThemisResult<(String, Option<String>)> {
    if let Some(hash_pos) = ref_str.find('#') {
        let file_path = ref_str[..hash_pos].to_string();
        let fragment = ref_str[hash_pos + 1..].to_string();
        
        if file_path.is_empty() {
            // This is an internal reference like "#/components/schemas/User"
            // Should not be processed by external resolver
            return Err(ThemisError::SchemaValidation {
                message: format!("Internal reference passed to external resolver: {ref_str}"),
            });
        }
        
        Ok((file_path, Some(fragment)))
    } else {
        Ok((ref_str.to_string(), None))
    }
}

/// Resolves a file path relative to a base path.
fn resolve_file_path(base_path: &Path, file_path: &str) -> PathBuf {
    let path = Path::new(file_path);
    
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_path.join(path)
    }
}

/// Extracts a value from a JSON document using a JSON pointer.
fn extract_json_pointer(doc: &Value, pointer: &str) -> ThemisResult<Value> {
    // JSON pointer must start with /
    let pointer = if pointer.starts_with('/') {
        pointer.to_string()
    } else {
        format!("/{pointer}")
    };

    doc.pointer(&pointer)
        .cloned()
        .ok_or_else(|| ThemisError::SchemaValidation {
            message: format!("JSON pointer not found: {pointer}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_reference_with_fragment() {
        let (file, fragment) = parse_reference("./schemas.yaml#/components/schemas/User").unwrap();
        assert_eq!(file, "./schemas.yaml");
        assert_eq!(fragment, Some("/components/schemas/User".to_string()));
    }

    #[test]
    fn test_parse_reference_without_fragment() {
        let (file, fragment) = parse_reference("./schemas/user.yaml").unwrap();
        assert_eq!(file, "./schemas/user.yaml");
        assert_eq!(fragment, None);
    }

    #[test]
    fn test_parse_reference_internal_ref_error() {
        let result = parse_reference("#/components/schemas/User");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_json_pointer() {
        let doc = json!({
            "components": {
                "schemas": {
                    "User": {
                        "type": "object"
                    }
                }
            }
        });

        let result = extract_json_pointer(&doc, "/components/schemas/User").unwrap();
        assert_eq!(result["type"], "object");
    }

    #[test]
    fn test_bundle_with_external_ref() {
        let temp_dir = TempDir::new().unwrap();

        // Create schemas file
        let schemas_path = temp_dir.path().join("schemas.yaml");
        let mut schemas_file = std::fs::File::create(&schemas_path).unwrap();
        writeln!(
            schemas_file,
            r#"User:
  type: object
  properties:
    id:
      type: integer
    name:
      type: string
"#
        )
        .unwrap();

        // Create main API file
        let api_path = temp_dir.path().join("api.yaml");
        let mut api_file = std::fs::File::create(&api_path).unwrap();
        writeln!(
            api_file,
            r##"openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /users:
    get:
      operationId: getUsers
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                $ref: "./schemas.yaml#/User"
"##
        )
        .unwrap();

        let mut bundler = OpenApiBundler::new(BundleOptions::default());
        let bundled = bundler.bundle_file(&api_path).unwrap();

        // The external ref should be resolved
        let schema = &bundled["paths"]["/users"]["get"]["responses"]["200"]["content"]["application/json"]["schema"];
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["id"].is_object());
        assert!(schema["properties"]["name"].is_object());
    }

    #[test]
    fn test_bundle_nested_refs() {
        let temp_dir = TempDir::new().unwrap();

        // Create common types file
        let common_path = temp_dir.path().join("common.yaml");
        let mut common_file = std::fs::File::create(&common_path).unwrap();
        writeln!(
            common_file,
            r#"Id:
  type: integer
  format: int64
"#
        )
        .unwrap();

        // Create schemas file that references common
        let schemas_path = temp_dir.path().join("schemas.yaml");
        let mut schemas_file = std::fs::File::create(&schemas_path).unwrap();
        writeln!(
            schemas_file,
            r##"User:
  type: object
  properties:
    id:
      $ref: "./common.yaml#/Id"
    name:
      type: string
"##
        )
        .unwrap();

        // Create main API file
        let api_path = temp_dir.path().join("api.yaml");
        let mut api_file = std::fs::File::create(&api_path).unwrap();
        writeln!(
            api_file,
            r##"openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /users:
    get:
      operationId: getUsers
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                $ref: "./schemas.yaml#/User"
"##
        )
        .unwrap();

        let mut bundler = OpenApiBundler::new(BundleOptions::default());
        let bundled = bundler.bundle_file(&api_path).unwrap();

        // Both external refs should be resolved
        let schema = &bundled["paths"]["/users"]["get"]["responses"]["200"]["content"]["application/json"]["schema"];
        assert_eq!(schema["type"], "object");
        let id_schema = &schema["properties"]["id"];
        assert_eq!(id_schema["type"], "integer");
        assert_eq!(id_schema["format"], "int64");
    }

    #[test]
    fn test_circular_reference_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create file A that references B
        let file_a = temp_dir.path().join("a.yaml");
        let mut a = std::fs::File::create(&file_a).unwrap();
        writeln!(
            a,
            r##"TypeA:
  $ref: "./b.yaml#/TypeB"
"##
        )
        .unwrap();

        // Create file B that references A (circular)
        let file_b = temp_dir.path().join("b.yaml");
        let mut b = std::fs::File::create(&file_b).unwrap();
        writeln!(
            b,
            r##"TypeB:
  $ref: "./a.yaml#/TypeA"
"##
        )
        .unwrap();

        let mut bundler = OpenApiBundler::new(BundleOptions::default());
        let result = bundler.bundle_file(&file_a);
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Circular reference"));
    }

    #[test]
    fn test_url_refs_disabled_by_default() {
        let content = r##"openapi: "3.1.0"
info:
  title: Test
  version: "1.0.0"
paths:
  /test:
    get:
      operationId: test
      responses:
        "200":
          description: OK
          content:
            application/json:
              schema:
                $ref: "https://example.com/schemas/user.json"
"##;

        let mut bundler = OpenApiBundler::new(BundleOptions::default());
        let result = bundler.bundle_content(content, Path::new("."));
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("URL references are disabled"));
    }

    #[test]
    fn test_internal_refs_preserved() {
        let content = r##"openapi: "3.1.0"
info:
  title: Test
  version: "1.0.0"
components:
  schemas:
    User:
      type: object
paths:
  /test:
    get:
      operationId: test
      responses:
        "200":
          description: OK
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"
"##;

        let mut bundler = OpenApiBundler::new(BundleOptions::default());
        let bundled = bundler.bundle_content(content, Path::new(".")).unwrap();
        
        // Internal refs should be preserved
        let schema = &bundled["paths"]["/test"]["get"]["responses"]["200"]["content"]["application/json"]["schema"];
        assert_eq!(schema["$ref"], "#/components/schemas/User");
    }
}
