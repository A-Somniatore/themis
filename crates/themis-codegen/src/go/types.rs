//! Go type generation from Themis schemas.
//!
//! Converts Themis schema types to Go type definitions including:
//! - Primitive types (string, int64, float64, bool)
//! - Structs with JSON tags
//! - Slices for arrays
//! - Pointers for optional fields
//! - Interfaces for complex schemas

use indexmap::IndexMap;
use themis_core::schema::{ArraySchema, EnumSchema, ObjectSchema, RefSchema, Schema};

/// Generator for Go type definitions.
#[derive(Debug, Clone)]
pub struct GoTypeGenerator {
    /// Package name for generated types
    package_name: String,
}

impl GoTypeGenerator {
    /// Creates a new Go type generator.
    #[must_use]
    pub const fn new(package_name: String) -> Self {
        Self { package_name }
    }

    /// Generates Go type definitions from a set of schemas.
    pub fn generate_types(&self, schemas: &IndexMap<String, Schema>) -> String {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package_name));

        // Imports
        output.push_str("import (\n");
        output.push_str("\t\"encoding/json\"\n");
        output.push_str("\t\"time\"\n");
        output.push_str(")\n\n");

        // Silence unused import warnings
        output.push_str("var _ = json.Marshal\n");
        output.push_str("var _ = time.Now\n\n");

        // Generate each schema
        for (name, schema) in schemas {
            match schema {
                Schema::Object(obj) => {
                    output.push_str(&Self::generate_struct(name, obj));
                    output.push('\n');
                }
                Schema::Enum(e) => {
                    output.push_str(&Self::generate_enum(name, e));
                    output.push('\n');
                }
                Schema::Ref(r) => {
                    output.push_str(&Self::generate_type_alias(name, r));
                    output.push('\n');
                }
                _ => {
                    // For primitive types, create a type alias
                    let go_type = Self::schema_to_go_type(schema);
                    output.push_str(&format!(
                        "// {name} is a type alias.\ntype {name} = {go_type}\n\n"
                    ));
                }
            }
        }

        output
    }

    /// Generates a Go struct from an object schema.
    fn generate_struct(name: &str, schema: &ObjectSchema) -> String {
        let mut output = String::new();

        // Doc comment
        if let Some(desc) = &schema.description {
            output.push_str(&format!("// {name} {desc}\n"));
        } else {
            output.push_str(&format!("// {name} represents the {name} type.\n"));
        }

        output.push_str(&format!("type {name} struct {{\n"));

        for (field_name, field_schema) in &schema.properties {
            let go_type = Self::schema_to_go_type(field_schema);
            let go_field_name = to_pascal_case(field_name);
            let is_required = schema.required.contains(field_name);

            // Use pointer for optional fields
            let field_type = if is_required {
                go_type
            } else {
                format!("*{go_type}")
            };

            // Add omitempty for optional fields
            let json_tag = if is_required {
                format!("`json:\"{field_name}\"`")
            } else {
                format!("`json:\"{field_name},omitempty\"`")
            };

            // Field description
            if let Some(desc) = get_schema_description(field_schema) {
                output.push_str(&format!("\t// {go_field_name} {desc}\n"));
            }

            output.push_str(&format!("\t{go_field_name} {field_type} {json_tag}\n"));
        }

        output.push_str("}\n");
        output
    }

    /// Generates a Go enum (using const and type alias).
    fn generate_enum(name: &str, schema: &EnumSchema) -> String {
        let mut output = String::new();

        // Doc comment
        if let Some(desc) = &schema.description {
            output.push_str(&format!("// {name} {desc}\n"));
        } else {
            output.push_str(&format!("// {name} represents an enumeration.\n"));
        }

        // Type definition
        output.push_str(&format!("type {name} string\n\n"));

        // Constants
        output.push_str("const (\n");
        for enum_value in &schema.values {
            // Get the string value from the serde_json::Value
            let value_string = enum_value.value.to_string();
            let value_str = enum_value.value.as_str().unwrap_or(&value_string);
            let const_name = format!("{name}{}", to_pascal_case(value_str));
            output.push_str(&format!("\t{const_name} {name} = \"{value_str}\"\n"));
        }
        output.push_str(")\n\n");

        // Valid values slice
        output.push_str(&format!(
            "// {name}Values contains all valid {name} values.\n"
        ));
        output.push_str(&format!("var {name}Values = []{name}{{\n"));
        for enum_value in &schema.values {
            let value_string = enum_value.value.to_string();
            let value_str = enum_value.value.as_str().unwrap_or(&value_string);
            let const_name = format!("{name}{}", to_pascal_case(value_str));
            output.push_str(&format!("\t{const_name},\n"));
        }
        output.push_str("}\n\n");

        // IsValid method
        output.push_str(&format!(
            "// IsValid checks if the {name} value is valid.\n"
        ));
        output.push_str(&format!("func (e {name}) IsValid() bool {{\n"));
        output.push_str(&format!("\tfor _, v := range {name}Values {{\n"));
        output.push_str("\t\tif e == v {\n");
        output.push_str("\t\t\treturn true\n");
        output.push_str("\t\t}\n");
        output.push_str("\t}\n");
        output.push_str("\treturn false\n");
        output.push_str("}\n");

        output
    }

    /// Generates a type alias for a reference.
    fn generate_type_alias(name: &str, schema: &RefSchema) -> String {
        let ref_type = ref_to_go_type(&schema.reference);
        format!("// {name} is an alias for {ref_type}.\ntype {name} = {ref_type}\n")
    }

    /// Converts a Themis schema to a Go type string.
    #[must_use]
    pub fn schema_to_go_type(schema: &Schema) -> String {
        match schema {
            Schema::String(s) => match s.format.as_deref() {
                Some("date-time" | "date") => "time.Time".to_string(),
                Some("binary" | "byte") => "[]byte".to_string(),
                _ => "string".to_string(),
            },
            Schema::Integer(i) => match i.format.as_deref() {
                Some("int32") => "int32".to_string(),
                _ => "int64".to_string(),
            },
            Schema::Number(n) => match n.format.as_deref() {
                Some("float") => "float32".to_string(),
                _ => "float64".to_string(),
            },
            Schema::Boolean(_) => "bool".to_string(),
            Schema::Null | Schema::OneOf(_) | Schema::AllOf(_) | Schema::AnyOf(_) => {
                "interface{}".to_string()
            }
            Schema::Array(arr) => Self::array_to_go_type(arr),
            Schema::Object(_) => "map[string]interface{}".to_string(),
            Schema::Ref(r) => ref_to_go_type(&r.reference),
            Schema::Enum(_) => "string".to_string(),
        }
    }

    /// Converts an array schema to a Go slice type.
    fn array_to_go_type(schema: &ArraySchema) -> String {
        let items = Self::schema_to_go_type(&schema.items);
        format!("[]{items}")
    }
}

