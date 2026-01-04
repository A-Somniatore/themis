//! Contract model representing a unified API contract.
//!
//! The [`Contract`] struct is the central data structure in Themis, representing
//! any API contract regardless of its source format (OpenAPI, Protobuf, GraphQL, etc.).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::operation::Operation;
use crate::schema::Schema;
use crate::version::Version;

/// The source format of a contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractFormat {
    /// OpenAPI 3.1 specification
    OpenApi,
    /// Protocol Buffers v3
    Protobuf,
    /// GraphQL Schema Definition Language
    GraphQl,
    /// AsyncAPI 3.0 specification
    AsyncApi,
}

/// Metadata about the contract and its owning service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Unique service identifier (e.g., "users-service")
    pub service_name: String,

    /// Human-readable service description
    #[serde(default)]
    pub description: Option<String>,

    /// Team or individual responsible for this service
    #[serde(default)]
    pub owner: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Documentation URL
    #[serde(default)]
    pub documentation_url: Option<String>,
}

/// A unified API contract representation.
///
/// This struct normalizes contracts from different source formats (OpenAPI, Protobuf,
/// GraphQL, AsyncAPI) into a common model that can be validated, compared, and used
/// for code generation.
///
/// # Example
///
/// ```rust
/// use themis_core::{Contract, Version};
/// use themis_core::contract::{ContractFormat, ContractMetadata};
///
/// let contract = Contract {
///     format: ContractFormat::OpenApi,
///     version: Version::new(1, 0, 0),
///     metadata: ContractMetadata {
///         service_name: "users-service".to_string(),
///         description: Some("User management API".to_string()),
///         owner: Some("platform-team".to_string()),
///         repository: None,
///         documentation_url: None,
///     },
///     operations: Default::default(),
///     schemas: Default::default(),
///     security_schemes: Default::default(),
/// };
///
/// assert_eq!(contract.metadata.service_name, "users-service");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    /// The source format of this contract
    pub format: ContractFormat,

    /// Semantic version of the contract
    pub version: Version,

    /// Contract and service metadata
    pub metadata: ContractMetadata,

    /// All operations defined in this contract, keyed by operation ID
    pub operations: HashMap<String, Operation>,

    /// All schemas defined in this contract, keyed by schema name
    pub schemas: HashMap<String, Schema>,

    /// Security schemes available in this contract
    pub security_schemes: HashMap<String, SecurityScheme>,
}

impl Contract {
    /// Creates a new contract with the given metadata.
    pub fn new(
        format: ContractFormat,
        version: Version,
        service_name: impl Into<String>,
    ) -> Self {
        Self {
            format,
            version,
            metadata: ContractMetadata {
                service_name: service_name.into(),
                description: None,
                owner: None,
                repository: None,
                documentation_url: None,
            },
            operations: HashMap::new(),
            schemas: HashMap::new(),
            security_schemes: HashMap::new(),
        }
    }

    /// Returns the service name from metadata.
    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.metadata.service_name
    }

    /// Returns the number of operations in this contract.
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Returns the number of schemas in this contract.
    #[must_use]
    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }
}

/// A security scheme definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityScheme {
    /// Type of security scheme
    #[serde(rename = "type")]
    pub scheme_type: SecuritySchemeType,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
}

/// Types of security schemes supported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecuritySchemeType {
    /// HTTP authentication (Bearer, Basic, etc.)
    Http {
        /// The HTTP authentication scheme (e.g., "bearer", "basic")
        scheme: String,
        /// Format hint for bearer tokens (e.g., "JWT")
        #[serde(default)]
        bearer_format: Option<String>,
    },
    /// API key authentication
    ApiKey {
        /// Where the API key is passed
        #[serde(rename = "in")]
        location: ApiKeyLocation,
        /// Name of the header, query param, or cookie
        name: String,
    },
    /// OAuth2 authentication
    OAuth2,
    /// OpenID Connect authentication
    OpenIdConnect {
        /// OpenID Connect discovery URL
        openid_connect_url: String,
    },
    /// Mutual TLS (mTLS) authentication
    MutualTls,
}

/// Location where an API key is passed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation {
    /// In a request header
    Header,
    /// In query parameters
    Query,
    /// In a cookie
    Cookie,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let contract = Contract::new(
            ContractFormat::OpenApi,
            Version::new(1, 0, 0),
            "test-service",
        );

        assert_eq!(contract.service_name(), "test-service");
        assert_eq!(contract.format, ContractFormat::OpenApi);
        assert_eq!(contract.operation_count(), 0);
        assert_eq!(contract.schema_count(), 0);
    }

    #[test]
    fn test_contract_serialization() {
        let contract = Contract::new(
            ContractFormat::OpenApi,
            Version::new(1, 2, 3),
            "users-service",
        );

        let json = serde_json::to_string(&contract).unwrap();
        let deserialized: Contract = serde_json::from_str(&json).unwrap();

        assert_eq!(contract, deserialized);
    }

    #[test]
    fn test_contract_format_serialization() {
        assert_eq!(
            serde_json::to_string(&ContractFormat::OpenApi).unwrap(),
            "\"openapi\""
        );
        assert_eq!(
            serde_json::to_string(&ContractFormat::Protobuf).unwrap(),
            "\"protobuf\""
        );
        assert_eq!(
            serde_json::to_string(&ContractFormat::GraphQl).unwrap(),
            "\"graphql\""
        );
        assert_eq!(
            serde_json::to_string(&ContractFormat::AsyncApi).unwrap(),
            "\"asyncapi\""
        );
    }
}
