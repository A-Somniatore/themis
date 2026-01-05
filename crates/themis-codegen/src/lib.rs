//! Code generation for Themis contracts.
//!
//! This crate generates strongly-typed code from Themis contracts in multiple
//! languages including Rust, TypeScript, and Python.
//!
//! # Example
//!
//! ```ignore
//! use themis_codegen::{CodeGenerator, RustGenerator, GeneratorConfig};
//! use themis_core::Contract;
//!
//! let contract = // ... parse contract
//! let config = GeneratorConfig::default();
//! let generator = RustGenerator::new(config);
//!
//! let output = generator.generate(&contract)?;
//! println!("{}", output.types);
//! ```
//!
//! # Generated Output
//!
//! Each generator produces:
//! - **Types**: Request/response structs and enums
//! - **Errors**: Error types with proper conversions
//! - **Handlers**: Trait definitions for implementing operations
//!
//! # Supported Languages
//!
//! - Rust (MVP)
//! - TypeScript (MVP)
//! - Python (MVP)
//! - C++ (future)
//! - Go (future)

// Allow some pedantic clippy lints that are overly strict for codegen
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]

mod config;
mod error;
mod rust;
mod traits;

pub use config::{GeneratorConfig, NamingConvention};
pub use error::{CodegenError, CodegenResult};
pub use rust::RustGenerator;
pub use traits::{CodeGenerator, GeneratedCode, GeneratedFile};
