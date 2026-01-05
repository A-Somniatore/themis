//! Main Rust code generator.

use super::types::RustTypeGenerator;
use crate::config::GeneratorConfig;
use crate::error::{CodegenError, CodegenResult};
use crate::traits::{CodeGenerator, GeneratedCode, GeneratedFile};
use std::fmt::Write;
use themis_core::operation::{Parameter, ParameterLocation};
use themis_core::{Contract, Operation, Schema};

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

        // Imports
        output.push_str("use super::types::*;\n");
        output.push_str("use std::error::Error as StdError;\n");
        output.push_str("use std::fmt;\n");
        output.push('\n');
        output.push_str("#[allow(unused_imports)]\n");
        output.push_str("use async_trait::async_trait;\n");
        output.push('\n');

        // Generate RequestContext placeholder
        output.push_str(&Self::generate_request_context());
        output.push('\n');

        // Generate error types
        output.push_str(&Self::generate_error_types(contract, &self.config));
        output.push('\n');

        // Generate request/response types for each operation
        for (op_id, operation) in &contract.operations {
            output.push_str(&Self::generate_operation_types(
                op_id,
                operation,
                &self.config,
            ));
            output.push('\n');
        }

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

    /// Generates the RequestContext struct.
    ///
    /// This is a placeholder that can be replaced with Archimedes' RequestContext
    /// once integration is established.
    fn generate_request_context() -> String {
        r"/// Request context containing metadata about the incoming request.
///
/// This struct provides access to request headers, authentication info,
/// and other contextual data. It will integrate with Archimedes' RequestContext
/// in production deployments.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request ID for tracing
    pub request_id: String,
    /// Optional authenticated user ID
    pub user_id: Option<String>,
    /// Request headers (key-value pairs)
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            request_id: String::new(),
            user_id: None,
            headers: std::collections::HashMap::new(),
        }
    }
}

impl RequestContext {
    /// Creates a new request context with the given request ID.
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            ..Default::default()
        }
    }

    /// Gets a header value by name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }
}

