//! Contract diffing utilities.
//!
//! Provides functions to compare two contracts and identify changes.

use crate::changes::{Addition, BreakingChange, Modification};
use crate::report::CompatibilityReport;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use themis_core::operation::{Parameter, Response};
use themis_core::schema::{EnumSchema, ObjectSchema};
use themis_core::{Contract, Operation, Schema};

/// Compares two contracts and produces a compatibility report.
///
/// # Arguments
///
/// * `old` - The old (baseline) contract
/// * `new` - The new contract to compare against the baseline
///
/// # Returns
///
/// A `CompatibilityReport` containing all detected changes.
#[must_use]
pub fn diff_contracts(old: &Contract, new: &Contract) -> CompatibilityReport {
    let mut report = CompatibilityReport::new();

    // Set versions
    report.set_old_version(old.version.to_string());
    report.set_new_version(new.version.to_string());

    // Compare operations
    diff_operations(&old.operations, &new.operations, &mut report);

    // Compare schemas
    diff_schemas(&old.schemas, &new.schemas, &mut report);

    // Compare security schemes
    diff_security_schemes(old, new, &mut report);

    report
}

/// Compares operations between two contracts.
fn diff_operations(
    old_ops: &HashMap<String, Operation>,
    new_ops: &HashMap<String, Operation>,
    report: &mut CompatibilityReport,
) {
    let old_ids: HashSet<&String> = old_ops.keys().collect();
    let new_ids: HashSet<&String> = new_ops.keys().collect();

    // Check for removed operations (BREAKING)
    for id in old_ids.difference(&new_ids) {
        let old_op = &old_ops[*id];
        report.add_breaking_change(BreakingChange::OperationRemoved {
            operation_id: (*id).clone(),
            path: old_op.path.clone(),
        });
    }

    // Check for added operations (ADDITION)
    for id in new_ids.difference(&old_ids) {
        let new_op = &new_ops[*id];
        report.add_addition(Addition::OperationAdded {
            operation_id: (*id).clone(),
            path: new_op.path.clone(),
            method: new_op.method.map(|m| m.to_string()),
        });
    }

    // Check for modified operations
    for id in old_ids.intersection(&new_ids) {
        let old_op = &old_ops[*id];
        let new_op = &new_ops[*id];
        diff_operation(id, old_op, new_op, report);
    }
}

/// Compares a single operation between versions.
fn diff_operation(
    id: &str,
    old_op: &Operation,
    new_op: &Operation,
    report: &mut CompatibilityReport,
) {
    // Check path change (BREAKING)
    if let (Some(old_path), Some(new_path)) = (&old_op.path, &new_op.path) {
        if old_path != new_path {
            report.add_breaking_change(BreakingChange::OperationPathChanged {
                operation_id: id.to_string(),
                old_path: old_path.clone(),
                new_path: new_path.clone(),
            });
        }
    }

    // Check method change (BREAKING)
    if let (Some(old_method), Some(new_method)) = (&old_op.method, &new_op.method) {
        if old_method != new_method {
            report.add_breaking_change(BreakingChange::OperationMethodChanged {
                operation_id: id.to_string(),
                path: new_op.path.clone(),
                old_method: old_method.to_string(),
                new_method: new_method.to_string(),
            });
        }
    }

    // Check parameters
    diff_parameters(id, &old_op.parameters, &new_op.parameters, report);

    // Check request body schemas
    if let (Some(old_req), Some(new_req)) = (&old_op.request_body, &new_op.request_body) {
        // Compare the schemas in the request body content
        for (media_type, old_media) in &old_req.content {
            if let Some(new_media) = new_req.content.get(media_type) {
                diff_schema_enum(
                    id,
                    &format!("request.{media_type}"),
                    &old_media.schema,
                    &new_media.schema,
                    report,
                );
            }
        }
    }

    // Check responses
    diff_responses(id, &old_op.responses, &new_op.responses, report);

    // Check description (MODIFICATION)
    if old_op.description != new_op.description {
        report.add_modification(Modification::DescriptionChanged {
            location: format!("operation.{id}"),
            old: old_op.description.clone(),
            new: new_op.description.clone(),
        });
    }

    // Check summary (MODIFICATION)
    if old_op.summary != new_op.summary {
        report.add_modification(Modification::SummaryChanged {
            operation_id: id.to_string(),
            old: old_op.summary.clone(),
            new: new_op.summary.clone(),
        });
    }

    // Check tags (MODIFICATION)
    if old_op.tags != new_op.tags {
        report.add_modification(Modification::TagsChanged {
            operation_id: id.to_string(),
            old: old_op.tags.clone(),
            new: new_op.tags.clone(),
        });
    }

    // Check deprecation (MODIFICATION)
    if old_op.deprecated != new_op.deprecated {
        report.add_modification(Modification::DeprecationChanged {
            operation_id: id.to_string(),
            deprecated: new_op.deprecated,
        });
    }
}

