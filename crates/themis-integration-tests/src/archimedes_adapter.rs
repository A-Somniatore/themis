//! Adapter for converting Themis artifacts to Archimedes contracts.
//!
//! This module bridges Themis artifacts and Archimedes contract types,
//! enabling runtime integration testing. It demonstrates how Archimedes
//! will consume Themis artifacts in production.
//!
//! # Key Integration Points
//!
//! 1. **Artifact Loading** - Load Themis artifacts into Archimedes
//! 2. **Operation Mapping** - Map artifact operations to Archimedes operations
//! 3. **Schema Adaptation** - Convert Themis schemas to Archimedes MockSchema
//! 4. **Metadata Preservation** - Maintain operation metadata for policy context

use themis_artifact::{Artifact, ArtifactOperation};
use themis_core::Schema;

/// Adapts a Themis artifact for use with Archimedes.
///
/// This adapter converts Themis artifacts into a format compatible with
/// Archimedes, demonstrating the integration pattern Archimedes will use
/// in production.
pub struct ArchimedesAdapter;

impl ArchimedesAdapter {
    /// Converts a Themis artifact to an Archimedes-compatible format.
    ///
    /// # Arguments
    ///
    /// * `artifact` - The Themis artifact to adapt
    ///
    /// # Returns
    ///
    /// Information about the artifact in Archimedes format
    pub fn adapt_artifact(artifact: &Artifact) -> AdaptedArtifact {
        let operations = artifact
            .operations
            .iter()
            .map(Self::adapt_operation)
            .collect();

        AdaptedArtifact {
            service: artifact.service.clone(),
            version: artifact.version.clone(),
            format: artifact.format.clone(),
            operations,
        }
    }

    /// Converts a Themis operation to Archimedes format.
    fn adapt_operation(op: &ArtifactOperation) -> AdaptedOperation {
        AdaptedOperation {
            operation_id: op.id.clone(),
            method: op.method.clone(),
            path: op.path.clone(),
            summary: op.summary.clone(),
            description: op.description.clone(),
            security_requirements: op.security.clone(),
            has_request_schema: op.request_schema.is_some(),
            has_response_schemas: !op.response_schemas.is_empty(),
            response_status_codes: op.response_schemas.keys().cloned().collect(),
            rate_limit_tier: op.metadata.as_ref().and_then(|m| m.rate_limit_tier.clone()),
            timeout_tier: op.metadata.as_ref().and_then(|m| m.timeout_tier.clone()),
            is_idempotent: op
                .metadata
                .as_ref()
                .and_then(|m| m.idempotent)
                .unwrap_or(false),
            tags: op.tags.clone(),
            deprecated: op.deprecated,
        }
    }

    /// Validates that a Themis schema is compatible with Archimedes.
    ///
    /// # Errors
    ///
    /// Returns `SchemaValidationError` if:
    /// - Schema exceeds maximum nesting depth (10 levels)
    pub fn validate_schema(schema: &Schema) -> Result<(), SchemaValidationError> {
        validate_schema_recursive(schema, 0)
    }
}

fn validate_schema_recursive(schema: &Schema, depth: usize) -> Result<(), SchemaValidationError> {
    // Check for excessive nesting
    if depth > 10 {
        return Err(SchemaValidationError::ExcessiveNesting);
    }

    match schema {
        // Primitive types are always valid
        Schema::String(_)
        | Schema::Integer(_)
        | Schema::Number(_)
        | Schema::Boolean(_)
        | Schema::Null
        | Schema::Ref(_)
        | Schema::Enum(_) => Ok(()),
        Schema::Array(arr) => validate_schema_recursive(&arr.items, depth + 1),
        Schema::Object(obj) => {
            for (_name, prop_schema) in &obj.properties {
                validate_schema_recursive(prop_schema, depth + 1)?;
            }
            Ok(())
        }
        Schema::OneOf(one_of) => {
            for schema_option in &one_of.schemas {
                validate_schema_recursive(schema_option, depth + 1)?;
            }
            Ok(())
        }
        Schema::AnyOf(any_of) => {
            for schema_option in &any_of.schemas {
                validate_schema_recursive(schema_option, depth + 1)?;
            }
            Ok(())
        }
        Schema::AllOf(all_of) => {
            for schema_option in &all_of.schemas {
                validate_schema_recursive(schema_option, depth + 1)?;
            }
            Ok(())
        }
    }
}

