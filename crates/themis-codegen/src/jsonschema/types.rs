//! JSON Schema type conversion.
//!
//! Converts Themis schema types to JSON Schema draft 2020-12 format.

#![allow(clippy::too_many_lines)] // Complex JSON Schema conversion requires detailed handling
#![allow(clippy::self_only_used_in_recursion)] // Recursive methods need self for consistency
#![allow(dead_code)] // Some methods for future extensibility

use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use themis_core::Schema;

/// Converts a Themis schema to JSON Schema format.
pub struct JsonSchemaConverter;

impl JsonSchemaConverter {
    /// Creates a new converter.
    pub const fn new() -> Self {
        Self
    }

    /// Converts a Themis schema to JSON Schema.
    pub fn convert(&self, schema: &Schema) -> Value {
        self.schema_to_json_schema(schema)
    }

    /// Converts a map of named schemas to individual JSON Schema objects.
    pub fn convert_schemas(&self, schemas: &IndexMap<String, Schema>) -> IndexMap<String, Value> {
        schemas
            .iter()
            .map(|(name, schema)| {
                let json_schema = self.create_root_schema(name, schema);
                (name.clone(), json_schema)
            })
            .collect()
    }

    /// Creates a root JSON Schema document for a named type.
    fn create_root_schema(&self, name: &str, schema: &Schema) -> Value {
        let mut root = self.schema_to_json_schema(schema);

        // If the result is an object, add $schema and title
        if let Value::Object(ref mut map) = root {
            // Insert at the beginning
            let mut new_map = Map::new();
            new_map.insert(
                "$schema".to_string(),
                json!("https://json-schema.org/draft/2020-12/schema"),
            );
            new_map.insert("$id".to_string(), json!(format!("{name}.json")));
            new_map.insert("title".to_string(), json!(name));

            // Merge the rest
            for (k, v) in map.iter() {
                new_map.insert(k.clone(), v.clone());
            }

            return Value::Object(new_map);
        }

        // For non-object schemas, wrap them
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": format!("{name}.json"),
            "title": name,
            "allOf": [root]
        })
    }

    /// Converts a Themis schema to a JSON Schema value.
    fn schema_to_json_schema(&self, schema: &Schema) -> Value {
        match schema {
            Schema::String(s) => {
                let mut obj = json!({ "type": "string" });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = s.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if let Some(ref format) = s.format {
                        map.insert("format".to_string(), json!(format));
                    }
                    if let Some(min) = s.min_length {
                        map.insert("minLength".to_string(), json!(min));
                    }
                    if let Some(max) = s.max_length {
                        map.insert("maxLength".to_string(), json!(max));
                    }
                    if let Some(ref pattern) = s.pattern {
                        map.insert("pattern".to_string(), json!(pattern));
                    }
                    if let Some(ref default) = s.default {
                        map.insert("default".to_string(), json!(default));
                    }
                    if s.nullable {
                        map.insert("type".to_string(), json!(["string", "null"]));
                    }
                }
                obj
            }

            Schema::Integer(i) => {
                let mut obj = json!({ "type": "integer" });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = i.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if let Some(ref format) = i.format {
                        // Map to JSON Schema format hints
                        let fmt = match format.as_str() {
                            "int32" => "int32",
                            "int64" => "int64",
                            _ => format.as_str(),
                        };
                        map.insert("format".to_string(), json!(fmt));
                    }
                    if let Some(min) = i.minimum {
                        map.insert("minimum".to_string(), json!(min));
                    }
                    if let Some(max) = i.maximum {
                        map.insert("maximum".to_string(), json!(max));
                    }
                    if let Some(default) = i.default {
                        map.insert("default".to_string(), json!(default));
                    }
                    if i.nullable {
                        map.insert("type".to_string(), json!(["integer", "null"]));
                    }
                }
                obj
            }

            Schema::Number(n) => {
                let mut obj = json!({ "type": "number" });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = n.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if let Some(ref format) = n.format {
                        let fmt = match format.as_str() {
                            "float" => "float",
                            "double" => "double",
                            _ => format.as_str(),
                        };
                        map.insert("format".to_string(), json!(fmt));
                    }
                    if let Some(min) = n.minimum {
                        map.insert("minimum".to_string(), json!(min));
                    }
                    if let Some(max) = n.maximum {
                        map.insert("maximum".to_string(), json!(max));
                    }
                    if let Some(default) = n.default {
                        map.insert("default".to_string(), json!(default));
                    }
                    if n.nullable {
                        map.insert("type".to_string(), json!(["number", "null"]));
                    }
                }
                obj
            }

            Schema::Boolean(b) => {
                let mut obj = json!({ "type": "boolean" });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = b.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if let Some(default) = b.default {
                        map.insert("default".to_string(), json!(default));
                    }
                    if b.nullable {
                        map.insert("type".to_string(), json!(["boolean", "null"]));
                    }
                }
                obj
            }

            Schema::Array(a) => {
                let items_schema = self.schema_to_json_schema(&a.items);
                let mut obj = json!({
                    "type": "array",
                    "items": items_schema
                });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = a.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if let Some(min) = a.min_items {
                        map.insert("minItems".to_string(), json!(min));
                    }
                    if let Some(max) = a.max_items {
                        map.insert("maxItems".to_string(), json!(max));
                    }
                    if a.unique_items {
                        map.insert("uniqueItems".to_string(), json!(true));
                    }
                    if a.nullable {
                        map.insert("type".to_string(), json!(["array", "null"]));
                    }
                }
                obj
            }

            Schema::Object(o) => {
                let properties: Map<String, Value> = o
                    .properties
                    .iter()
                    .map(|(name, prop_schema)| {
                        (name.clone(), self.schema_to_json_schema(prop_schema))
                    })
                    .collect();

                let mut obj = json!({
                    "type": "object",
                    "properties": properties
                });

                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = o.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if !o.required.is_empty() {
                        map.insert("required".to_string(), json!(o.required));
                    }
                    if let Some(ref additional) = o.additional_properties {
                        map.insert(
                            "additionalProperties".to_string(),
                            self.schema_to_json_schema(additional),
                        );
                    } else {
                        // Default to no additional properties for strict typing
                        map.insert("additionalProperties".to_string(), json!(false));
                    }
                    if o.nullable {
                        map.insert("type".to_string(), json!(["object", "null"]));
                    }
                }
                obj
            }

            Schema::Ref(r) => {
                // Convert internal refs to JSON Schema $ref format
                let ref_name = r
                    .reference
                    .split('/')
                    .next_back()
                    .unwrap_or(&r.reference)
                    .to_string();
                json!({ "$ref": format!("{ref_name}.json") })
            }

            Schema::OneOf(one) => {
                let schemas: Vec<Value> = one
                    .schemas
                    .iter()
                    .map(|s| self.schema_to_json_schema(s))
                    .collect();
                let mut obj = json!({ "oneOf": schemas });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = one.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if let Some(ref disc) = one.discriminator {
                        map.insert(
                            "discriminator".to_string(),
                            json!({
                                "propertyName": disc.property_name,
                                "mapping": disc.mapping
                            }),
                        );
                    }
                }
                obj
            }

            Schema::AllOf(all) => {
                let schemas: Vec<Value> = all
                    .schemas
                    .iter()
                    .map(|s| self.schema_to_json_schema(s))
                    .collect();
                let mut obj = json!({ "allOf": schemas });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = all.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                }
                obj
            }

            Schema::AnyOf(any) => {
                let schemas: Vec<Value> = any
                    .schemas
                    .iter()
                    .map(|s| self.schema_to_json_schema(s))
                    .collect();
                let mut obj = json!({ "anyOf": schemas });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = any.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                }
                obj
            }

            Schema::Enum(e) => {
                let values: Vec<Value> = e.values.iter().map(|v| v.value.clone()).collect();
                let mut obj = json!({ "enum": values });
                if let Value::Object(ref mut map) = obj {
                    if let Some(ref desc) = e.description {
                        map.insert("description".to_string(), json!(desc));
                    }
                    if e.nullable {
                        // Add null to enum values
                        if let Some(Value::Array(ref mut vec)) = map.get_mut("enum") {
                            vec.push(Value::Null);
                        }
                    }
                }
                obj
            }

            Schema::Null => json!({ "type": "null" }),
        }
    }
}

