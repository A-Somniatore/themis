//! `AsyncAPI` 3.0 parser and validator for Themis contract governance.
//!
//! This crate provides functionality for parsing `AsyncAPI` 3.0 specifications
//! and converting them into Themis's unified contract model.
//!
//! # Features
//!
//! - Parse `AsyncAPI` 3.0 YAML/JSON specifications
//! - Convert to Themis Contract format
//! - Validate `AsyncAPI` documents against rules
//! - Normalize specifications for consistency
//!
//! # Example
//!
//! ```ignore
//! use themis_asyncapi::parse;
//!
//! let yaml = r#"
//! asyncapi: 3.0.0
//! info:
//!   title: User Events
//!   version: 1.0.0
//! channels:
//!   userCreated:
//!     messages:
//!       userCreatedMessage:
//!         payload:
//!           type: object
//!           properties:
//!             userId:
//!               type: string
//! "#;
//!
//! let contract = parse(yaml)?;
//! println!("Found {} operations", contract.operations.len());
//! ```

mod error;
mod normalizer;
mod parser;
mod validator;

pub use error::AsyncApiError;
pub use normalizer::{AsyncApiNormalizer, NormalizerOptions};
pub use parser::AsyncApiParser;
pub use validator::AsyncApiValidator;

use themis_core::contract::Contract;

/// Convenience function to parse an `AsyncAPI` specification.
///
/// # Arguments
///
/// * `input` - The `AsyncAPI` specification as a YAML or JSON string
///
/// # Returns
///
/// A parsed `Contract` representing the `AsyncAPI` specification
///
/// # Errors
///
/// Returns `AsyncApiError` if parsing fails
pub fn parse(input: &str) -> Result<Contract, AsyncApiError> {
    AsyncApiParser::parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_convenience_function() {
        let yaml = r#"
asyncapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
channels: {}
operations: {}
"#;

        let result = parse(yaml);
        assert!(result.is_ok());
    }
}