/// Compares parameters between operations.
fn diff_parameters(
    operation_id: &str,
    old_params: &[Parameter],
    new_params: &[Parameter],
    report: &mut CompatibilityReport,
) {
    let old_by_name: HashMap<&str, &Parameter> =
        old_params.iter().map(|p| (p.name.as_str(), p)).collect();
    let new_by_name: HashMap<&str, &Parameter> =
        new_params.iter().map(|p| (p.name.as_str(), p)).collect();

    let old_names: HashSet<&str> = old_by_name.keys().copied().collect();
    let new_names: HashSet<&str> = new_by_name.keys().copied().collect();

    // Check for added parameters
    for name in new_names.difference(&old_names) {
        let new_param = new_by_name[*name];
        if new_param.required {
            // Adding a required parameter is BREAKING
            report.add_breaking_change(BreakingChange::RequiredFieldAdded {
                operation_id: operation_id.to_string(),
                location: format!("parameter.{name}"),
                field: (*name).to_string(),
            });
        } else {
            // Adding optional parameter is fine
            report.add_addition(Addition::OptionalFieldAdded {
                operation_id: operation_id.to_string(),
                location: "parameter".to_string(),
                field: (*name).to_string(),
            });
        }
    }

    // Check for modified parameters
    for name in old_names.intersection(&new_names) {
        let old_param = old_by_name[*name];
        let new_param = new_by_name[*name];

        // Check if parameter became required (BREAKING)
        if !old_param.required && new_param.required {
            report.add_breaking_change(BreakingChange::FieldBecameRequired {
                operation_id: operation_id.to_string(),
                location: "parameter".to_string(),
                field: (*name).to_string(),
            });
        }
    }
}

/// Compares response schemas between operations.
fn diff_responses(
    operation_id: &str,
    old_responses: &HashMap<String, Response>,
    new_responses: &HashMap<String, Response>,
    report: &mut CompatibilityReport,
) {
    let old_codes: HashSet<&String> = old_responses.keys().collect();
    let new_codes: HashSet<&String> = new_responses.keys().collect();

    // Check for modified responses
    for code in old_codes.intersection(&new_codes) {
        let old_response = &old_responses[*code];
        let new_response = &new_responses[*code];

        // Compare content schemas
        for (media_type, old_media) in &old_response.content {
            if let Some(new_media) = new_response.content.get(media_type) {
                diff_schema_enum(
                    operation_id,
                    &format!("response.{code}.{media_type}"),
                    &old_media.schema,
                    &new_media.schema,
                    report,
                );
            }
        }
    }
}

/// Compares two Schema enums and records changes.
fn diff_schema_enum(
    operation_id: &str,
    location: &str,
    old_schema: &Schema,
    new_schema: &Schema,
    report: &mut CompatibilityReport,
) {
    // Check for type change (BREAKING)
    if schema_type_name(old_schema) != schema_type_name(new_schema) {
        report.add_breaking_change(BreakingChange::FieldTypeChanged {
            operation_id: operation_id.to_string(),
            location: location.to_string(),
            field: "(root)".to_string(),
            old_type: schema_type_name(old_schema).to_string(),
            new_type: schema_type_name(new_schema).to_string(),
        });
        return;
    }

    // Compare based on schema type
    match (old_schema, new_schema) {
        (Schema::Object(old_obj), Schema::Object(new_obj)) => {
            diff_object_schemas(operation_id, location, old_obj, new_obj, report);
        }
        (Schema::Enum(old_enum), Schema::Enum(new_enum)) => {
            diff_enum_schemas(location, old_enum, new_enum, report);
        }
        (Schema::Array(old_arr), Schema::Array(new_arr)) => {
            diff_schema_enum(
                operation_id,
                &format!("{location}.items"),
                &old_arr.items,
                &new_arr.items,
                report,
            );
        }
        _ => {
            // For other types, no further comparison needed
        }
    }
}

