//! Change types for compatibility analysis.
//!
//! Defines the different kinds of changes that can occur between contract versions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A breaking change that requires a major version bump.
///
/// Breaking changes are changes that would cause existing clients to fail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BreakingChange {
    /// An operation was removed from the contract.
    OperationRemoved {
        /// The ID of the removed operation.
        operation_id: String,
        /// The path of the removed operation.
        path: Option<String>,
    },

    /// An operation's path was changed.
    OperationPathChanged {
        /// The ID of the affected operation.
        operation_id: String,
        /// The old path.
        old_path: String,
        /// The new path.
        new_path: String,
    },

    /// An operation's HTTP method was changed.
    OperationMethodChanged {
        /// The ID of the affected operation.
        operation_id: String,
        /// The path of the operation.
        path: Option<String>,
        /// The old HTTP method.
        old_method: String,
        /// The new HTTP method.
        new_method: String,
    },

    /// A required field was added to a request body.
    RequiredFieldAdded {
        /// The ID of the affected operation.
        operation_id: String,
        /// The location (request or response).
        location: String,
        /// The name of the added required field.
        field: String,
    },

    /// A field was removed from a response.
    FieldRemoved {
        /// The ID of the affected operation.
        operation_id: String,
        /// The location (request or response).
        location: String,
        /// The name of the removed field.
        field: String,
    },

    /// A field's type was changed.
    FieldTypeChanged {
        /// The ID of the affected operation.
        operation_id: String,
        /// The location (request or response).
        location: String,
        /// The name of the affected field.
        field: String,
        /// The old type.
        old_type: String,
        /// The new type.
        new_type: String,
    },

    /// A field was made required (was previously optional).
    FieldBecameRequired {
        /// The ID of the affected operation.
        operation_id: String,
        /// The location (request or response).
        location: String,
        /// The name of the field.
        field: String,
    },

    /// An enum value was removed.
    EnumValueRemoved {
        /// The schema name.
        schema_name: String,
        /// The removed enum value.
        value: String,
    },

    /// A security scheme was removed.
    SecuritySchemeRemoved {
        /// The name of the removed security scheme.
        scheme_name: String,
    },

    /// A schema was removed.
    SchemaRemoved {
        /// The name of the removed schema.
        schema_name: String,
    },
}

impl fmt::Display for BreakingChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationRemoved { operation_id, path } => {
                if let Some(p) = path {
                    write!(f, "BREAK001: Operation '{operation_id}' removed (was at {p})")
                } else {
                    write!(f, "BREAK001: Operation '{operation_id}' removed")
                }
            }
            Self::OperationPathChanged {
                operation_id,
                old_path,
                new_path,
            } => write!(
                f,
                "BREAK002: Operation '{operation_id}' path changed from '{old_path}' to '{new_path}'"
            ),
            Self::OperationMethodChanged {
                operation_id,
                old_method,
                new_method,
                ..
            } => write!(
                f,
                "BREAK003: Operation '{operation_id}' method changed from {old_method} to {new_method}"
            ),
            Self::RequiredFieldAdded {
                operation_id,
                location,
                field,
            } => write!(
                f,
                "BREAK004: Required field '{field}' added to {location} of '{operation_id}'"
            ),
            Self::FieldRemoved {
                operation_id,
                location,
                field,
            } => write!(
                f,
                "BREAK005: Field '{field}' removed from {location} of '{operation_id}'"
            ),
            Self::FieldTypeChanged {
                operation_id,
                location,
                field,
                old_type,
                new_type,
            } => write!(
                f,
                "BREAK006: Field '{field}' in {location} of '{operation_id}' changed type from {old_type} to {new_type}"
            ),
            Self::FieldBecameRequired {
                operation_id,
                location,
                field,
            } => write!(
                f,
                "BREAK007: Field '{field}' in {location} of '{operation_id}' became required"
            ),
            Self::EnumValueRemoved { schema_name, value } => {
                write!(f, "BREAK008: Enum value '{value}' removed from '{schema_name}'")
            }
            Self::SecuritySchemeRemoved { scheme_name } => {
                write!(f, "BREAK009: Security scheme '{scheme_name}' removed")
            }
            Self::SchemaRemoved { schema_name } => {
                write!(f, "BREAK010: Schema '{schema_name}' removed")
            }
        }
    }
}

