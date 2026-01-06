//! Python type generation from schemas.

// Allow some pedantic clippy lints that are acceptable in generated code
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::unused_self)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_wraps)]

use crate::config::GeneratorConfig;
use crate::error::CodegenResult;
use heck::{ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::Write;
use themis_core::schema::{
    AllOfSchema, AnyOfSchema, ArraySchema, EnumSchema, ObjectSchema, OneOfSchema, Schema,
};

/// Generates Python type definitions from schemas.
pub struct PythonTypeGenerator<'a> {
    config: &'a GeneratorConfig,
    /// Maps schema names to their generated Python type names
    type_map: HashMap<String, String>,
}

impl<'a> PythonTypeGenerator<'a> {
    /// Creates a new type generator.
    pub fn new(config: &'a GeneratorConfig) -> Self {
        Self {
            config,
            type_map: HashMap::new(),
        }
    }

    /// Generates Python dataclasses from a map of named schemas.
    pub fn generate_types(&mut self, schemas: &IndexMap<String, Schema>) -> CodegenResult<String> {
        let mut output = String::new();

        // Generate imports
        output.push_str("from __future__ import annotations\n\n");
        output.push_str("from dataclasses import dataclass, field\n");
        output.push_str("from datetime import datetime\n");
        output.push_str("from enum import Enum\n");
        output.push_str("from typing import Any, Optional, Union\n");
        output.push_str("from uuid import UUID\n");
        output.push_str("\n\n");

        // First pass: register all type names
        for name in schemas.keys() {
            let py_name = self.schema_name_to_py_type(name);
            self.type_map.insert(name.clone(), py_name);
        }

        // Second pass: generate types
        for (name, schema) in schemas {
            let type_code = self.generate_schema_type(name, schema)?;
            output.push_str(&type_code);
            output.push_str("\n\n");
        }

        Ok(output)
    }

    /// Generates a Python type for a named schema.
    fn generate_schema_type(&self, name: &str, schema: &Schema) -> CodegenResult<String> {
        let py_name = self.schema_name_to_py_type(name);

        match schema {
            Schema::Object(obj) => self.generate_dataclass(&py_name, obj),
            Schema::Enum(enum_schema) => Ok(self.generate_enum(&py_name, enum_schema)),
            Schema::OneOf(one_of) => Ok(self.generate_union_type(&py_name, one_of)),
            Schema::AllOf(all_of) => self.generate_merged_type(&py_name, all_of),
            Schema::AnyOf(any_of) => Ok(self.generate_any_of_union(&py_name, any_of)),
            _ => {
                // For simple types, generate a type alias
                let inner_type = self.schema_to_py_type(schema)?;
                Ok(format!("{py_name} = {inner_type}"))
            }
        }
    }

    /// Generates a Python dataclass from an object schema.
    fn generate_dataclass(&self, name: &str, obj: &ObjectSchema) -> CodegenResult<String> {
        let mut output = String::new();

        // Docstring
        if let Some(desc) = &obj.description {
            if self.config.include_docs {
                // We'll add the docstring inside the class
            }
            let _ = desc; // Use later
        }

        // Dataclass decorator
        output.push_str("@dataclass\n");
        let _ = writeln!(output, "class {name}:");

        // Docstring inside class
        if self.config.include_docs {
            if let Some(desc) = &obj.description {
                output.push_str(&format_docstring(desc, 1));
            }
        }

        // Collect required and optional properties
        let mut required_props: Vec<(&String, &Schema)> = Vec::new();
        let mut optional_props: Vec<(&String, &Schema)> = Vec::new();

        for (prop_name, prop_schema) in &obj.properties {
            if obj.required.contains(prop_name) {
                required_props.push((prop_name, prop_schema));
            } else {
                optional_props.push((prop_name, prop_schema));
            }
        }

        // Sort for deterministic output
        required_props.sort_by_key(|(name, _)| *name);
        optional_props.sort_by_key(|(name, _)| *name);

        // Generate required properties first (Python requires this order)
        for (prop_name, prop_schema) in &required_props {
            let py_name = prop_name.to_snake_case();
            let py_type = self.schema_to_py_type(prop_schema)?;

            // Property docstring as comment
            if self.config.include_docs {
                if let Some(desc) = prop_schema.description() {
                    let _ = writeln!(output, "    # {desc}");
                }
            }
            let _ = writeln!(output, "    {py_name}: {py_type}");
        }

        // Generate optional properties with defaults
        for (prop_name, prop_schema) in &optional_props {
            let py_name = prop_name.to_snake_case();
            let py_type = self.schema_to_py_type(prop_schema)?;

            // Property docstring as comment
            if self.config.include_docs {
                if let Some(desc) = prop_schema.description() {
                    let _ = writeln!(output, "    # {desc}");
                }
            }
            let _ = writeln!(output, "    {py_name}: Optional[{py_type}] = None");
        }

        // If no properties, add pass
        if obj.properties.is_empty() {
            output.push_str("    pass\n");
        }

        Ok(output)
    }