/// Returns the type name of a schema.
const fn schema_type_name(schema: &Schema) -> &'static str {
    match schema {
        Schema::String(_) => "string",
        Schema::Integer(_) => "integer",
        Schema::Number(_) => "number",
        Schema::Boolean(_) => "boolean",
        Schema::Array(_) => "array",
        Schema::Object(_) => "object",
        Schema::Ref(_) => "ref",
        Schema::OneOf(_) => "oneOf",
        Schema::AllOf(_) => "allOf",
        Schema::AnyOf(_) => "anyOf",
        Schema::Enum(_) => "enum",
        Schema::Null => "null",
    }
}

/// Compares object schemas.
fn diff_object_schemas(
    operation_id: &str,
    location: &str,
    old_obj: &ObjectSchema,
    new_obj: &ObjectSchema,
    report: &mut CompatibilityReport,
) {
    let old_names: HashSet<&String> = old_obj.properties.keys().collect();
    let new_names: HashSet<&String> = new_obj.properties.keys().collect();
    let old_required_set: HashSet<&String> = old_obj.required.iter().collect();
    let new_required_set: HashSet<&String> = new_obj.required.iter().collect();

    let is_response = location.contains("response");
    let is_request = location.contains("request");

    // Check for removed properties
    for name in old_names.difference(&new_names) {
        if is_response {
            // Removing a field from response is BREAKING
            report.add_breaking_change(BreakingChange::FieldRemoved {
                operation_id: operation_id.to_string(),
                location: location.to_string(),
                field: (*name).clone(),
            });
        }
    }

    // Check for added properties
    for name in new_names.difference(&old_names) {
        let is_now_required = new_required_set.contains(*name);

        if is_request && is_now_required {
            // Adding a required field to request is BREAKING
            report.add_breaking_change(BreakingChange::RequiredFieldAdded {
                operation_id: operation_id.to_string(),
                location: location.to_string(),
                field: (*name).clone(),
            });
        } else if is_response {
            // Adding a field to response is fine
            report.add_addition(Addition::ResponseFieldAdded {
                operation_id: operation_id.to_string(),
                field: (*name).clone(),
            });
        } else {
            // Adding optional field to request
            report.add_addition(Addition::OptionalFieldAdded {
                operation_id: operation_id.to_string(),
                location: location.to_string(),
                field: (*name).clone(),
            });
        }
    }

    // Check for modified properties
    for name in old_names.intersection(&new_names) {
        let old_prop = &old_obj.properties[*name];
        let new_prop = &new_obj.properties[*name];

        // Check type change (BREAKING)
        if schema_type_name(old_prop) != schema_type_name(new_prop) {
            report.add_breaking_change(BreakingChange::FieldTypeChanged {
                operation_id: operation_id.to_string(),
                location: location.to_string(),
                field: (*name).clone(),
                old_type: schema_type_name(old_prop).to_string(),
                new_type: schema_type_name(new_prop).to_string(),
            });
        }

        // Check if field became required (BREAKING for request)
        let was_required = old_required_set.contains(*name);
        let is_now_required = new_required_set.contains(*name);

        if !was_required && is_now_required && is_request {
            report.add_breaking_change(BreakingChange::FieldBecameRequired {
                operation_id: operation_id.to_string(),
                location: location.to_string(),
                field: (*name).clone(),
            });
        }

        // Recursively check nested objects
        if let (Schema::Object(old_nested), Schema::Object(new_nested)) = (old_prop, new_prop) {
            diff_object_schemas(
                operation_id,
                &format!("{location}.{name}"),
                old_nested,
                new_nested,
                report,
            );
        }
    }
}

/// Compares enum schemas.
fn diff_enum_schemas(
    location: &str,
    old_enum: &EnumSchema,
    new_enum: &EnumSchema,
    report: &mut CompatibilityReport,
) {
    let old_values: HashSet<String> = old_enum
        .values
        .iter()
        .filter_map(|v| v.value.as_str().map(String::from))
        .collect();
    let new_values: HashSet<String> = new_enum
        .values
        .iter()
        .filter_map(|v| v.value.as_str().map(String::from))
        .collect();

    // Removed enum values are BREAKING
    for value in old_values.difference(&new_values) {
        report.add_breaking_change(BreakingChange::EnumValueRemoved {
            schema_name: location.to_string(),
            value: value.clone(),
        });
    }

    // Added enum values are additions
    for value in new_values.difference(&old_values) {
        report.add_addition(Addition::EnumValueAdded {
            schema_name: location.to_string(),
            value: value.clone(),
        });
    }
}

