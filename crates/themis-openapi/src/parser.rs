//! OpenAPI 3.1 parser.
//!
//! Parses OpenAPI specifications from YAML or JSON into the internal Themis model.

use indexmap::IndexMap;
use openapiv3::{
    OpenAPI, Operation as OpenApiOperation, Parameter as OpenApiParameter, PathItem, ReferenceOr,
    Schema as OpenApiSchema, SchemaKind, SecurityScheme as OpenApiSecurityScheme, StatusCode,
    Type as OpenApiType,
};
use std::collections::HashMap;
use std::path::Path;
use themis_core::{
    contract::{
        ApiKeyLocation, Contract, ContractFormat, ContractMetadata, SecurityScheme,
        SecuritySchemeType,
    },
    operation::{
        Header, HttpMethod, MediaType, Operation, Parameter, ParameterLocation, RequestBody,
        Response, SecurityRequirement, ThemisOperationMetadata,
    },
    schema::{
        AllOfSchema, AnyOfSchema, ArraySchema, BooleanSchema, EnumSchema, EnumValue, IntegerSchema,
        NumberSchema, ObjectSchema, OneOfSchema, RefSchema, Schema, StringSchema,
    },
    version::Version,
    ThemisError, ThemisResult,
};

/// Parses an OpenAPI 3.1 specification from a string.
///
/// # Arguments
///
/// * `content` - The OpenAPI specification as YAML or JSON string
///
/// # Returns
///
/// A normalized [`Contract`] representation of the OpenAPI spec.
///
/// # Errors
///
/// Returns [`ThemisError`] if:
/// - The content is not valid YAML/JSON
/// - The content is not a valid OpenAPI 3.0/3.1 specification
/// - Required fields are missing (e.g., operationId)
///
/// # Example
///
/// ```rust,ignore
/// use themis_openapi::parse_openapi;
///
/// let yaml = r#"
/// openapi: "3.1.0"
/// info:
///   title: Users API
///   version: "1.0.0"
/// paths:
///   /users:
///     get:
///       operationId: listUsers
///       responses:
///         "200":
///           description: Success
/// "#;
///
/// let contract = parse_openapi(yaml)?;
/// assert_eq!(contract.operation_count(), 1);
/// ```
pub fn parse_openapi(content: &str) -> ThemisResult<Contract> {
    // Try YAML first, then JSON
    let openapi: OpenAPI = serde_yaml::from_str(content).or_else(|yaml_err| {
        serde_json::from_str(content).map_err(|json_err| ThemisError::YamlParse {
            path: "<string>".into(),
            message: format!("Failed to parse as YAML ({yaml_err}) or JSON ({json_err})"),
        })
    })?;

    convert_openapi_to_contract(&openapi)
}

