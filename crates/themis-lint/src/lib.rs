//! # Themis Lint
//!
//! Linting rules for Themis contract governance.
//!
//! This crate provides configurable lint rules that enforce best practices
//! and Themis conventions on API contracts.
//!
//! ## Usage
//!
//! ```rust
//! use themis_lint::{LintReporter, LintConfig};
//! use themis_core::Contract;
//!
//! // Create a linter with default configuration
//! let linter = LintReporter::new(LintConfig::default());
//!
//! // Or with custom configuration
//! let config = LintConfig::strict();
//! let linter = LintReporter::new(config);
//!
//! // Lint a contract
//! // let report = linter.lint(&contract);
//! ```
//!
//! ## Rule Categories
//!
//! - **Naming** (`naming/*`): Naming convention checks
//! - **Documentation** (`docs/*`): Documentation completeness
//! - **Security** (`security/*`): Security best practices
//! - **Versioning** (`versioning/*`): Version rule checks

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod reporter;
pub mod rule;
pub mod rules;

pub use reporter::{LintConfig, LintIssue, LintReport, LintReporter, Severity};
pub use rule::{Rule, RuleConfig};

/// Re-export of all built-in rules.
pub mod builtin {
    pub use crate::rules::{documentation, naming};
}
