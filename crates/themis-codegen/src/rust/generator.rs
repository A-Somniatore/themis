//! Main Rust code generator.

use super::types::RustTypeGenerator;
use crate::config::GeneratorConfig;
use crate::error::{CodegenError, CodegenResult};
use crate::traits::{CodeGenerator, GeneratedCode, GeneratedFile};
use std::fmt::Write;
use themis_core::Contract;

/// Rust code generator.
///
/// Generates Rust types, error types, and handler traits from Themis contracts.
pub struct RustGenerator {
    config: GeneratorConfig,
}

impl RustGenerator {
    /// Creates a new Rust generator with the given configuration.
    pub const fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Creates a new Rust generator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Generates the types module.
    fn generate_types(&self, contract: &Contract) -> CodegenResult<String> {
        let mut type_gen = RustTypeGenerator::new(&self.config);

        // Generate header
        let mut output = String::new();
        output.push_str(&Self::generate_header(contract));

        // Generate types from schemas
        let types_code = type_gen.generate_types(&contract.schemas)?;
        output.push_str(&types_code);

        Ok(output)
    }

    /// Generates the handlers module.
    fn generate_handlers(&self, contract: &Contract) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&Self::generate_header(contract));
        output.push_str("use super::types::*;\n");
        output.push('\n');
        output.push_str("#[allow(unused_imports)]\n");
        output.push_str("use async_trait::async_trait;\n");
        output.push('\n');

        // Generate handler trait for each operation
        for (op_id, operation) in &contract.operations {
            output.push_str(&Self::generate_handler_trait(
                op_id,
                operation,
                &self.config,
            ));
            output.push('\n');
        }

        // Generate service struct that combines all handlers
        output.push_str(&Self::generate_service_struct(contract));

        output
    }

    /// Generates a handler trait for an operation.
    fn generate_handler_trait(
        op_id: &str,
        operation: &themis_core::Operation,
        config: &GeneratorConfig,
    ) -> String {
        let mut output = String::new();

        // Doc comment
        if config.include_docs {
            if let Some(desc) = &operation.description {
                let _ = writeln!(output, "/// {desc}");
            } else if let Some(summary) = &operation.summary {
                let _ = writeln!(output, "/// {summary}");
            }
        }

        let trait_name = format!("{}Handler", heck::AsUpperCamelCase(op_id));
        let request_type = Self::infer_request_type(operation);
        let response_type = Self::infer_response_type(operation);

        output.push_str("#[async_trait]\n");
        let _ = writeln!(output, "pub trait {trait_name}: Send + Sync + 'static {{");
        output.push_str("    /// Handles the request.\n");
        let _ = writeln!(
            output,
            "    async fn handle(\n        &self,\n        request: {request_type},\n    ) -> Result<{response_type}, Box<dyn std::error::Error + Send + Sync>>;"
        );
        output.push_str("}\n");

        output
    }

    /// Generates the service struct that combines all handlers.
    fn generate_service_struct(contract: &Contract) -> String {
        let mut output = String::new();

        let service_name = Self::service_name(contract);
        let service_struct = format!("{service_name}Service");

        // Doc comment
        let _ = writeln!(output, "/// All handlers for {service_name}.");

        // Struct with generic parameters
        let _ = write!(output, "pub struct {service_struct}");

        if !contract.operations.is_empty() {
            output.push('<');
            let generics: Vec<_> = contract
                .operations
                .keys()
                .map(|op_id| heck::AsUpperCamelCase(op_id).to_string())
                .collect();
            output.push_str(&generics.join(", "));
            output.push_str(">\n");

            // Where clause
            output.push_str("where\n");
            for op_id in contract.operations.keys() {
                let generic = heck::AsUpperCamelCase(op_id).to_string();
                let trait_name = format!("{generic}Handler");
                let _ = writeln!(output, "    {generic}: {trait_name},");
            }
        }

        output.push_str("{\n");

        // Fields
        for op_id in contract.operations.keys() {
            let field_name = heck::AsSnakeCase(op_id).to_string();
            let generic = heck::AsUpperCamelCase(op_id).to_string();
            let _ = writeln!(output, "    pub {field_name}: {generic},");
        }

        output.push_str("}\n");

        output
    }

    /// Generates the file header.
    fn generate_header(contract: &Contract) -> String {
        format!(
            "// Auto-generated by themis-codegen. DO NOT EDIT.\n\
             // Contract: {} v{}\n\
             // Generated: {}\n\n",
            contract.metadata.service_name,
            contract.version,
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
        )
    }

    /// Infers the request type for an operation.
    fn infer_request_type(operation: &themis_core::Operation) -> String {
        // Look for request body schema
        if let Some(ref body) = operation.request_body {
            if let Some(content) = body.content.get("application/json") {
                let schema = &content.schema;
                if let themis_core::Schema::Ref(r) = schema {
                    let name = r.reference.rsplit('/').next().unwrap_or("Request");
                    return name.to_string();
                }
            }
        }

        // Default to a unit struct
        format!("{}Request", heck::AsUpperCamelCase(&operation.operation_id))
    }

    /// Infers the response type for an operation.
    fn infer_response_type(operation: &themis_core::Operation) -> String {
        // Look for 200/201 response schema
        for status in ["200", "201", "202"] {
            if let Some(response) = operation.responses.get(status) {
                if let Some(content) = response.content.get("application/json") {
                    let schema = &content.schema;
                    if let themis_core::Schema::Ref(r) = schema {
                        let name = r.reference.rsplit('/').next().unwrap_or("Response");
                        return name.to_string();
                    }
                }
            }
        }

        // Default to unit type
        "()".to_string()
    }

    /// Gets the service name from the contract.
    fn service_name(contract: &Contract) -> String {
        heck::AsUpperCamelCase(&contract.metadata.service_name).to_string()
    }

    /// Generates the mod.rs file.
    fn generate_mod_file() -> String {
        "// Auto-generated by themis-codegen. DO NOT EDIT.\n\n\
         pub mod types;\n\
         pub mod handlers;\n\n\
         pub use types::*;\n\
         pub use handlers::*;\n"
            .to_string()
    }
}