/// Parses an OpenAPI 3.1 specification from a file.
///
/// # Arguments
///
/// * `path` - Path to the OpenAPI specification file
///
/// # Returns
///
/// A normalized [`Contract`] representation of the OpenAPI spec.
///
/// # Errors
///
/// Returns [`ThemisError`] if:
/// - The file cannot be read
/// - The content is not valid YAML/JSON
/// - The content is not a valid OpenAPI 3.0/3.1 specification
pub fn parse_openapi_file(path: &Path) -> ThemisResult<Contract> {
    let content = std::fs::read_to_string(path).map_err(|e| ThemisError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_openapi(&content)
}

/// Converts an `openapiv3::OpenAPI` structure to a Themis `Contract`.
fn convert_openapi_to_contract(openapi: &OpenAPI) -> ThemisResult<Contract> {
    // Parse version from info.version
    let version = parse_api_version(&openapi.info.version)?;

    // Create contract metadata
    let metadata = ContractMetadata {
        service_name: openapi.info.title.clone(),
        description: openapi.info.description.clone(),
        owner: extract_owner_from_extensions(openapi),
        repository: extract_repository_from_extensions(openapi),
        documentation_url: openapi.external_docs.as_ref().map(|d| d.url.clone()),
    };

    let mut contract = Contract {
        format: ContractFormat::OpenApi,
        version,
        metadata,
        operations: HashMap::new(),
        schemas: IndexMap::new(),
        security_schemes: HashMap::new(),
    };

    // Convert security schemes
    if let Some(components) = &openapi.components {
        for (name, scheme_ref) in &components.security_schemes {
            if let ReferenceOr::Item(scheme) = scheme_ref {
                contract
                    .security_schemes
                    .insert(name.clone(), convert_security_scheme(scheme));
            }
        }

        // Convert schemas
        for (name, schema_ref) in &components.schemas {
            if let ReferenceOr::Item(schema) = schema_ref {
                contract
                    .schemas
                    .insert(name.clone(), convert_schema(schema));
            }
        }
    }

    // Convert paths to operations
    for (path, path_item_ref) in &openapi.paths.paths {
        if let ReferenceOr::Item(path_item) = path_item_ref {
            convert_path_item_to_operations(&mut contract, path, path_item)?;
        }
    }

    Ok(contract)
}

/// Parses a version string into a `Version`.
fn parse_api_version(version_str: &str) -> ThemisResult<Version> {
    // Strip 'v' prefix if present (common in APIs)
    let clean_version = version_str.strip_prefix('v').unwrap_or(version_str);
    clean_version
        .parse::<Version>()
        .map_err(|_| ThemisError::InvalidVersion {
            version: version_str.to_string(),
            reason: "Expected semantic version (e.g., '1.0.0' or 'v1.2.3')".to_string(),
        })
}

/// Extracts owner from x-themis-owner extension or contact info.
fn extract_owner_from_extensions(openapi: &OpenAPI) -> Option<String> {
    openapi
        .info
        .extensions
        .get("x-themis-owner")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| openapi.info.contact.as_ref().and_then(|c| c.name.clone()))
}

