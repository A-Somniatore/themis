//! # Themis Compat
//!
//! Breaking change detection and compatibility analysis for Themis contracts.
//!
//! This crate provides tools for comparing contract versions and detecting
//! breaking changes that would require a major version bump.
//!
//! ## Usage
//!
//! ```ignore
//! use themis_compat::{CompatibilityChecker, check_compatibility};
//!
//! // Simple comparison
//! let report = check_compatibility(&old_contract, &new_contract);
//! if !report.is_compatible {
//!     println!("Breaking changes detected!");
//!     for change in &report.breaking_changes {
//!         println!("  - {}", change);
//!     }
//! }
//!
//! // With version validation
//! let checker = CompatibilityChecker::new();
//! match checker.check(&old_contract, &new_contract) {
//!     Ok(report) => println!("Version bump is valid"),
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```
//!
//! ## Change Categories
//!
//! Changes are categorized into three types:
//!
//! - **Breaking**: Changes that break existing clients (require major version bump)
//! - **Additions**: Backwards-compatible new features (require minor version bump)
//! - **Modifications**: Non-functional changes like descriptions (patch version bump)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod changes;
mod checker;
mod differ;
mod report;

// Re-export change types
pub use changes::{Addition, BreakingChange, Modification};

// Re-export checker API
pub use checker::{
    check_compatibility, validate_version_bump, CompatibilityChecker, CompatibilityConfig,
    CompatibilityError,
};

// Re-export report types
pub use report::{CompatibilityReport, SuggestedBump};

// Re-export differ for advanced use
pub use differ::diff_contracts;
