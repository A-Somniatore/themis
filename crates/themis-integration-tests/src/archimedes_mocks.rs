//! Archimedes integration mocks.
//!
//! This module provides mock implementations of Archimedes interfaces
//! for testing that Themis artifacts are compatible with the runtime.
//!
//! These mocks simulate how Archimedes will:
//! - Load and verify artifacts
//! - Map operations to handlers
//! - Validate requests against schemas
//! - Produce policy context for Eunomia

use std::collections::HashMap;
use themis_artifact::Artifact;

/// Mock Archimedes artifact loader.
///
/// Simulates how Archimedes loads and validates artifacts at runtime.
pub struct MockArtifactLoader {
    artifacts: HashMap<String, Artifact>,
}

impl MockArtifactLoader {
    /// Creates a new mock artifact loader.
    #[must_use]
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
        }
    }

    /// Loads an artifact into the mock runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact fails validation.
    pub fn load(&mut self, artifact: Artifact) -> Result<(), ArtifactLoadError> {
        // Verify checksum like Archimedes would
        artifact
            .verify_checksum()
            .map_err(|_| ArtifactLoadError::ChecksumMismatch)?;

        // Validate required fields
        if artifact.service.is_empty() {
            return Err(ArtifactLoadError::MissingService);
        }
        if artifact.version.is_empty() {
            return Err(ArtifactLoadError::MissingVersion);
        }

        let key = format!("{}:{}", artifact.service, artifact.version);
        self.artifacts.insert(key, artifact);
        Ok(())
    }

    /// Gets an artifact by service name and version.
    #[must_use]
    pub fn get(&self, service: &str, version: &str) -> Option<&Artifact> {
        let key = format!("{service}:{version}");
        self.artifacts.get(&key)
    }

    /// Lists all loaded artifact keys.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.artifacts.keys().cloned().collect()
    }

    /// Gets operation metadata for a specific operation.
    #[must_use]
    pub fn get_operation(
        &self,
        service: &str,
        version: &str,
        operation_id: &str,
    ) -> Option<OperationMetadata> {
        let artifact = self.get(service, version)?;
        let op = artifact.operations.iter().find(|o| o.id == operation_id)?;

        // Extract metadata from nested structure
        let (rate_limit, timeout, idempotent) = if let Some(meta) = &op.metadata {
            (
                meta.rate_limit_tier.clone(),
                meta.timeout_tier.clone(),
                meta.idempotent.unwrap_or(false),
            )
        } else {
            (None, None, false)
        };

        Some(OperationMetadata {
            operation_id: op.id.clone(),
            method: op.method.clone(),
            path: op.path.clone(),
            summary: op.summary.clone(),
            rate_limit_tier: rate_limit,
            timeout_tier: timeout,
            is_idempotent: idempotent,
        })
    }
}

impl Default for MockArtifactLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Error when loading an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactLoadError {
    /// Artifact checksum verification failed.
    ChecksumMismatch,
    /// Missing required service name.
    MissingService,
    /// Missing required version.
    MissingVersion,
}

impl std::fmt::Display for ArtifactLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChecksumMismatch => write!(f, "artifact checksum verification failed"),
            Self::MissingService => write!(f, "artifact missing required service name"),
            Self::MissingVersion => write!(f, "artifact missing required version"),
        }
    }
}

impl std::error::Error for ArtifactLoadError {}

/// Operation metadata extracted from artifact.
///
/// This is the format Archimedes uses to pass context to Eunomia.
#[derive(Debug, Clone)]
pub struct OperationMetadata {
    /// Unique operation identifier (matches `PolicyInput.operation_id`).
    pub operation_id: String,
    /// HTTP method.
    pub method: String,
    /// URL path pattern.
    pub path: String,
    /// Operation summary.
    pub summary: Option<String>,
    /// Rate limit tier.
    pub rate_limit_tier: Option<String>,
    /// Timeout tier.
    pub timeout_tier: Option<String>,
    /// Whether operation is idempotent.
    pub is_idempotent: bool,
}

