//! `AsyncAPI` 3.0 parser for Themis contract governance.
//!
//! Parses `AsyncAPI` 3.0 specifications and converts them to Themis Contract format.

use std::collections::HashMap;
use themis_core::{
    contract::{Contract, ContractFormat, ContractMetadata},
    operation::{HttpMethod, MediaType, Operation, RequestBody, Response},
    schema::{
        ArraySchema, BooleanSchema, EnumSchema, EnumValue, IntegerSchema, NumberSchema,
        ObjectSchema, RefSchema, Schema, StringSchema,
    },
    version::Version,
};

use crate::error::AsyncApiError;
use indexmap::IndexMap;

/// Parser for `AsyncAPI` 3.0 specifications.
pub struct AsyncApiParser;

impl AsyncApiParser {
    /// Parse an `AsyncAPI` specification from YAML or JSON.
    ///
    /// # Arguments
    ///
    /// * `input` - The `AsyncAPI` specification as a string
    ///
    /// # Returns
    ///
    /// A `Contract` representing the parsed specification
    ///
    /// # Errors
    ///
    /// Returns `AsyncApiError` if parsing fails
    pub fn parse(input: &str) -> Result<Contract, AsyncApiError> {
        let doc: serde_yaml::Value = serde_yaml::from_str(input)?;
        Self::parse_document(&doc)
    }

