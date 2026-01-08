//! JSON Schema code generator.
//!
//! Generates JSON Schema files from Themis contracts for use with
//! generic code generation tools like `quicktype`, `datamodel-code-generator`, etc.

mod generator;
mod types;

pub use generator::JsonSchemaGenerator;
