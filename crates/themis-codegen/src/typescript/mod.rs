//! TypeScript code generation.
//!
//! This module generates TypeScript interfaces, fetch clients, and
//! Express/Fastify handler types from Themis contracts.

mod generator;
mod types;

pub use generator::TypeScriptGenerator;