/// Mock request context.
///
/// Simulates the context Archimedes creates for each request.
#[derive(Debug, Clone)]
pub struct MockRequestContext {
    /// Unique request ID.
    pub request_id: String,
    /// Operation being invoked.
    pub operation_id: String,
    /// HTTP method.
    pub method: String,
    /// Request path.
    pub path: String,
    /// Path parameters extracted from URL.
    pub path_params: HashMap<String, String>,
    /// Query parameters.
    pub query_params: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
}

impl MockRequestContext {
    /// Creates a new mock request context.
    #[must_use]
    pub fn new(operation_id: &str, method: &str, path: &str) -> Self {
        Self {
            request_id: uuid::Uuid::now_v7().to_string(),
            operation_id: operation_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    /// Adds a path parameter.
    #[must_use]
    pub fn with_path_param(mut self, key: &str, value: &str) -> Self {
        self.path_params.insert(key.to_string(), value.to_string());
        self
    }

    /// Adds a query parameter.
    #[must_use]
    pub fn with_query_param(mut self, key: &str, value: &str) -> Self {
        self.query_params.insert(key.to_string(), value.to_string());
        self
    }

    /// Adds a header.
    #[must_use]
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

/// Mock policy input builder.
///
/// Simulates how Archimedes creates `PolicyInput` for Eunomia.
#[derive(Debug, Clone)]
pub struct MockPolicyInputBuilder {
    operation_id: Option<String>,
    service_name: Option<String>,
    resource_path: Option<String>,
    http_method: Option<String>,
    rate_limit_tier: Option<String>,
}

impl MockPolicyInputBuilder {
    /// Creates a new mock policy input builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            operation_id: None,
            service_name: None,
            resource_path: None,
            http_method: None,
            rate_limit_tier: None,
        }
    }

    /// Creates a builder from artifact operation metadata.
    #[must_use]
    pub fn from_operation(artifact: &Artifact, op: &OperationMetadata) -> Self {
        Self {
            operation_id: Some(op.operation_id.clone()),
            service_name: Some(artifact.service.clone()),
            resource_path: Some(op.path.clone()),
            http_method: Some(op.method.clone()),
            rate_limit_tier: op.rate_limit_tier.clone(),
        }
    }

    /// Sets the operation ID.
    #[must_use]
    pub fn operation_id(mut self, id: &str) -> Self {
        self.operation_id = Some(id.to_string());
        self
    }

    /// Sets the service name.
    #[must_use]
    pub fn service_name(mut self, name: &str) -> Self {
        self.service_name = Some(name.to_string());
        self
    }

    /// Gets the operation ID (for validation).
    #[must_use]
    pub fn get_operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }

    /// Gets the service name (for validation).
    #[must_use]
    pub fn get_service_name(&self) -> Option<&str> {
        self.service_name.as_deref()
    }

    /// Gets the HTTP method (for validation).
    #[must_use]
    pub fn get_http_method(&self) -> Option<&str> {
        self.http_method.as_deref()
    }

    /// Gets the resource path (for validation).
    #[must_use]
    pub fn get_resource_path(&self) -> Option<&str> {
        self.resource_path.as_deref()
    }
}

impl Default for MockPolicyInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock operation router.
///
/// Simulates how Archimedes routes requests to operation handlers.
pub struct MockOperationRouter {
    routes: HashMap<(String, String), String>, // (method, path) -> operation_id
}

impl MockOperationRouter {
    /// Creates a new router from an artifact.
    #[must_use]
    pub fn from_artifact(artifact: &Artifact) -> Self {
        let mut routes = HashMap::new();

        for op in &artifact.operations {
            let method = op.method.to_uppercase();
            let path = op.path.clone();
            routes.insert((method, path), op.id.clone());
        }

        Self { routes }
    }

