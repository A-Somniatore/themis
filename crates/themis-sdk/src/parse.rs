//! Contract parsing functionality.
//!
//! This module provides functions for parsing contracts from various sources.

use std::path::Path;

use themis_core::contract::ContractFormat;
use themis_core::Contract;

use crate::error::{SdkError, SdkResult};

/// Parse a contract from a file path.
///
/// The format is automatically detected based on file content.
///
/// # Arguments
///
/// * `path` - Path to the contract file
///
/// # Returns
///
/// The parsed contract
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The contract format cannot be detected
/// - The contract is invalid
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::parse::parse_file;
///
/// let contract = parse_file("api.yaml")?;
/// println!("Parsed {} operations", contract.operations.len());
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> SdkResult<Contract> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| SdkError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse_string(&content)
}

/// Parse a contract from a string.
///
/// The format is automatically detected based on content.
///
/// # Arguments
///
/// * `content` - The contract content as a string
///
/// # Returns
///
/// The parsed contract
///
/// # Errors
///
/// Returns an error if:
/// - The contract format cannot be detected
/// - The contract is invalid
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::parse::parse_string;
///
/// let yaml = r#"
/// openapi: "3.1.0"
/// info:
///   title: My API
///   version: "1.0.0"
/// paths: {}
/// "#;
/// let contract = parse_string(yaml)?;
/// ```
pub fn parse_string(content: &str) -> SdkResult<Contract> {
    // Detect format and parse
    let format = detect_format(content)?;
    parse_with_format(content, format)
}

/// Parse a contract with a specific format.
///
/// # Arguments
///
/// * `content` - The contract content as a string
/// * `format` - The contract format
///
/// # Returns
///
/// The parsed contract
///
/// # Errors
///
/// Returns an error if the contract is invalid
pub fn parse_with_format(content: &str, format: ContractFormat) -> SdkResult<Contract> {
    match format {
        ContractFormat::OpenApi => {
            themis_openapi::parse_openapi(content).map_err(|e| SdkError::Parse {
                message: e.to_string(),
            })
        }
        ContractFormat::Protobuf => {
            // Protobuf needs a service name, we use a placeholder that gets updated
            themis_protobuf::parse(content, "Service").map_err(|e| SdkError::Parse {
                message: e.to_string(),
            })
        }
        ContractFormat::GraphQl => {
            // GraphQL needs a service name, we use a placeholder
            themis_graphql::parse(content, "Service").map_err(|e| SdkError::Parse {
                message: e.to_string(),
            })
        }
        ContractFormat::AsyncApi => {
            themis_asyncapi::parse(content).map_err(|e| SdkError::Parse {
                message: e.to_string(),
            })
        }
    }
}

/// Detect the contract format from content.
///
/// # Arguments
///
/// * `content` - The contract content as a string
///
/// # Returns
///
/// The detected contract format
///
/// # Errors
///
/// Returns an error if the format cannot be detected
pub fn detect_format(content: &str) -> SdkResult<ContractFormat> {
    // Try to detect OpenAPI (YAML or JSON with openapi field)
    if content.contains("openapi:") || content.contains("\"openapi\"") {
        return Ok(ContractFormat::OpenApi);
    }

    // Try to detect AsyncAPI
    if content.contains("asyncapi:") || content.contains("\"asyncapi\"") {
        return Ok(ContractFormat::AsyncApi);
    }

    // Try to detect GraphQL (type definitions, schema keyword)
    if content.contains("type Query")
        || content.contains("type Mutation")
        || content.contains("schema {")
    {
        return Ok(ContractFormat::GraphQl);
    }

    // Try to detect Protobuf (syntax, message, service keywords)
    if content.contains("syntax = \"proto")
        || (content.contains("message ") && content.contains("{"))
        || content.contains("service ")
    {
        return Ok(ContractFormat::Protobuf);
    }

    Err(SdkError::UnsupportedFormat {
        format: "unknown".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_openapi_yaml() {
        let content = r#"
openapi: "3.1.0"
info:
  title: Test API
  version: "1.0.0"
paths: {}
"#;
        let format = detect_format(content).unwrap();
        assert_eq!(format, ContractFormat::OpenApi);
    }

    #[test]
    fn test_detect_openapi_json() {
        let content = r#"{"openapi": "3.1.0", "info": {}, "paths": {}}"#;
        let format = detect_format(content).unwrap();
        assert_eq!(format, ContractFormat::OpenApi);
    }

    #[test]
    fn test_detect_asyncapi() {
        let content = r#"
asyncapi: "3.0.0"
info:
  title: Test Events
  version: "1.0.0"
channels: {}
"#;
        let format = detect_format(content).unwrap();
        assert_eq!(format, ContractFormat::AsyncApi);
    }

    #[test]
    fn test_detect_graphql() {
        let content = r#"
type Query {
  users: [User!]!
}

type User {
  id: ID!
  name: String!
}
"#;
        let format = detect_format(content).unwrap();
        assert_eq!(format, ContractFormat::GraphQl);
    }

    #[test]
    fn test_detect_protobuf() {
        let content = r#"
syntax = "proto3";

message User {
  string id = 1;
  string name = 2;
}
"#;
        let format = detect_format(content).unwrap();
        assert_eq!(format, ContractFormat::Protobuf);
    }

    #[test]
    fn test_detect_unknown_format() {
        let content = "random text that is not a contract";
        let result = detect_format(content);
        assert!(result.is_err());
    }
}