impl Default for JsonSchemaConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::schema::{
        ArraySchema, BooleanSchema, EnumSchema, EnumValue, IntegerSchema, NumberSchema,
        ObjectSchema, RefSchema, StringSchema,
    };

    #[test]
    fn test_string_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::String(StringSchema {
            description: Some("A name".to_string()),
            format: Some("email".to_string()),
            min_length: Some(1),
            max_length: Some(100),
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], "string");
        assert_eq!(result["description"], "A name");
        assert_eq!(result["format"], "email");
        assert_eq!(result["minLength"], 1);
        assert_eq!(result["maxLength"], 100);
    }

    #[test]
    fn test_integer_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::Integer(IntegerSchema {
            description: Some("Age".to_string()),
            format: Some("int32".to_string()),
            minimum: Some(0),
            maximum: Some(150),
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], "integer");
        assert_eq!(result["minimum"], 0);
        assert_eq!(result["maximum"], 150);
    }

    #[test]
    fn test_number_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::Number(NumberSchema {
            description: Some("Price".to_string()),
            format: Some("double".to_string()),
            minimum: Some(0.0),
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], "number");
        assert_eq!(result["format"], "double");
    }

    #[test]
    fn test_boolean_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::Boolean(BooleanSchema {
            description: Some("Is active".to_string()),
            default: Some(false),
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], "boolean");
        assert_eq!(result["default"], false);
    }

    #[test]
    fn test_array_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::Array(ArraySchema {
            description: Some("A list of items".to_string()),
            items: Box::new(Schema::String(StringSchema::default())),
            min_items: Some(1),
            max_items: Some(10),
            unique_items: true,
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], "array");
        assert_eq!(result["items"]["type"], "string");
        assert_eq!(result["minItems"], 1);
        assert_eq!(result["maxItems"], 10);
        assert_eq!(result["uniqueItems"], true);
    }

    #[test]
    fn test_object_schema() {
        let converter = JsonSchemaConverter::new();
        let mut properties = IndexMap::new();
        properties.insert("name".to_string(), Schema::String(StringSchema::default()));
        properties.insert(
            "age".to_string(),
            Schema::Integer(IntegerSchema::default()),
        );

        let schema = Schema::Object(ObjectSchema {
            description: Some("A user".to_string()),
            properties,
            required: vec!["name".to_string()],
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], "object");
        assert_eq!(result["properties"]["name"]["type"], "string");
        assert_eq!(result["properties"]["age"]["type"], "integer");
        assert_eq!(result["required"], json!(["name"]));
    }

    #[test]
    fn test_ref_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::Ref(RefSchema {
            reference: "#/components/schemas/User".to_string(),
        });

        let result = converter.convert(&schema);
        assert_eq!(result["$ref"], "User.json");
    }

    #[test]
    fn test_enum_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::Enum(EnumSchema {
            description: Some("Status values".to_string()),
            values: vec![
                EnumValue {
                    value: json!("active"),
                    description: None,
                },
                EnumValue {
                    value: json!("inactive"),
                    description: None,
                },
            ],
            nullable: false,
        });

        let result = converter.convert(&schema);
        assert_eq!(result["enum"], json!(["active", "inactive"]));
    }

    #[test]
    fn test_nullable_string() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::String(StringSchema {
            nullable: true,
            ..Default::default()
        });

        let result = converter.convert(&schema);
        assert_eq!(result["type"], json!(["string", "null"]));
    }

    #[test]
    fn test_root_schema() {
        let converter = JsonSchemaConverter::new();
        let schema = Schema::String(StringSchema::default());

        let mut schemas = IndexMap::new();
        schemas.insert("Name".to_string(), schema);

        let result = converter.convert_schemas(&schemas);
        let name_schema = result.get("Name").unwrap();

        assert_eq!(
            name_schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(name_schema["$id"], "Name.json");
        assert_eq!(name_schema["title"], "Name");
    }
}