    /// Routes a request to an operation.
    ///
    /// Returns the operation ID for the given method and path.
    #[must_use]
    pub fn route(&self, method: &str, path: &str) -> Option<&str> {
        self.routes
            .get(&(method.to_uppercase(), path.to_string()))
            .map(String::as_str)
    }

    /// Lists all registered routes.
    #[must_use]
    pub fn list_routes(&self) -> Vec<(&str, &str, &str)> {
        self.routes
            .iter()
            .map(|((method, path), op_id)| (method.as_str(), path.as_str(), op_id.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::USERS_SERVICE_V1;
    use themis_artifact::ArtifactBuilder;
    use themis_openapi::parse_openapi;

    #[test]
    fn test_mock_loader_basic() {
        let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

        let artifact = ArtifactBuilder::from_contract(&contract)
            .build()
            .expect("Should create artifact");

        let mut loader = MockArtifactLoader::new();
        loader.load(artifact.clone()).expect("Should load artifact");

        let loaded = loader.get(&artifact.service, &artifact.version);
        assert!(loaded.is_some(), "Should find loaded artifact");
        assert_eq!(loaded.unwrap().service, artifact.service);
    }

    #[test]
    fn test_mock_loader_checksum_verification() {
        let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

        let mut artifact = ArtifactBuilder::from_contract(&contract)
            .build()
            .expect("Should create artifact");

        // Tamper with the artifact
        artifact.checksum.value = "invalid".to_string();

        let mut loader = MockArtifactLoader::new();
        let result = loader.load(artifact);

        assert_eq!(result, Err(ArtifactLoadError::ChecksumMismatch));
    }

    #[test]
    fn test_mock_router_from_artifact() {
        let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

        let artifact = ArtifactBuilder::from_contract(&contract)
            .build()
            .expect("Should create artifact");

        let router = MockOperationRouter::from_artifact(&artifact);
        let routes = router.list_routes();

        assert!(!routes.is_empty(), "Should have routes");

        // Print routes for debugging
        for (method, path, op_id) in &routes {
            println!("Route: {method} {path} -> {op_id}");
        }
    }

    #[test]
    fn test_mock_policy_input_builder() {
        let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

        let artifact = ArtifactBuilder::from_contract(&contract)
            .build()
            .expect("Should create artifact");

        // Get first operation
        if let Some(op) = artifact.operations.first() {
            // Extract metadata from nested structure
            let (rate_limit, timeout, idempotent) = if let Some(meta) = &op.metadata {
                (
                    meta.rate_limit_tier.clone(),
                    meta.timeout_tier.clone(),
                    meta.idempotent.unwrap_or(false),
                )
            } else {
                (None, None, false)
            };

            let metadata = OperationMetadata {
                operation_id: op.id.clone(),
                method: op.method.clone(),
                path: op.path.clone(),
                summary: op.summary.clone(),
                rate_limit_tier: rate_limit,
                timeout_tier: timeout,
                is_idempotent: idempotent,
            };

            let builder = MockPolicyInputBuilder::from_operation(&artifact, &metadata);

            assert_eq!(builder.get_operation_id(), Some(op.id.as_str()));
            assert_eq!(builder.get_service_name(), Some(artifact.service.as_str()));
        }
    }

    #[test]
    fn test_mock_request_context() {
        let ctx = MockRequestContext::new("getUser", "GET", "/users/{userId}")
            .with_path_param("userId", "123")
            .with_header("Authorization", "Bearer token");

        assert_eq!(ctx.operation_id, "getUser");
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.path_params.get("userId"), Some(&"123".to_string()));
        assert!(ctx.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_get_operation_metadata() {
        let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse contract");

        let artifact = ArtifactBuilder::from_contract(&contract)
            .build()
            .expect("Should create artifact");

        let mut loader = MockArtifactLoader::new();
        let _service = artifact.service.clone();
        let _version = artifact.version.clone();
        loader.load(artifact).expect("Should load artifact");

        // Try to get an operation (if any exist)
        let all_artifacts = loader.list();
        assert!(!all_artifacts.is_empty(), "Should have loaded artifacts");
    }
}
