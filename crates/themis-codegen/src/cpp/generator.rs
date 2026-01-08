//! Main C++ code generator.

// Allow some pedantic clippy lints that are acceptable in code generators
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_wraps)]

use super::types::CppTypeGenerator;
use crate::config::GeneratorConfig;
use crate::error::CodegenResult;
use crate::traits::{CodeGenerator, GeneratedCode, GeneratedFile};
use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fmt::Write;
use themis_core::operation::ParameterLocation;
use themis_core::{Contract, Operation};

/// C++ code generator.
///
/// Generates C++ header files with structs, enums, and handler interfaces
/// from Themis contracts. Uses modern C++17/20 features including:
/// - `std::optional` for nullable fields
/// - `std::variant` for union types
/// - `std::vector` for arrays
/// - `nlohmann::json` for JSON serialization (external dependency)
pub struct CppGenerator {
    config: GeneratorConfig,
}

impl CppGenerator {
    /// Creates a new C++ generator with the given configuration.
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Creates a new C++ generator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Generates the types header file.
    fn generate_types_header(&self, contract: &Contract) -> CodegenResult<String> {
        let type_gen = CppTypeGenerator::new(&self.config);
        let guard_name = Self::header_guard_name(contract, "types");

        let mut output = String::new();

        // Header
        output.push_str(&Self::generate_header(contract));

        // Include guard
        let _ = writeln!(output, "#ifndef {guard_name}");
        let _ = writeln!(output, "#define {guard_name}");
        output.push('\n');

        // Includes
        output.push_str("#include <string>\n");
        output.push_str("#include <vector>\n");
        output.push_str("#include <optional>\n");
        output.push_str("#include <variant>\n");
        output.push_str("#include <cstdint>\n");
        output.push_str("#include <chrono>\n");
        output.push_str("#include <nlohmann/json.hpp>\n");
        output.push('\n');

        // Namespace
        let namespace = self.namespace_name(contract);
        let _ = writeln!(output, "namespace {namespace} {{");
        output.push('\n');

        // Generate types from schemas
        let types_code = type_gen.generate_types(&contract.schemas)?;
        output.push_str(&types_code);

        // Close namespace
        let _ = writeln!(output, "}} // namespace {namespace}");
        output.push('\n');

        // End include guard
        let _ = writeln!(output, "#endif // {guard_name}");

        Ok(output)
    }

    /// Generates the handlers header file.
    fn generate_handlers_header(&self, contract: &Contract) -> CodegenResult<String> {
        let guard_name = Self::header_guard_name(contract, "handlers");
        let namespace = self.namespace_name(contract);
        let service_name = Self::service_name(contract);

        let mut output = String::new();

        // Header
        output.push_str(&Self::generate_header(contract));

        // Include guard
        let _ = writeln!(output, "#ifndef {guard_name}");
        let _ = writeln!(output, "#define {guard_name}");
        output.push('\n');

        // Includes
        output.push_str("#include <string>\n");
        output.push_str("#include <memory>\n");
        output.push_str("#include <functional>\n");
        output.push_str("#include <expected>\n");
        output.push_str("#include <map>\n");
        let types_header = format!("{}_types.hpp", contract.metadata.service_name.to_snake_case());
        let _ = writeln!(output, "#include \"{types_header}\"");
        output.push('\n');

        // Namespace
        let _ = writeln!(output, "namespace {namespace} {{");
        output.push('\n');

        // Request context
        output.push_str(&Self::generate_request_context());
        output.push('\n');

        // Error types
        output.push_str(&self.generate_error_types(&service_name));
        output.push('\n');

        // Request/Response types for each operation
        for (op_id, operation) in &contract.operations {
            output.push_str(&self.generate_operation_types(op_id, operation)?);
            output.push('\n');
        }

        // Handler interfaces
        for (op_id, operation) in &contract.operations {
            output.push_str(&self.generate_handler_interface(op_id, operation, &service_name));
            output.push('\n');
        }

        // Service interface
        output.push_str(&self.generate_service_interface(contract, &service_name));

        // Close namespace
        let _ = writeln!(output, "}} // namespace {namespace}");
        output.push('\n');

        // End include guard
        let _ = writeln!(output, "#endif // {guard_name}");

        Ok(output)
    }

