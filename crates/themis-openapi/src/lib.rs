//! # Themis `OpenAPI`
//!
//! `OpenAPI` 3.1 parser and normalizer for Themis.
//!
//! This crate provides functionality to parse `OpenAPI` 3.1 specifications and
//! normalize them into the unified Themis contract model.
//!
//! ## Example
//!
//! ```rust,ignore
//! use themis_openapi::parse_openapi;
//!
//! let yaml = std::fs::read_to_string("api.yaml")?;
//! let contract = parse_openapi(&yaml)?;
//! println!("Parsed {} operations", contract.operation_count());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
// TODO: Fix these clippy warnings in a follow-up PR
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]

pub mod normalizer;
pub mod parser;
pub mod validator;

pub use normalizer::{NormalizerOptions, OpenApiNormalizer};
pub use parser::parse_openapi;
pub use validator::{
    validate_contract, validate_openapi, Severity, ValidationIssue, ValidationResult,
};
