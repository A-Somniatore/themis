//! Schema types for contract data models.
//!
//! Provides a unified schema representation that can express types from
//! `OpenAPI`, Protobuf, GraphQL, and `AsyncAPI`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A schema definition representing a data type.
///
/// This is a unified representation that can express types from any
/// supported contract format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Schema {
    /// A string type
    String(StringSchema),

    /// An integer type
    Integer(IntegerSchema),

    /// A floating-point number type
    Number(NumberSchema),

    /// A boolean type
    Boolean(BooleanSchema),

    /// An array type
    Array(ArraySchema),

    /// An object type
    Object(ObjectSchema),

    /// A reference to another schema
    Ref(RefSchema),

    /// A union of multiple schemas (oneOf)
    OneOf(OneOfSchema),

    /// An intersection of multiple schemas (allOf)
    AllOf(AllOfSchema),

    /// Any of multiple schemas (anyOf)
    AnyOf(AnyOfSchema),

    /// An enumeration type
    Enum(EnumSchema),

    /// Null type
    Null,
}

impl Default for Schema {
    fn default() -> Self {
        Self::Object(ObjectSchema::default())
    }
}

/// String schema with optional constraints.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StringSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Format hint (e.g., "uuid", "email", "date-time")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Minimum length constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Regex pattern constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// Integer schema with optional constraints.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IntegerSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Format hint (e.g., "int32", "int64")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Minimum value constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,

    /// Maximum value constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<i64>,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// Number (floating-point) schema with optional constraints.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NumberSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Format hint (e.g., "float", "double")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Minimum value constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value constraint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<f64>,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// Boolean schema.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BooleanSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// Array schema.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ArraySchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Schema for array items
    pub items: Box<Schema>,

    /// Minimum number of items
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    /// Maximum number of items
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    /// Whether items must be unique
    #[serde(default)]
    pub unique_items: bool,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// Object schema with properties.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ObjectSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Property definitions
    #[serde(default)]
    pub properties: HashMap<String, Schema>,

    /// Required property names
    #[serde(default)]
    pub required: Vec<String>,

    /// Schema for additional properties (if allowed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<Schema>>,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// Reference to another schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefSchema {
    /// Reference path (e.g., "#/components/schemas/User")
    #[serde(rename = "$ref")]
    pub reference: String,
}

/// Union schema (oneOf).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OneOfSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Possible schemas (exactly one must match)
    #[serde(rename = "oneOf")]
    pub schemas: Vec<Schema>,

    /// Discriminator for polymorphism
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
}

/// Intersection schema (allOf).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AllOfSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Schemas that all must match
    #[serde(rename = "allOf")]
    pub schemas: Vec<Schema>,
}

/// Any-of schema (anyOf).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AnyOfSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Schemas where at least one must match
    #[serde(rename = "anyOf")]
    pub schemas: Vec<Schema>,
}

/// Enumeration schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumSchema {
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Possible enum values
    pub values: Vec<EnumValue>,

    /// Whether the field is nullable
    #[serde(default)]
    pub nullable: bool,
}

/// A single enum value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumValue {
    /// The enum value
    pub value: serde_json::Value,

    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Discriminator for polymorphic schemas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discriminator {
    /// Property name that contains the type discriminator
    pub property_name: String,

    /// Mapping of discriminator values to schema references
    #[serde(default)]
    pub mapping: HashMap<String, String>,
}

impl Schema {
    /// Creates a string schema.
    #[must_use]
    pub fn string() -> Self {
        Self::String(StringSchema::default())
    }

    /// Creates an integer schema.
    #[must_use]
    pub fn integer() -> Self {
        Self::Integer(IntegerSchema::default())
    }

    /// Creates a number schema.
    #[must_use]
    pub fn number() -> Self {
        Self::Number(NumberSchema::default())
    }

    /// Creates a boolean schema.
    #[must_use]
    pub fn boolean() -> Self {
        Self::Boolean(BooleanSchema::default())
    }