/// Compares top-level schemas between contracts.
fn diff_schemas(
    old_schemas: &IndexMap<String, Schema>,
    new_schemas: &IndexMap<String, Schema>,
    report: &mut CompatibilityReport,
) {
    let old_names: HashSet<&String> = old_schemas.keys().collect();
    let new_names: HashSet<&String> = new_schemas.keys().collect();

    // Check for removed schemas (potentially BREAKING)
    for name in old_names.difference(&new_names) {
        report.add_breaking_change(BreakingChange::SchemaRemoved {
            schema_name: (*name).clone(),
        });
    }

    // Check for added schemas (ADDITION)
    for name in new_names.difference(&old_names) {
        report.add_addition(Addition::SchemaAdded {
            schema_name: (*name).clone(),
        });
    }

    // Check for modified schemas
    for name in old_names.intersection(&new_names) {
        let old_schema = &old_schemas[*name];
        let new_schema = &new_schemas[*name];

        // Check type change
        if schema_type_name(old_schema) != schema_type_name(new_schema) {
            report.add_breaking_change(BreakingChange::FieldTypeChanged {
                operation_id: format!("schema.{name}"),
                location: "schema".to_string(),
                field: (*name).clone(),
                old_type: schema_type_name(old_schema).to_string(),
                new_type: schema_type_name(new_schema).to_string(),
            });
        }

        // Check enum values
        if let (Schema::Enum(old_enum), Schema::Enum(new_enum)) = (old_schema, new_schema) {
            diff_enum_schemas(name, old_enum, new_enum, report);
        }
    }
}

