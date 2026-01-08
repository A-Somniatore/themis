//! Protobuf parser for Themis contracts.
//!
//! This module provides the core parsing logic for converting Protocol Buffer v3
//! definitions into Themis [`Contract`] objects.

use std::collections::HashMap;
use std::path::Path;

use indexmap::IndexMap;
use themis_core::contract::{Contract, ContractFormat, ContractMetadata};
use themis_core::operation::{HttpMethod, MediaType, Operation, RequestBody, Response};
use themis_core::schema::{
    ArraySchema, BooleanSchema, EnumSchema, EnumValue, IntegerSchema, NumberSchema, ObjectSchema,
    RefSchema, Schema, StringSchema,
};
use themis_core::Version;

use crate::error::{ProtobufError, Result};

/// Protobuf parser for Themis contracts.
///
/// Parses Protocol Buffer v3 definitions and normalizes them into the unified
/// Themis [`Contract`] model.
#[derive(Debug, Default)]
pub struct ProtoParser {
    /// Include paths for resolving imports.
    include_paths: Vec<String>,
}

impl ProtoParser {
    /// Creates a new protobuf parser.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_paths: Vec::new(),
        }
    }

    /// Adds an include path for resolving imports.
    pub fn add_include_path(&mut self, path: impl Into<String>) {
        self.include_paths.push(path.into());
    }

    /// Parses a protobuf string and returns a Themis Contract.
    ///
    /// # Arguments
    ///
    /// * `content` - The protobuf file content
    /// * `service_name` - Default service name if not specified in proto
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if parsing fails.
    pub fn parse(&self, content: &str, service_name: &str) -> Result<Contract> {
        // Parse the protobuf content manually
        let parsed = ParsedProto::parse(content);
        Self::normalize(&parsed, service_name)
    }

    /// Parses a protobuf file from disk.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .proto file
    /// * `service_name` - Default service name if not specified in proto
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if the file cannot be read or parsed.
    pub fn parse_file(&self, path: &Path, service_name: &str) -> Result<Contract> {
        let content = std::fs::read_to_string(path).map_err(|e| ProtobufError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.parse(&content, service_name)
    }

    /// Normalizes parsed protobuf into a Themis Contract.
    fn normalize(parsed: &ParsedProto, service_name: &str) -> Result<Contract> {
        let mut operations = HashMap::new();
        let mut schemas = IndexMap::new();

        // Extract version from package name
        let version = parsed
            .package
            .as_ref()
            .and_then(|p| Self::parse_version_from_package(p))
            .unwrap_or_else(|| Version::new(1, 0, 0));

        // Determine service name
        let actual_service_name = parsed
            .services
            .first()
            .map_or_else(|| service_name.to_string(), |s| Self::to_kebab_case(&s.name));

        // Convert messages to schemas
        for message in &parsed.messages {
            let schema = Self::message_to_schema(message);
            schemas.insert(message.name.clone(), schema);
        }

        // Convert enums to schemas
        for enum_def in &parsed.enums {
            let schema = Self::enum_to_schema(enum_def);
            schemas.insert(enum_def.name.clone(), schema);
        }

        // Convert services to operations
        for service in &parsed.services {
            for method in &service.methods {
                let operation = Self::method_to_operation(method, &service.name);
                operations.insert(operation.operation_id.clone(), operation);
            }
        }

        // If no services found, return error
        if operations.is_empty() {
            return Err(ProtobufError::NoServiceFound);
        }

        Ok(Contract {
            format: ContractFormat::Protobuf,
            version,
            metadata: ContractMetadata {
                service_name: actual_service_name,
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas,
            security_schemes: HashMap::new(),
        })
    }

    /// Parses a version from a package string.
    fn parse_version_from_package(package: &str) -> Option<Version> {
        for part in package.split('.') {
            if part.starts_with('v') && part.len() > 1 {
                let version_part = &part[1..];
                if let Some(idx) = version_part.find(|c: char| !c.is_ascii_digit()) {
                    let major_str = &version_part[..idx];
                    if let Ok(major) = major_str.parse::<u32>() {
                        let suffix = &version_part[idx..];
                        let minor = if suffix.starts_with("beta") || suffix.starts_with("alpha") {
                            let num_start = if suffix.starts_with("beta") { 4 } else { 5 };
                            suffix[num_start..].parse::<u32>().unwrap_or(0)
                        } else {
                            0
                        };
                        return Some(Version::new(major, minor, 0));
                    }
                } else if let Ok(major) = version_part.parse::<u32>() {
                    return Some(Version::new(major, 0, 0));
                }
            }
        }
        None
    }

    /// Converts a service name to kebab-case.
    fn to_kebab_case(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
            .strip_suffix("-service")
            .map_or_else(|| result.clone(), ToString::to_string)
    }

    /// Converts a protobuf method to a Themis Operation.
    fn method_to_operation(method: &ProtoMethod, service_name: &str) -> Operation {
        let operation_id = Self::to_camel_case(&method.name);

        // Create request body content
        let mut request_content = HashMap::new();
        request_content.insert(
            "application/grpc".to_string(),
            MediaType {
                schema: Schema::Ref(RefSchema {
                    reference: format!("#/components/schemas/{}", method.input_type),
                }),
            },
        );

        let request_body = Some(RequestBody {
            description: Some(format!("Request message for {}", method.name)),
            required: true,
            content: request_content,
        });

        // Create response content
        let mut response_content = HashMap::new();
        response_content.insert(
            "application/grpc".to_string(),
            MediaType {
                schema: Schema::Ref(RefSchema {
                    reference: format!("#/components/schemas/{}", method.output_type),
                }),
            },
        );

        let mut responses = HashMap::new();
        responses.insert(
            "200".to_string(),
            Response {
                description: format!("Successful response for {}", method.name),
                content: response_content,
                headers: HashMap::new(),
            },
        );

        Operation {
            operation_id,
            summary: Some(format!("{service_name}.{}", method.name)),
            description: method.comment.clone(),
            method: Some(HttpMethod::Post),
            path: Some(format!("/{service_name}/{}", method.name)),
            parameters: Vec::new(),
            request_body,
            responses,
            security: Vec::new(),
            deprecated: false,
            tags: vec![service_name.to_string()],
            themis_metadata: None,
        }
    }

    /// Converts a string to `camelCase`.
    fn to_camel_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for (i, c) in s.chars().enumerate() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else if i == 0 {
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Converts a protobuf message to a Themis Schema.
    fn message_to_schema(message: &ProtoMessage) -> Schema {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();

        for field in &message.fields {
            let field_schema = Self::field_type_to_schema(&field.field_type, field.repeated);

            if !field.optional && !field.repeated {
                required.push(field.name.clone());
            }

            properties.insert(field.name.clone(), field_schema);
        }

        Schema::Object(ObjectSchema {
            description: message.comment.clone(),
            properties,
            required,
            additional_properties: None,
            nullable: false,
        })
    }

    /// Converts a protobuf field type to a Themis Schema.
    fn field_type_to_schema(field_type: &str, repeated: bool) -> Schema {
        let base_schema = match field_type {
            "string" => Schema::String(StringSchema::default()),
            "bytes" => Schema::String(StringSchema {
                format: Some("byte".to_string()),
                ..Default::default()
            }),
            "bool" => Schema::Boolean(BooleanSchema::default()),
            "int32" | "sint32" | "sfixed32" => Schema::Integer(IntegerSchema {
                format: Some("int32".to_string()),
                ..Default::default()
            }),
            "int64" | "sint64" | "sfixed64" => Schema::Integer(IntegerSchema {
                format: Some("int64".to_string()),
                ..Default::default()
            }),
            "uint32" | "fixed32" => Schema::Integer(IntegerSchema {
                format: Some("uint32".to_string()),
                minimum: Some(0),
                ..Default::default()
            }),
            "uint64" | "fixed64" => Schema::Integer(IntegerSchema {
                format: Some("uint64".to_string()),
                minimum: Some(0),
                ..Default::default()
            }),
            "float" => Schema::Number(NumberSchema {
                format: Some("float".to_string()),
                ..Default::default()
            }),
            "double" => Schema::Number(NumberSchema {
                format: Some("double".to_string()),
                ..Default::default()
            }),
            // Well-known types
            "google.protobuf.Timestamp" => Schema::String(StringSchema {
                format: Some("date-time".to_string()),
                ..Default::default()
            }),
            "google.protobuf.Duration" => Schema::String(StringSchema {
                format: Some("duration".to_string()),
                ..Default::default()
            }),
            // Custom message types - create a reference
            _ => Schema::Ref(RefSchema {
                reference: format!("#/components/schemas/{field_type}"),
            }),
        };

        if repeated {
            Schema::Array(ArraySchema {
                description: None,
                items: Box::new(base_schema),
                min_items: None,
                max_items: None,
                unique_items: false,
                nullable: false,
            })
        } else {
            base_schema
        }
    }

    /// Converts a protobuf enum to a Themis Schema.
    fn enum_to_schema(enum_def: &ProtoEnum) -> Schema {
        let values: Vec<EnumValue> = enum_def
            .values
            .iter()
            .map(|(name, _number)| EnumValue {
                value: serde_json::Value::String(name.clone()),
                description: None,
            })
            .collect();

        Schema::Enum(EnumSchema {
            description: enum_def.comment.clone(),
            values,
            nullable: false,
        })
    }
}

/// Parsed protobuf representation.
#[derive(Debug, Default)]
struct ParsedProto {
    /// Package name (e.g., "myservice.v1")
    package: Option<String>,
    /// Service definitions
    services: Vec<ProtoService>,
    /// Message definitions
    messages: Vec<ProtoMessage>,
    /// Enum definitions
    enums: Vec<ProtoEnum>,
}

/// Mutable parsing state.
#[derive(Default)]
struct ParseState {
    current_comment: Option<String>,
    in_service: bool,
    in_message: bool,
    in_enum: bool,
    brace_depth: usize,
    current_service: Option<ProtoService>,
    current_message: Option<ProtoMessage>,
    current_enum: Option<ProtoEnum>,
}

impl ParsedProto {
    /// Parses protobuf content.
    fn parse(content: &str) -> Self {
        let mut parsed = Self::default();
        let mut state = ParseState::default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            Self::process_line(trimmed, &mut parsed, &mut state);
        }

        parsed
    }

    /// Processes a single line of protobuf content.
    fn process_line(trimmed: &str, parsed: &mut Self, state: &mut ParseState) {
        // Collect comments
        if trimmed.starts_with("//") {
            let comment_text = trimmed.trim_start_matches("//").trim();
            state.current_comment = Some(comment_text.to_string());
            return;
        }

        // Parse package
        if trimmed.starts_with("package ") {
            Self::parse_package(trimmed, parsed);
            return;
        }

        // Track braces
        state.brace_depth += trimmed.matches('{').count();
        state.brace_depth = state.brace_depth.saturating_sub(trimmed.matches('}').count());

        // Service parsing
        if Self::try_parse_service(trimmed, parsed, state) {
            return;
        }

        // Message parsing
        if Self::try_parse_message(trimmed, parsed, state) {
            return;
        }

        // Enum parsing
        if Self::try_parse_enum(trimmed, parsed, state) {
            return;
        }

        // Clear comment if not used
        state.current_comment = None;
    }

    /// Parses the package declaration.
    fn parse_package(trimmed: &str, parsed: &mut Self) {
        let pkg = trimmed
            .trim_start_matches("package ")
            .trim_end_matches(';')
            .trim();
        parsed.package = Some(pkg.to_string());
    }

    /// Attempts to parse service-related lines.
    fn try_parse_service(trimmed: &str, parsed: &mut Self, state: &mut ParseState) -> bool {
        // Parse service start
        if trimmed.starts_with("service ") && trimmed.contains('{') {
            let name = trimmed
                .trim_start_matches("service ")
                .split('{')
                .next()
                .unwrap_or("")
                .trim();
            state.current_service = Some(ProtoService {
                name: name.to_string(),
                methods: Vec::new(),
                comment: state.current_comment.take(),
            });
            state.in_service = true;
            return true;
        }

        // Parse service end
        if state.in_service && state.brace_depth == 0 && trimmed.contains('}') {
            if let Some(service) = state.current_service.take() {
                parsed.services.push(service);
            }
            state.in_service = false;
            return true;
        }

        // Parse RPC method
        if state.in_service && trimmed.starts_with("rpc ") {
            if let Some(method) = Self::parse_rpc_line(trimmed, state.current_comment.take()) {
                if let Some(ref mut service) = state.current_service {
                    service.methods.push(method);
                }
            }
            return true;
        }

        false
    }

    /// Attempts to parse message-related lines.
    fn try_parse_message(trimmed: &str, parsed: &mut Self, state: &mut ParseState) -> bool {
        // Parse message start
        if trimmed.starts_with("message ") && trimmed.contains('{') {
            let name = trimmed
                .trim_start_matches("message ")
                .split('{')
                .next()
                .unwrap_or("")
                .trim();
            state.current_message = Some(ProtoMessage {
                name: name.to_string(),
                fields: Vec::new(),
                comment: state.current_comment.take(),
            });
            state.in_message = true;
            return true;
        }

        // Parse message end
        if state.in_message && state.brace_depth == 0 && trimmed.contains('}') {
            if let Some(message) = state.current_message.take() {
                parsed.messages.push(message);
            }
            state.in_message = false;
            return true;
        }

        // Parse message field
        if state.in_message && !trimmed.starts_with("//") && trimmed.contains('=') {
            if let Some(field) = Self::parse_field_line(trimmed, state.current_comment.take()) {
                if let Some(ref mut message) = state.current_message {
                    message.fields.push(field);
                }
            }
            return true;
        }

        false
    }

    /// Attempts to parse enum-related lines.
    fn try_parse_enum(trimmed: &str, parsed: &mut Self, state: &mut ParseState) -> bool {
        // Parse enum start
        if trimmed.starts_with("enum ") && trimmed.contains('{') {
            let name = trimmed
                .trim_start_matches("enum ")
                .split('{')
                .next()
                .unwrap_or("")
                .trim();
            state.current_enum = Some(ProtoEnum {
                name: name.to_string(),
                values: Vec::new(),
                comment: state.current_comment.take(),
            });
            state.in_enum = true;
            return true;
        }

        // Parse enum end
        if state.in_enum && state.brace_depth == 0 && trimmed.contains('}') {
            if let Some(enum_def) = state.current_enum.take() {
                parsed.enums.push(enum_def);
            }
            state.in_enum = false;
            return true;
        }

        // Parse enum value
        if state.in_enum && trimmed.contains('=') && !trimmed.starts_with("option") {
            if let Some((name, number)) = Self::parse_enum_value_line(trimmed) {
                if let Some(ref mut enum_def) = state.current_enum {
                    enum_def.values.push((name, number));
                }
            }
            return true;
        }

        false
    }

    /// Parses an RPC line.
    fn parse_rpc_line(line: &str, comment: Option<String>) -> Option<ProtoMethod> {
        // rpc GetUser(GetUserRequest) returns (GetUserResponse);
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let name = parts[1].split('(').next()?.to_string();
        let input_type = parts[1]
            .split('(')
            .nth(1)?
            .trim_end_matches(')')
            .to_string();

        // Find the returns clause
        let returns_idx = parts.iter().position(|&p| p == "returns")?;
        let output_part = parts.get(returns_idx + 1)?;
        let output_type = output_part
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim_end_matches(';')
            .to_string();

        Some(ProtoMethod {
            name,
            input_type,
            output_type,
            comment,
        })
    }

    /// Parses a field line.
    fn parse_field_line(line: &str, comment: Option<String>) -> Option<ProtoField> {
        let trimmed = line.trim().trim_end_matches(';');

        // Handle map fields
        if trimmed.starts_with("map<") {
            let map_end = trimmed.find('>')?;
            let rest = &trimmed[map_end + 1..].trim();
            let parts: Vec<&str> = rest.split('=').collect();
            let name = parts.first()?.trim().to_string();

            return Some(ProtoField {
                name,
                field_type: "map".to_string(),
                repeated: false,
                optional: false,
                comment,
            });
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let mut idx = 0;
        let mut repeated = false;
        let mut optional = false;

        // Check for modifiers
        if parts[idx] == "repeated" {
            repeated = true;
            idx += 1;
        } else if parts[idx] == "optional" {
            optional = true;
            idx += 1;
        }

        if idx >= parts.len() {
            return None;
        }

        let field_type = parts[idx].to_string();
        idx += 1;

        if idx >= parts.len() {
            return None;
        }

        let name = parts[idx].to_string();

        Some(ProtoField {
            name,
            field_type,
            repeated,
            optional,
            comment,
        })
    }

    /// Parses an enum value line.
    fn parse_enum_value_line(line: &str) -> Option<(String, i32)> {
        let trimmed = line.trim().trim_end_matches(';');
        let parts: Vec<&str> = trimmed.split('=').collect();
        if parts.len() != 2 {
            return None;
        }

        let name = parts[0].trim().to_string();
        let number = parts[1].trim().parse::<i32>().ok()?;

        Some((name, number))
    }
}

/// Protobuf service definition.
#[derive(Debug)]
struct ProtoService {
    name: String,
    methods: Vec<ProtoMethod>,
    #[allow(dead_code)]
    comment: Option<String>,
}

/// Protobuf RPC method.
#[derive(Debug)]
struct ProtoMethod {
    name: String,
    input_type: String,
    output_type: String,
    comment: Option<String>,
}

/// Protobuf message definition.
#[derive(Debug)]
struct ProtoMessage {
    name: String,
    fields: Vec<ProtoField>,
    comment: Option<String>,
}

/// Protobuf field.
#[derive(Debug)]
struct ProtoField {
    name: String,
    field_type: String,
    repeated: bool,
    optional: bool,
    #[allow(dead_code)]
    comment: Option<String>,
}

/// Protobuf enum definition.
#[derive(Debug)]
struct ProtoEnum {
    name: String,
    values: Vec<(String, i32)>,
    comment: Option<String>,
}

/// Parses a protobuf string content and returns a Contract.
///
/// # Arguments
///
/// * `content` - The protobuf file content
/// * `service_name` - Default service name
///
/// # Errors
///
/// Returns [`ProtobufError`] if parsing fails.
pub fn parse_proto(content: &str, service_name: &str) -> Result<Contract> {
    let parser = ProtoParser::new();
    parser.parse(content, service_name)
}

/// Parses a protobuf file and returns a Contract.
///
/// # Arguments
///
/// * `path` - Path to the .proto file
/// * `service_name` - Default service name
///
/// # Errors
///
/// Returns [`ProtobufError`] if the file cannot be read or parsed.
pub fn parse_proto_file(path: &Path, service_name: &str) -> Result<Contract> {
    let parser = ProtoParser::new();
    parser.parse_file(path, service_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_from_package() {
        assert_eq!(
            ProtoParser::parse_version_from_package("myservice.v1"),
            Some(Version::new(1, 0, 0))
        );
        assert_eq!(
            ProtoParser::parse_version_from_package("myservice.v2"),
            Some(Version::new(2, 0, 0))
        );
        assert_eq!(
            ProtoParser::parse_version_from_package("com.example.api.v3"),
            Some(Version::new(3, 0, 0))
        );
        assert_eq!(ProtoParser::parse_version_from_package("myservice"), None);
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(ProtoParser::to_kebab_case("UsersService"), "users");
        assert_eq!(ProtoParser::to_kebab_case("OrdersService"), "orders");
        assert_eq!(
            ProtoParser::to_kebab_case("MyLongServiceName"),
            "my-long-service-name"
        );
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(ProtoParser::to_camel_case("GetUser"), "getUser");
        assert_eq!(ProtoParser::to_camel_case("get_user"), "getUser");
        assert_eq!(ProtoParser::to_camel_case("ListItems"), "listItems");
    }

    #[test]
    fn test_parse_simple_proto() {
        let proto = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc GetItem(GetItemRequest) returns (GetItemResponse);
    rpc CreateItem(CreateItemRequest) returns (CreateItemResponse);
}

message GetItemRequest {
    string id = 1;
}

message GetItemResponse {
    string id = 1;
    string name = 2;
}

message CreateItemRequest {
    string name = 1;
}

message CreateItemResponse {
    string id = 1;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert_eq!(contract.format, ContractFormat::Protobuf);
        assert_eq!(contract.version, Version::new(1, 0, 0));
        assert_eq!(contract.operations.len(), 2);
        assert!(contract.operations.contains_key("getItem"));
        assert!(contract.operations.contains_key("createItem"));
    }

    #[test]
    fn test_parse_proto_with_enums() {
        let proto = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc GetItem(GetItemRequest) returns (GetItemResponse);
}

enum Status {
    UNKNOWN = 0;
    ACTIVE = 1;
    INACTIVE = 2;
}

message GetItemRequest {
    string id = 1;
}

message GetItemResponse {
    string id = 1;
    Status status = 2;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert!(contract.schemas.contains_key("Status"));

        if let Schema::Enum(enum_schema) = &contract.schemas["Status"] {
            assert_eq!(enum_schema.values.len(), 3);
        } else {
            panic!("Expected enum schema");
        }
    }

    #[test]
    fn test_parse_proto_with_nested_messages() {
        let proto = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
}

message GetUserRequest {
    string user_id = 1;
}

message GetUserResponse {
    User user = 1;
}

message User {
    string id = 1;
    string email = 2;
    Address address = 3;
}

message Address {
    string street = 1;
    string city = 2;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert!(contract.schemas.contains_key("User"));
        assert!(contract.schemas.contains_key("Address"));
    }

    #[test]
    fn test_parse_proto_no_service() {
        let proto = r#"
syntax = "proto3";

package test.v1;

message Request {
    string id = 1;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtobufError::NoServiceFound));
    }

    #[test]
    fn test_parse_proto_with_repeated_fields() {
        let proto = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc ListItems(ListItemsRequest) returns (ListItemsResponse);
}

message ListItemsRequest {
    int32 page_size = 1;
}

message ListItemsResponse {
    repeated Item items = 1;
}

message Item {
    string id = 1;
    repeated string tags = 2;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        if let Schema::Object(obj) = &contract.schemas["ListItemsResponse"] {
            if let Some(Schema::Array(_)) = obj.properties.get("items") {
                // Expected
            } else {
                panic!("Expected items to be an array schema");
            }
        } else {
            panic!("Expected object schema");
        }
    }

    #[test]
    fn test_parse_proto_with_optional_fields() {
        let proto = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc GetItem(GetItemRequest) returns (GetItemResponse);
}

message GetItemRequest {
    string id = 1;
}

message GetItemResponse {
    string id = 1;
    optional string description = 2;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        if let Schema::Object(obj) = &contract.schemas["GetItemResponse"] {
            assert!(!obj.required.contains(&"description".to_string()));
            assert!(obj.required.contains(&"id".to_string()));
        } else {
            panic!("Expected object schema");
        }
    }

    #[test]
    fn test_operation_generated_correctly() {
        let proto = r#"
syntax = "proto3";

package users.v1;

service UsersService {
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
}

message GetUserRequest {
    string user_id = 1;
}

message GetUserResponse {
    string id = 1;
    string name = 2;
}
"#;

        let result = parse_proto(proto, "users-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        let op = &contract.operations["getUser"];

        assert_eq!(op.operation_id, "getUser");
        assert_eq!(op.method, Some(HttpMethod::Post));
        assert!(op.path.as_ref().unwrap().contains("GetUser"));
        assert!(op.request_body.is_some());
        assert!(op.responses.contains_key("200"));
    }

    #[test]
    fn test_field_types() {
        let proto = r#"
syntax = "proto3";

package test.v1;

service TestService {
    rpc Test(TestRequest) returns (TestResponse);
}

message TestRequest {
    string str_field = 1;
    int32 int32_field = 2;
    int64 int64_field = 3;
    uint32 uint32_field = 4;
    uint64 uint64_field = 5;
    float float_field = 6;
    double double_field = 7;
    bool bool_field = 8;
    bytes bytes_field = 9;
}

message TestResponse {
    string result = 1;
}
"#;

        let result = parse_proto(proto, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        if let Schema::Object(obj) = &contract.schemas["TestRequest"] {
            assert!(matches!(
                obj.properties.get("str_field"),
                Some(Schema::String(_))
            ));
            assert!(matches!(
                obj.properties.get("int32_field"),
                Some(Schema::Integer(_))
            ));
            assert!(matches!(
                obj.properties.get("float_field"),
                Some(Schema::Number(_))
            ));
            assert!(matches!(
                obj.properties.get("bool_field"),
                Some(Schema::Boolean(_))
            ));
        } else {
            panic!("Expected object schema");
        }
    }
}
