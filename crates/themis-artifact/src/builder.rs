//! Artifact builder for constructing artifacts.

use crate::artifact::{Artifact, ArtifactMetadata, Checksum, ARTIFACT_SCHEMA_VERSION};
use crate::error::{ArtifactError, ArtifactResult};
use crate::operation::ArtifactOperation;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use std::collections::HashMap;
use themis_core::{Contract, Schema};

/// Builder for creating artifacts.
///
/// # Example
///
/// ```ignore
/// use themis_artifact::ArtifactBuilder;
///
/// let artifact = ArtifactBuilder::new()
///     .service("users-service")
///     .version("1.0.0")
///     .format("openapi", "3.1.0")
///     .owner("platform-team")
///     .build()?;
/// ```
#[derive(Debug, Default)]
pub struct ArtifactBuilder {
    version: Option<String>,
    service: Option<String>,
    format: Option<String>,
    format_version: Option<String>,
    git_commit: Option<String>,
    git_repository: Option<String>,
    owner: Option<String>,
    operations: Vec<ArtifactOperation>,
    schemas: HashMap<String, Schema>,
    raw_contract: Option<Vec<u8>>,
    custom_metadata: HashMap<String, serde_json::Value>,
}

impl ArtifactBuilder {
    /// Creates a new artifact builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder from an existing contract.
    ///
    /// This extracts operations and schemas from the contract.
    pub fn from_contract(contract: &Contract) -> Self {
        let mut builder = Self::new()
            .service(&contract.metadata.service_name)
            .version(contract.version.to_string());

        // Set format based on contract format
        let (format, format_version) = match contract.format {
            themis_core::contract::ContractFormat::OpenApi => ("openapi", "3.1.0"),
            themis_core::contract::ContractFormat::Protobuf => ("protobuf", "3"),
            themis_core::contract::ContractFormat::GraphQl => ("graphql", "June2018"),
            themis_core::contract::ContractFormat::AsyncApi => ("asyncapi", "3.0.0"),
        };
        builder = builder.format(format, format_version);

        // Set owner from contract metadata if available
        if let Some(owner) = &contract.metadata.owner {
            builder = builder.owner(owner);
        }

        // Set git repository from contract metadata if available
        if let Some(repo) = &contract.metadata.repository {
            builder = builder.git_repository(repo);
        }

        // Add operations from contract
        for (op_id, operation) in &contract.operations {
            let method = operation
                .method
                .as_ref()
                .map(|m| m.to_string().to_uppercase())
                .unwrap_or_else(|| "GET".to_string());
            let path = operation.path.as_deref().unwrap_or("/");

            let mut artifact_op = ArtifactOperation::new(op_id, &method, path);

            // Set summary if present
            if let Some(summary) = &operation.summary {
                artifact_op = artifact_op.with_summary(summary);
            }

            // Set description if present
            if let Some(description) = &operation.description {
                artifact_op = artifact_op.with_description(description);
            }

            // Set deprecated flag
            if operation.deprecated {
                artifact_op.deprecated = true;
            }

            builder.operations.push(artifact_op);
        }

        // Add schemas from contract
        for (name, schema) in &contract.schemas {
            builder.schemas.insert(name.clone(), schema.clone());
        }

        builder
    }

    /// Sets the service name.
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    /// Sets the contract version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the contract format and version.
    pub fn format(mut self, format: impl Into<String>, version: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self.format_version = Some(version.into());
        self
    }

    /// Sets the git commit SHA.
    pub fn git_commit(mut self, commit: impl Into<String>) -> Self {
        self.git_commit = Some(commit.into());
        self
    }

    /// Sets the git repository URL.
    pub fn git_repository(mut self, repository: impl Into<String>) -> Self {
        self.git_repository = Some(repository.into());
        self
    }

    /// Sets the owner.
    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Adds an operation to the artifact.
    pub fn add_operation(mut self, operation: ArtifactOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Adds multiple operations to the artifact.
    pub fn add_operations(
        mut self,
        operations: impl IntoIterator<Item = ArtifactOperation>,
    ) -> Self {
        self.operations.extend(operations);
        self
    }

    /// Adds a schema to the artifact.
    pub fn add_schema(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.schemas.insert(name.into(), schema);
        self
    }

    /// Sets the raw contract content.
    pub fn raw_contract(mut self, content: impl Into<Vec<u8>>) -> Self {
        self.raw_contract = Some(content.into());
        self
    }

    /// Adds custom metadata.
    pub fn custom_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.custom_metadata.insert(key.into(), value.into());
        self
    }

