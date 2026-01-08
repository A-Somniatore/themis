//! JSON Schema generator implementation.
//!
//! Generates JSON Schema files from Themis contracts for use with
//! generic code generation tools.

#![allow(dead_code)] // Some methods are used for future extensibility

use super::types::JsonSchemaConverter;
use crate::config::GeneratorConfig;
use crate::error::CodegenResult;
use crate::traits::{CodeGenerator, GeneratedCode, GeneratedFile};
use themis_core::Contract;

/// JSON Schema generator.
///
/// Generates JSON Schema files from Themis contracts. Each schema type
/// produces a separate `.json` file that can be used with tools like:
///
/// - `quicktype` - Generate types for Go, Java, C#, Swift, etc.
/// - `datamodel-code-generator` - Generate Python/Pydantic models
/// - `json-schema-to-typescript` - Generate TypeScript interfaces
/// - `jsonschema2pojo` - Generate Java POJOs
///
/// # Example
///
/// ```ignore
/// use themis_codegen::{JsonSchemaGenerator, GeneratorConfig, CodeGenerator};
/// use themis_core::Contract;
///
/// let generator = JsonSchemaGenerator::new(GeneratorConfig::default());
/// let output = generator.generate(&contract)?;
///
/// // Each schema becomes a separate file:
/// // User.json, Order.json, Error.json, etc.
/// ```
pub struct JsonSchemaGenerator {
    config: GeneratorConfig,
}

impl JsonSchemaGenerator {
    /// Creates a new JSON Schema generator with the given configuration.
    pub const fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Creates a new JSON Schema generator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Generates an index file listing all schemas.
    #[allow(clippy::unused_self)] // Keep self for future config usage
    fn generate_index(&self, contract: &Contract) -> String {
        let mut output = String::new();

        output.push_str("{\n");
        output.push_str("  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n");
        output.push_str(&format!(
            "  \"$id\": \"{}_schemas.json\",\n",
            contract.metadata.service_name.to_lowercase().replace(' ', "_")
        ));
        output.push_str(&format!(
            "  \"title\": \"{} Schemas\",\n",
            &contract.metadata.service_name
        ));
        if let Some(ref desc) = contract.metadata.description {
            output.push_str(&format!("  \"description\": \"{desc}\",\n"));
        }

        // Add $defs with references to all schemas
        output.push_str("  \"$defs\": {\n");

        let schema_names: Vec<&String> = contract.schemas.keys().collect();
        for (i, name) in schema_names.iter().enumerate() {
            output.push_str(&format!("    \"{name}\": {{ \"$ref\": \"{name}.json\" }}"));
            if i < schema_names.len() - 1 {
                output.push_str(",\n");
            } else {
                output.push('\n');
            }
        }

        output.push_str("  }\n");
        output.push_str("}\n");

        output
    }
}

