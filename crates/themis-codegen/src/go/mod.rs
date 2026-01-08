//! Go code generator for Themis contracts.
//!
//! This module provides code generation for Go, producing:
//! - Type definitions as Go structs with JSON tags
//! - Handler interfaces for API operations
//! - Request/response types
//!
//! # Example
//!
//! ```ignore
//! use themis_codegen::{CodeGenerator, GoGenerator};
//!
//! let generator = GoGenerator::new();
//! let files = generator.generate(&contract)?;
//!
//! for (filename, content) in files {
//!     println!("Generated: {}", filename);
//! }
//! ```

mod generator;
mod types;

pub use generator::GoGenerator;
