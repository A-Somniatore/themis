//! Themis Protobuf v3 Parser
//!
//! This crate provides parsing capabilities for Protocol Buffer v3 contract files,
//! extracting service definitions, RPC methods, and message types into the unified
//! Themis [`Contract`] model.
//!
//! # Features
//!
//! - Parse `.proto` files to extract service definitions
//! - Extract RPC methods as operations with full metadata
//! - Convert message types to Themis schemas
//! - Support for Themis extensions (`themis.service`, `themis.operation`, etc.)
//! - Support for Google API HTTP annotations
//!
//! # Example
//!
//! ```rust,ignore
//! use themis_protobuf::parse_proto;
//!
//! let proto_content = r#"
//! syntax = "proto3";
//! package myservice.v1;
//!
//! service MyService {
//!     rpc GetItem(GetItemRequest) returns (GetItemResponse);
//! }
//!
//! message GetItemRequest {
//!     string id = 1;
//! }
//!
//! message GetItemResponse {
//!     string id = 1;
//!     string name = 2;
//! }
//! "#;
//!
//! let contract = parse_proto(proto_content)?;
//! assert_eq!(contract.operations.len(), 1);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod normalizer;
pub mod parser;
pub mod validator;

pub use error::{ProtobufError, Result};
pub use parser::{parse_proto, parse_proto_file, ProtoParser};
pub use validator::ProtoValidator;

use themis_core::Contract;

/// Parse a protobuf string and return a Themis Contract.
///
/// This is a convenience function that combines parsing and normalization.
///
/// # Arguments
///
/// * `content` - The protobuf file content as a string
/// * `service_name` - The service name to use if not specified in the proto
///
/// # Errors
///
/// Returns [`ProtobufError`] if parsing fails or the proto is invalid.
///
/// # Example
///
/// ```rust,ignore
/// use themis_protobuf::parse;
///
/// let contract = parse(proto_content, "my-service")?;
/// ```
pub fn parse(content: &str, service_name: &str) -> Result<Contract> {
    let parser = ProtoParser::new();
    parser.parse(content, service_name)
}
