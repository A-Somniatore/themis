//! Python code generation.
//!
//! This module generates Python dataclasses, httpx clients, and
//! FastAPI handler signatures from Themis contracts.

mod generator;
mod types;

pub use generator::PythonGenerator;
