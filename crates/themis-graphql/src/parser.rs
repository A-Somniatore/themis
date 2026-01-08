//! GraphQL parser for Themis contracts.
//!
//! This module provides the core parsing logic for converting GraphQL SDL
//! definitions into Themis [`Contract`] objects.
//!
//! ## Directive Support
//!
//! The parser extracts Themis-specific metadata from GraphQL directives:
//!
//! - `@themis(service: String!, owner: String!)` on SCHEMA
//! - `@operation(operationId: String!, rateLimitTier: RateLimitTier, ...)` on FIELD_DEFINITION
//! - `@deprecated(reason: String!, sunset: String)` on FIELD_DEFINITION

use std::collections::HashMap;
use std::path::Path;

use graphql_parser::schema::{
    Definition, Directive, Document, Field, InputValue, ObjectType, SchemaDefinition, Type,
    TypeDefinition, Value,
};
use indexmap::IndexMap;
use themis_core::contract::{Contract, ContractFormat, ContractMetadata};
use themis_core::operation::{
    MediaType, Operation, Parameter, ParameterLocation, RequestBody, Response,
    SecurityRequirement, ThemisOperationMetadata,
};
use themis_core::schema::{
    ArraySchema, BooleanSchema, EnumSchema, EnumValue, IntegerSchema, NumberSchema, ObjectSchema,
    OneOfSchema, RefSchema, Schema, StringSchema,
};
use themis_core::Version;

use crate::error::{GraphqlError, Result};

/// GraphQL parser for Themis contracts.
///
/// Parses GraphQL SDL definitions and normalizes them into the unified
/// Themis [`Contract`] model. Extracts Themis-specific metadata from
/// `@themis` and `@operation` directives.
#[derive(Debug, Default)]
pub struct GraphqlParser {
    /// Include paths for resolving imports.
    _include_paths: Vec<String>,
}

/// Metadata extracted from the `@themis` schema directive.
#[derive(Debug, Clone, Default)]
pub struct ThemisSchemaDirective {
    /// Service name from `@themis(service: "...")`
    pub service: Option<String>,
    /// Owner from `@themis(owner: "...")`
    pub owner: Option<String>,
}

/// Metadata extracted from the `@operation` field directive.
#[derive(Debug, Clone, Default)]
pub struct ThemisOperationDirective {
    /// Operation ID from `@operation(operationId: "...")`
    pub operation_id: Option<String>,
    /// Rate limit tier from `@operation(rateLimitTier: ...)`
    pub rate_limit_tier: Option<String>,
    /// Timeout tier from `@operation(timeoutTier: ...)`
    pub timeout_tier: Option<String>,
    /// Whether the operation is idempotent
    pub idempotent: Option<bool>,
    /// Security schemes from `@operation(security: [...])`
    pub security: Vec<String>,
}