/// Compares security schemes between contracts.
fn diff_security_schemes(old: &Contract, new: &Contract, report: &mut CompatibilityReport) {
    let old_names: HashSet<&String> = old.security_schemes.keys().collect();
    let new_names: HashSet<&String> = new.security_schemes.keys().collect();

    // Removed security schemes are BREAKING
    for name in old_names.difference(&new_names) {
        report.add_breaking_change(BreakingChange::SecuritySchemeRemoved {
            scheme_name: (*name).clone(),
        });
    }

    // Added security schemes are additions
    for name in new_names.difference(&old_names) {
        report.add_addition(Addition::SecuritySchemeAdded {
            scheme_name: (*name).clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use themis_core::contract::{ContractFormat, ContractMetadata};
    use themis_core::operation::{HttpMethod, ParameterLocation};
    use themis_core::schema::StringSchema;
    use themis_core::Version;

    fn create_test_operation(id: &str, path: &str, method: HttpMethod) -> Operation {
        let mut op = Operation::new(id);
        op.path = Some(path.to_string());
        op.method = Some(method);
        op
    }

    fn create_test_contract() -> Contract {
        Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "test-service".to_string(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: HashMap::new(),
            schemas: IndexMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    #[test]
    fn test_no_changes_is_compatible() {
        let old = create_test_contract();
        let new = create_test_contract();
        let report = diff_contracts(&old, &new);

        assert!(report.is_compatible);
        assert!(report.is_unchanged());
    }

    #[test]
    fn test_operation_removed_is_breaking() {
        let mut old = create_test_contract();
        old.operations.insert(
            "getUser".to_string(),
            create_test_operation("getUser", "/users/{id}", HttpMethod::Get),
        );

        let new = create_test_contract();
        let report = diff_contracts(&old, &new);

        assert!(!report.is_compatible);
        assert_eq!(report.breaking_changes.len(), 1);
        assert!(matches!(
            &report.breaking_changes[0],
            BreakingChange::OperationRemoved { operation_id, .. } if operation_id == "getUser"
        ));
    }

    #[test]
    fn test_operation_added_is_addition() {
        let old = create_test_contract();

        let mut new = create_test_contract();
        new.operations.insert(
            "getUser".to_string(),
            create_test_operation("getUser", "/users/{id}", HttpMethod::Get),
        );

        let report = diff_contracts(&old, &new);

        assert!(report.is_compatible);
        assert_eq!(report.additions.len(), 1);
        assert!(matches!(
            &report.additions[0],
            Addition::OperationAdded { operation_id, .. } if operation_id == "getUser"
        ));
    }

    #[test]
    fn test_path_changed_is_breaking() {
        let mut old = create_test_contract();
        old.operations.insert(
            "getUser".to_string(),
            create_test_operation("getUser", "/users/{id}", HttpMethod::Get),
        );

        let mut new = create_test_contract();
        new.operations.insert(
            "getUser".to_string(),
            create_test_operation("getUser", "/api/users/{id}", HttpMethod::Get),
        );

        let report = diff_contracts(&old, &new);

        assert!(!report.is_compatible);
        assert!(report.breaking_changes.iter().any(|c| matches!(
            c,
            BreakingChange::OperationPathChanged { operation_id, .. } if operation_id == "getUser"
        )));
    }

    #[test]
    fn test_method_changed_is_breaking() {
        let mut old = create_test_contract();
        old.operations.insert(
            "updateUser".to_string(),
            create_test_operation("updateUser", "/users/{id}", HttpMethod::Put),
        );

        let mut new = create_test_contract();
        new.operations.insert(
            "updateUser".to_string(),
            create_test_operation("updateUser", "/users/{id}", HttpMethod::Patch),
        );

        let report = diff_contracts(&old, &new);

        assert!(!report.is_compatible);
        assert!(report.breaking_changes.iter().any(|c| matches!(
            c,
            BreakingChange::OperationMethodChanged { old_method, new_method, .. }
            if old_method == "PUT" && new_method == "PATCH"
        )));
    }

    #[test]
    fn test_required_parameter_added_is_breaking() {
        let mut old = create_test_contract();
        old.operations.insert(
            "getUser".to_string(),
            create_test_operation("getUser", "/users", HttpMethod::Get),
        );

        let mut new = create_test_contract();
        let mut op = create_test_operation("getUser", "/users", HttpMethod::Get);
        op.parameters.push(Parameter {
            name: "filter".to_string(),
            location: ParameterLocation::Query,
            required: true,
            schema: Schema::String(StringSchema::default()),
            description: None,
            deprecated: false,
        });
        new.operations.insert("getUser".to_string(), op);

        let report = diff_contracts(&old, &new);

        assert!(!report.is_compatible);
        assert!(report.breaking_changes.iter().any(|c| matches!(
            c,
            BreakingChange::RequiredFieldAdded { field, .. } if field == "filter"
        )));
    }

    #[test]
    fn test_optional_parameter_added_is_addition() {
        let mut old = create_test_contract();
        old.operations.insert(
            "getUser".to_string(),
            create_test_operation("getUser", "/users", HttpMethod::Get),
        );

        let mut new = create_test_contract();
        let mut op = create_test_operation("getUser", "/users", HttpMethod::Get);
        op.parameters.push(Parameter {
            name: "limit".to_string(),
            location: ParameterLocation::Query,
            required: false,
            schema: Schema::String(StringSchema::default()),
            description: None,
            deprecated: false,
        });
        new.operations.insert("getUser".to_string(), op);

        let report = diff_contracts(&old, &new);

        assert!(report.is_compatible);
        assert!(report.additions.iter().any(|a| matches!(
            a,
            Addition::OptionalFieldAdded { field, .. } if field == "limit"
        )));
    }

    #[test]
    fn test_schema_removed_is_breaking() {
        let mut old = create_test_contract();
        old.schemas
            .insert("User".to_string(), Schema::Object(ObjectSchema::default()));

        let new = create_test_contract();
        let report = diff_contracts(&old, &new);

        assert!(!report.is_compatible);
        assert!(report.breaking_changes.iter().any(|c| matches!(
            c,
            BreakingChange::SchemaRemoved { schema_name, .. } if schema_name == "User"
        )));
    }

    #[test]
    fn test_schema_added_is_addition() {
        let old = create_test_contract();

        let mut new = create_test_contract();
        new.schemas
            .insert("User".to_string(), Schema::Object(ObjectSchema::default()));

        let report = diff_contracts(&old, &new);

        assert!(report.is_compatible);
        assert!(report.additions.iter().any(|a| matches!(
            a,
            Addition::SchemaAdded { schema_name, .. } if schema_name == "User"
        )));
    }

    #[test]
    fn test_description_changed_is_modification() {
        let mut old = create_test_contract();
        let mut old_op = create_test_operation("getUser", "/users/{id}", HttpMethod::Get);
        old_op.description = Some("Old description".to_string());
        old.operations.insert("getUser".to_string(), old_op);

        let mut new = create_test_contract();
        let mut new_op = create_test_operation("getUser", "/users/{id}", HttpMethod::Get);
        new_op.description = Some("New description".to_string());
        new.operations.insert("getUser".to_string(), new_op);

        let report = diff_contracts(&old, &new);

        assert!(report.is_compatible);
        assert!(!report.modifications.is_empty());
    }

    #[test]
    fn test_versions_captured_in_report() {
        let mut old = create_test_contract();
        old.version = Version::new(1, 0, 0);

        let mut new = create_test_contract();
        new.version = Version::new(2, 0, 0);

        let report = diff_contracts(&old, &new);

        assert_eq!(report.old_version, Some("1.0.0".to_string()));
        assert_eq!(report.new_version, Some("2.0.0".to_string()));
    }
}