    /// Generates a Python Enum from an enum schema.
    fn generate_enum(&self, name: &str, enum_schema: &EnumSchema) -> String {
        let mut output = String::new();

        let _ = writeln!(output, "class {name}(str, Enum):");

        // Docstring
        if self.config.include_docs {
            if let Some(desc) = &enum_schema.description {
                output.push_str(&format_docstring(desc, 1));
            }
        }

        // Generate enum values
        for value in &enum_schema.values {
            let value_str = match &value.value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            // Python enum member names should be uppercase snake_case
            let member_name = value_str.to_uppercase().replace('-', "_");
            let _ = writeln!(output, "    {member_name} = \"{value_str}\"");
        }

        output
    }

    /// Generates a Union type from oneOf schema.
    fn generate_union_type(&self, name: &str, one_of: &OneOfSchema) -> String {
        let mut output = String::new();

        if self.config.include_docs {
            if let Some(desc) = &one_of.description {
                output.push_str(&format!("# {desc}\n"));
            }
        }

        let variants: Vec<String> = one_of
            .schemas
            .iter()
            .filter_map(|s| self.schema_to_py_type(s).ok())
            .collect();

        let _ = writeln!(output, "{name} = Union[{}]", variants.join(", "));
        output
    }

    /// Generates a merged dataclass from allOf schema.
    fn generate_merged_type(&self, name: &str, all_of: &AllOfSchema) -> CodegenResult<String> {
        let mut output = String::new();

        // For allOf, we create a dataclass that inherits from all refs
        // or merges properties from inline objects

        // Collect all properties from all schemas
        let mut all_properties: IndexMap<String, Schema> = IndexMap::new();
        let mut all_required: Vec<String> = Vec::new();
        let mut base_classes: Vec<String> = Vec::new();

        for schema in &all_of.schemas {
            match schema {
                Schema::Ref(r) => {
                    let type_name = r
                        .reference
                        .split('/')
                        .next_back()
                        .unwrap_or(&r.reference)
                        .to_upper_camel_case();
                    base_classes.push(type_name);
                }
                Schema::Object(obj) => {
                    for (prop_name, prop_schema) in &obj.properties {
                        all_properties.insert(prop_name.clone(), prop_schema.clone());
                    }
                    all_required.extend(obj.required.clone());
                }
                _ => {}
            }
        }

        // Generate dataclass with inheritance if there are base classes
        output.push_str("@dataclass\n");
        if base_classes.is_empty() {
            let _ = writeln!(output, "class {name}:");
        } else {
            let _ = writeln!(output, "class {name}({}):", base_classes.join(", "));
        }

        if self.config.include_docs {
            if let Some(desc) = &all_of.description {
                output.push_str(&format_docstring(desc, 1));
            }
        }

        // Sort properties for deterministic output
        let mut sorted_props: Vec<_> = all_properties.iter().collect();
        sorted_props.sort_by_key(|(name, _)| *name);

        // Generate properties (required first)
        let mut required_written = false;
        for (prop_name, prop_schema) in &sorted_props {
            if all_required.contains(prop_name) {
                let py_name = prop_name.to_snake_case();
                let py_type = self.schema_to_py_type(prop_schema)?;
                let _ = writeln!(output, "    {py_name}: {py_type}");
                required_written = true;
            }
        }

        // Then optional
        for (prop_name, prop_schema) in &sorted_props {
            if !all_required.contains(prop_name) {
                let py_name = prop_name.to_snake_case();
                let py_type = self.schema_to_py_type(prop_schema)?;
                let _ = writeln!(output, "    {py_name}: Optional[{py_type}] = None");
                required_written = true;
            }
        }

        if !required_written && base_classes.is_empty() {
            output.push_str("    pass\n");
        }

        Ok(output)
    }

    /// Generates a Union type from anyOf schema.
    fn generate_any_of_union(&self, name: &str, any_of: &AnyOfSchema) -> String {
        let mut output = String::new();

        if self.config.include_docs {
            if let Some(desc) = &any_of.description {
                output.push_str(&format!("# {desc}\n"));
            }
        }

        let variants: Vec<String> = any_of
            .schemas
            .iter()
            .filter_map(|s| self.schema_to_py_type(s).ok())
            .collect();

        let _ = writeln!(output, "{name} = Union[{}]", variants.join(", "));
        output
    }

