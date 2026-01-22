//! Themis SDK - Programmatic Access to Contract Governance
//!
//! This crate provides a unified Rust SDK for working with Themis contract governance.
//! It exposes the functionality of the Themis CLI as a library, enabling programmatic
//! access to contract parsing, validation, linting, compatibility checking, and code generation.
//!
//! # Features
//!
//! - **Multi-format parsing**: OpenAPI 3.1, Protobuf v3, GraphQL SDL, AsyncAPI 3.0
//! - **Validation**: Schema validation with comprehensive error reporting
//! - **Linting**: Configurable linting rules for contract quality
//! - **Compatibility**: Breaking change detection between contract versions
//! - **Code Generation**: Generate types and handlers for Rust, TypeScript, Python, Go, C++
//! - **Artifacts**: Create and publish immutable contract artifacts
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use themis_sdk::{Themis, ContractFormat};
//!
//! // Create a Themis instance
//! let themis = Themis::new();
//!
//! // Parse an OpenAPI contract
//! let contract = themis.parse_file("api.yaml", ContractFormat::OpenApi)?;
//!
//! // Validate the contract
//! let result = themis.validate(&contract)?;
//! if !result.is_valid() {
//!     for error in result.errors() {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//!
//! // Lint the contract
//! let lint_result = themis.lint(&contract, Default::default())?;
//!
//! // Generate Rust code
//! let code = themis.generate(&contract, Language::Rust, Default::default())?;
//! ```
//!
//! # Architecture
//!
//! The SDK is organized around the [`Themis`] struct, which provides the main entry point
//! for all operations. Individual operations are also available through dedicated modules:
//!
//! - [`parse`] - Contract parsing for all supported formats
//! - [`validate`] - Contract validation
//! - [`lint`] - Contract linting with configurable rules
//! - [`compat`] - Compatibility checking between versions
//! - [`codegen`] - Code generation for multiple languages
//! - [`artifact`] - Artifact creation and management

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod sdk;

pub mod artifact;
pub mod codegen;
pub mod compat;
pub mod lint;
pub mod parse;
pub mod validate;

pub use error::{SdkError, SdkResult};
pub use sdk::Themis;

// Re-export core types for convenience
pub use themis_core::contract::ContractFormat;
pub use themis_core::{Contract, Operation, Schema, Version};

// Re-export common configuration types
pub use themis_codegen::GeneratorConfig;
pub use themis_lint::LintConfig;

/// Supported programming languages for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Rust with serde derives
    Rust,
    /// TypeScript with interfaces
    TypeScript,
    /// Python with dataclasses
    Python,
    /// Go with structs
    Go,
    /// C++ with structs
    Cpp,
    /// JSON Schema output
    JsonSchema,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::TypeScript => write!(f, "typescript"),
            Self::Python => write!(f, "python"),
            Self::Go => write!(f, "go"),
            Self::Cpp => write!(f, "cpp"),
            Self::JsonSchema => write!(f, "json-schema"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_display() {
        assert_eq!(Language::Rust.to_string(), "rust");
        assert_eq!(Language::TypeScript.to_string(), "typescript");
        assert_eq!(Language::Python.to_string(), "python");
        assert_eq!(Language::Go.to_string(), "go");
        assert_eq!(Language::Cpp.to_string(), "cpp");
        assert_eq!(Language::JsonSchema.to_string(), "json-schema");
    }
}