    /// Generates the `CMakeLists.txt` file.
    fn generate_cmake(contract: &Contract) -> String {
        let project_name = contract.metadata.service_name.to_snake_case();

        format!(
            r"# Auto-generated by themis-codegen. DO NOT EDIT.
# CMake configuration for {service_name} API types
# Contract version: {version}

cmake_minimum_required(VERSION 3.14)
project({project_name}_api VERSION {version})

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find nlohmann_json package
find_package(nlohmann_json 3.11.0 REQUIRED)

# Create interface library for the API types
add_library({project_name}_api INTERFACE)
target_include_directories({project_name}_api INTERFACE ${{CMAKE_CURRENT_SOURCE_DIR}})
target_link_libraries({project_name}_api INTERFACE nlohmann_json::nlohmann_json)

# Export targets
install(TARGETS {project_name}_api EXPORT {project_name}_api-targets)
install(EXPORT {project_name}_api-targets
    FILE {project_name}_api-targets.cmake
    NAMESPACE {project_name}::
    DESTINATION lib/cmake/{project_name}_api
)
",
            service_name = contract.metadata.service_name,
            version = contract.version,
            project_name = project_name,
        )
    }

    /// Generates the file header comment.
    fn generate_header(contract: &Contract) -> String {
        format!(
            r"// Auto-generated by themis-codegen. DO NOT EDIT.
// Contract: {} v{}
// Generated at: {}

",
            contract.metadata.service_name,
            contract.version,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    /// Generates the header guard name.
    fn header_guard_name(contract: &Contract, suffix: &str) -> String {
        format!(
            "{}_{}_{}_HPP",
            contract.metadata.service_name.to_snake_case().to_uppercase(),
            suffix.to_uppercase(),
            contract.version.to_string().replace('.', "_")
        )
    }

    /// Returns the namespace name.
    fn namespace_name(&self, contract: &Contract) -> String {
        self.config
            .module_name
            .clone()
            .unwrap_or_else(|| contract.metadata.service_name.to_snake_case())
    }

    /// Returns the service name in `PascalCase`.
    fn service_name(contract: &Contract) -> String {
        contract.metadata.service_name.to_upper_camel_case()
    }

    /// Generates the `RequestContext` struct.
    fn generate_request_context() -> String {
        r"/**
 * Request context containing metadata about the incoming request.
 *
 * Provides access to request headers, authentication info,
 * and other contextual data.
 */
struct RequestContext {
    /** Request ID for tracing */
    std::string request_id;
    /** Optional authenticated user ID */
    std::optional<std::string> user_id;
    /** Request headers (key-value pairs) */
    std::map<std::string, std::string> headers;

    RequestContext() = default;
    
    explicit RequestContext(std::string req_id) 
        : request_id(std::move(req_id)) {}
    
    /** Gets a header value by name */
    std::optional<std::string> header(const std::string& name) const {
        auto it = headers.find(name);
        if (it != headers.end()) {
            return it->second;
        }
        return std::nullopt;
    }
};

"
        .to_string()
    }

    /// Generates error types for the service.
    fn generate_error_types(&self, service_name: &str) -> String {
        let mut output = String::new();

        if self.config.include_docs {
            let _ = writeln!(output, "/** Error codes for {service_name} operations. */");
        }
        let _ = writeln!(output, "enum class {service_name}ErrorCode {{");
        let _ = writeln!(output, "    BadRequest = 400,");
        let _ = writeln!(output, "    Unauthorized = 401,");
        let _ = writeln!(output, "    Forbidden = 403,");
        let _ = writeln!(output, "    NotFound = 404,");
        let _ = writeln!(output, "    Conflict = 409,");
        let _ = writeln!(output, "    UnprocessableEntity = 422,");
        let _ = writeln!(output, "    Internal = 500");
        let _ = writeln!(output, "}};");
        output.push('\n');

        if self.config.include_docs {
            let _ = writeln!(output, "/** Error type for {service_name} operations. */");
        }
        let _ = writeln!(output, "struct {service_name}Error {{");
        let _ = writeln!(output, "    {service_name}ErrorCode code;");
        let _ = writeln!(output, "    std::string message;");
        output.push('\n');
        let _ = writeln!(output, "    {service_name}Error() = default;");
        let _ = writeln!(
            output,
            "    {service_name}Error({service_name}ErrorCode c, std::string msg)"
        );
        let _ = writeln!(output, "        : code(c), message(std::move(msg)) {{}}");
        output.push('\n');
        let _ = writeln!(output, "    /** Returns the HTTP status code. */");
        let _ = writeln!(
            output,
            "    int status_code() const {{ return static_cast<int>(code); }}"
        );
        let _ = writeln!(output, "}};");

        output
    }

    /// Generates request/response types for an operation.
    fn generate_operation_types(
        &self,
        op_id: &str,
        operation: &Operation,
    ) -> CodegenResult<String> {
        let mut output = String::new();
        let type_prefix = op_id.to_upper_camel_case();
        let type_gen = CppTypeGenerator::new(&self.config);

        // Request type
        if self.config.include_docs {
            if let Some(desc) = &operation.description {
                let _ = writeln!(output, "/** Request for: {desc} */");
            } else {
                let _ = writeln!(output, "/** Request for {op_id}. */");
            }
        }

        let _ = writeln!(output, "struct {type_prefix}Request {{");

        // Path parameters
        for param in &operation.parameters {
            if param.location == ParameterLocation::Path {
                let cpp_type = type_gen.schema_to_cpp_type(&param.schema)?;
                let field = param.name.to_snake_case();

                if self.config.include_docs {
                    if let Some(desc) = &param.description {
                        let _ = writeln!(output, "    /** {desc} */");
                    }
                }
                let _ = writeln!(output, "    {cpp_type} {field};");
            }
        }

        // Query parameters
        for param in &operation.parameters {
            if param.location == ParameterLocation::Query {
                let cpp_type = type_gen.schema_to_cpp_type(&param.schema)?;
                let field = param.name.to_snake_case();

                if self.config.include_docs {
                    if let Some(desc) = &param.description {
                        let _ = writeln!(output, "    /** {desc} */");
                    }
                }

                if param.required {
                    let _ = writeln!(output, "    {cpp_type} {field};");
                } else {
                    let _ = writeln!(output, "    std::optional<{cpp_type}> {field};");
                }
            }
        }

        // Request body
        if let Some(body) = &operation.request_body {
            if let Some(media_type) = body.content.get("application/json") {
                let cpp_type = type_gen.schema_to_cpp_type(&media_type.schema)?;
                if self.config.include_docs {
                    let _ = writeln!(output, "    /** Request body */");
                }
                let _ = writeln!(output, "    {cpp_type} body;");
            }
        }

        let _ = writeln!(output, "}};");
        output.push('\n');

        // Response type
        if self.config.include_docs {
            let _ = writeln!(output, "/** Response from {op_id}. */");
        }

        // Find success response schema
        let success_schema = operation
            .responses
            .iter()
            .filter(|(code, _)| code.starts_with('2'))
            .find_map(|(_, response)| {
                response
                    .content
                    .get("application/json")
                    .map(|media| &media.schema)
            });

        if let Some(schema) = success_schema {
            let cpp_type = type_gen.schema_to_cpp_type(schema)?;
            let _ = writeln!(output, "using {type_prefix}Response = {cpp_type};");
        } else {
            // No response body
            let _ = writeln!(output, "using {type_prefix}Response = void;");
        }

        Ok(output)
    }

    /// Generates a handler interface for an operation.
    fn generate_handler_interface(
        &self,
        op_id: &str,
        operation: &Operation,
        service_name: &str,
    ) -> String {
        let mut output = String::new();
        let handler_name = format!("{}Handler", op_id.to_upper_camel_case());
        let request_type = format!("{}Request", op_id.to_upper_camel_case());
        let response_type = format!("{}Response", op_id.to_upper_camel_case());

        if self.config.include_docs {
            let _ = writeln!(output, "/**");
            let _ = writeln!(output, " * Handler interface for {op_id}.");
            if let Some(desc) = &operation.description {
                let _ = writeln!(output, " *");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
            }
            let _ = writeln!(output, " */");
        }

        let _ = writeln!(output, "class {handler_name} {{");
        let _ = writeln!(output, "public:");
        let _ = writeln!(output, "    virtual ~{handler_name}() = default;");
        output.push('\n');
        let _ = writeln!(
            output,
            "    /**"
        );
        let _ = writeln!(output, "     * Handles the {op_id} operation.");
        let _ = writeln!(output, "     * @param ctx Request context");
        let _ = writeln!(output, "     * @param request The request data");
        let _ = writeln!(output, "     * @return Response or error");
        let _ = writeln!(output, "     */");
        let _ = writeln!(
            output,
            "    virtual std::expected<{response_type}, {service_name}Error> handle("
        );
        let _ = writeln!(output, "        const RequestContext& ctx,");
        let _ = writeln!(output, "        const {request_type}& request) = 0;");
        let _ = writeln!(output, "}};");

        output
    }

    /// Generates the service interface that combines all handlers.
    fn generate_service_interface(&self, contract: &Contract, service_name: &str) -> String {
        let mut output = String::new();

        if self.config.include_docs {
            let _ = writeln!(output, "/**");
            let _ = writeln!(output, " * Service interface for {service_name}.");
            if let Some(desc) = &contract.metadata.description {
                let _ = writeln!(output, " *");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
            }
            let _ = writeln!(output, " */");
        }

        let _ = writeln!(output, "class {service_name}Service {{");
        let _ = writeln!(output, "public:");
        let _ = writeln!(output, "    virtual ~{service_name}Service() = default;");
        output.push('\n');

        // Add virtual methods for each operation
        for (op_id, operation) in &contract.operations {
            let method_name = op_id.to_snake_case();
            let request_type = format!("{}Request", op_id.to_upper_camel_case());
            let response_type = format!("{}Response", op_id.to_upper_camel_case());

            if self.config.include_docs {
                if let Some(desc) = &operation.description {
                    let _ = writeln!(output, "    /** {desc} */");
                }
            }

            let _ = writeln!(
                output,
                "    virtual std::expected<{response_type}, {service_name}Error> {method_name}("
            );
            let _ = writeln!(output, "        const RequestContext& ctx,");
            let _ = writeln!(output, "        const {request_type}& request) = 0;");
            output.push('\n');
        }

        let _ = writeln!(output, "}};");

        output
    }
}

impl CodeGenerator for CppGenerator {
    fn language_name(&self) -> &'static str {
        "C++"
    }

    fn file_extension(&self) -> &'static str {
        "hpp"
    }