/// A backwards-compatible addition (requires minor version bump).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Addition {
    /// A new operation was added.
    OperationAdded {
        /// The ID of the new operation.
        operation_id: String,
        /// The path of the new operation.
        path: Option<String>,
        /// The HTTP method.
        method: Option<String>,
    },

    /// An optional field was added to a request.
    OptionalFieldAdded {
        /// The ID of the affected operation.
        operation_id: String,
        /// The location (request or response).
        location: String,
        /// The name of the added field.
        field: String,
    },

    /// A field was added to a response.
    ResponseFieldAdded {
        /// The ID of the affected operation.
        operation_id: String,
        /// The name of the added field.
        field: String,
    },

    /// A new enum value was added.
    EnumValueAdded {
        /// The schema name.
        schema_name: String,
        /// The added enum value.
        value: String,
    },

    /// A new security scheme was added.
    SecuritySchemeAdded {
        /// The name of the new security scheme.
        scheme_name: String,
    },

    /// A new schema was added.
    SchemaAdded {
        /// The name of the new schema.
        schema_name: String,
    },
}

impl fmt::Display for Addition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationAdded {
                operation_id,
                path,
                method,
            } => {
                let method_str = method.as_deref().unwrap_or("?");
                let path_str = path.as_deref().unwrap_or("?");
                write!(
                    f,
                    "ADD001: Operation '{operation_id}' added ({method_str} {path_str})"
                )
            }
            Self::OptionalFieldAdded {
                operation_id,
                location,
                field,
            } => write!(
                f,
                "ADD002: Optional field '{field}' added to {location} of '{operation_id}'"
            ),
            Self::ResponseFieldAdded {
                operation_id,
                field,
            } => write!(
                f,
                "ADD003: Field '{field}' added to response of '{operation_id}'"
            ),
            Self::EnumValueAdded { schema_name, value } => {
                write!(f, "ADD004: Enum value '{value}' added to '{schema_name}'")
            }
            Self::SecuritySchemeAdded { scheme_name } => {
                write!(f, "ADD005: Security scheme '{scheme_name}' added")
            }
            Self::SchemaAdded { schema_name } => {
                write!(f, "ADD006: Schema '{schema_name}' added")
            }
        }
    }
}

/// A non-functional modification (requires patch version bump).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Modification {
    /// A description was changed.
    DescriptionChanged {
        /// The location of the change.
        location: String,
        /// The old description.
        old: Option<String>,
        /// The new description.
        new: Option<String>,
    },

    /// A summary was changed.
    SummaryChanged {
        /// The operation ID.
        operation_id: String,
        /// The old summary.
        old: Option<String>,
        /// The new summary.
        new: Option<String>,
    },

    /// Tags were changed.
    TagsChanged {
        /// The operation ID.
        operation_id: String,
        /// The old tags.
        old: Vec<String>,
        /// The new tags.
        new: Vec<String>,
    },

    /// Deprecation status changed.
    DeprecationChanged {
        /// The operation ID.
        operation_id: String,
        /// Whether the operation is now deprecated.
        deprecated: bool,
    },
}

impl fmt::Display for Modification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DescriptionChanged { location, .. } => {
                write!(f, "MOD001: Description changed at '{location}'")
            }
            Self::SummaryChanged { operation_id, .. } => {
                write!(f, "MOD002: Summary changed for operation '{operation_id}'")
            }
            Self::TagsChanged { operation_id, .. } => {
                write!(f, "MOD003: Tags changed for operation '{operation_id}'")
            }
            Self::DeprecationChanged {
                operation_id,
                deprecated,
            } => {
                if *deprecated {
                    write!(f, "MOD004: Operation '{operation_id}' marked as deprecated")
                } else {
                    write!(
                        f,
                        "MOD004: Operation '{operation_id}' unmarked as deprecated"
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaking_change_display() {
        let change = BreakingChange::OperationRemoved {
            operation_id: "getUser".to_string(),
            path: Some("/users/{id}".to_string()),
        };
        assert!(change.to_string().contains("BREAK001"));
        assert!(change.to_string().contains("getUser"));
    }

    #[test]
    fn test_addition_display() {
        let addition = Addition::OperationAdded {
            operation_id: "listUsers".to_string(),
            path: Some("/users".to_string()),
            method: Some("GET".to_string()),
        };
        assert!(addition.to_string().contains("ADD001"));
        assert!(addition.to_string().contains("listUsers"));
    }

    #[test]
    fn test_modification_display() {
        let modification = Modification::DescriptionChanged {
            location: "paths./users.get".to_string(),
            old: Some("Old desc".to_string()),
            new: Some("New desc".to_string()),
        };
        assert!(modification.to_string().contains("MOD001"));
    }

    #[test]
    fn test_breaking_change_serialization() {
        let change = BreakingChange::FieldTypeChanged {
            operation_id: "getUser".to_string(),
            location: "response".to_string(),
            field: "age".to_string(),
            old_type: "string".to_string(),
            new_type: "integer".to_string(),
        };

        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains("field_type_changed"));

        let parsed: BreakingChange = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, change);
    }
}