    /// Builds the artifact.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing.
    pub fn build(self) -> ArtifactResult<Artifact> {
        let service = self
            .service
            .ok_or_else(|| ArtifactError::missing_field("service"))?;
        let version = self
            .version
            .ok_or_else(|| ArtifactError::missing_field("version"))?;
        let format = self.format.unwrap_or_else(|| "openapi".to_string());
        let format_version = self.format_version.unwrap_or_else(|| "3.1.0".to_string());

        let raw_contract = self.raw_contract.map(|bytes| STANDARD.encode(bytes));

        let metadata = ArtifactMetadata {
            created_at: Utc::now(),
            git_commit: self.git_commit,
            git_repository: self.git_repository,
            owner: self.owner,
            custom: self.custom_metadata,
        };

        // Build artifact without final checksum
        let mut artifact = Artifact {
            schema: ARTIFACT_SCHEMA_VERSION.to_string(),
            version,
            service,
            format,
            format_version,
            metadata,
            checksum: Checksum::sha256(""),
            operations: self.operations,
            schemas: self.schemas,
            raw_contract,
        };

        // Compute and set the checksum
        artifact.checksum = Checksum::sha256(artifact.compute_checksum());

        Ok(artifact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .build()
            .unwrap();

        assert_eq!(artifact.service, "test-service");
        assert_eq!(artifact.version, "1.0.0");
        assert_eq!(artifact.format, "openapi");
        assert_eq!(artifact.format_version, "3.1.0");
    }

    #[test]
    fn test_builder_with_format() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .format("protobuf", "3")
            .build()
            .unwrap();

        assert_eq!(artifact.format, "protobuf");
        assert_eq!(artifact.format_version, "3");
    }

    #[test]
    fn test_builder_with_git_info() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .git_commit("abc123")
            .git_repository("https://github.com/org/repo")
            .build()
            .unwrap();

        assert_eq!(artifact.metadata.git_commit, Some("abc123".to_string()));
        assert_eq!(
            artifact.metadata.git_repository,
            Some("https://github.com/org/repo".to_string())
        );
    }

    #[test]
    fn test_builder_with_owner() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .owner("platform-team")
            .build()
            .unwrap();

        assert_eq!(artifact.metadata.owner, Some("platform-team".to_string()));
    }

    #[test]
    fn test_builder_with_operations() {
        let op = ArtifactOperation::new("getUser", "GET", "/users/{id}");
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .add_operation(op)
            .build()
            .unwrap();

        assert_eq!(artifact.operations.len(), 1);
        assert_eq!(artifact.operations[0].id, "getUser");
    }

    #[test]
    fn test_builder_with_raw_contract() {
        let raw = b"openapi: 3.1.0\ninfo:\n  title: Test";
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .raw_contract(raw.to_vec())
            .build()
            .unwrap();

        assert!(artifact.raw_contract.is_some());
        let decoded = STANDARD.decode(artifact.raw_contract.unwrap()).unwrap();
        assert_eq!(decoded, raw);
    }

    #[test]
    fn test_builder_with_custom_metadata() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .custom_metadata("team", serde_json::json!("backend"))
            .custom_metadata("priority", serde_json::json!(1))
            .build()
            .unwrap();

        assert_eq!(
            artifact.metadata.custom.get("team"),
            Some(&serde_json::json!("backend"))
        );
        assert_eq!(
            artifact.metadata.custom.get("priority"),
            Some(&serde_json::json!(1))
        );
    }

    #[test]
    fn test_builder_checksum_is_set() {
        let artifact = ArtifactBuilder::new()
            .service("test-service")
            .version("1.0.0")
            .build()
            .unwrap();

        assert!(!artifact.checksum.value.is_empty());
        assert!(artifact.verify_checksum().is_ok());
    }

    #[test]
    fn test_builder_missing_service() {
        let result = ArtifactBuilder::new().version("1.0.0").build();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ArtifactError::MissingField { .. }
        ));
    }

    #[test]
    fn test_builder_missing_version() {
        let result = ArtifactBuilder::new().service("test-service").build();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ArtifactError::MissingField { .. }
        ));
    }

    #[test]
    fn test_builder_from_contract() {
        use themis_core::contract::{ContractFormat, ContractMetadata};
        use themis_core::operation::HttpMethod;
        use themis_core::{Contract, Operation, Version};

        let mut operations = HashMap::new();
        operations.insert(
            "getUser".to_string(),
            Operation {
                operation_id: "getUser".to_string(),
                method: Some(HttpMethod::Get),
                path: Some("/users/{id}".to_string()),
                summary: Some("Get a user".to_string()),
                description: None,
                tags: vec!["users".to_string()],
                request_body: None,
                responses: HashMap::new(),
                parameters: vec![],
                deprecated: false,
                security: vec![],
                themis_metadata: None,
            },
        );

        let contract = Contract {
            format: ContractFormat::OpenApi,
            version: Version::new(1, 0, 0),
            metadata: ContractMetadata {
                service_name: "users-service".to_string(),
                description: None,
                owner: Some("platform-team".to_string()),
                repository: None,
                documentation_url: None,
            },
            operations,
            schemas: HashMap::new(),
            security_schemes: HashMap::new(),
        };

        let artifact = ArtifactBuilder::from_contract(&contract).build().unwrap();

        assert_eq!(artifact.service, "users-service");
        assert_eq!(artifact.version, "1.0.0");
        assert_eq!(artifact.operations.len(), 1);
        assert_eq!(artifact.operations[0].id, "getUser");
        assert_eq!(artifact.operations[0].method, "GET");
        assert_eq!(artifact.metadata.owner, Some("platform-team".to_string()));
    }
}