    /// Converts an OpenAPI schema name to a Python type name.
    fn schema_name_to_py_type(&self, name: &str) -> String {
        name.to_upper_camel_case()
    }

    /// Converts a schema to its Python type representation.
    pub fn schema_to_py_type(&self, schema: &Schema) -> CodegenResult<String> {
        match schema {
            Schema::String(s) => Ok(self.string_schema_to_py_type(s)),
            Schema::Integer(_) => Ok("int".to_string()),
            Schema::Number(_) => Ok("float".to_string()),
            Schema::Boolean(_) => Ok("bool".to_string()),
            Schema::Array(arr) => self.array_schema_to_py_type(arr),
            Schema::Object(obj) => {
                if obj.properties.is_empty() && obj.additional_properties.is_none() {
                    Ok("dict[str, Any]".to_string())
                } else if obj.properties.is_empty() {
                    // additionalProperties case - use dict
                    if let Some(additional) = &obj.additional_properties {
                        let value_type = self.schema_to_py_type(additional)?;
                        Ok(format!("dict[str, {value_type}]"))
                    } else {
                        Ok("dict[str, Any]".to_string())
                    }
                } else {
                    // Inline object - generate as dict for simplicity
                    self.generate_inline_object(obj)
                }
            }
            Schema::Ref(r) => {
                // Extract the type name from the ref
                let type_name = r
                    .reference
                    .split('/')
                    .next_back()
                    .unwrap_or(&r.reference)
                    .to_upper_camel_case();
                Ok(type_name)
            }
            Schema::Enum(e) => {
                // For inline enums, use Literal type
                let variants: Vec<String> = e
                    .values
                    .iter()
                    .map(|v| match &v.value {
                        serde_json::Value::String(s) => format!("\"{s}\""),
                        other => format!("\"{other}\""),
                    })
                    .collect();
                Ok(format!("Literal[{}]", variants.join(", ")))
            }
            Schema::OneOf(one_of) => {
                let variants: Vec<String> = one_of
                    .schemas
                    .iter()
                    .filter_map(|s| self.schema_to_py_type(s).ok())
                    .collect();
                Ok(format!("Union[{}]", variants.join(", ")))
            }
            Schema::AllOf(all_of) => {
                // For inline allOf, just use the first concrete type or Any
                for schema in &all_of.schemas {
                    if let Schema::Ref(r) = schema {
                        let type_name = r
                            .reference
                            .split('/')
                            .next_back()
                            .unwrap_or(&r.reference)
                            .to_upper_camel_case();
                        return Ok(type_name);
                    }
                }
                Ok("Any".to_string())
            }
            Schema::AnyOf(any_of) => {
                let variants: Vec<String> = any_of
                    .schemas
                    .iter()
                    .filter_map(|s| self.schema_to_py_type(s).ok())
                    .collect();
                Ok(format!("Union[{}]", variants.join(", ")))
            }
            Schema::Null => Ok("None".to_string()),
        }
    }

    /// Converts a string schema to Python type.
    fn string_schema_to_py_type(&self, s: &themis_core::schema::StringSchema) -> String {
        // Check for common formats
        if let Some(format) = &s.format {
            match format.as_str() {
                // Python uses datetime for both date-time and date
                "date-time" | "date" => return "datetime".to_string(),
                "uuid" => return "UUID".to_string(),
                "binary" | "byte" => return "bytes".to_string(),
                _ => {}
            }
        }
        "str".to_string()
    }

    /// Converts an array schema to Python type.
    fn array_schema_to_py_type(&self, arr: &ArraySchema) -> CodegenResult<String> {
        let item_type = self.schema_to_py_type(&arr.items)?;
        Ok(format!("list[{item_type}]"))
    }

    /// Generates an inline object type as a TypedDict-like dict.
    fn generate_inline_object(&self, obj: &ObjectSchema) -> CodegenResult<String> {
        // For complex inline objects, we just use dict[str, Any]
        // A more sophisticated implementation could generate TypedDict
        if obj.properties.len() > 3 {
            return Ok("dict[str, Any]".to_string());
        }

        // Generate type information for each property for documentation
        // but Python doesn't have a literal dict type syntax in typing,
        // so we return dict[str, Any] for inline objects
        for (_prop_name, prop_schema) in &obj.properties {
            // Validate that we can convert each property schema
            let _py_type = self.schema_to_py_type(prop_schema)?;
        }
        // For inline, we just use dict[str, Any] or a TypedDict would be needed
        Ok("dict[str, Any]".to_string())
    }
}

