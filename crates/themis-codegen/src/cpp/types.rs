//! C++ type generation utilities.

use crate::config::{GeneratorConfig, NamingConvention};
use crate::error::CodegenResult;
use heck::{ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use std::fmt::Write;
use themis_core::schema::Schema;

/// Generates C++ types from Themis schemas.
pub struct CppTypeGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> CppTypeGenerator<'a> {
    /// Creates a new C++ type generator.
    pub const fn new(config: &'a GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generates all types from a schema map.
    pub fn generate_types(&self, schemas: &IndexMap<String, Schema>) -> CodegenResult<String> {
        let mut output = String::new();

        for (name, schema) in schemas {
            let type_code = self.generate_type(name, schema)?;
            output.push_str(&type_code);
            output.push('\n');
        }

        Ok(output)
    }

    /// Generates a single type from a schema.
    pub fn generate_type(&self, name: &str, schema: &Schema) -> CodegenResult<String> {
        let type_name = self.format_type_name(name);

        match schema {
            Schema::Object(obj) => self.generate_struct(&type_name, obj, schema),
            Schema::Enum(enum_schema) => Ok(self.generate_enum(&type_name, enum_schema)),
            Schema::AllOf(all_of) => self.generate_all_of(&type_name, all_of),
            Schema::OneOf(one_of) => self.generate_one_of(&type_name, one_of),
            Schema::AnyOf(any_of) => self.generate_any_of(&type_name, any_of),
            Schema::Array(arr) => self.generate_type_alias(&type_name, &arr.items, true),
            Schema::String(_)
            | Schema::Integer(_)
            | Schema::Number(_)
            | Schema::Boolean(_)
            | Schema::Null => self.generate_primitive_alias(&type_name, schema),
            Schema::Ref(ref_schema) => {
                Ok(self.generate_reference_alias(&type_name, &ref_schema.reference))
            }
        }
    }
    /// Generates a C++ struct from an object schema.
    fn generate_struct(
        &self,
        name: &str,
        obj: &themis_core::schema::ObjectSchema,
        schema: &Schema,
    ) -> CodegenResult<String> {
        let mut output = String::new();

        // Doc comment
        if self.config.include_docs {
            if let Some(desc) = schema.description() {
                let _ = writeln!(output, "/**");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
                let _ = writeln!(output, " */");
            }
        }

        let _ = writeln!(output, "struct {name} {{");

        // Generate fields
        for (field_name, prop_schema) in &obj.properties {
            let cpp_type = self.schema_to_cpp_type(prop_schema)?;
            let field = self.format_field_name(field_name);
            let is_required = obj.required.contains(field_name);

            // Doc comment for field
            if self.config.include_docs {
                if let Some(desc) = prop_schema.description() {
                    let _ = writeln!(output, "    /** {desc} */");
                }
            }

            // Use std::optional for non-required fields
            if is_required {
                let _ = writeln!(output, "    {cpp_type} {field};");
            } else {
                let _ = writeln!(output, "    std::optional<{cpp_type}> {field};");
            }
        }

        // Generate default constructor
        output.push('\n');
        let _ = writeln!(output, "    {name}() = default;");

        // Generate comparison operators (C++20 spaceship)
        let _ = writeln!(output, "    auto operator<=>(const {name}&) const = default;");

        let _ = writeln!(output, "}};");

        Ok(output)
    }

    /// Generates a C++ enum from an enum schema.
    fn generate_enum(
        &self,
        name: &str,
        enum_schema: &themis_core::schema::EnumSchema,
    ) -> String {
        let mut output = String::new();

        if self.config.include_docs {
            if let Some(desc) = &enum_schema.description {
                let _ = writeln!(output, "/**");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
                let _ = writeln!(output, " */");
            }
        }

        let _ = writeln!(output, "enum class {name} {{");

        for (i, value) in enum_schema.values.iter().enumerate() {
            // Convert JSON value to string for enum variant name
            let variant = match &value.value {
                serde_json::Value::String(s) => s.to_upper_camel_case(),
                serde_json::Value::Number(n) => format!("Value{n}"),
                _ => format!("Value{i}"),
            };

            if let Some(desc) = &value.description {
                let _ = writeln!(output, "    /** {desc} */");
            }

            if i < enum_schema.values.len() - 1 {
                let _ = writeln!(output, "    {variant},");
            } else {
                let _ = writeln!(output, "    {variant}");
            }
        }

        let _ = writeln!(output, "}};");

        // Generate to_string helper
        output.push('\n');
        let _ = writeln!(
            output,
            "inline std::string to_string(const {name}& value) {{"
        );
        let _ = writeln!(output, "    switch (value) {{");
        for value in &enum_schema.values {
            let variant = match &value.value {
                serde_json::Value::String(s) => s.to_upper_camel_case(),
                serde_json::Value::Number(n) => format!("Value{n}"),
                _ => continue,
            };
            let string_val = match &value.value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };
            let _ = writeln!(
                output,
                "        case {name}::{variant}: return \"{string_val}\";"
            );
        }
        let _ = writeln!(output, "        default: return \"unknown\";");
        let _ = writeln!(output, "    }}");
        let _ = writeln!(output, "}}");

        output
    }

    /// Generates a type for `allOf` schema.
    fn generate_all_of(
        &self,
        name: &str,
        all_of: &themis_core::schema::AllOfSchema,
    ) -> CodegenResult<String> {
        let mut output = String::new();
        let mut combined_props: IndexMap<String, Schema> = IndexMap::new();
        let mut combined_required: Vec<String> = Vec::new();

        // Combine all properties from referenced schemas
        for schema in &all_of.schemas {
            if let Schema::Object(obj) = schema {
                combined_props.extend(obj.properties.clone());
                combined_required.extend(obj.required.clone());
            }
        }

        // Generate combined struct
        if self.config.include_docs {
            if let Some(desc) = &all_of.description {
                let _ = writeln!(output, "/**");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
                let _ = writeln!(output, " */");
            }
        }

        let _ = writeln!(output, "struct {name} {{");

        for (field_name, prop_schema) in &combined_props {
            let cpp_type = self.schema_to_cpp_type(prop_schema)?;
            let field = self.format_field_name(field_name);
            let is_required = combined_required.iter().any(|r| r == field_name);

            if is_required {
                let _ = writeln!(output, "    {cpp_type} {field};");
            } else {
                let _ = writeln!(output, "    std::optional<{cpp_type}> {field};");
            }
        }

        output.push('\n');
        let _ = writeln!(output, "    {name}() = default;");
        let _ = writeln!(output, "    auto operator<=>(const {name}&) const = default;");
        let _ = writeln!(output, "}};");

        Ok(output)
    }

    /// Generates a variant type for `oneOf` schema using `std::variant`.
    fn generate_one_of(
        &self,
        name: &str,
        one_of: &themis_core::schema::OneOfSchema,
    ) -> CodegenResult<String> {
        let mut output = String::new();

        if self.config.include_docs {
            if let Some(desc) = &one_of.description {
                let _ = writeln!(output, "/**");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
                let _ = writeln!(output, " */");
            }
        }

        let mut variant_types: Vec<String> = Vec::new();
        for schema in &one_of.schemas {
            let cpp_type = self.schema_to_cpp_type(schema)?;
            variant_types.push(cpp_type);
        }

        let variants = variant_types.join(", ");
        let _ = writeln!(output, "using {name} = std::variant<{variants}>;");

        Ok(output)
    }

    /// Generates a variant type for `anyOf` schema.
    fn generate_any_of(
        &self,
        name: &str,
        any_of: &themis_core::schema::AnyOfSchema,
    ) -> CodegenResult<String> {
        let mut output = String::new();

        if self.config.include_docs {
            if let Some(desc) = &any_of.description {
                let _ = writeln!(output, "/**");
                for line in desc.lines() {
                    let _ = writeln!(output, " * {line}");
                }
                let _ = writeln!(output, " */");
            }
        }

        let mut variant_types: Vec<String> = Vec::new();
        for schema in &any_of.schemas {
            let cpp_type = self.schema_to_cpp_type(schema)?;
            variant_types.push(cpp_type);
        }

        let variants = variant_types.join(", ");
        let _ = writeln!(output, "using {name} = std::variant<{variants}>;");

        Ok(output)
    }

    /// Generates a type alias for array types.
    fn generate_type_alias(
        &self,
        name: &str,
        item_schema: &Schema,
        _is_array: bool,
    ) -> CodegenResult<String> {
        let item_type = self.schema_to_cpp_type(item_schema)?;
        Ok(format!("using {name} = std::vector<{item_type}>;\n"))
    }

    /// Generates a type alias for primitive types.
    fn generate_primitive_alias(&self, name: &str, schema: &Schema) -> CodegenResult<String> {
        let cpp_type = self.schema_to_cpp_type(schema)?;
        Ok(format!("using {name} = {cpp_type};\n"))
    }

    /// Generates a type alias for references.
    fn generate_reference_alias(&self, name: &str, ref_path: &str) -> String {
        let ref_type = self.ref_to_cpp_type(ref_path);
        format!("using {name} = {ref_type};\n")
    }

    /// Converts a schema to a C++ type string.
    pub fn schema_to_cpp_type(&self, schema: &Schema) -> CodegenResult<String> {
        let cpp_type = match schema {
            Schema::String(s) => {
                if s.format.as_deref() == Some("date-time") {
                    "std::chrono::system_clock::time_point".to_string()
                } else if s.format.as_deref() == Some("date") {
                    "std::chrono::year_month_day".to_string()
                } else if s.format.as_deref() == Some("uuid") {
                    "std::string".to_string() // Could use boost::uuids::uuid
                } else if s.format.as_deref() == Some("binary") {
                    "std::vector<uint8_t>".to_string()
                } else {
                    "std::string".to_string()
                }
            }
            Schema::Integer(i) => {
                match i.format.as_deref() {
                    Some("int32") => "int32_t",
                    _ => "int64_t", // Default to int64
                }
                .to_string()
            }
            Schema::Number(n) => {
                match n.format.as_deref() {
                    Some("float") => "float",
                    _ => "double", // Default to double
                }
                .to_string()
            }
            Schema::Boolean(_) => "bool".to_string(),
            Schema::Null => "std::nullptr_t".to_string(),
            Schema::Array(arr) => {
                let item_type = self.schema_to_cpp_type(&arr.items)?;
                format!("std::vector<{item_type}>")
            }
            Schema::Object(_) => {
                // Inline object - use JSON for complex cases
                "nlohmann::json".to_string()
            }
            Schema::Ref(ref_schema) => self.ref_to_cpp_type(&ref_schema.reference),
            Schema::Enum(_) => "std::string".to_string(), // Enums need special handling
            Schema::AllOf(_) | Schema::OneOf(_) | Schema::AnyOf(_) => {
                "nlohmann::json".to_string()
            }
        };

        // Handle nullable
        if schema.is_nullable() {
            Ok(format!("std::optional<{cpp_type}>"))
        } else {
            Ok(cpp_type)
        }
    }

    /// Converts a $ref path to a C++ type.
    fn ref_to_cpp_type(&self, ref_path: &str) -> String {
        // Handle #/components/schemas/TypeName format
        ref_path
            .strip_prefix("#/components/schemas/")
            .or_else(|| ref_path.strip_prefix("#/definitions/"))
            .map_or_else(
                || self.format_type_name(ref_path),
                |type_name| self.format_type_name(type_name),
            )
    }

    /// Formats a type name according to configuration.
    fn format_type_name(&self, name: &str) -> String {
        let formatted = match self.config.type_naming {
            NamingConvention::PascalCase => name.to_upper_camel_case(),
            NamingConvention::CamelCase => {
                let pascal = name.to_upper_camel_case();
                let mut chars = pascal.chars();
                chars.next().map_or_else(String::new, |c| {
                    c.to_lowercase().collect::<String>() + chars.as_str()
                })
            }
            NamingConvention::SnakeCase => name.to_snake_case(),
            NamingConvention::ScreamingSnakeCase => name.to_snake_case().to_uppercase(),
            NamingConvention::KebabCase => name.to_snake_case().replace('_', "-"),
        };

        // Apply prefix/suffix
        let with_prefix = match &self.config.type_prefix {
            Some(p) => format!("{p}{formatted}"),
            None => formatted,
        };

        match &self.config.type_suffix {
            Some(s) => format!("{with_prefix}{s}"),
            None => with_prefix,
        }
    }

    /// Formats a field name according to configuration.
    fn format_field_name(&self, name: &str) -> String {
        match self.config.field_naming {
            NamingConvention::PascalCase => name.to_upper_camel_case(),
            NamingConvention::CamelCase => {
                let pascal = name.to_upper_camel_case();
                let mut chars = pascal.chars();
                chars.next().map_or_else(String::new, |c| {
                    c.to_lowercase().collect::<String>() + chars.as_str()
                })
            }
            NamingConvention::SnakeCase => name.to_snake_case(),
            NamingConvention::ScreamingSnakeCase => name.to_snake_case().to_uppercase(),
            NamingConvention::KebabCase => name.to_snake_case().replace('_', "-"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::schema::{EnumSchema, EnumValue, ObjectSchema, StringSchema};

    fn default_config() -> GeneratorConfig {
        GeneratorConfig::default()
    }

    #[test]
    fn test_primitive_types() {
        let config = default_config();
        let gen = CppTypeGenerator::new(&config);

        let string_schema = Schema::String(StringSchema {
            format: None,
            pattern: None,
            min_length: None,
            max_length: None,
            description: None,
            nullable: false,
            default: None,
        });
        assert_eq!(gen.schema_to_cpp_type(&string_schema).unwrap(), "std::string");

        let int_schema = Schema::Integer(themis_core::schema::IntegerSchema {
            format: Some("int32".to_string()),
            minimum: None,
            maximum: None,
            description: None,
            nullable: false,
            default: None,
        });
        assert_eq!(gen.schema_to_cpp_type(&int_schema).unwrap(), "int32_t");

        let bool_schema = Schema::Boolean(themis_core::schema::BooleanSchema {
            description: None,
            nullable: false,
            default: None,
        });
        assert_eq!(gen.schema_to_cpp_type(&bool_schema).unwrap(), "bool");
    }

    #[test]
    fn test_array_type() {
        let config = default_config();
        let gen = CppTypeGenerator::new(&config);

        let arr_schema = Schema::Array(themis_core::schema::ArraySchema {
            items: Box::new(Schema::String(StringSchema {
                format: None,
                pattern: None,
                min_length: None,
                max_length: None,
                description: None,
                nullable: false,
                default: None,
            })),
            min_items: None,
            max_items: None,
            unique_items: false,
            description: None,
            nullable: false,
        });

        assert_eq!(
            gen.schema_to_cpp_type(&arr_schema).unwrap(),
            "std::vector<std::string>"
        );
    }

    #[test]
    fn test_nullable_type() {
        let config = default_config();
        let gen = CppTypeGenerator::new(&config);

        let nullable_string = Schema::String(StringSchema {
            format: None,
            pattern: None,
            min_length: None,
            max_length: None,
            description: None,
            nullable: true,
            default: None,
        });

        assert_eq!(
            gen.schema_to_cpp_type(&nullable_string).unwrap(),
            "std::optional<std::string>"
        );
    }

    #[test]
    fn test_generate_struct() {
        let config = default_config();
        let gen = CppTypeGenerator::new(&config);

        let mut properties = IndexMap::new();
        properties.insert(
            "user_name".to_string(),
            Schema::String(StringSchema {
                format: None,
                pattern: None,
                min_length: None,
                max_length: None,
                description: Some("The user's name".to_string()),
                nullable: false,
                default: None,
            }),
        );
        properties.insert(
            "age".to_string(),
            Schema::Integer(themis_core::schema::IntegerSchema {
                format: None,
                minimum: None,
                maximum: None,
                description: None,
                nullable: false,
                default: None,
            }),
        );

        let obj = ObjectSchema {
            properties,
            required: vec!["user_name".to_string()],
            additional_properties: None,
            description: Some("A user object".to_string()),
            nullable: false,
        };

        let schema = Schema::Object(obj.clone());
        let result = gen.generate_struct("User", &obj, &schema).unwrap();

        assert!(result.contains("struct User"));
        assert!(result.contains("std::string user_name;"));
        assert!(result.contains("std::optional<int64_t> age;"));
    }

    #[test]
    fn test_generate_enum() {
        let config = default_config();
        let gen = CppTypeGenerator::new(&config);

        let enum_schema = EnumSchema {
            values: vec![
                EnumValue {
                    value: serde_json::Value::String("active".to_string()),
                    description: Some("User is active".to_string()),
                },
                EnumValue {
                    value: serde_json::Value::String("inactive".to_string()),
                    description: None,
                },
            ],
            description: Some("User status".to_string()),
            nullable: false,
        };

        let result = gen.generate_enum("Status", &enum_schema);

        assert!(result.contains("enum class Status"));
        assert!(result.contains("Active"));
        assert!(result.contains("Inactive"));
        assert!(result.contains("to_string"));
    }
}