/// Extracts repository URL from extensions.
fn extract_repository_from_extensions(openapi: &OpenAPI) -> Option<String> {
    openapi
        .info
        .extensions
        .get("x-themis-repository")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Converts a path item to one or more operations.
fn convert_path_item_to_operations(
    contract: &mut Contract,
    path: &str,
    path_item: &PathItem,
) -> ThemisResult<()> {
    // Common parameters for all operations on this path
    let common_params: Vec<Parameter> = path_item
        .parameters
        .iter()
        .filter_map(|p| match p {
            ReferenceOr::Item(param) => Some(convert_parameter(param)),
            ReferenceOr::Reference { .. } => None, // TODO: Resolve refs
        })
        .collect();

    // Helper to insert operation and check for duplicates
    let mut insert_operation = |operation: Operation| -> ThemisResult<()> {
        let op_id = operation.operation_id.clone();
        if contract.operations.contains_key(&op_id) {
            return Err(ThemisError::SchemaValidation {
                message: format!("Duplicate operationId '{op_id}' found"),
            });
        }
        contract.operations.insert(op_id, operation);
        Ok(())
    };

    // Convert each HTTP method
    if let Some(op) = &path_item.get {
        let operation = convert_operation(op, path, HttpMethod::Get, &common_params)?;
        insert_operation(operation)?;
    }
    if let Some(op) = &path_item.post {
        let operation = convert_operation(op, path, HttpMethod::Post, &common_params)?;
        insert_operation(operation)?;
    }
    if let Some(op) = &path_item.put {
        let operation = convert_operation(op, path, HttpMethod::Put, &common_params)?;
        insert_operation(operation)?;
    }
    if let Some(op) = &path_item.patch {
        let operation = convert_operation(op, path, HttpMethod::Patch, &common_params)?;
        insert_operation(operation)?;
    }
    if let Some(op) = &path_item.delete {
        let operation = convert_operation(op, path, HttpMethod::Delete, &common_params)?;
        insert_operation(operation)?;
    }
    if let Some(op) = &path_item.head {
        let operation = convert_operation(op, path, HttpMethod::Head, &common_params)?;
        insert_operation(operation)?;
    }
    if let Some(op) = &path_item.options {
        let operation = convert_operation(op, path, HttpMethod::Options, &common_params)?;
        insert_operation(operation)?;
    }

    Ok(())
}

/// Converts an OpenAPI operation to a Themis operation.
fn convert_operation(
    op: &OpenApiOperation,
    path: &str,
    method: HttpMethod,
    common_params: &[Parameter],
) -> ThemisResult<Operation> {
    // operationId is required in Themis
    let operation_id = op
        .operation_id
        .clone()
        .ok_or_else(|| ThemisError::MissingField {
            field: "operationId".to_string(),
            context: format!("{method} {path}"),
        })?;

    // Convert parameters
    let mut parameters: Vec<Parameter> = common_params.to_vec();
    for param_ref in &op.parameters {
        if let ReferenceOr::Item(param) = param_ref {
            parameters.push(convert_parameter(param));
        }
    }

    // Convert request body
    let request_body = op.request_body.as_ref().and_then(|rb| match rb {
        ReferenceOr::Item(body) => Some(convert_request_body(body)),
        ReferenceOr::Reference { .. } => None, // TODO: Resolve refs
    });

    // Convert responses
    let mut responses = HashMap::new();
    for (status, response_ref) in &op.responses.responses {
        if let ReferenceOr::Item(response) = response_ref {
            let status_str = match status {
                StatusCode::Code(code) => code.to_string(),
                StatusCode::Range(r) => format!("{r}XX"),
            };
            responses.insert(status_str, convert_response(response));
        }
    }
    if let Some(ReferenceOr::Item(response)) = &op.responses.default {
        responses.insert("default".to_string(), convert_response(response));
    }

    // Convert security requirements
    let security: Vec<SecurityRequirement> = op
        .security
        .as_ref()
        .map(|reqs| reqs.iter().flat_map(convert_security_requirement).collect())
        .unwrap_or_default();

    // Extract Themis metadata from extensions
    let themis_metadata = extract_themis_metadata(&op.extensions);

    Ok(Operation {
        operation_id,
        summary: op.summary.clone(),
        description: op.description.clone(),
        method: Some(method),
        path: Some(path.to_string()),
        parameters,
        request_body,
        responses,
        security,
        deprecated: op.deprecated,
        tags: op.tags.clone(),
        themis_metadata,
    })
}

/// Converts an OpenAPI parameter to a Themis parameter.
fn convert_parameter(param: &OpenApiParameter) -> Parameter {
    let data = match param {
        OpenApiParameter::Query { parameter_data, .. }
        | OpenApiParameter::Header { parameter_data, .. }
        | OpenApiParameter::Path { parameter_data, .. }
        | OpenApiParameter::Cookie { parameter_data, .. } => parameter_data,
    };

    let location = match param {
        OpenApiParameter::Query { .. } => ParameterLocation::Query,
        OpenApiParameter::Header { .. } => ParameterLocation::Header,
        OpenApiParameter::Path { .. } => ParameterLocation::Path,
        OpenApiParameter::Cookie { .. } => ParameterLocation::Cookie,
    };

    let schema = match &data.format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
            ReferenceOr::Item(schema) => convert_schema(schema),
            ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                reference: reference.clone(),
            }),
        },
        openapiv3::ParameterSchemaOrContent::Content(_) => {
            // Default to string for content-type parameters
            Schema::String(StringSchema::default())
        }
    };

    Parameter {
        name: data.name.clone(),
        location,
        description: data.description.clone(),
        required: data.required,
        deprecated: data.deprecated.unwrap_or(false),
        schema,
    }
}

/// Converts an OpenAPI request body to a Themis request body.
fn convert_request_body(body: &openapiv3::RequestBody) -> RequestBody {
    let mut content = HashMap::new();

    for (media_type_name, media_type) in &body.content {
        if let Some(schema_ref) = &media_type.schema {
            let schema = match schema_ref {
                ReferenceOr::Item(schema) => convert_schema(schema),
                ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                    reference: reference.clone(),
                }),
            };
            content.insert(media_type_name.clone(), MediaType { schema });
        }
    }

    RequestBody {
        description: body.description.clone(),
        required: body.required,
        content,
    }
}