    /// Parse a pre-parsed YAML document.
    ///
    /// # Errors
    ///
    /// Returns `AsyncApiError` if the document structure is invalid
    pub fn parse_document(doc: &serde_yaml::Value) -> Result<Contract, AsyncApiError> {
        Self::validate_version(doc)?;

        let (metadata, version, name) = Self::parse_metadata(doc)?;
        let operations = Self::parse_operations(doc)?;
        let schemas = Self::parse_component_schemas(doc);

        Ok(Contract {
            format: ContractFormat::AsyncApi,
            version,
            metadata: ContractMetadata {
                service_name: name,
                description: metadata.description,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas,
            security_schemes: HashMap::new(),
        })
    }

    /// Validate `AsyncAPI` version is 3.x
    fn validate_version(doc: &serde_yaml::Value) -> Result<(), AsyncApiError> {
        match doc.get("asyncapi") {
            Some(serde_yaml::Value::String(version)) => {
                if version.starts_with("3.") {
                    Ok(())
                } else {
                    Err(AsyncApiError::InvalidVersion(version.clone()))
                }
            }
            Some(_) => Err(AsyncApiError::InvalidVersion("non-string".to_string())),
            None => Err(AsyncApiError::MissingField("asyncapi".to_string())),
        }
    }

    /// Parse contract metadata from info section.
    fn parse_metadata(
        doc: &serde_yaml::Value,
    ) -> Result<(ParsedMetadata, Version, String), AsyncApiError> {
        let info = doc
            .get("info")
            .ok_or_else(|| AsyncApiError::MissingField("info".to_string()))?;

        let title = info
            .get("title")
            .and_then(serde_yaml::Value::as_str)
            .ok_or_else(|| AsyncApiError::MissingField("info.title".to_string()))?
            .to_string();

        let version_str = info
            .get("version")
            .and_then(serde_yaml::Value::as_str)
            .ok_or_else(|| AsyncApiError::MissingField("info.version".to_string()))?;

        let version = Self::parse_version(version_str);

        let description = info
            .get("description")
            .and_then(serde_yaml::Value::as_str)
            .map(String::from);

        Ok((ParsedMetadata { description }, version, title))
    }

    /// Parse a semver version string.
    fn parse_version(version_str: &str) -> Version {
        let parts: Vec<&str> = version_str.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        Version::new(major, minor, patch)
    }

    /// Parse operations from channels and operations sections.
    fn parse_operations(
        doc: &serde_yaml::Value,
    ) -> Result<HashMap<String, Operation>, AsyncApiError> {
        let mut operations = HashMap::new();

        // In `AsyncAPI` 3.0, operations are defined separately from channels
        if let Some(serde_yaml::Value::Mapping(ops)) = doc.get("operations") {
            for (name, op_def) in ops {
                let op_name = name
                    .as_str()
                    .ok_or_else(|| AsyncApiError::InvalidOperation("invalid key".to_string()))?;

                let operation = Self::parse_operation(op_name, op_def, doc);
                operations.insert(op_name.to_string(), operation);
            }
        }

        // Also parse channels directly if they have inline messages
        if operations.is_empty() {
            if let Some(serde_yaml::Value::Mapping(channels)) = doc.get("channels") {
                for (name, channel_def) in channels {
                    let channel_name = name.as_str().unwrap_or("unknown");

                    if let Some(publish) = channel_def.get("publish") {
                        let op_id = publish
                            .get("operationId")
                            .and_then(serde_yaml::Value::as_str)
                            .unwrap_or(channel_name);
                        let operation = Self::parse_channel_operation(op_id, publish, "publish");
                        operations.insert(op_id.to_string(), operation);
                    }

                    if let Some(subscribe) = channel_def.get("subscribe") {
                        let op_id = subscribe
                            .get("operationId")
                            .and_then(serde_yaml::Value::as_str)
                            .unwrap_or(channel_name);
                        let operation =
                            Self::parse_channel_operation(op_id, subscribe, "subscribe");
                        operations.insert(op_id.to_string(), operation);
                    }

                    // `AsyncAPI` 3.0 style - messages in channel
                    if let Some(serde_yaml::Value::Mapping(messages)) = channel_def.get("messages")
                    {
                        for (msg_name, msg_def) in messages {
                            let msg_name_str = msg_name.as_str().unwrap_or("unknown");
                            let op_id = format!("{channel_name}_{msg_name_str}");
                            let operation = Self::parse_message_operation(&op_id, msg_def);
                            operations.insert(op_id, operation);
                        }
                    }
                }
            }
        }

        Ok(operations)
    }

    /// Parse an operation from the operations section.
    fn parse_operation(
        name: &str,
        op_def: &serde_yaml::Value,
        doc: &serde_yaml::Value,
    ) -> Operation {
        let action = op_def
            .get("action")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or("send");

        let description = op_def
            .get("description")
            .or_else(|| op_def.get("summary"))
            .and_then(serde_yaml::Value::as_str)
            .map(String::from);

        let summary = op_def
            .get("summary")
            .and_then(serde_yaml::Value::as_str)
            .map(String::from);

        // Map action to HTTP method
        let method = match action {
            "receive" => Some(HttpMethod::Get),
            _ => Some(HttpMethod::Post),
        };

        let path = op_def
            .get("channel")
            .and_then(|c| c.get("$ref"))
            .and_then(serde_yaml::Value::as_str)
            .map(|r| r.replace("#/channels/", "/"));

        let request_body = Self::parse_operation_messages(op_def, doc);

        let responses = if action == "receive" {
            let mut resp = HashMap::new();
            if let Some(body) = &request_body {
                let response = Response {
                    description: description.clone().unwrap_or_default(),
                    content: body.content.clone(),
                    headers: HashMap::new(),
                };
                resp.insert("200".to_string(), response);
            }
            resp
        } else {
            let mut resp = HashMap::new();
            resp.insert(
                "200".to_string(),
                Response {
                    description: "Message sent".to_string(),
                    content: HashMap::new(),
                    headers: HashMap::new(),
                },
            );
            resp
        };

        let request = if action == "send" { request_body } else { None };

        Operation {
            operation_id: name.to_string(),
            summary,
            description,
            method,
            path,
            parameters: Vec::new(),
            request_body: request,
            responses,
            security: Vec::new(),
            deprecated: op_def
                .get("deprecated")
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            tags: Vec::new(),
            themis_metadata: None,
        }
    }

    /// Parse messages from an operation.
    fn parse_operation_messages(
        op_def: &serde_yaml::Value,
        doc: &serde_yaml::Value,
    ) -> Option<RequestBody> {
        // `AsyncAPI` 3.0: messages array
        if let Some(serde_yaml::Value::Sequence(messages)) = op_def.get("messages") {
            if let Some(first_msg) = messages.first() {
                let msg = first_msg
                    .get("$ref")
                    .map_or(first_msg, |ref_path| {
                        Self::resolve_ref(ref_path.as_str().unwrap_or(""), doc)
                            .unwrap_or(first_msg)
                    });

                if let Some(payload) = msg.get("payload") {
                    let schema = Self::parse_schema(payload);
                    let mut content = HashMap::new();
                    content.insert("application/json".to_string(), MediaType { schema });
                    return Some(RequestBody {
                        description: msg
                            .get("description")
                            .and_then(serde_yaml::Value::as_str)
                            .map(String::from),
                        content,
                        required: true,
                    });
                }
            }
        }

        // Direct message property
        if let Some(message) = op_def.get("message") {
            if let Some(payload) = message.get("payload") {
                let schema = Self::parse_schema(payload);
                let mut content = HashMap::new();
                content.insert("application/json".to_string(), MediaType { schema });
                return Some(RequestBody {
                    description: message
                        .get("description")
                        .and_then(serde_yaml::Value::as_str)
                        .map(String::from),
                    content,
                    required: true,
                });
            }
        }

        None
    }

    /// Parse a channel operation (publish/subscribe).
    fn parse_channel_operation(name: &str, op_def: &serde_yaml::Value, action: &str) -> Operation {
        let description = op_def
            .get("description")
            .or_else(|| op_def.get("summary"))
            .and_then(serde_yaml::Value::as_str)
            .map(String::from);

        let method = match action {
            "subscribe" => Some(HttpMethod::Get),
            _ => Some(HttpMethod::Post),
        };

        let request_body = op_def.get("message").and_then(|message| {
            message.get("payload").map(|payload| {
                let schema = Self::parse_schema(payload);
                let mut content = HashMap::new();
                content.insert("application/json".to_string(), MediaType { schema });
                RequestBody {
                    description: message
                        .get("description")
                        .and_then(serde_yaml::Value::as_str)
                        .map(String::from),
                    content,
                    required: true,
                }
            })
        });

        let mut responses = HashMap::new();
        responses.insert(
            "200".to_string(),
            Response {
                description: "Success".to_string(),
                content: HashMap::new(),
                headers: HashMap::new(),
            },
        );

        Operation {
            operation_id: name.to_string(),
            summary: None,
            description,
            method,
            path: Some(format!("/{name}")),
            parameters: Vec::new(),
            request_body,
            responses,
            security: Vec::new(),
            deprecated: false,
            tags: Vec::new(),
            themis_metadata: None,
        }
    }

    /// Parse a message as an operation.
    fn parse_message_operation(name: &str, msg_def: &serde_yaml::Value) -> Operation {
        let description = msg_def
            .get("description")
            .and_then(serde_yaml::Value::as_str)
            .map(String::from);

        let request_body = msg_def.get("payload").map(|payload| {
            let schema = Self::parse_schema(payload);
            let mut content = HashMap::new();
            content.insert("application/json".to_string(), MediaType { schema });
            RequestBody {
                description: description.clone(),
                content,
                required: true,
            }
        });

        let mut responses = HashMap::new();
        responses.insert(
            "200".to_string(),
            Response {
                description: "Success".to_string(),
                content: HashMap::new(),
                headers: HashMap::new(),
            },
        );

        Operation {
            operation_id: name.to_string(),
            summary: None,
            description,
            method: Some(HttpMethod::Post),
            path: Some(format!("/{name}")),
            parameters: Vec::new(),
            request_body,
            responses,
            security: Vec::new(),
            deprecated: false,
            tags: Vec::new(),
            themis_metadata: None,
        }
    }

    /// Parse component schemas.
    fn parse_component_schemas(doc: &serde_yaml::Value) -> IndexMap<String, Schema> {
        let mut schemas = IndexMap::new();

        if let Some(components) = doc.get("components") {
            if let Some(serde_yaml::Value::Mapping(schema_defs)) = components.get("schemas") {
                for (name, schema_def) in schema_defs {
                    if let Some(name_str) = name.as_str() {
                        let schema = Self::parse_schema(schema_def);
                        schemas.insert(name_str.to_string(), schema);
                    }
                }
            }

            if let Some(serde_yaml::Value::Mapping(messages)) = components.get("messages") {
                for (name, msg_def) in messages {
                    if let Some(name_str) = name.as_str() {
                        if let Some(payload) = msg_def.get("payload") {
                            let schema = Self::parse_schema(payload);
                            schemas.insert(format!("{name_str}Payload"), schema);
                        }
                    }
                }
            }
        }

        schemas
    }

    /// Parse a schema definition.
    fn parse_schema(value: &serde_yaml::Value) -> Schema {
        if let Some(ref_path) = value.get("$ref").and_then(serde_yaml::Value::as_str) {
            return Schema::Ref(RefSchema {
                reference: ref_path.to_string(),
            });
        }

        let type_value = value.get("type").and_then(serde_yaml::Value::as_str);

        match type_value {
            Some("string") => Self::parse_string_schema(value),
            Some("integer") => Self::parse_integer_schema(value),
            Some("number") => Self::parse_number_schema(value),
            Some("boolean") => Schema::Boolean(BooleanSchema {
                description: Self::get_description(value),
                default: None,
                nullable: false,
            }),
            Some("array") => Self::parse_array_schema(value),
            Some("object") => Self::parse_object_schema(value),
            Some("null") => Schema::Null,
            _ => {
                if value.get("enum").is_some() {
                    return Self::parse_enum_schema(value);
                }
                if value.get("properties").is_some() {
                    return Self::parse_object_schema(value);
                }
                Schema::Object(ObjectSchema {
                    description: Self::get_description(value),
                    properties: IndexMap::new(),
                    required: Vec::new(),
                    additional_properties: None,
                    nullable: false,
                })
            }
        }
    }

    /// Parse a string schema.
    fn parse_string_schema(value: &serde_yaml::Value) -> Schema {
        Schema::String(StringSchema {
            description: Self::get_description(value),
            min_length: Self::get_usize(value, "minLength"),
            max_length: Self::get_usize(value, "maxLength"),
            pattern: value
                .get("pattern")
                .and_then(serde_yaml::Value::as_str)
                .map(String::from),
            format: value
                .get("format")
                .and_then(serde_yaml::Value::as_str)
                .map(String::from),
            default: None,
            nullable: false,
        })
    }

    /// Parse an integer schema.
    fn parse_integer_schema(value: &serde_yaml::Value) -> Schema {
        Schema::Integer(IntegerSchema {
            description: Self::get_description(value),
            minimum: value.get("minimum").and_then(serde_yaml::Value::as_i64),
            maximum: value.get("maximum").and_then(serde_yaml::Value::as_i64),
            format: value
                .get("format")
                .and_then(serde_yaml::Value::as_str)
                .map(String::from),
            default: None,
            nullable: false,
        })
    }

    /// Parse a number schema.
    fn parse_number_schema(value: &serde_yaml::Value) -> Schema {
        Schema::Number(NumberSchema {
            description: Self::get_description(value),
            minimum: value.get("minimum").and_then(serde_yaml::Value::as_f64),
            maximum: value.get("maximum").and_then(serde_yaml::Value::as_f64),
            format: value
                .get("format")
                .and_then(serde_yaml::Value::as_str)
                .map(String::from),
            default: None,
            nullable: false,
        })
    }

    /// Parse an array schema.
    fn parse_array_schema(value: &serde_yaml::Value) -> Schema {
        let items = value.get("items").map_or_else(
            || {
                Box::new(Schema::Object(ObjectSchema {
                    description: None,
                    properties: IndexMap::new(),
                    required: Vec::new(),
                    additional_properties: None,
                    nullable: false,
                }))
            },
            |i| Box::new(Self::parse_schema(i)),
        );

        Schema::Array(ArraySchema {
            description: Self::get_description(value),
            items,
            min_items: Self::get_usize(value, "minItems"),
            max_items: Self::get_usize(value, "maxItems"),
            unique_items: false,
            nullable: false,
        })
    }

    /// Parse an object schema.
    fn parse_object_schema(value: &serde_yaml::Value) -> Schema {
        let mut properties = IndexMap::new();

        if let Some(serde_yaml::Value::Mapping(props)) = value.get("properties") {
            for (name, prop_def) in props {
                if let Some(name_str) = name.as_str() {
                    properties.insert(name_str.to_string(), Self::parse_schema(prop_def));
                }
            }
        }

        let required = value
            .get("required")
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .filter_map(serde_yaml::Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Schema::Object(ObjectSchema {
            description: Self::get_description(value),
            properties,
            required,
            additional_properties: None,
            nullable: false,
        })
    }

    /// Parse an enum schema.
    fn parse_enum_schema(value: &serde_yaml::Value) -> Schema {
        let values = value
            .get("enum")
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .map(|v| EnumValue {
                        value: Self::yaml_to_json(v),
                        description: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        Schema::Enum(EnumSchema {
            description: Self::get_description(value),
            values,
            nullable: false,
        })
    }

    /// Convert YAML value to JSON value for enum.
    fn yaml_to_json(value: &serde_yaml::Value) -> serde_json::Value {
        match value {
            serde_yaml::Value::Null => serde_json::Value::Null,
            serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
            serde_yaml::Value::Number(n) => n.as_i64().map_or_else(
                || {
                    n.as_f64().map_or(serde_json::Value::Null, |f| {
                        serde_json::Number::from_f64(f)
                            .map_or(serde_json::Value::Null, serde_json::Value::Number)
                    })
                },
                |i| serde_json::Value::Number(i.into()),
            ),
            serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
            serde_yaml::Value::Sequence(seq) => {
                serde_json::Value::Array(seq.iter().map(Self::yaml_to_json).collect())
            }
            serde_yaml::Value::Mapping(map) => {
                let obj: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), Self::yaml_to_json(v))))
                    .collect();
                serde_json::Value::Object(obj)
            }
            serde_yaml::Value::Tagged(tagged) => Self::yaml_to_json(&tagged.value),
        }
    }