    fn generate(&self, contract: &Contract) -> CodegenResult<GeneratedCode> {
        let mut output = GeneratedCode::new();
        let base_name = contract.metadata.service_name.to_snake_case();

        // Generate types header
        let types_content = self.generate_types_header(contract)?;
        output.add_file(GeneratedFile::new(
            format!("{base_name}_types.hpp"),
            types_content,
        ));

        // Generate handlers header
        let handlers_content = self.generate_handlers_header(contract)?;
        output.add_file(GeneratedFile::new(
            format!("{base_name}_handlers.hpp"),
            handlers_content,
        ));

        // Generate CMakeLists.txt
        let cmake_content = Self::generate_cmake(contract);
        output.add_file(GeneratedFile::new_no_overwrite(
            "CMakeLists.txt",
            cmake_content,
        ));

        Ok(output)
    }

    fn config(&self) -> &GeneratorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::{MediaType, Parameter, RequestBody, Response};
    use themis_core::schema::{ObjectSchema, RefSchema, Schema, StringSchema};
    use themis_core::Version;

    fn create_test_contract() -> Contract {
        let mut schemas = IndexMap::new();
        let mut properties = IndexMap::new();

        properties.insert(
            "id".to_string(),
            Schema::String(StringSchema {
                format: Some("uuid".to_string()),
                pattern: None,
                min_length: None,
                max_length: None,
                description: Some("User ID".to_string()),
                nullable: false,
                default: None,
            }),
        );
        properties.insert(
            "name".to_string(),
            Schema::String(StringSchema {
                format: None,
                pattern: None,
                min_length: None,
                max_length: None,
                description: Some("User name".to_string()),
                nullable: false,
                default: None,
            }),
        );

        schemas.insert(
            "User".to_string(),
            Schema::Object(ObjectSchema {
                properties,
                required: vec!["id".to_string(), "name".to_string()],
                additional_properties: None,
                description: Some("A user in the system".to_string()),
                nullable: false,
            }),
        );

        let mut operations = std::collections::HashMap::new();

        // GetUser operation
        let mut get_responses = std::collections::HashMap::new();
        let mut get_content = std::collections::HashMap::new();
        get_content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema::Ref(RefSchema {
                    reference: "#/components/schemas/User".to_string(),
                }),
            },
        );
        get_responses.insert(
            "200".to_string(),
            Response {
                description: "Success".to_string(),
                content: get_content,
                headers: std::collections::HashMap::new(),
            },
        );

        operations.insert(
            "getUser".to_string(),
            Operation {
                operation_id: "getUser".to_string(),
                summary: Some("Get a user by ID".to_string()),
                description: Some("Retrieves a user by their unique identifier".to_string()),
                method: Some(themis_core::operation::HttpMethod::Get),
                path: Some("/users/{id}".to_string()),
                parameters: vec![Parameter {
                    name: "id".to_string(),
                    location: ParameterLocation::Path,
                    description: Some("User ID".to_string()),
                    required: true,
                    schema: Schema::String(StringSchema {
                        format: Some("uuid".to_string()),
                        pattern: None,
                        min_length: None,
                        max_length: None,
                        description: None,
                        nullable: false,
                        default: None,
                    }),
                    deprecated: false,
                }],
                request_body: None,
                responses: get_responses,
                security: vec![],
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        // CreateUser operation
        let mut create_responses = std::collections::HashMap::new();
        let mut create_content = std::collections::HashMap::new();
        create_content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema::Ref(RefSchema {
                    reference: "#/components/schemas/User".to_string(),
                }),
            },
        );
        create_responses.insert(
            "201".to_string(),
            Response {
                description: "Created".to_string(),
                content: create_content.clone(),
                headers: std::collections::HashMap::new(),
            },
        );

        let mut req_body_content = std::collections::HashMap::new();
        req_body_content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema::Ref(RefSchema {
                    reference: "#/components/schemas/User".to_string(),
                }),
            },
        );

        operations.insert(
            "createUser".to_string(),
            Operation {
                operation_id: "createUser".to_string(),
                summary: Some("Create a new user".to_string()),
                description: Some("Creates a new user in the system".to_string()),
                method: Some(themis_core::operation::HttpMethod::Post),
                path: Some("/users".to_string()),
                parameters: vec![],
                request_body: Some(RequestBody {
                    description: Some("User to create".to_string()),
                    required: true,
                    content: req_body_content,
                }),
                responses: create_responses,
                security: vec![],
                deprecated: false,
                tags: vec![],
                themis_metadata: None,
            },
        );

        Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "UserService".to_string(),
                description: Some("User management service".to_string()),
                owner: Some("platform-team".to_string()),
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas,
            security_schemes: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_generate_basic_contract() {
        let generator = CppGenerator::with_defaults();
        let contract = create_test_contract();

        let result = generator.generate(&contract).unwrap();

        assert_eq!(result.files.len(), 3);

        // Check types header
        let types_file = result.get_file("user_service_types.hpp").unwrap();
        assert!(types_file.content.contains("#ifndef USER_SERVICE_TYPES_"));
        assert!(types_file.content.contains("namespace user_service"));
        assert!(types_file.content.contains("struct User"));
        assert!(types_file.content.contains("std::string id;"));
        assert!(types_file.content.contains("std::string name;"));

        // Check handlers header
        let handlers_file = result.get_file("user_service_handlers.hpp").unwrap();
        assert!(handlers_file.content.contains("#ifndef USER_SERVICE_HANDLERS_"));
        assert!(handlers_file.content.contains("struct RequestContext"));
        assert!(handlers_file.content.contains("enum class UserServiceErrorCode"));
        assert!(handlers_file.content.contains("class GetUserHandler"));
        assert!(handlers_file.content.contains("class CreateUserHandler"));
        assert!(handlers_file.content.contains("class UserServiceService"));

        // Check CMakeLists.txt
        let cmake_file = result.get_file("CMakeLists.txt").unwrap();
        assert!(cmake_file.content.contains("project(user_service_api"));
        assert!(cmake_file.content.contains("nlohmann_json"));
        assert!(!cmake_file.overwrite); // Should not overwrite
    }

    #[test]
    fn test_language_name() {
        let generator = CppGenerator::with_defaults();
        assert_eq!(generator.language_name(), "C++");
    }

    #[test]
    fn test_file_extension() {
        let generator = CppGenerator::with_defaults();
        assert_eq!(generator.file_extension(), "hpp");
    }

    #[test]
    fn test_request_context_generation() {
        let context = CppGenerator::generate_request_context();
        assert!(context.contains("struct RequestContext"));
        assert!(context.contains("std::string request_id;"));
        assert!(context.contains("std::optional<std::string> user_id;"));
        assert!(context.contains("std::map<std::string, std::string> headers;"));
    }

    #[test]
    fn test_error_types_generation() {
        let generator = CppGenerator::with_defaults();
        let errors = generator.generate_error_types("TestService");

        assert!(errors.contains("enum class TestServiceErrorCode"));
        assert!(errors.contains("BadRequest = 400"));
        assert!(errors.contains("struct TestServiceError"));
        assert!(errors.contains("status_code()"));
    }

    #[test]
    fn test_handler_interface_generation() {
        let generator = CppGenerator::with_defaults();
        let operation = Operation {
            operation_id: "getUser".to_string(),
            summary: Some("Get user".to_string()),
            description: Some("Get a user by ID".to_string()),
            method: Some(themis_core::operation::HttpMethod::Get),
            path: Some("/users/{id}".to_string()),
            parameters: vec![],
            request_body: None,
            responses: std::collections::HashMap::new(),
            security: vec![],
            deprecated: false,
            tags: vec![],
            themis_metadata: None,
        };

        let handler = generator.generate_handler_interface("getUser", &operation, "UserService");

        assert!(handler.contains("class GetUserHandler"));
        assert!(handler.contains("virtual ~GetUserHandler() = default;"));
        assert!(handler.contains("std::expected<GetUserResponse, UserServiceError>"));
        assert!(handler.contains("const RequestContext& ctx"));
        assert!(handler.contains("const GetUserRequest& request"));
    }

    #[test]
    fn test_namespace_from_config() {
        let config = GeneratorConfig::default().with_module_name("custom_namespace");
        let generator = CppGenerator::new(config);
        let contract = create_test_contract();

        let namespace = generator.namespace_name(&contract);
        assert_eq!(namespace, "custom_namespace");
    }

    #[test]
    fn test_header_guard_name() {
        let contract = create_test_contract();
        let guard = CppGenerator::header_guard_name(&contract, "types");

        assert!(guard.starts_with("USER_SERVICE_TYPES_"));
        assert!(guard.ends_with("_HPP"));
    }
}