impl GraphqlParser {
    /// Creates a new GraphQL parser.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _include_paths: Vec::new(),
        }
    }

    /// Parses a GraphQL SDL string and returns a Themis Contract.
    ///
    /// # Arguments
    ///
    /// * `content` - The GraphQL SDL content
    /// * `service_name` - Service name for metadata
    ///
    /// # Errors
    ///
    /// Returns [`GraphqlError`] if parsing fails.
    pub fn parse(&self, content: &str, service_name: &str) -> Result<Contract> {
        let document: Document<'_, String> =
            graphql_parser::parse_schema(content).map_err(GraphqlError::from)?;

        Self::normalize(&document, service_name)
    }

    /// Parses a GraphQL file from disk.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .graphql file
    /// * `service_name` - Service name for metadata
    ///
    /// # Errors
    ///
    /// Returns [`GraphqlError`] if the file cannot be read or parsed.
    pub fn parse_file(&self, path: &Path, service_name: &str) -> Result<Contract> {
        let content = std::fs::read_to_string(path).map_err(|e| GraphqlError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.parse(&content, service_name)
    }

    /// Normalizes a parsed GraphQL document into a Themis Contract.
    fn normalize(document: &Document<'_, String>, service_name: &str) -> Result<Contract> {
        let mut operations = HashMap::new();
        let mut schemas = IndexMap::new();

        // Extract @themis directive from schema definition if present
        let mut themis_directive = ThemisSchemaDirective::default();
        for definition in &document.definitions {
            if let Definition::SchemaDefinition(schema_def) = definition {
                themis_directive = Self::extract_themis_directive(schema_def);
                break;
            }
        }

        // Process all definitions
        for definition in &document.definitions {
            match definition {
                Definition::TypeDefinition(type_def) => {
                    Self::process_type_definition(type_def, &mut schemas, &mut operations);
                }
                Definition::SchemaDefinition(_)
                | Definition::TypeExtension(_)
                | Definition::DirectiveDefinition(_) => {
                    // Schema definitions processed above
                    // Type extensions and directive definitions handled implicitly
                }
            }
        }

        // Ensure we have at least a Query type
        if operations.is_empty() {
            return Err(GraphqlError::NoQueryType);
        }

        // Use @themis directive values or fall back to provided service_name
        let final_service_name = themis_directive.service.unwrap_or_else(|| service_name.to_string());

        Ok(Contract {
            format: ContractFormat::GraphQl,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: final_service_name,
                description: None,
                owner: themis_directive.owner,
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas,
            security_schemes: HashMap::new(),
        })
    }

    /// Extracts `@themis` directive from a schema definition.
    ///
    /// The directive format is:
    /// ```graphql
    /// schema @themis(service: "users-service", owner: "platform-team") {
    ///   query: Query
    /// }
    /// ```
    fn extract_themis_directive(schema_def: &SchemaDefinition<'_, String>) -> ThemisSchemaDirective {
        let mut result = ThemisSchemaDirective::default();

        for directive in &schema_def.directives {
            if directive.name == "themis" {
                result.service = Self::extract_string_arg(directive, "service");
                result.owner = Self::extract_string_arg(directive, "owner");
                break;
            }
        }

        result
    }

    /// Extracts a string argument from a directive.
    fn extract_string_arg(directive: &Directive<'_, String>, arg_name: &str) -> Option<String> {
        for (name, value) in &directive.arguments {
            if name == arg_name {
                if let Value::String(s) = value {
                    return Some(s.clone());
                }
            }
        }
        None
    }

    /// Extracts a boolean argument from a directive.
    fn extract_bool_arg(directive: &Directive<'_, String>, arg_name: &str) -> Option<bool> {
        for (name, value) in &directive.arguments {
            if name == arg_name {
                if let Value::Boolean(b) = value {
                    return Some(*b);
                }
            }
        }
        None
    }

    /// Extracts an enum value argument from a directive.
    fn extract_enum_arg(directive: &Directive<'_, String>, arg_name: &str) -> Option<String> {
        for (name, value) in &directive.arguments {
            if name == arg_name {
                if let Value::Enum(e) = value {
                    return Some(e.clone());
                }
            }
        }
        None
    }

    /// Extracts a list of enum values from a directive argument.
    fn extract_enum_list_arg(directive: &Directive<'_, String>, arg_name: &str) -> Vec<String> {
        for (name, value) in &directive.arguments {
            if name == arg_name {
                if let Value::List(items) = value {
                    return items
                        .iter()
                        .filter_map(|v| {
                            if let Value::Enum(e) = v {
                                Some(e.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Extracts `@operation` directive from a field definition.
    ///
    /// The directive format is:
    /// ```graphql
    /// user(id: ID!): User @operation(
    ///   operationId: "getUser"
    ///   rateLimitTier: STANDARD
    ///   timeoutTier: FAST
    ///   security: [SPIFFE, BEARER]
    /// )
    /// ```
    fn extract_operation_directive(field: &Field<'_, String>) -> ThemisOperationDirective {
        let mut result = ThemisOperationDirective::default();

        for directive in &field.directives {
            if directive.name == "operation" {
                result.operation_id = Self::extract_string_arg(directive, "operationId");
                result.rate_limit_tier = Self::extract_enum_arg(directive, "rateLimitTier");
                result.timeout_tier = Self::extract_enum_arg(directive, "timeoutTier");
                result.idempotent = Self::extract_bool_arg(directive, "idempotent");
                result.security = Self::extract_enum_list_arg(directive, "security");
                break;
            }
        }

        result
    }

    /// Processes a type definition.
    fn process_type_definition(
        type_def: &TypeDefinition<'_, String>,
        schemas: &mut IndexMap<String, Schema>,
        operations: &mut HashMap<String, Operation>,
    ) {
        match type_def {
            TypeDefinition::Object(obj) => {
                // Check if this is Query, Mutation, or Subscription
                match obj.name.as_str() {
                    "Query" => Self::process_root_type(obj, operations, "query"),
                    "Mutation" => Self::process_root_type(obj, operations, "mutation"),
                    "Subscription" => Self::process_root_type(obj, operations, "subscription"),
                    _ => {
                        // Regular object type - add to schemas
                        let schema = Self::object_type_to_schema(obj);
                        schemas.insert(obj.name.clone(), schema);
                    }
                }
            }
            TypeDefinition::InputObject(input) => {
                let schema = Self::input_object_to_schema(input);
                schemas.insert(input.name.clone(), schema);
            }
            TypeDefinition::Enum(enum_def) => {
                let schema = Self::enum_to_schema(enum_def);
                schemas.insert(enum_def.name.clone(), schema);
            }
            TypeDefinition::Scalar(scalar) => {
                // Custom scalars are treated as strings
                let schema = Schema::String(StringSchema {
                    description: scalar.description.clone(),
                    format: Some(format!("scalar:{}", scalar.name)),
                    ..Default::default()
                });
                schemas.insert(scalar.name.clone(), schema);
            }
            TypeDefinition::Interface(iface) => {
                let schema = Self::interface_to_schema(iface);
                schemas.insert(iface.name.clone(), schema);
            }
            TypeDefinition::Union(union_def) => {
                let schema = Self::union_to_schema(union_def);
                schemas.insert(union_def.name.clone(), schema);
            }
        }
    }

    /// Processes a root type (Query, Mutation, or Subscription).
    fn process_root_type(
        obj: &ObjectType<'_, String>,
        operations: &mut HashMap<String, Operation>,
        operation_type: &str,
    ) {
        for field in &obj.fields {
            let operation = Self::field_to_operation(field, operation_type);
            operations.insert(operation.operation_id.clone(), operation);
        }
    }

    /// Converts a GraphQL field to a Themis Operation.
    fn field_to_operation(field: &Field<'_, String>, operation_type: &str) -> Operation {
        // Extract @operation directive if present
        let op_directive = Self::extract_operation_directive(field);

        // Use operationId from directive or generate from field name
        let operation_id = op_directive
            .operation_id
            .unwrap_or_else(|| format!("{operation_type}_{}", field.name));

        // Convert arguments to parameters
        let parameters: Vec<Parameter> = field
            .arguments
            .iter()
            .map(|arg| Self::input_value_to_parameter(arg))
            .collect();

        // Create request body if there are arguments
        let request_body = if !parameters.is_empty() && operation_type != "query" {
            let mut properties = IndexMap::new();
            let mut required = Vec::new();

            for arg in &field.arguments {
                let schema = Self::graphql_type_to_schema(&arg.value_type);
                if is_non_null_type(&arg.value_type) {
                    required.push(arg.name.clone());
                }
                properties.insert(arg.name.clone(), schema);
            }

            let input_schema = Schema::Object(ObjectSchema {
                description: Some(format!("Input for {}", field.name)),
                properties,
                required,
                additional_properties: None,
                nullable: false,
            });

            let mut content = HashMap::new();
            content.insert("application/json".to_string(), MediaType { schema: input_schema });

            Some(RequestBody {
                description: Some(format!("Request body for {} operation", field.name)),
                required: true,
                content,
            })
        } else {
            None
        };

        // Create response
        let response_schema = Self::graphql_type_to_schema(&field.field_type);
        let mut response_content = HashMap::new();
        response_content.insert("application/json".to_string(), MediaType { schema: response_schema });

        let mut responses = HashMap::new();
        responses.insert(
            "200".to_string(),
            Response {
                description: format!("Successful {operation_type} response"),
                content: response_content,
                headers: HashMap::new(),
            },
        );

        // Check for deprecated directive
        let deprecated = field
            .directives
            .iter()
            .any(|d| d.name == "deprecated");

        // Build ThemisOperationMetadata if directive provided meaningful data
        let themis_metadata = if op_directive.rate_limit_tier.is_some()
            || op_directive.timeout_tier.is_some()
            || op_directive.idempotent.is_some()
        {
            Some(ThemisOperationMetadata {
                rate_limit_tier: op_directive.rate_limit_tier,
                timeout_tier: op_directive.timeout_tier,
                idempotent: op_directive.idempotent,
            })
        } else {
            None
        };

        // Convert security schemes from directive to security requirements
        let security: Vec<SecurityRequirement> = op_directive
            .security
            .iter()
            .map(|s| SecurityRequirement {
                scheme: s.to_lowercase(),
                scopes: Vec::new(),
            })
            .collect();

        Operation {
            operation_id,
            summary: Some(field.name.clone()),
            description: field.description.clone(),
            method: None, // GraphQL doesn't use HTTP methods
            path: Some(format!("/{operation_type}/{}", field.name)),
            parameters: if operation_type == "query" { parameters } else { Vec::new() },
            request_body,
            responses,
            security,
            deprecated,
            tags: vec![operation_type.to_string()],
            themis_metadata,
        }
    }

    /// Converts an input value to a parameter.
    fn input_value_to_parameter( arg: &InputValue<'_, String>) -> Parameter {
        Parameter {
            name: arg.name.clone(),
            location: ParameterLocation::Query,
            description: arg.description.clone(),
            required: is_non_null_type(&arg.value_type),
            deprecated: false,
            schema: Self::graphql_type_to_schema(&arg.value_type),
        }
    }

    /// Converts a GraphQL object type to a Themis Schema.
    fn object_type_to_schema( obj: &ObjectType<'_, String>) -> Schema {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();

        for field in &obj.fields {
            let schema = Self::graphql_type_to_schema(&field.field_type);
            if is_non_null_type(&field.field_type) {
                required.push(field.name.clone());
            }
            properties.insert(field.name.clone(), schema);
        }

        Schema::Object(ObjectSchema {
            description: obj.description.clone(),
            properties,
            required,
            additional_properties: None,
            nullable: false,
        })
    }

    /// Converts a GraphQL input object type to a Themis Schema.
    fn input_object_to_schema(
        input: &graphql_parser::schema::InputObjectType<'_, String>,
    ) -> Schema {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();

        for field in &input.fields {
            let schema = Self::graphql_type_to_schema(&field.value_type);
            if is_non_null_type(&field.value_type) {
                required.push(field.name.clone());
            }
            properties.insert(field.name.clone(), schema);
        }

        Schema::Object(ObjectSchema {
            description: input.description.clone(),
            properties,
            required,
            additional_properties: None,
            nullable: false,
        })
    }

    /// Converts a GraphQL enum type to a Themis Schema.
    fn enum_to_schema(enum_def: &graphql_parser::schema::EnumType<'_, String>) -> Schema {
        let values: Vec<EnumValue> = enum_def
            .values
            .iter()
            .map(|v| EnumValue {
                value: serde_json::Value::String(v.name.clone()),
                description: v.description.clone(),
            })
            .collect();

        Schema::Enum(EnumSchema {
            description: enum_def.description.clone(),
            values,
            nullable: false,
        })
    }

    /// Converts a GraphQL interface type to a Themis Schema.
    fn interface_to_schema(
        iface: &graphql_parser::schema::InterfaceType<'_, String>,
    ) -> Schema {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();

        for field in &iface.fields {
            let schema = Self::graphql_type_to_schema(&field.field_type);
            if is_non_null_type(&field.field_type) {
                required.push(field.name.clone());
            }
            properties.insert(field.name.clone(), schema);
        }

        Schema::Object(ObjectSchema {
            description: iface.description.clone(),
            properties,
            required,
            additional_properties: None,
            nullable: false,
        })
    }

    /// Converts a GraphQL union type to a Themis Schema.
    fn union_to_schema(union_def: &graphql_parser::schema::UnionType<'_, String>) -> Schema {
        let variants: Vec<Schema> = union_def
            .types
            .iter()
            .map(|t| {
                Schema::Ref(RefSchema {
                    reference: format!("#/components/schemas/{t}"),
                })
            })
            .collect();

        Schema::OneOf(OneOfSchema {
            description: union_def.description.clone(),
            schemas: variants,
            discriminator: None,
        })
    }

    /// Converts a GraphQL type to a Themis Schema.
    fn graphql_type_to_schema(gql_type: &Type<'_, String>) -> Schema {
        match gql_type {
            Type::NamedType(name) => Self::named_type_to_schema(name),
            Type::ListType(inner) => Schema::Array(ArraySchema {
                description: None,
                items: Box::new(Self::graphql_type_to_schema(inner)),
                min_items: None,
                max_items: None,
                unique_items: false,
                nullable: true,
            }),
            Type::NonNullType(inner) => {
                let mut schema = Self::graphql_type_to_schema(inner);
                // Mark as non-nullable
                match &mut schema {
                    Schema::String(s) => s.nullable = false,
                    Schema::Integer(i) => i.nullable = false,
                    Schema::Number(n) => n.nullable = false,
                    Schema::Boolean(b) => b.nullable = false,
                    Schema::Array(a) => a.nullable = false,
                    Schema::Object(o) => o.nullable = false,
                    Schema::Enum(e) => e.nullable = false,
                    _ => {}
                }
                schema
            }
        }
    }

    /// Converts a named GraphQL type to a schema.
    fn named_type_to_schema(name: &str) -> Schema {
        match name {
            "String" => Schema::String(StringSchema {
                nullable: true,
                ..Default::default()
            }),
            "Int" => Schema::Integer(IntegerSchema {
                format: Some("int32".to_string()),
                nullable: true,
                ..Default::default()
            }),
            "Float" => Schema::Number(NumberSchema {
                format: Some("double".to_string()),
                nullable: true,
                ..Default::default()
            }),
            "Boolean" => Schema::Boolean(BooleanSchema {
                nullable: true,
                ..Default::default()
            }),
            "ID" => Schema::String(StringSchema {
                format: Some("id".to_string()),
                nullable: true,
                ..Default::default()
            }),
            // Custom types - create a reference
            _ => Schema::Ref(RefSchema {
                reference: format!("#/components/schemas/{name}"),
            }),
        }
    }
}

/// Checks if a GraphQL type is non-null.
const fn is_non_null_type(gql_type: &Type<'_, String>) -> bool {
    matches!(gql_type, Type::NonNullType(_))
}

/// Parses a GraphQL SDL string and returns a Contract.
///
/// # Arguments
///
/// * `content` - The GraphQL SDL content
/// * `service_name` - Service name for metadata
///
/// # Errors
///
/// Returns [`GraphqlError`] if parsing fails.
pub fn parse_graphql(content: &str, service_name: &str) -> Result<Contract> {
    let parser = GraphqlParser::new();
    parser.parse(content, service_name)
}

/// Parses a GraphQL file and returns a Contract.
///
/// # Arguments
///
/// * `path` - Path to the .graphql file
/// * `service_name` - Service name for metadata
///
/// # Errors
///
/// Returns [`GraphqlError`] if the file cannot be read or parsed.
pub fn parse_graphql_file(path: &Path, service_name: &str) -> Result<Contract> {
    let parser = GraphqlParser::new();
    parser.parse_file(path, service_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
        let schema = r#"
            type Query {
                hello: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert_eq!(contract.format, ContractFormat::GraphQl);
        assert_eq!(contract.operations.len(), 1);
        assert!(contract.operations.contains_key("query_hello"));
    }

    #[test]
    fn test_parse_schema_with_types() {
        let schema = r#"
            type Query {
                user(id: ID!): User
                users: [User!]!
            }

            type User {
                id: ID!
                name: String!
                email: String
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert_eq!(contract.operations.len(), 2);
        assert!(contract.operations.contains_key("query_user"));
        assert!(contract.operations.contains_key("query_users"));
        assert!(contract.schemas.contains_key("User"));
    }

    #[test]
    fn test_parse_schema_with_mutation() {
        let schema = r#"
            type Query {
                user(id: ID!): User
            }

            type Mutation {
                createUser(name: String!, email: String!): User
            }

            type User {
                id: ID!
                name: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert_eq!(contract.operations.len(), 2);
        assert!(contract.operations.contains_key("query_user"));
        assert!(contract.operations.contains_key("mutation_createUser"));
    }

    #[test]
    fn test_parse_schema_with_enum() {
        let schema = r#"
            type Query {
                usersByStatus(status: UserStatus!): [User!]!
            }

            enum UserStatus {
                ACTIVE
                INACTIVE
                PENDING
            }

            type User {
                id: ID!
                status: UserStatus!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert!(contract.schemas.contains_key("UserStatus"));

        if let Schema::Enum(enum_schema) = &contract.schemas["UserStatus"] {
            assert_eq!(enum_schema.values.len(), 3);
        } else {
            panic!("Expected enum schema");
        }
    }

    #[test]
    fn test_parse_schema_with_input_type() {
        let schema = r#"
            type Query {
                user(id: ID!): User
            }

            type Mutation {
                createUser(input: CreateUserInput!): User
            }

            input CreateUserInput {
                name: String!
                email: String!
                age: Int
            }

            type User {
                id: ID!
                name: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert!(contract.schemas.contains_key("CreateUserInput"));
    }

    #[test]
    fn test_parse_schema_no_query() {
        let schema = r#"
            type User {
                id: ID!
                name: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphqlError::NoQueryType));
    }

    #[test]
    fn test_parse_schema_with_deprecated() {
        let schema = r#"
            type Query {
                oldUser(id: ID!): User @deprecated(reason: "Use user instead")
                user(id: ID!): User
            }

            type User {
                id: ID!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        let old_op = &contract.operations["query_oldUser"];
        assert!(old_op.deprecated);
    }

    #[test]
    fn test_operation_parameters() {
        let schema = r#"
            type Query {
                user(id: ID!, includeDeleted: Boolean): User
            }

            type User {
                id: ID!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        let op = &contract.operations["query_user"];
        assert_eq!(op.parameters.len(), 2);

        let id_param = op.parameters.iter().find(|p| p.name == "id").unwrap();
        assert!(id_param.required);

        let bool_param = op
            .parameters
            .iter()
            .find(|p| p.name == "includeDeleted")
            .unwrap();
        assert!(!bool_param.required);
    }

    #[test]
    fn test_graphql_types_to_schema() {
        let schema = r#"
            type Query {
                test: TestType
            }

            type TestType {
                stringField: String!
                intField: Int!
                floatField: Float!
                boolField: Boolean!
                idField: ID!
                optionalString: String
                listField: [String!]!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        if let Schema::Object(obj) = &contract.schemas["TestType"] {
            assert!(obj.required.contains(&"stringField".to_string()));
            assert!(obj.required.contains(&"intField".to_string()));
            assert!(!obj.required.contains(&"optionalString".to_string()));

            assert!(matches!(
                obj.properties.get("stringField"),
                Some(Schema::String(_))
            ));
            assert!(matches!(
                obj.properties.get("intField"),
                Some(Schema::Integer(_))
            ));
            assert!(matches!(
                obj.properties.get("floatField"),
                Some(Schema::Number(_))
            ));
            assert!(matches!(
                obj.properties.get("boolField"),
                Some(Schema::Boolean(_))
            ));
            assert!(matches!(
                obj.properties.get("listField"),
                Some(Schema::Array(_))
            ));
        } else {
            panic!("Expected object schema");
        }
    }

    #[test]
    fn test_parse_interface() {
        let schema = r#"
            type Query {
                node(id: ID!): Node
            }

            interface Node {
                id: ID!
            }

            type User implements Node {
                id: ID!
                name: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        assert!(contract.schemas.contains_key("Node"));
        assert!(contract.schemas.contains_key("User"));
    }

    #[test]
    fn test_parse_union() {
        let schema = r#"
            type Query {
                search(query: String!): SearchResult
            }

            union SearchResult = User | Post

            type User {
                id: ID!
            }

            type Post {
                id: ID!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        assert!(contract.schemas.contains_key("SearchResult"));

        if let Schema::OneOf(one_of) = &contract.schemas["SearchResult"] {
            assert_eq!(one_of.schemas.len(), 2);
        } else {
            panic!("Expected OneOf schema");
        }
    }

    #[test]
    fn test_themis_schema_directive() {
        let schema = r#"
            schema @themis(service: "users-service", owner: "platform-team") {
                query: Query
            }

            type Query {
                hello: String!
            }
        "#;

        let result = parse_graphql(schema, "fallback-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        // Service name should come from directive, not fallback
        assert_eq!(contract.metadata.service_name, "users-service");
        assert_eq!(contract.metadata.owner, Some("platform-team".to_string()));
    }

    #[test]
    fn test_themis_directive_fallback() {
        let schema = r#"
            type Query {
                hello: String!
            }
        "#;

        let result = parse_graphql(schema, "fallback-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        // Should use fallback service name when no directive
        assert_eq!(contract.metadata.service_name, "fallback-service");
        assert_eq!(contract.metadata.owner, None);
    }

    #[test]
    fn test_operation_directive_basic() {
        let schema = r#"
            schema @themis(service: "users-service", owner: "platform-team") {
                query: Query
            }

            type Query {
                user(id: ID!): User @operation(
                    operationId: "getUser"
                    rateLimitTier: STANDARD
                    timeoutTier: FAST
                )
            }

            type User {
                id: ID!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        // Operation ID should come from directive
        assert!(contract.operations.contains_key("getUser"));

        let op = &contract.operations["getUser"];
        assert!(op.themis_metadata.is_some());

        let meta = op.themis_metadata.as_ref().unwrap();
        assert_eq!(meta.rate_limit_tier, Some("STANDARD".to_string()));
        assert_eq!(meta.timeout_tier, Some("FAST".to_string()));
    }

    #[test]
    fn test_operation_directive_with_security() {
        let schema = r#"
            schema @themis(service: "users-service", owner: "platform-team") {
                query: Query
            }

            type Query {
                user(id: ID!): User @operation(
                    operationId: "getUser"
                    rateLimitTier: STANDARD
                    timeoutTier: FAST
                    security: [SPIFFE, BEARER]
                )
            }

            type User {
                id: ID!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        let op = &contract.operations["getUser"];

        // Should have security requirements
        assert_eq!(op.security.len(), 2);
        assert_eq!(op.security[0].scheme, "spiffe");
        assert_eq!(op.security[1].scheme, "bearer");
    }

    #[test]
    fn test_operation_directive_with_idempotent() {
        let schema = r#"
            schema @themis(service: "users-service", owner: "platform-team") {
                query: Query
                mutation: Mutation
            }

            type Query {
                user(id: ID!): User
            }

            type Mutation {
                updateUser(id: ID!, name: String!): User @operation(
                    operationId: "updateUser"
                    rateLimitTier: STRICT
                    timeoutTier: STANDARD
                    idempotent: true
                )
            }

            type User {
                id: ID!
                name: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();
        assert!(contract.operations.contains_key("updateUser"));

        let op = &contract.operations["updateUser"];
        assert!(op.themis_metadata.is_some());

        let meta = op.themis_metadata.as_ref().unwrap();
        assert_eq!(meta.idempotent, Some(true));
        assert_eq!(meta.rate_limit_tier, Some("STRICT".to_string()));
    }

    #[test]
    fn test_operation_without_directive_generates_id() {
        let schema = r#"
            type Query {
                user(id: ID!): User
            }

            type User {
                id: ID!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok());

        let contract = result.unwrap();
        // Without @operation directive, ID should be generated from field name
        assert!(contract.operations.contains_key("query_user"));
    }

    #[test]
    fn test_multiple_operations_with_directives() {
        let schema = r#"
            schema @themis(service: "users-service", owner: "platform-team") {
                query: Query
                mutation: Mutation
            }

            type Query {
                user(id: ID!): User @operation(
                    operationId: "getUser"
                    rateLimitTier: STANDARD
                    timeoutTier: FAST
                    security: [BEARER]
                )
                users: [User!]! @operation(
                    operationId: "listUsers"
                    rateLimitTier: HIGH
                    timeoutTier: STANDARD
                )
            }

            type Mutation {
                createUser(name: String!): User @operation(
                    operationId: "createUser"
                    rateLimitTier: STRICT
                    timeoutTier: SLOW
                    idempotent: false
                )
            }

            type User {
                id: ID!
                name: String!
            }
        "#;

        let result = parse_graphql(schema, "test-service");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let contract = result.unwrap();

        // All three operations should exist with correct IDs
        assert!(contract.operations.contains_key("getUser"));
        assert!(contract.operations.contains_key("listUsers"));
        assert!(contract.operations.contains_key("createUser"));

        // Check getUser
        let get_user = &contract.operations["getUser"];
        assert_eq!(get_user.security.len(), 1);
        let meta = get_user.themis_metadata.as_ref().unwrap();
        assert_eq!(meta.rate_limit_tier, Some("STANDARD".to_string()));
        assert_eq!(meta.timeout_tier, Some("FAST".to_string()));

        // Check listUsers
        let list_users = &contract.operations["listUsers"];
        let meta = list_users.themis_metadata.as_ref().unwrap();
        assert_eq!(meta.rate_limit_tier, Some("HIGH".to_string()));

        // Check createUser
        let create_user = &contract.operations["createUser"];
        let meta = create_user.themis_metadata.as_ref().unwrap();
        assert_eq!(meta.idempotent, Some(false));
        assert_eq!(meta.rate_limit_tier, Some("STRICT".to_string()));
        assert_eq!(meta.timeout_tier, Some("SLOW".to_string()));
    }
}