    /// Creates an array schema with the given item schema.
    #[must_use]
    pub fn array(items: Self) -> Self {
        Self::Array(ArraySchema {
            items: Box::new(items),
            ..Default::default()
        })
    }

    /// Creates an object schema.
    #[must_use]
    pub fn object() -> Self {
        Self::Object(ObjectSchema::default())
    }

    /// Creates a reference schema.
    #[must_use]
    pub fn reference(ref_path: impl Into<String>) -> Self {
        Self::Ref(RefSchema {
            reference: ref_path.into(),
        })
    }

    /// Returns true if this schema is nullable.
    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        match self {
            Self::String(s) => s.nullable,
            Self::Integer(i) => i.nullable,
            Self::Number(n) => n.nullable,
            Self::Boolean(b) => b.nullable,
            Self::Array(a) => a.nullable,
            Self::Object(o) => o.nullable,
            Self::Enum(e) => e.nullable,
            Self::Null => true,
            Self::Ref(_) | Self::OneOf(_) | Self::AllOf(_) | Self::AnyOf(_) => false,
        }
    }

    /// Returns the description if available.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::String(s) => s.description.as_deref(),
            Self::Integer(i) => i.description.as_deref(),
            Self::Number(n) => n.description.as_deref(),
            Self::Boolean(b) => b.description.as_deref(),
            Self::Array(a) => a.description.as_deref(),
            Self::Object(o) => o.description.as_deref(),
            Self::OneOf(o) => o.description.as_deref(),
            Self::AllOf(a) => a.description.as_deref(),
            Self::AnyOf(a) => a.description.as_deref(),
            Self::Enum(e) => e.description.as_deref(),
            Self::Ref(_) | Self::Null => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_constructors() {
        assert!(matches!(Schema::string(), Schema::String(_)));
        assert!(matches!(Schema::integer(), Schema::Integer(_)));
        assert!(matches!(Schema::number(), Schema::Number(_)));
        assert!(matches!(Schema::boolean(), Schema::Boolean(_)));
        assert!(matches!(Schema::object(), Schema::Object(_)));
    }

    #[test]
    fn test_array_schema() {
        let schema = Schema::array(Schema::string());
        if let Schema::Array(arr) = schema {
            assert!(matches!(*arr.items, Schema::String(_)));
        } else {
            panic!("Expected array schema");
        }
    }

    #[test]
    fn test_reference_schema() {
        let schema = Schema::reference("#/components/schemas/User");
        if let Schema::Ref(r) = schema {
            assert_eq!(r.reference, "#/components/schemas/User");
        } else {
            panic!("Expected ref schema");
        }
    }

    #[test]
    fn test_schema_nullable() {
        let mut string_schema = StringSchema::default();
        assert!(!Schema::String(string_schema.clone()).is_nullable());

        string_schema.nullable = true;
        assert!(Schema::String(string_schema).is_nullable());

        assert!(Schema::Null.is_nullable());
    }

    #[test]
    fn test_object_schema() {
        let mut obj = ObjectSchema::default();
        obj.properties.insert("name".to_string(), Schema::string());
        obj.properties.insert("age".to_string(), Schema::integer());
        obj.required.push("name".to_string());

        let schema = Schema::Object(obj);
        if let Schema::Object(o) = schema {
            assert_eq!(o.properties.len(), 2);
            assert_eq!(o.required.len(), 1);
        } else {
            panic!("Expected object schema");
        }
    }

    #[test]
    fn test_schema_serialization() {
        let schema = Schema::String(StringSchema {
            description: Some("A test string".to_string()),
            format: Some("email".to_string()),
            ..Default::default()
        });

        let json = serde_json::to_string(&schema).unwrap();
        let deserialized: Schema = serde_json::from_str(&json).unwrap();
        assert_eq!(schema, deserialized);
    }
}
