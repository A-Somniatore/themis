//! # Themis Core
//!
//! Core types and traits for the Themis contract governance system.
//!
//! This crate provides the foundational data structures used across all Themis
//! components, including:
//!
//! - [`Contract`] - The unified contract representation
//! - [`Operation`] - API operation definitions
//! - [`Schema`] - Type schema definitions
//! - [`Version`] - Semantic versioning support
//! - [`error`] - Standardized error types
//!
//! ## Shared Platform Types
//!
//! This crate re-exports types from `themis-platform-types` that are shared
//! across Themis, Archimedes, and Eunomia:
//!
//! - [`ThemisErrorEnvelope`] - Standard API error format
//! - [`ErrorCode`] - Standard error codes
//! - [`RequestId`] - UUID v7 request identifier
//! - [`SemanticVersion`] - `SemVer` 2.0 version type
//!
//! ## Example
//!
//! ```rust
//! use themis_core::{Contract, Version};
//!
//! let version = Version::new(1, 0, 0);
//! assert_eq!(version.to_string(), "1.0.0");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod contract;
pub mod error;
pub mod operation;
pub mod schema;
pub mod version;

// Re-export main types at crate root for convenience
pub use contract::Contract;
pub use error::{ThemisError, ThemisResult};
pub use operation::Operation;
pub use schema::Schema;
pub use version::Version;

// Re-export shared platform types for convenience
pub use themis_platform_types::{
    ErrorCode, FieldError, RequestId, SemanticVersion, ThemisErrorEnvelope,
};