    /// Get description from a value.
    fn get_description(value: &serde_yaml::Value) -> Option<String> {
        value
            .get("description")
            .and_then(serde_yaml::Value::as_str)
            .map(String::from)
    }

    /// Get a usize value from a YAML field with proper truncation handling.
    #[allow(clippy::cast_possible_truncation)]
    fn get_usize(value: &serde_yaml::Value, field: &str) -> Option<usize> {
        value
            .get(field)
            .and_then(serde_yaml::Value::as_u64)
            .map(|v| v as usize)
    }

    /// Resolve a $ref path to the actual value.
    fn resolve_ref<'a>(
        ref_path: &str,
        doc: &'a serde_yaml::Value,
    ) -> Option<&'a serde_yaml::Value> {
        if !ref_path.starts_with("#/") {
            return None;
        }

        let path_parts: Vec<&str> = ref_path[2..].split('/').collect();
        let mut current = doc;

        for part in path_parts {
            current = current.get(part)?;
        }

        Some(current)
    }
}

/// Intermediate metadata structure.
struct ParsedMetadata {
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_asyncapi() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
channels: {}
operations: {}
"#;

        let contract = AsyncApiParser::parse(yaml).unwrap();
        assert_eq!(contract.metadata.service_name, "Test API");
        assert_eq!(contract.version, Version::new(1, 0, 0));
        assert_eq!(contract.format, ContractFormat::AsyncApi);
    }

    #[test]
    fn test_parse_with_description() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: User Events
  version: 2.0.0
  description: Events for user management