impl CodeGenerator for JsonSchemaGenerator {
    fn language_name(&self) -> &'static str {
        "JSON Schema"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }

    fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    fn generate(&self, contract: &Contract) -> CodegenResult<GeneratedCode> {
        let mut output = GeneratedCode::new();
        let converter = JsonSchemaConverter::new();

        // Convert all schemas to JSON Schema
        let json_schemas = converter.convert_schemas(&contract.schemas);

        // Generate a file for each schema
        for (name, schema) in &json_schemas {
            let content = serde_json::to_string_pretty(schema)
                .unwrap_or_else(|_| format!("{{\"error\": \"Failed to serialize {name}\"}}"));

            output.add_file(GeneratedFile::new(format!("{name}.json"), content));
        }

        // Generate index file
        let index_content = self.generate_index(contract);
        let index_name = format!(
            "{}_schemas.json",
            contract.metadata.service_name.to_lowercase().replace(' ', "_")
        );
        output.add_file(GeneratedFile::new(index_name, index_content));

        // Add warning if no schemas found
        if contract.schemas.is_empty() {
            output
                .warnings
                .push("No schemas found in contract".to_string());
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use themis_core::schema::{IntegerSchema, ObjectSchema, StringSchema};
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::Schema;

    fn create_test_contract() -> Contract {
        let mut schemas = IndexMap::new();

        // User schema
        let mut user_props = IndexMap::new();
        user_props.insert(
            "id".to_string(),
            Schema::String(StringSchema {
                format: Some("uuid".to_string()),
                ..Default::default()
            }),
        );
        user_props.insert(
            "name".to_string(),
            Schema::String(StringSchema::default()),
        );
        user_props.insert(
            "age".to_string(),
            Schema::Integer(IntegerSchema {
                minimum: Some(0),
                ..Default::default()
            }),
        );

        schemas.insert(
            "User".to_string(),
            Schema::Object(ObjectSchema {
                description: Some("A user in the system".to_string()),
                properties: user_props,
                required: vec!["id".to_string(), "name".to_string()],
                ..Default::default()
            }),
        );

        Contract {
            format: ContractFormat::OpenApi,
            version: "1.0.0".parse().unwrap(),
            metadata: ContractMetadata {
                service_name: "Users Service".to_string(),
                description: Some("User management API".to_string()),
                owner: None,
                repository: None,
                documentation_url: None,
            },
            schemas,
            operations: Default::default(),
            security_schemes: Default::default(),
        }
    }

    #[test]
    fn test_generate_creates_schema_files() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        // Should have User.json and index file
        assert_eq!(result.files.len(), 2);

        let user_file = result.files.iter().find(|f| f.path == "User.json");
        assert!(user_file.is_some());

        let index_file = result
            .files
            .iter()
            .find(|f| f.path == "users_service_schemas.json");
        assert!(index_file.is_some());
    }

    #[test]
    fn test_generated_schema_is_valid_json() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        for file in &result.files {
            let parsed: serde_json::Value = serde_json::from_str(&file.content)
                .unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", file.path, e));
            assert!(parsed.is_object());
        }
    }

    #[test]
    fn test_schema_has_correct_draft() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        let user_file = result
            .files
            .iter()
            .find(|f| f.path == "User.json")
            .unwrap();
        let schema: serde_json::Value = serde_json::from_str(&user_file.content).unwrap();

        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
    }

    #[test]
    fn test_schema_has_title_and_id() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        let user_file = result
            .files
            .iter()
            .find(|f| f.path == "User.json")
            .unwrap();
        let schema: serde_json::Value = serde_json::from_str(&user_file.content).unwrap();

        assert_eq!(schema["$id"], "User.json");
        assert_eq!(schema["title"], "User");
    }

    #[test]
    fn test_schema_properties() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        let user_file = result
            .files
            .iter()
            .find(|f| f.path == "User.json")
            .unwrap();
        let schema: serde_json::Value = serde_json::from_str(&user_file.content).unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["id"].is_object());
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["properties"]["age"].is_object());
    }

    #[test]
    fn test_schema_required_fields() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        let user_file = result
            .files
            .iter()
            .find(|f| f.path == "User.json")
            .unwrap();
        let schema: serde_json::Value = serde_json::from_str(&user_file.content).unwrap();

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("id")));
        assert!(required.contains(&serde_json::json!("name")));
    }

    #[test]
    fn test_index_file_references_all_schemas() {
        let contract = create_test_contract();
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        let index_file = result
            .files
            .iter()
            .find(|f| f.path.ends_with("_schemas.json"))
            .unwrap();
        let index: serde_json::Value = serde_json::from_str(&index_file.content).unwrap();

        assert!(index["$defs"]["User"].is_object());
        assert_eq!(index["$defs"]["User"]["$ref"], "User.json");
    }

    #[test]
    fn test_empty_contract_produces_warning() {
        let contract = Contract {
            format: ContractFormat::OpenApi,
            version: "1.0.0".parse().unwrap(),
            metadata: ContractMetadata {
                service_name: "Empty".to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: Default::default(),
            schemas: Default::default(),
            security_schemes: Default::default(),
        };
        let generator = JsonSchemaGenerator::with_defaults();

        let result = generator.generate(&contract).unwrap();

        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("No schemas"));
    }

    #[test]
    fn test_const_new() {
        // Verify const fn works at compile time
        const _: JsonSchemaGenerator = JsonSchemaGenerator::new(GeneratorConfig::new());
    }
}