/// Converts an OpenAPI response to a Themis response.
fn convert_response(response: &openapiv3::Response) -> Response {
    let mut content = HashMap::new();

    for (media_type_name, media_type) in &response.content {
        if let Some(schema_ref) = &media_type.schema {
            let schema = match schema_ref {
                ReferenceOr::Item(schema) => convert_schema(schema),
                ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                    reference: reference.clone(),
                }),
            };
            content.insert(media_type_name.clone(), MediaType { schema });
        }
    }

    let mut headers = HashMap::new();
    for (header_name, header_ref) in &response.headers {
        if let ReferenceOr::Item(header) = header_ref {
            if let openapiv3::ParameterSchemaOrContent::Schema(s) = &header.format {
                let schema = match s {
                    ReferenceOr::Item(schema) => convert_schema(schema),
                    ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                        reference: reference.clone(),
                    }),
                };
                headers.insert(
                    header_name.clone(),
                    Header {
                        description: header.description.clone(),
                        required: header.required,
                        schema,
                    },
                );
            }
        }
    }

    Response {
        description: response.description.clone(),
        content,
        headers,
    }
}

/// Converts an OpenAPI security requirement to Themis security requirements.
fn convert_security_requirement(req: &IndexMap<String, Vec<String>>) -> Vec<SecurityRequirement> {
    req.iter()
        .map(|(name, scopes)| SecurityRequirement {
            scheme: name.clone(),
            scopes: scopes.clone(),
        })
        .collect()
}

/// Converts an OpenAPI security scheme to a Themis security scheme.
fn convert_security_scheme(scheme: &OpenApiSecurityScheme) -> SecurityScheme {
    let scheme_type = match scheme {
        OpenApiSecurityScheme::APIKey { location, name, .. } => {
            let api_key_location = match location {
                openapiv3::APIKeyLocation::Query => ApiKeyLocation::Query,
                openapiv3::APIKeyLocation::Header => ApiKeyLocation::Header,
                openapiv3::APIKeyLocation::Cookie => ApiKeyLocation::Cookie,
            };
            SecuritySchemeType::ApiKey {
                location: api_key_location,
                name: name.clone(),
            }
        }
        OpenApiSecurityScheme::HTTP {
            scheme,
            bearer_format,
            ..
        } => SecuritySchemeType::Http {
            scheme: scheme.clone(),
            bearer_format: bearer_format.clone(),
        },
        OpenApiSecurityScheme::OAuth2 { .. } => SecuritySchemeType::OAuth2,
        OpenApiSecurityScheme::OpenIDConnect {
            open_id_connect_url,
            ..
        } => SecuritySchemeType::OpenIdConnect {
            openid_connect_url: open_id_connect_url.clone(),
        },
    };

    let description = match scheme {
        OpenApiSecurityScheme::APIKey { description, .. }
        | OpenApiSecurityScheme::HTTP { description, .. }
        | OpenApiSecurityScheme::OAuth2 { description, .. }
        | OpenApiSecurityScheme::OpenIDConnect { description, .. } => description.clone(),
    };

    SecurityScheme {
        scheme_type,
        description,
    }
}

/// Extracts Themis-specific metadata from operation extensions.
fn extract_themis_metadata(
    extensions: &IndexMap<String, serde_json::Value>,
) -> Option<ThemisOperationMetadata> {
    let rate_limit = extensions
        .get("x-themis-rate-limit-tier")
        .and_then(|v| v.as_str())
        .map(String::from);

    let timeout = extensions
        .get("x-themis-timeout-tier")
        .and_then(|v| v.as_str())
        .map(String::from);

    let idempotent = extensions
        .get("x-themis-idempotent")
        .and_then(serde_json::Value::as_bool);

    if rate_limit.is_some() || timeout.is_some() || idempotent.is_some() {
        Some(ThemisOperationMetadata {
            rate_limit_tier: rate_limit,
            timeout_tier: timeout,
            idempotent,
        })
    } else {
        None
    }
}