"
        .to_string()
    }

    /// Generates error types for the service.
    fn generate_error_types(contract: &Contract, config: &GeneratorConfig) -> String {
        let mut output = String::new();
        let service_name = Self::service_name(contract);

        // Generate main error enum
        if config.include_docs {
            let _ = writeln!(output, "/// Error type for {service_name} operations.");
        }
        let _ = writeln!(output, "#[derive(Debug)]");
        let _ = writeln!(output, "pub enum {service_name}Error {{");
        let _ = writeln!(output, "    /// Bad request (400)");
        let _ = writeln!(output, "    BadRequest(String),");
        let _ = writeln!(output, "    /// Unauthorized (401)");
        let _ = writeln!(output, "    Unauthorized(String),");
        let _ = writeln!(output, "    /// Forbidden (403)");
        let _ = writeln!(output, "    Forbidden(String),");
        let _ = writeln!(output, "    /// Not found (404)");
        let _ = writeln!(output, "    NotFound(String),");
        let _ = writeln!(output, "    /// Conflict (409)");
        let _ = writeln!(output, "    Conflict(String),");
        let _ = writeln!(output, "    /// Unprocessable entity (422)");
        let _ = writeln!(output, "    UnprocessableEntity(String),");
        let _ = writeln!(output, "    /// Internal server error (500)");
        let _ = writeln!(output, "    Internal(String),");
        let _ = writeln!(output, "    /// Custom error with status code");
        let _ = writeln!(output, "    Custom {{ status: u16, message: String }},");
        let _ = writeln!(output, "}}");
        output.push('\n');

        // Implement Display
        let _ = writeln!(output, "impl fmt::Display for {service_name}Error {{");
        let _ = writeln!(
            output,
            "    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{"
        );
        let _ = writeln!(output, "        match self {{");
        let _ = writeln!(
            output,
            "            Self::BadRequest(msg) => write!(f, \"Bad request: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::Unauthorized(msg) => write!(f, \"Unauthorized: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::Forbidden(msg) => write!(f, \"Forbidden: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::NotFound(msg) => write!(f, \"Not found: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::Conflict(msg) => write!(f, \"Conflict: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::UnprocessableEntity(msg) => write!(f, \"Unprocessable entity: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::Internal(msg) => write!(f, \"Internal error: {{msg}}\"),"
        );
        let _ = writeln!(
            output,
            "            Self::Custom {{ status, message }} => write!(f, \"Error {{status}}: {{message}}\"),"
        );
        let _ = writeln!(output, "        }}");
        let _ = writeln!(output, "    }}");
        let _ = writeln!(output, "}}");
        output.push('\n');

        // Implement Error
        let _ = writeln!(output, "impl StdError for {service_name}Error {{}}");
        output.push('\n');

        // Implement status code method
        let _ = writeln!(output, "impl {service_name}Error {{");
        let _ = writeln!(
            output,
            "    /// Returns the HTTP status code for this error."
        );
        let _ = writeln!(output, "    pub const fn status_code(&self) -> u16 {{");
        let _ = writeln!(output, "        match self {{");
        let _ = writeln!(output, "            Self::BadRequest(_) => 400,");
        let _ = writeln!(output, "            Self::Unauthorized(_) => 401,");
        let _ = writeln!(output, "            Self::Forbidden(_) => 403,");
        let _ = writeln!(output, "            Self::NotFound(_) => 404,");
        let _ = writeln!(output, "            Self::Conflict(_) => 409,");
        let _ = writeln!(output, "            Self::UnprocessableEntity(_) => 422,");
        let _ = writeln!(output, "            Self::Internal(_) => 500,");
        let _ = writeln!(
            output,
            "            Self::Custom {{ status, .. }} => *status,"
        );
        let _ = writeln!(output, "        }}");
        let _ = writeln!(output, "    }}");
        let _ = writeln!(output, "}}");

        output
    }

    /// Generates request and response types for an operation.
    fn generate_operation_types(
        op_id: &str,
        operation: &Operation,
        config: &GeneratorConfig,
    ) -> String {
        let mut output = String::new();
        let type_prefix = heck::AsUpperCamelCase(op_id).to_string();

        // Generate Request type
        output.push_str(&Self::generate_request_type(
            &type_prefix,
            operation,
            config,
        ));
        output.push('\n');

        // Generate Response type
        output.push_str(&Self::generate_response_type(
            &type_prefix,
            operation,
            config,
        ));

        output
    }

    /// Generates the request type for an operation.
    fn generate_request_type(
        type_prefix: &str,
        operation: &Operation,
        config: &GeneratorConfig,
    ) -> String {
        let mut output = String::new();
        let request_type = format!("{type_prefix}Request");

        if config.include_docs {
            if let Some(desc) = &operation.description {
                let _ = writeln!(output, "/// Request for: {desc}");
            } else if let Some(summary) = &operation.summary {
                let _ = writeln!(output, "/// Request for: {summary}");
            } else {
                let _ = writeln!(output, "/// Request type for {type_prefix}.");
            }
        }

        let _ = writeln!(
            output,
            "#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]"
        );
        let _ = writeln!(output, "pub struct {request_type} {{");

        // Add path parameters
        for param in &operation.parameters {
            if param.location == ParameterLocation::Path {
                let field_name = heck::AsSnakeCase(&param.name).to_string();
                let rust_type = Self::param_to_rust_type(param);
                if config.include_docs {
                    if let Some(desc) = &param.description {
                        let _ = writeln!(output, "    /// {desc}");
                    }
                }
                let _ = writeln!(output, "    pub {field_name}: {rust_type},");
            }
        }

        // Add query parameters
        for param in &operation.parameters {
            if param.location == ParameterLocation::Query {
                let field_name = heck::AsSnakeCase(&param.name).to_string();
                let rust_type = Self::param_to_rust_type(param);
                let rust_type = if param.required {
                    rust_type
                } else {
                    format!("Option<{rust_type}>")
                };
                if config.include_docs {
                    if let Some(desc) = &param.description {
                        let _ = writeln!(output, "    /// {desc}");
                    }
                }
                let _ = writeln!(output, "    pub {field_name}: {rust_type},");
            }
        }

        // Add body field if there's a request body
        if let Some(ref body) = operation.request_body {
            if let Some(content) = body.content.get("application/json") {
                let body_type = match &content.schema {
                    Schema::Ref(r) => r
                        .reference
                        .rsplit('/')
                        .next()
                        .unwrap_or("serde_json::Value")
                        .to_string(),
                    _ => "serde_json::Value".to_string(),
                };
                if config.include_docs {
                    let _ = writeln!(output, "    /// Request body");
                }
                let _ = writeln!(output, "    pub body: {body_type},");
            }
        }

        let _ = writeln!(output, "}}");

        output
    }

    /// Generates the response type for an operation.
    fn generate_response_type(
        type_prefix: &str,
        operation: &Operation,
        config: &GeneratorConfig,
    ) -> String {
        let mut output = String::new();
        let response_type = format!("{type_prefix}Response");

        if config.include_docs {
            let _ = writeln!(output, "/// Response type for {type_prefix}.");
        }

        let _ = writeln!(
            output,
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]"
        );
        let _ = writeln!(output, "#[serde(untagged)]");
        let _ = writeln!(output, "pub enum {response_type} {{");

        // Look for success responses (2xx)
        let mut has_variants = false;
        for (status, response) in &operation.responses {
            if status.starts_with('2') {
                has_variants = true;
                let variant_name = Self::status_to_variant_name(status);

                if let Some(content) = response.content.get("application/json") {
                    let body_type = match &content.schema {
                        Schema::Ref(r) => r
                            .reference
                            .rsplit('/')
                            .next()
                            .unwrap_or("serde_json::Value")
                            .to_string(),
                        _ => "serde_json::Value".to_string(),
                    };
                    if config.include_docs && !response.description.is_empty() {
                        let _ = writeln!(output, "    /// {}", response.description);
                    }
                    let _ = writeln!(output, "    {variant_name}({body_type}),");
                } else {
                    // No body
                    if config.include_docs && !response.description.is_empty() {
                        let _ = writeln!(output, "    /// {}", response.description);
                    }
                    let _ = writeln!(output, "    {variant_name},");
                }
            }
        }

        // If no success responses found, add a default Ok variant
        if !has_variants {
            let _ = writeln!(output, "    /// Success with no body");
            let _ = writeln!(output, "    Ok,");
        }

        let _ = writeln!(output, "}}");

        output
    }

    /// Converts a parameter to its Rust type.
    fn param_to_rust_type(param: &Parameter) -> String {
        match &param.schema {
            Schema::String(s) => match s.format.as_deref() {
                Some("uuid") => "uuid::Uuid".to_string(),
                Some("date-time") => "chrono::DateTime<chrono::Utc>".to_string(),
                Some("date") => "chrono::NaiveDate".to_string(),
                _ => "String".to_string(),
            },
            Schema::Integer(_) => "i64".to_string(),
            Schema::Number(_) => "f64".to_string(),
            Schema::Boolean(_) => "bool".to_string(),
            Schema::Array(a) => {
                let item_type = match &*a.items {
                    Schema::String(_) => "String",
                    Schema::Integer(_) => "i64",
                    Schema::Number(_) => "f64",
                    Schema::Boolean(_) => "bool",
                    _ => "serde_json::Value",
                };
                format!("Vec<{item_type}>")
            }
            _ => "serde_json::Value".to_string(),
        }
    }

    /// Converts an HTTP status code to a variant name.
    fn status_to_variant_name(status: &str) -> String {
        match status {
            "200" => "Ok".to_string(),
            "201" => "Created".to_string(),
            "202" => "Accepted".to_string(),
            "204" => "NoContent".to_string(),
            _ => format!("Status{status}"),
        }
    }

    /// Generates a handler trait for an operation.
    fn generate_handler_trait(
        op_id: &str,
        operation: &Operation,
        config: &GeneratorConfig,
    ) -> String {
        let mut output = String::new();

        // Doc comment
        if config.include_docs {
            if let Some(desc) = &operation.description {
                let _ = writeln!(output, "/// Handler for: {desc}");
            } else if let Some(summary) = &operation.summary {
                let _ = writeln!(output, "/// Handler for: {summary}");
            }
            if let Some(ref method) = operation.method {
                if let Some(ref path) = operation.path {
                    let _ = writeln!(output, "///");
                    let _ = writeln!(output, "/// {method:?} {path}");
                }
            }
        }

        let trait_name = format!("{}Handler", heck::AsUpperCamelCase(op_id));
        let type_prefix = heck::AsUpperCamelCase(op_id).to_string();
        let request_type = format!("{type_prefix}Request");
        let response_type = format!("{type_prefix}Response");

        // Get the service name from the parent - we'll use a generic error for now
        // since we don't have access to the contract here
        output.push_str("#[async_trait]\n");
        let _ = writeln!(output, "pub trait {trait_name}: Send + Sync + 'static {{");
        output.push_str("    /// Handles the request with context.\n");
        let _ = writeln!(output, "    ///");
        let _ = writeln!(output, "    /// # Arguments");
        let _ = writeln!(output, "    ///");
        let _ = writeln!(
            output,
            "    /// * `ctx` - Request context containing metadata, headers, and auth info"
        );
        let _ = writeln!(
            output,
            "    /// * `request` - The typed request parameters and body"
        );
        let _ = writeln!(output, "    ///");
        let _ = writeln!(output, "    /// # Returns");
        let _ = writeln!(output, "    ///");
        let _ = writeln!(output, "    /// The typed response or a service error");
        let _ = writeln!(
            output,
            "    async fn handle(\n        &self,\n        ctx: &RequestContext,\n        request: {request_type},\n    ) -> Result<{response_type}, Box<dyn std::error::Error + Send + Sync>>;"
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

    fn create_contract_with_parameters() -> Contract {
        let mut contract = Contract::new(
            ContractFormat::OpenApi,
            Version::new(1, 0, 0),
            "user-service",
        );

        // Add User schema
        contract.schemas.insert(
            "User".to_string(),
            Schema::Object(ObjectSchema {
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

        // Add operation with path and query parameters
        let mut op = Operation::new("listUsers");
        op.method = Some(HttpMethod::Get);
        op.path = Some("/organizations/{orgId}/users".to_string());
        op.summary = Some("List users in organization".to_string());

        // Add path parameter
        op.parameters.push(themis_core::operation::Parameter {
            name: "orgId".to_string(),
            location: ParameterLocation::Path,
            description: Some("Organization ID".to_string()),
            required: true,
            deprecated: false,
            schema: Schema::String(StringSchema {
                format: Some("uuid".to_string()),
                ..Default::default()
            }),
        });

        // Add query parameters
        op.parameters.push(themis_core::operation::Parameter {
            name: "page".to_string(),
            location: ParameterLocation::Query,
            description: Some("Page number".to_string()),
            required: false,
            deprecated: false,
            schema: Schema::Integer(themis_core::schema::IntegerSchema::default()),
        });

        op.parameters.push(themis_core::operation::Parameter {
            name: "limit".to_string(),
            location: ParameterLocation::Query,
            description: Some("Results per page".to_string()),
            required: true,
            deprecated: false,
            schema: Schema::Integer(themis_core::schema::IntegerSchema::default()),
        });

        contract.operations.insert("listUsers".to_string(), op);
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
    fn test_rust_generator_generates_request_context() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        assert!(handlers_file.content.contains("pub struct RequestContext"));
        assert!(handlers_file.content.contains("pub request_id: String"));
        assert!(handlers_file
            .content
            .contains("pub user_id: Option<String>"));
        assert!(handlers_file
            .content
            .contains("pub headers: std::collections::HashMap<String, String>"));
    }

    #[test]
    fn test_rust_generator_generates_error_types() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        assert!(handlers_file.content.contains("pub enum TestServiceError"));
        assert!(handlers_file.content.contains("BadRequest(String)"));
        assert!(handlers_file.content.contains("NotFound(String)"));
        assert!(handlers_file.content.contains("Internal(String)"));
        assert!(handlers_file
            .content
            .contains("fn status_code(&self) -> u16"));
    }

    #[test]
    fn test_rust_generator_generates_request_types() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        // Should generate GetUserRequest struct
        assert!(handlers_file.content.contains("pub struct GetUserRequest"));
    }

    #[test]
    fn test_rust_generator_generates_response_types() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        // Should generate GetUserResponse enum
        assert!(handlers_file.content.contains("pub enum GetUserResponse"));
    }

    #[test]
    fn test_rust_generator_handler_uses_request_context() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        // Handler should accept RequestContext parameter
        assert!(handlers_file.content.contains("ctx: &RequestContext"));
    }

    #[test]
    fn test_rust_generator_generates_service_struct() {
        let generator = RustGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        assert!(handlers_file
            .content
            .contains("pub struct TestServiceService"));
        assert!(handlers_file.content.contains("GetUser: GetUserHandler"));
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

    #[test]
    fn test_rust_generator_with_path_parameters() {
        let generator = RustGenerator::with_defaults();
        let contract = create_contract_with_parameters();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        // Should have path parameter in request struct
        assert!(handlers_file
            .content
            .contains("pub struct ListUsersRequest"));
        assert!(handlers_file.content.contains("pub org_id: uuid::Uuid"));
    }

    #[test]
    fn test_rust_generator_with_query_parameters() {
        let generator = RustGenerator::with_defaults();
        let contract = create_contract_with_parameters();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        // Required query param should not be Option
        assert!(handlers_file.content.contains("pub limit: i64"));
        // Optional query param should be Option
        assert!(handlers_file.content.contains("pub page: Option<i64>"));
    }

    #[test]
    fn test_rust_generator_request_has_serde_derives() {
        let generator = RustGenerator::with_defaults();
        let contract = create_contract_with_parameters();

        let result = generator.generate(&contract).unwrap();
        let handlers_file = result.get_file("handlers.rs").unwrap();

        // Request types should have serde derives
        assert!(handlers_file
            .content
            .contains("serde::Serialize, serde::Deserialize"));
    }
}