/// Converts a $ref path to a Go type name.
fn ref_to_go_type(reference: &str) -> String {
    // Extract type name from reference like "#/components/schemas/User"
    reference
        .rsplit('/')
        .next()
        .map_or_else(|| "interface{}".to_string(), to_pascal_case)
}

/// Converts a string to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    // Handle common acronyms
    result
        .replace("Id", "ID")
        .replace("Url", "URL")
        .replace("Uri", "URI")
        .replace("Http", "HTTP")
        .replace("Https", "HTTPS")
        .replace("Api", "API")
        .replace("Json", "JSON")
        .replace("Xml", "XML")
}

/// Gets the description from a schema if available.
fn get_schema_description(schema: &Schema) -> Option<&str> {
    match schema {
        Schema::String(s) => s.description.as_deref(),
        Schema::Integer(i) => i.description.as_deref(),
        Schema::Number(n) => n.description.as_deref(),
        Schema::Boolean(b) => b.description.as_deref(),
        Schema::Object(o) => o.description.as_deref(),
        Schema::Array(a) => a.description.as_deref(),
        Schema::Enum(e) => e.description.as_deref(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::schema::{BooleanSchema, IntegerSchema, StringSchema};

    #[test]
    fn test_primitive_type_mapping() {
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&Schema::String(StringSchema::default())),
            "string"
        );
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&Schema::Integer(IntegerSchema::default())),
            "int64"
        );
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&Schema::Boolean(BooleanSchema::default())),
            "bool"
        );
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&Schema::Null),
            "interface{}"
        );
    }

    #[test]
    fn test_integer_formats() {
        let int32 = Schema::Integer(IntegerSchema {
            format: Some("int32".to_string()),
            ..Default::default()
        });
        assert_eq!(GoTypeGenerator::schema_to_go_type(&int32), "int32");

        let int64 = Schema::Integer(IntegerSchema {
            format: Some("int64".to_string()),
            ..Default::default()
        });
        assert_eq!(GoTypeGenerator::schema_to_go_type(&int64), "int64");
    }

    #[test]
    fn test_string_formats() {
        let datetime = Schema::String(StringSchema {
            format: Some("date-time".to_string()),
            ..Default::default()
        });
        assert_eq!(GoTypeGenerator::schema_to_go_type(&datetime), "time.Time");

        let binary = Schema::String(StringSchema {
            format: Some("binary".to_string()),
            ..Default::default()
        });
        assert_eq!(GoTypeGenerator::schema_to_go_type(&binary), "[]byte");
    }

    #[test]
    fn test_array_type() {
        let string_array = Schema::Array(ArraySchema {
            items: Box::new(Schema::String(StringSchema::default())),
            ..Default::default()
        });
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&string_array),
            "[]string"
        );

        let nested_array = Schema::Array(ArraySchema {
            items: Box::new(Schema::Array(ArraySchema {
                items: Box::new(Schema::Integer(IntegerSchema::default())),
                ..Default::default()
            })),
            ..Default::default()
        });
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&nested_array),
            "[][]int64"
        );
    }

    #[test]
    fn test_nullable_via_schema() {
        // In the real schema, nullable is a field on the type, not a wrapper
        let nullable_string = Schema::String(StringSchema {
            nullable: true,
            ..Default::default()
        });
        // The type is still "string", nullable is handled at the struct level
        assert_eq!(
            GoTypeGenerator::schema_to_go_type(&nullable_string),
            "string"
        );
    }

    #[test]
    fn test_reference_type() {
        let ref_schema = Schema::Ref(RefSchema {
            reference: "#/components/schemas/User".to_string(),
        });
        assert_eq!(GoTypeGenerator::schema_to_go_type(&ref_schema), "User");
    }

    #[test]
    fn test_generate_struct() {
        let mut properties = IndexMap::new();
        properties.insert(
            "id".to_string(),
            Schema::Integer(IntegerSchema {
                format: Some("int64".to_string()),
                description: Some("User ID".to_string()),
                ..Default::default()
            }),
        );
        properties.insert(
            "name".to_string(),
            Schema::String(StringSchema {
                description: Some("User name".to_string()),
                ..Default::default()
            }),
        );
        properties.insert("email".to_string(), Schema::String(StringSchema::default()));

        let obj = ObjectSchema {
            properties,
            required: vec!["id".to_string(), "name".to_string()],
            description: Some("represents a user in the system.".to_string()),
            ..Default::default()
        };

        let output = GoTypeGenerator::generate_struct("User", &obj);

        assert!(output.contains("type User struct {"));
        assert!(output.contains("ID int64 `json:\"id\"`"));
        assert!(output.contains("Name string `json:\"name\"`"));
        assert!(output.contains("Email *string `json:\"email,omitempty\"`"));
        assert!(output.contains("// User represents a user in the system."));
    }

    #[test]
    fn test_generate_enum() {
        use themis_core::schema::EnumValue;

        let enum_schema = EnumSchema {
            values: vec![
                EnumValue {
                    value: serde_json::Value::String("pending".to_string()),
                    description: None,
                },
                EnumValue {
                    value: serde_json::Value::String("active".to_string()),
                    description: None,
                },
                EnumValue {
                    value: serde_json::Value::String("completed".to_string()),
                    description: None,
                },
            ],
            description: Some("represents task status.".to_string()),
            nullable: false,
        };

        let output = GoTypeGenerator::generate_enum("Status", &enum_schema);

        assert!(output.contains("type Status string"));
        assert!(output.contains("StatusPending Status = \"pending\""));
        assert!(output.contains("StatusActive Status = \"active\""));
        assert!(output.contains("StatusCompleted Status = \"completed\""));
        assert!(output.contains("var StatusValues = []Status{"));
        assert!(output.contains("func (e Status) IsValid() bool {"));
    }

    #[test]
    fn test_pascal_case_conversion() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("user-id"), "UserID");
        assert_eq!(to_pascal_case("api_url"), "APIURL");
        assert_eq!(to_pascal_case("http_request"), "HTTPRequest");
    }

    #[test]
    fn test_generate_types() {
        let gen = GoTypeGenerator::new("models".to_string());

        let mut schemas = IndexMap::new();
        schemas.insert(
            "User".to_string(),
            Schema::Object(ObjectSchema {
                properties: {
                    let mut props = IndexMap::new();
                    props.insert("id".to_string(), Schema::Integer(IntegerSchema::default()));
                    props
                },
                required: vec!["id".to_string()],
                ..Default::default()
            }),
        );

        let output = gen.generate_types(&schemas);

        assert!(output.contains("package models"));
        assert!(output.contains("import ("));
        assert!(output.contains("type User struct {"));
    }
}