/// Formats a description as a Python docstring.
pub fn format_docstring(description: &str, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);
    let lines: Vec<&str> = description.lines().collect();

    if lines.len() == 1 && lines[0].len() < 70 {
        format!("{indent}\"\"\"{}\"\"\"\n", lines[0])
    } else {
        let mut output = format!("{indent}\"\"\"\n");
        for line in lines {
            let _ = writeln!(output, "{indent}{line}");
        }
        let _ = writeln!(output, "{indent}\"\"\"");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::schema::{EnumValue, StringSchema};

    #[test]
    fn test_string_schema_to_py_type() {
        let config = GeneratorConfig::default();
        let gen = PythonTypeGenerator::new(&config);

        let schema = StringSchema::default();
        assert_eq!(gen.string_schema_to_py_type(&schema), "str");
    }

    #[test]
    fn test_datetime_format_to_py_type() {
        let config = GeneratorConfig::default();
        let gen = PythonTypeGenerator::new(&config);

        let schema = StringSchema {
            format: Some("date-time".to_string()),
            ..Default::default()
        };
        assert_eq!(gen.string_schema_to_py_type(&schema), "datetime");
    }

    #[test]
    fn test_uuid_format_to_py_type() {
        let config = GeneratorConfig::default();
        let gen = PythonTypeGenerator::new(&config);

        let schema = StringSchema {
            format: Some("uuid".to_string()),
            ..Default::default()
        };
        assert_eq!(gen.string_schema_to_py_type(&schema), "UUID");
    }

    #[test]
    fn test_array_schema_to_py_type() {
        let config = GeneratorConfig::default();
        let gen = PythonTypeGenerator::new(&config);

        let arr = ArraySchema {
            description: None,
            items: Box::new(Schema::String(StringSchema::default())),
            min_items: None,
            max_items: None,
            unique_items: false,
            nullable: false,
        };
        assert_eq!(gen.array_schema_to_py_type(&arr).unwrap(), "list[str]");
    }

    #[test]
    fn test_ref_schema_to_py_type() {
        let config = GeneratorConfig::default();
        let gen = PythonTypeGenerator::new(&config);

        let schema = Schema::Ref(themis_core::schema::RefSchema {
            reference: "#/components/schemas/User".to_string(),
        });
        assert_eq!(gen.schema_to_py_type(&schema).unwrap(), "User");
    }

    #[test]
    fn test_generate_enum() {
        let config = GeneratorConfig::default();
        let gen = PythonTypeGenerator::new(&config);

        let enum_schema = EnumSchema {
            description: None,
            values: vec![
                EnumValue {
                    value: serde_json::Value::String("ACTIVE".to_string()),
                    description: None,
                },
                EnumValue {
                    value: serde_json::Value::String("INACTIVE".to_string()),
                    description: None,
                },
            ],
            nullable: false,
        };

        let result = gen.generate_enum("Status", &enum_schema);
        assert!(result.contains("class Status(str, Enum):"));
        assert!(result.contains("ACTIVE = \"ACTIVE\""));
        assert!(result.contains("INACTIVE = \"INACTIVE\""));
    }

    #[test]
    fn test_format_docstring_single_line() {
        let result = format_docstring("A simple description", 0);
        assert_eq!(result, "\"\"\"A simple description\"\"\"\n");
    }

    #[test]
    fn test_format_docstring_multi_line() {
        let result = format_docstring("Line 1\nLine 2", 0);
        assert!(result.contains("\"\"\"\n"));
        assert!(result.contains("Line 1\n"));
        assert!(result.contains("Line 2\n"));
    }

    #[test]
    fn test_format_docstring_with_indent() {
        let result = format_docstring("Description", 1);
        assert!(result.starts_with("    \"\"\""));
    }

    #[test]
    fn test_generate_simple_dataclass() {
        let config = GeneratorConfig {
            include_docs: false,
            ..Default::default()
        };
        let gen = PythonTypeGenerator::new(&config);

        let obj = ObjectSchema {
            description: None,
            properties: {
                let mut props = IndexMap::new();
                props.insert("id".to_string(), Schema::String(StringSchema::default()));
                props.insert("name".to_string(), Schema::String(StringSchema::default()));
                props
            },
            required: vec!["id".to_string()],
            additional_properties: None,
            nullable: false,
        };

        let result = gen.generate_dataclass("User", &obj).unwrap();
        assert!(result.contains("@dataclass"));
        assert!(result.contains("class User:"));
        assert!(result.contains("id: str"));
        assert!(result.contains("name: Optional[str] = None"));
    }
}
