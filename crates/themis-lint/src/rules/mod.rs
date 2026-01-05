//! Lint rules for contract validation.
//!
//! This module contains all built-in lint rules organized by category:
//!
//! - [`naming`]: Naming convention checks (camelCase, kebab-case, `PascalCase`)
//! - [`documentation`]: Documentation completeness checks
//! - [`security`]: Security best practice checks (coming soon)
//! - [`versioning`]: Versioning rule checks (coming soon)

pub mod documentation;
pub mod naming;
pub mod security;
pub mod versioning;

use crate::rule::Rule;

/// Returns all built-in lint rules.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut rules = Vec::new();
    rules.extend(naming::all_rules());
    rules.extend(documentation::all_rules());
    // TODO: Add security rules
    // TODO: Add versioning rules
    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        // 3 naming + 3 documentation = 6
        assert_eq!(rules.len(), 6);
    }
}