/// Converts an OpenAPI schema to a Themis schema.
fn convert_schema(schema: &OpenApiSchema) -> Schema {
    match &schema.schema_kind {
        SchemaKind::Type(schema_type) => convert_type_schema(schema_type, schema),
        SchemaKind::OneOf { one_of } => {
            let schemas: Vec<Schema> = one_of
                .iter()
                .map(|s| match s {
                    ReferenceOr::Item(schema) => convert_schema(schema),
                    ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                        reference: reference.clone(),
                    }),
                })
                .collect();
            Schema::OneOf(OneOfSchema {
                schemas,
                description: schema.schema_data.description.clone(),
                discriminator: None, // TODO: Handle discriminator
            })
        }
        SchemaKind::AllOf { all_of } => {
            let schemas: Vec<Schema> = all_of
                .iter()
                .map(|s| match s {
                    ReferenceOr::Item(schema) => convert_schema(schema),
                    ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                        reference: reference.clone(),
                    }),
                })
                .collect();
            Schema::AllOf(AllOfSchema {
                schemas,
                description: schema.schema_data.description.clone(),
            })
        }
        SchemaKind::AnyOf { any_of } => {
            let schemas: Vec<Schema> = any_of
                .iter()
                .map(|s| match s {
                    ReferenceOr::Item(schema) => convert_schema(schema),
                    ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                        reference: reference.clone(),
                    }),
                })
                .collect();
            Schema::AnyOf(AnyOfSchema {
                schemas,
                description: schema.schema_data.description.clone(),
            })
        }
        SchemaKind::Not { .. } => {
            // Not directly supported, return a generic object
            Schema::Object(ObjectSchema::default())
        }
        SchemaKind::Any(any) => convert_any_schema(any, schema),
    }
}

/// Converts an AnySchema to a Themis schema.
fn convert_any_schema(any: &openapiv3::AnySchema, schema: &OpenApiSchema) -> Schema {
    // Check if it has items (array)
    if let Some(items) = &any.items {
        let item_schema = match items {
            ReferenceOr::Item(schema) => Box::new(convert_schema(schema)),
            ReferenceOr::Reference { reference } => Box::new(Schema::Ref(RefSchema {
                reference: reference.clone(),
            })),
        };
        return Schema::Array(ArraySchema {
            items: item_schema,
            description: schema.schema_data.description.clone(),
            min_items: any.min_items,
            max_items: any.max_items,
            unique_items: any.unique_items.unwrap_or(false),
            nullable: schema.schema_data.nullable,
        });
    }

    // Check if it has properties (object)
    if !any.properties.is_empty() {
        return convert_object_from_any(any, schema);
    }

    // Check if it's an enum
    if !any.enumeration.is_empty() {
        let values: Vec<EnumValue> = any
            .enumeration
            .iter()
            .map(|v| EnumValue {
                value: v.clone(),
                description: None,
            })
            .collect();
        return Schema::Enum(EnumSchema {
            values,
            description: schema.schema_data.description.clone(),
            nullable: schema.schema_data.nullable,
        });
    }

    // Default to generic object
    Schema::Object(ObjectSchema {
        description: schema.schema_data.description.clone(),
        ..Default::default()
    })
}

