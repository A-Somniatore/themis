//! Lint rules for contract validation.
//!
//! This module contains all built-in lint rules organized by category:
//!
//! - [`naming`]: Naming convention checks (camelCase, kebab-case, `PascalCase`)
//! - [`documentation`]: Documentation completeness checks
//! - [`security`]: Security best practice checks
//! - [`versioning`]: Versioning rule checks
//! - [`protobuf`]: Protobuf-specific checks
//! - [`graphql`]: GraphQL-specific checks
//! - [`asyncapi`]: AsyncAPI-specific checks

pub mod asyncapi;
pub mod documentation;
pub mod graphql;
pub mod naming;
pub mod protobuf;
pub mod security;
pub mod versioning;

use crate::rule::Rule;

/// Returns all built-in lint rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut rules = Vec::new();
    rules.extend(naming::all_rules());
    rules.extend(documentation::all_rules());
    rules.extend(security::all_rules());
    rules.extend(versioning::all_rules());
    rules.extend(protobuf::all_rules());
    rules.extend(graphql::all_rules());
    rules.extend(asyncapi::all_rules());
    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        // 3 naming + 3 documentation + 4 security + 4 versioning + 2 protobuf + 2 graphql + 3 asyncapi = 21
        assert_eq!(rules.len(), 21);
    }
}