channels: {}
operations: {}
"#;

        let contract = AsyncApiParser::parse(yaml).unwrap();
        assert_eq!(
            contract.metadata.description,
            Some("Events for user management".to_string())
        );
    }

    #[test]
    fn test_parse_with_operations() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: User Events
  version: 1.0.0
channels:
  userCreated:
    messages:
      userCreatedMessage:
        payload:
          type: object
          properties:
            userId:
              type: string
operations:
  sendUserCreated:
    action: send
    channel:
      $ref: '#/channels/userCreated'
    messages:
      - $ref: '#/channels/userCreated/messages/userCreatedMessage'
"#;

        let contract = AsyncApiParser::parse(yaml).unwrap();
        assert!(!contract.operations.is_empty());
    }

    #[test]
    fn test_parse_component_schemas() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
channels: {}
operations: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        email:
          type: string
      required:
        - id
"#;

        let contract = AsyncApiParser::parse(yaml).unwrap();
        assert!(contract.schemas.contains_key("User"));
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
asyncapi: 2.0.0
info:
  title: Test API
  version: 1.0.0
"#;

        let result = AsyncApiParser::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_info() {
        let yaml = r#"
asyncapi: 3.0.0
"#;

        let result = AsyncApiParser::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_schema() {
        let yaml = r#"
type: string
description: User email
format: email
minLength: 5
maxLength: 100
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let schema = AsyncApiParser::parse_schema(&value);

        if let Schema::String(s) = schema {
            assert_eq!(s.description, Some("User email".to_string()));
            assert_eq!(s.format, Some("email".to_string()));
            assert_eq!(s.min_length, Some(5));
            assert_eq!(s.max_length, Some(100));
        } else {
            panic!("Expected String schema");
        }
    }

    #[test]
    fn test_parse_integer_schema() {
        let yaml = r#"
type: integer
description: User age
minimum: 0
maximum: 150
format: int32
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let schema = AsyncApiParser::parse_schema(&value);

        if let Schema::Integer(i) = schema {
            assert_eq!(i.description, Some("User age".to_string()));
            assert_eq!(i.minimum, Some(0));
            assert_eq!(i.maximum, Some(150));
        } else {
            panic!("Expected Integer schema");
        }
    }

    #[test]
    fn test_parse_array_schema() {
        let yaml = r#"
type: array
items:
  type: string
minItems: 1
maxItems: 10
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let schema = AsyncApiParser::parse_schema(&value);

        if let Schema::Array(a) = schema {
            assert!(matches!(*a.items, Schema::String(_)));
            assert_eq!(a.min_items, Some(1));
            assert_eq!(a.max_items, Some(10));
        } else {
            panic!("Expected Array schema");
        }
    }

    #[test]
    fn test_parse_enum_schema() {
        let yaml = r#"
enum:
  - active
  - inactive
  - pending
description: User status
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let schema = AsyncApiParser::parse_schema(&value);

        if let Schema::Enum(e) = schema {
            assert_eq!(e.description, Some("User status".to_string()));
            assert_eq!(e.values.len(), 3);
        } else {
            panic!("Expected Enum schema, got {:?}", schema);
        }
    }
}