/// Converts an OpenAPI type schema to a Themis schema.
fn convert_type_schema(schema_type: &OpenApiType, schema: &OpenApiSchema) -> Schema {
    match schema_type {
        OpenApiType::String(string_type) => {
            // Check if this is an enum
            if string_type.enumeration.is_empty() {
                let format_str = format!("{:?}", string_type.format);
                Schema::String(StringSchema {
                    description: schema.schema_data.description.clone(),
                    format: if format_str == "Empty" {
                        None
                    } else {
                        Some(format_str.to_lowercase())
                    },
                    min_length: string_type.min_length,
                    max_length: string_type.max_length,
                    pattern: string_type.pattern.clone(),
                    default: schema
                        .schema_data
                        .default
                        .as_ref()
                        .and_then(serde_json::Value::as_str)
                        .map(String::from),
                    nullable: schema.schema_data.nullable,
                })
            } else {
                let values: Vec<EnumValue> = string_type
                    .enumeration
                    .iter()
                    .filter_map(|v| {
                        v.as_ref().map(|s| EnumValue {
                            value: serde_json::Value::String(s.clone()),
                            description: None,
                        })
                    })
                    .collect();
                Schema::Enum(EnumSchema {
                    values,
                    description: schema.schema_data.description.clone(),
                    nullable: schema.schema_data.nullable,
                })
            }
        }
        OpenApiType::Number(number_type) => {
            let format_str = format!("{:?}", number_type.format);
            Schema::Number(NumberSchema {
                description: schema.schema_data.description.clone(),
                format: if format_str == "Empty" {
                    None
                } else {
                    Some(format_str.to_lowercase())
                },
                minimum: number_type.minimum,
                maximum: number_type.maximum,
                default: schema
                    .schema_data
                    .default
                    .as_ref()
                    .and_then(serde_json::Value::as_f64),
                nullable: schema.schema_data.nullable,
            })
        }
        OpenApiType::Integer(integer_type) => {
            let format_str = format!("{:?}", integer_type.format);
            Schema::Integer(IntegerSchema {
                description: schema.schema_data.description.clone(),
                format: if format_str == "Empty" {
                    None
                } else {
                    Some(format_str.to_lowercase())
                },
                minimum: integer_type.minimum,
                maximum: integer_type.maximum,
                default: schema
                    .schema_data
                    .default
                    .as_ref()
                    .and_then(serde_json::Value::as_i64),
                nullable: schema.schema_data.nullable,
            })
        }
        OpenApiType::Boolean(_) => Schema::Boolean(BooleanSchema {
            description: schema.schema_data.description.clone(),
            default: schema
                .schema_data
                .default
                .as_ref()
                .and_then(serde_json::Value::as_bool),
            nullable: schema.schema_data.nullable,
        }),
        OpenApiType::Array(array_type) => {
            let items = array_type.items.as_ref().map_or_else(
                || Box::new(Schema::Object(ObjectSchema::default())),
                |items| match items {
                    ReferenceOr::Item(schema) => Box::new(convert_schema(schema)),
                    ReferenceOr::Reference { reference } => Box::new(Schema::Ref(RefSchema {
                        reference: reference.clone(),
                    })),
                },
            );

            Schema::Array(ArraySchema {
                items,
                description: schema.schema_data.description.clone(),
                min_items: array_type.min_items,
                max_items: array_type.max_items,
                unique_items: array_type.unique_items,
                nullable: schema.schema_data.nullable,
            })
        }
        OpenApiType::Object(object_type) => {
            let properties: IndexMap<String, Schema> = object_type
                .properties
                .iter()
                .map(|(name, prop)| {
                    let prop_schema = match prop {
                        ReferenceOr::Item(s) => convert_schema(s),
                        ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                            reference: reference.clone(),
                        }),
                    };
                    (name.clone(), prop_schema)
                })
                .collect();

            let additional_properties =
                object_type
                    .additional_properties
                    .as_ref()
                    .and_then(|ap| match ap {
                        openapiv3::AdditionalProperties::Any(allowed) => {
                            if *allowed {
                                Some(Box::new(Schema::Object(ObjectSchema::default())))
                            } else {
                                None
                            }
                        }
                        openapiv3::AdditionalProperties::Schema(schema_ref) => {
                            match schema_ref.as_ref() {
                                ReferenceOr::Item(s) => Some(Box::new(convert_schema(s))),
                                ReferenceOr::Reference { reference } => {
                                    Some(Box::new(Schema::Ref(RefSchema {
                                        reference: reference.clone(),
                                    })))
                                }
                            }
                        }
                    });

            Schema::Object(ObjectSchema {
                description: schema.schema_data.description.clone(),
                properties,
                required: object_type.required.clone(),
                additional_properties,
                nullable: schema.schema_data.nullable,
            })
        }
    }
}

