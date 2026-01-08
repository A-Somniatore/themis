//! GraphQL schema parser and normalizer for Themis contracts.
//!
//! This crate provides parsing and normalization of GraphQL SDL (Schema Definition Language)
//! into the unified Themis [`Contract`] model.
//!
//! # Supported Features
//!
//! - Type definitions (Object, Input, Interface, Union, Enum, Scalar)
//! - Query, Mutation, and Subscription operations
//! - Field arguments and types
//! - Directives (@deprecated, etc.)
//! - Schema-level documentation (descriptions)
//!
//! # Example
//!
//! ```ignore
//! use themis_graphql::parse_graphql;
//!
//! let schema = r#"
//! type Query {
//!     user(id: ID!): User
//!     users: [User!]!
//! }
//!
//! type User {
//!     id: ID!
//!     name: String!
//!     email: String
//! }
//! "#;
//!
//! let contract = parse_graphql(schema, "users-service")?;
//! println!("Found {} operations", contract.operations.len());
//! ```

pub mod error;
pub mod normalizer;
pub mod parser;
pub mod validator;

pub use error::{GraphqlError, Result};
pub use normalizer::NormalizerOptions;
pub use parser::{parse_graphql, parse_graphql_file, GraphqlParser};
pub use validator::{GraphqlValidator, ValidationResult};

use themis_core::Contract;

/// Parses GraphQL SDL content and returns a Themis Contract.
///
/// This is a convenience function that creates a default parser
/// and parses the provided content.
///
/// # Arguments
///
/// * `content` - The GraphQL SDL content to parse
/// * `service_name` - The name of the service (used for metadata)
///
/// # Errors
///
/// Returns [`GraphqlError`] if:
/// - The SDL syntax is invalid
/// - No Query type is defined
/// - Type definitions are invalid
///
/// # Example
///
/// ```ignore
/// use themis_graphql::parse;
///
/// let schema = r#"
/// type Query {
///     hello: String!
/// }
/// "#;
///
/// let contract = parse(schema, "hello-service")?;
/// ```
pub fn parse(content: &str, service_name: &str) -> Result<Contract> {
    parse_graphql(content, service_name)
}