impl CodeGenerator for RustGenerator {
    fn language_name(&self) -> &'static str {
        "Rust"
    }

    fn file_extension(&self) -> &'static str {
        "rs"
    }

    fn generate(&self, contract: &Contract) -> CodegenResult<GeneratedCode> {
        let mut output = GeneratedCode::new();

        // Validate contract has operations
        if contract.operations.is_empty() {
            return Err(CodegenError::invalid_contract("no operations defined"));
        }

        // Generate types
        let types_content = self.generate_types(contract)?;
        output.add_file(GeneratedFile::new("types.rs", types_content));

        // Generate handlers
        let handlers_content = self.generate_handlers(contract);
        output.add_file(GeneratedFile::new("handlers.rs", handlers_content));

        // Generate mod.rs
        output.add_file(GeneratedFile::new("mod.rs", Self::generate_mod_file()));

        // Add warnings for unsupported features
        if contract.operations.values().any(|op| {
            op.request_body
                .as_ref()
                .is_some_and(|b| b.content.contains_key("multipart/form-data"))
        }) {
            output.add_warning("multipart/form-data request bodies are not fully supported");
        }

        Ok(output)
    }

    fn config(&self) -> &GeneratorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use themis_core::contract::ContractFormat;
    use themis_core::operation::HttpMethod;
    use themis_core::schema::{ObjectSchema, StringSchema};
    use themis_core::{Operation, Schema, Version};

    fn create_test_contract() -> Contract {
        let mut contract = Contract::new(
            ContractFormat::OpenApi,
            Version::new(1, 0, 0),
            "test-service",
        );

        // Add a schema
        contract.schemas.insert(
            "User".to_string(),
            Schema::Object(ObjectSchema {
                description: Some("A user".to_string()),
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "id".to_string(),
                        Schema::String(StringSchema {
                            format: Some("uuid".to_string()),
                            ..Default::default()
                        }),
                    );
                    props.insert("name".to_string(), Schema::String(StringSchema::default()));
                    props
                },
                required: vec!["id".to_string()],
                ..Default::default()
            }),
        );

        // Add an operation
        let mut op = Operation::new("getUser");
        op.method = Some(HttpMethod::Get);
        op.path = Some("/users/{id}".to_string());
        op.summary = Some("Get a user".to_string());
        op.description = Some("Retrieves a user by ID".to_string());
        contract.operations.insert("getUser".to_string(), op);

        contract
    }

    #[test]
    fn test_rust_generator_creates_files() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        assert!(result.has_files());
        assert_eq!(result.files.len(), 3); // types.rs, handlers.rs, mod.rs

        assert!(result.get_file("types.rs").is_some());
        assert!(result.get_file("handlers.rs").is_some());
        assert!(result.get_file("mod.rs").is_some());
    }

    #[test]
    fn test_rust_generator_types_content() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let types_file = result.get_file("types.rs").unwrap();

        assert!(types_file.content.contains("pub struct User"));
        assert!(types_file.content.contains("pub id: uuid::Uuid"));
        assert!(types_file.content.contains("Serialize, Deserialize"));
    }

    #[test]
    fn test_rust_generator_handlers_content() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        assert!(handlers_file.content.contains("pub trait GetUserHandler"));
        assert!(handlers_file.content.contains("async fn handle"));
    }

    #[test]
    fn test_rust_generator_mod_file() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let mod_file = result.get_file("mod.rs").unwrap();

        assert!(mod_file.content.contains("pub mod types;"));
        assert!(mod_file.content.contains("pub mod handlers;"));
    }

    #[test]
    fn test_rust_generator_empty_contract_fails() {
        let generator = RustGenerator::with_defaults();
        let contract = Contract::new(ContractFormat::OpenApi, Version::new(1, 0, 0), "empty");

        let result = generator.generate(&contract);
        assert!(result.is_err());
    }

    #[test]
    fn test_rust_generator_language_name() {
        let generator = RustGenerator::with_defaults();
        assert_eq!(generator.language_name(), "Rust");
        assert_eq!(generator.file_extension(), "rs");
    }
}