/// Converts an AnySchema with properties to an Object schema.
fn convert_object_from_any(any: &openapiv3::AnySchema, schema: &OpenApiSchema) -> Schema {
    let properties: IndexMap<String, Schema> = any
        .properties
        .iter()
        .map(|(name, prop)| {
            let prop_schema = match prop {
                ReferenceOr::Item(s) => convert_schema(s),
                ReferenceOr::Reference { reference } => Schema::Ref(RefSchema {
                    reference: reference.clone(),
                }),
            };
            (name.clone(), prop_schema)
        })
        .collect();

    Schema::Object(ObjectSchema {
        description: schema.schema_data.description.clone(),
        properties,
        required: any.required.clone(),
        additional_properties: None,
        nullable: schema.schema_data.nullable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_OPENAPI: &str = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths: {}
"#;

    const SIMPLE_OPENAPI: &str = r#"
openapi: "3.1.0"
info:
  title: Users API
  version: "1.0.0"
  description: User management API
paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      responses:
        "200":
          description: Success
    post:
      operationId: createUser
      summary: Create a user
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateUserRequest'
      responses:
        "201":
          description: Created
  /users/{userId}:
    get:
      operationId: getUser
      summary: Get a user by ID
      parameters:
        - name: userId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: Success
components:
  schemas:
    CreateUserRequest:
      type: object
      required:
        - email
        - name
      properties:
        email:
          type: string
          format: email
        name:
          type: string
          minLength: 1
          maxLength: 100
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
"#;

    #[test]
    fn test_parse_minimal_openapi() {
        let contract = parse_openapi(MINIMAL_OPENAPI).unwrap();
        assert_eq!(contract.metadata.service_name, "Test API");
        assert_eq!(contract.version, Version::new(1, 0, 0));
        assert_eq!(contract.operation_count(), 0);
    }

    #[test]
    fn test_parse_simple_openapi() {
        let contract = parse_openapi(SIMPLE_OPENAPI).unwrap();

        assert_eq!(contract.metadata.service_name, "Users API");
        assert_eq!(
            contract.metadata.description,
            Some("User management API".to_string())
        );
        assert_eq!(contract.operation_count(), 3);

        // Check operations
        assert!(contract.operations.contains_key("listUsers"));
        assert!(contract.operations.contains_key("createUser"));
        assert!(contract.operations.contains_key("getUser"));

        // Check listUsers operation
        let list_users = contract.operations.get("listUsers").unwrap();
        assert_eq!(list_users.method, Some(HttpMethod::Get));
        assert_eq!(list_users.path, Some("/users".to_string()));
        assert_eq!(list_users.summary, Some("List all users".to_string()));

        // Check getUser has parameter
        let get_user = contract.operations.get("getUser").unwrap();
        assert_eq!(get_user.parameters.len(), 1);
        assert_eq!(get_user.parameters[0].name, "userId");
        assert_eq!(get_user.parameters[0].location, ParameterLocation::Path);
        assert!(get_user.parameters[0].required);

        // Check schemas
        assert_eq!(contract.schema_count(), 1);
        assert!(contract.schemas.contains_key("CreateUserRequest"));

        // Check security schemes
        assert_eq!(contract.security_schemes.len(), 1);
        assert!(contract.security_schemes.contains_key("bearerAuth"));
    }

    #[test]
    fn test_missing_operation_id() {
        let yaml = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /test:
    get:
      responses:
        "200":
          description: OK
"#;
        let result = parse_openapi(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("operationId"));
    }

    #[test]
    fn test_parse_json_openapi() {
        let json = r#"{
            "openapi": "3.1.0",
            "info": {
                "title": "JSON API",
                "version": "2.0.0"
            },
            "paths": {}
        }"#;

        let contract = parse_openapi(json).unwrap();
        assert_eq!(contract.metadata.service_name, "JSON API");
        assert_eq!(contract.version, Version::new(2, 0, 0));
    }

    #[test]
    fn test_themis_extensions() {
        let yaml = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
  x-themis-owner: platform-team
  x-themis-repository: https://github.com/example/api
paths:
  /test:
    get:
      operationId: testOp
      x-themis-rate-limit-tier: standard
      x-themis-timeout-tier: fast
      x-themis-idempotent: true
      responses:
        "200":
          description: OK
"#;
        let contract = parse_openapi(yaml).unwrap();

        assert_eq!(contract.metadata.owner, Some("platform-team".to_string()));
        assert_eq!(
            contract.metadata.repository,
            Some("https://github.com/example/api".to_string())
        );

        let op = contract.operations.get("testOp").unwrap();
        let meta = op.themis_metadata.as_ref().unwrap();
        assert_eq!(meta.rate_limit_tier, Some("standard".to_string()));
        assert_eq!(meta.timeout_tier, Some("fast".to_string()));
        assert_eq!(meta.idempotent, Some(true));
    }

    #[test]
    fn test_schema_conversion() {
        let yaml = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths: {}
components:
  schemas:
    User:
      type: object
      required:
        - id
        - email
      properties:
        id:
          type: string
          format: uuid
        email:
          type: string
          format: email
        age:
          type: integer
          minimum: 0
          maximum: 150
        score:
          type: number
          format: double
        active:
          type: boolean
        tags:
          type: array
          items:
            type: string
          minItems: 1
        status:
          type: string
          enum:
            - active
            - inactive
            - pending
"#;
        let contract = parse_openapi(yaml).unwrap();

        let user_schema = contract.schemas.get("User").unwrap();
        if let Schema::Object(obj) = user_schema {
            assert_eq!(obj.properties.len(), 7);

            // Check required fields
            assert!(obj.required.contains(&"id".to_string()));
            assert!(obj.required.contains(&"email".to_string()));
            assert!(!obj.required.contains(&"age".to_string()));

            // Check string with format
            if let Schema::String(s) = obj.properties.get("id").unwrap() {
                assert!(s.format.is_some());
            } else {
                panic!("Expected string schema for id");
            }

            // Check integer with constraints
            if let Schema::Integer(i) = obj.properties.get("age").unwrap() {
                assert_eq!(i.minimum, Some(0));
                assert_eq!(i.maximum, Some(150));
            } else {
                panic!("Expected integer schema for age");
            }

            // Check array
            if let Schema::Array(arr) = obj.properties.get("tags").unwrap() {
                assert_eq!(arr.min_items, Some(1));
            } else {
                panic!("Expected array schema for tags");
            }

            // Check enum
            if let Schema::Enum(e) = obj.properties.get("status").unwrap() {
                assert_eq!(e.values.len(), 3);
            } else {
                panic!("Expected enum schema for status");
            }
        } else {
            panic!("Expected object schema for User");
        }
    }

    #[test]
    fn test_security_schemes() {
        let yaml = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths: {}
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
    apiKey:
      type: apiKey
      in: header
      name: X-API-Key
    oauth2:
      type: oauth2
      flows:
        authorizationCode:
          authorizationUrl: https://example.com/oauth/authorize
          tokenUrl: https://example.com/oauth/token
          scopes:
            read: Read access
            write: Write access
"#;
        let contract = parse_openapi(yaml).unwrap();

        assert_eq!(contract.security_schemes.len(), 3);

        // Check bearer auth
        let bearer = contract.security_schemes.get("bearerAuth").unwrap();
        if let SecuritySchemeType::Http {
            scheme,
            bearer_format,
        } = &bearer.scheme_type
        {
            assert_eq!(scheme, "bearer");
            assert_eq!(bearer_format, &Some("JWT".to_string()));
        } else {
            panic!("Expected HTTP scheme for bearerAuth");
        }

        // Check API key
        let api_key = contract.security_schemes.get("apiKey").unwrap();
        if let SecuritySchemeType::ApiKey { location, name } = &api_key.scheme_type {
            assert_eq!(*location, ApiKeyLocation::Header);
            assert_eq!(name, "X-API-Key");
        } else {
            panic!("Expected ApiKey scheme for apiKey");
        }

        // Check OAuth2
        let oauth2 = contract.security_schemes.get("oauth2").unwrap();
        assert!(matches!(oauth2.scheme_type, SecuritySchemeType::OAuth2));
    }
}