/// Adapted artifact for Archimedes compatibility.
#[derive(Debug, Clone)]
pub struct AdaptedArtifact {
    /// Service name
    pub service: String,
    /// Version
    pub version: String,
    /// Format (e.g., "openapi")
    pub format: String,
    /// Adapted operations
    pub operations: Vec<AdaptedOperation>,
}

impl AdaptedArtifact {
    /// Gets an operation by ID.
    #[must_use]
    pub fn get_operation(&self, operation_id: &str) -> Option<&AdaptedOperation> {
        self.operations
            .iter()
            .find(|op| op.operation_id == operation_id)
    }

    /// Finds an operation by method and path.
    #[must_use]
    pub fn find_operation_by_route(&self, method: &str, path: &str) -> Option<&AdaptedOperation> {
        self.operations
            .iter()
            .find(|op| op.method == method && Self::path_matches(&op.path, path))
    }

    /// Checks if a request path matches an operation path pattern.
    fn path_matches(pattern: &str, request_path: &str) -> bool {
        // Convert path pattern like "/users/{userId}" to regex
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|p| !p.is_empty()).collect();
        let request_parts: Vec<&str> = request_path.split('/').filter(|p| !p.is_empty()).collect();

        if pattern_parts.len() != request_parts.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(request_parts.iter())
            .all(|(pattern_part, request_part)| {
                // Pattern part is either a literal or a parameter like {userId}
                if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                    true // Parameters match any value
                } else {
                    pattern_part == request_part // Literals must match exactly
                }
            })
    }
}

/// Adapted operation for Archimedes compatibility.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AdaptedOperation {
    /// Operation ID (e.g., "getUser")
    pub operation_id: String,
    /// HTTP method
    pub method: String,
    /// URL path pattern
    pub path: String,
    /// Short summary
    pub summary: Option<String>,
    /// Long description
    pub description: Option<String>,
    /// Security requirements
    pub security_requirements: Vec<String>,
    /// Whether operation has a request schema
    pub has_request_schema: bool,
    /// Whether operation has response schemas
    pub has_response_schemas: bool,
    /// HTTP status codes for responses
    pub response_status_codes: Vec<String>,
    /// Rate limit tier
    pub rate_limit_tier: Option<String>,
    /// Timeout tier
    pub timeout_tier: Option<String>,
    /// Whether operation is idempotent
    pub is_idempotent: bool,
    /// Tags for grouping
    pub tags: Vec<String>,
    /// Whether operation is deprecated
    pub deprecated: bool,
}

/// Schema validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaValidationError {
    /// Schema nesting exceeds maximum depth
    ExcessiveNesting,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use themis_artifact::ArtifactBuilder;
    use themis_core::contract::ContractFormat;
    use themis_core::Contract;
    use themis_core::Version;

    #[test]
    fn test_adapt_artifact() {
        let artifact = ArtifactBuilder::new()
            .service("users-service")
            .version("1.0.0")
            .build()
            .unwrap();

        let adapted = ArchimedesAdapter::adapt_artifact(&artifact);

        assert_eq!(adapted.service, "users-service");
        assert_eq!(adapted.version, "1.0.0");
        assert!(!adapted.format.is_empty());
    }

    #[test]
    fn test_path_matching() {
        assert!(AdaptedArtifact::path_matches(
            "/users/{userId}",
            "/users/123"
        ));
        assert!(AdaptedArtifact::path_matches("/users", "/users"));
        assert!(!AdaptedArtifact::path_matches(
            "/users/{userId}",
            "/orders/123"
        ));
        assert!(!AdaptedArtifact::path_matches(
            "/users/{userId}",
            "/users/123/extra"
        ));
    }

    #[test]
    fn test_schema_validation_depth() {
        use themis_core::schema::ArraySchema;

        // Create deeply nested schema
        let mut schema = Schema::String(Default::default());
        for _ in 0..10 {
            schema = Schema::Array(ArraySchema {
                items: Box::new(schema),
                ..Default::default()
            });
        }

        // Should succeed at depth 10
        assert!(ArchimedesAdapter::validate_schema(&schema).is_ok());

        // Exceed depth limit
        schema = Schema::Array(ArraySchema {
            items: Box::new(schema),
            ..Default::default()
        });

        assert_eq!(
            ArchimedesAdapter::validate_schema(&schema),
            Err(SchemaValidationError::ExcessiveNesting)
        );
    }

    #[test]
    fn test_adapted_artifact_operations() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .build()
            .unwrap();

        let adapted = ArchimedesAdapter::adapt_artifact(&artifact);

        // Should be able to query by operation_id (if any exist)
        let _op = adapted.get_operation("nonexistent");
    }
}
