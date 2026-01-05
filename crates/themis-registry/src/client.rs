//! OCI registry client implementation.

use crate::cache::{ArtifactCache, CacheConfig};
use crate::config::RegistryConfig;
use crate::error::{RegistryError, RegistryResult};
use crate::oci::{
    descriptor_from_data, sha256_hex, OciDescriptor, OciErrors, OciManifest, TagsList,
    OCI_MANIFEST_MEDIA_TYPE, THEMIS_ARTIFACT_MEDIA_TYPE,
};
use crate::reference::ArtifactReference;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, LOCATION};
use reqwest::{Client, StatusCode};
use std::time::Duration;
use themis_artifact::Artifact;
use tracing::{debug, info, trace};

/// OCI registry client for Themis artifacts.
///
/// Implements the OCI Distribution Specification for pushing and pulling
/// contract artifacts.
pub struct RegistryClient {
    config: RegistryConfig,
    http: Client,
    cache: Option<ArtifactCache>,
}

impl RegistryClient {
    /// Creates a new registry client with the given configuration.
    #[must_use]
    pub fn new(config: RegistryConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to create HTTP client");

        let cache = config
            .cache_dir
            .as_ref()
            .and_then(|dir| ArtifactCache::new(CacheConfig::new(dir)).ok());

        Self {
            config,
            http,
            cache,
        }
    }

    /// Creates a client with a custom HTTP client.
    #[must_use]
    pub fn with_http_client(config: RegistryConfig, http: Client) -> Self {
        let cache = config
            .cache_dir
            .as_ref()
            .and_then(|dir| ArtifactCache::new(CacheConfig::new(dir)).ok());

        Self {
            config,
            http,
            cache,
        }
    }

    /// Publishes an artifact to the registry.
    ///
    /// # Workflow
    ///
    /// 1. Check if artifact already exists (returns error if it does)
    /// 2. Upload artifact content as a blob
    /// 3. Upload empty config blob
    /// 4. Create and push manifest
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = RegistryClient::new(config);
    /// client.publish(&artifact).await?;
    /// ```
    pub async fn publish(&self, artifact: &Artifact) -> RegistryResult<()> {
        let reference = ArtifactReference::new(&artifact.service).with_tag(&artifact.version);

        info!(
            service = %artifact.service,
            version = %artifact.version,
            "publishing artifact"
        );

        // Step 1: Check if already exists
        if self.exists(&artifact.service, &artifact.version).await? {
            return Err(RegistryError::already_exists(
                &artifact.service,
                &artifact.version,
            ));
        }

        // Step 2: Serialize artifact
        let artifact_json = artifact.to_json().map_err(|e| {
            RegistryError::SerializationError(format!("failed to serialize artifact: {e}"))
        })?;
        let artifact_bytes = artifact_json.as_bytes();

        // Step 3: Upload artifact blob
        let artifact_descriptor = self
            .upload_blob(&reference, THEMIS_ARTIFACT_MEDIA_TYPE, artifact_bytes)
            .await?;

        debug!(digest = %artifact_descriptor.digest, "uploaded artifact blob");

        // Step 4: Upload empty config
        let config_bytes = b"{}";
        let config_descriptor = self
            .upload_blob(
                &reference,
                "application/vnd.oci.image.config.v1+json",
                config_bytes,
            )
            .await?;

        debug!(digest = %config_descriptor.digest, "uploaded config blob");

        // Step 5: Create and upload manifest
        let manifest = OciManifest::for_artifact(artifact_descriptor, config_descriptor)
            .with_annotation("org.opencontainers.image.title", &artifact.service)
            .with_annotation("org.opencontainers.image.version", &artifact.version)
            .with_annotation("com.themis.artifact.format", &artifact.format)
            .with_annotation("com.themis.artifact.checksum", &artifact.checksum.value);

        self.upload_manifest(&reference, &manifest).await?;

        info!(
            service = %artifact.service,
            version = %artifact.version,
            "published artifact successfully"
        );

        // Update cache
        if let Some(ref cache) = self.cache {
            let _ = cache.put(
                self.config.namespace.as_deref(),
                &artifact.service,
                &artifact.version,
                artifact,
            );
        }

        Ok(())
    }

    /// Fetches an artifact from the registry.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let artifact = client.fetch("users-api", "1.0.0").await?;
    /// ```
    pub async fn fetch(&self, service: &str, version: &str) -> RegistryResult<Artifact> {
        info!(service, version, "fetching artifact");

        // Get manifest
        let reference = ArtifactReference::new(service).with_tag(version);
        let manifest = self.get_manifest(&reference).await?;

        // Get artifact layer
        let layer = manifest.artifact_layer().ok_or_else(|| {
            RegistryError::InvalidManifest("no artifact layer in manifest".into())
        })?;

        // Download artifact blob
        let blob = self.download_blob(&reference, &layer.digest).await?;

        // Parse artifact
        let artifact: Artifact = serde_json::from_slice(&blob)?;

        // Verify checksum if enabled
        if self.config.verify_checksums {
            artifact.verify_checksum().map_err(|e| {
                RegistryError::checksum_mismatch(&artifact.checksum.value, e.to_string())
            })?;
        }

        // Update cache
        if let Some(ref cache) = self.cache {
            let _ = cache.put(
                self.config.namespace.as_deref(),
                service,
                version,
                &artifact,
            );
        }

        info!(service, version, "fetched artifact successfully");
        Ok(artifact)
    }

    /// Fetches an artifact, using cache if available.
    pub async fn fetch_cached(&self, service: &str, version: &str) -> RegistryResult<Artifact> {
        // Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(artifact) = cache.get(self.config.namespace.as_deref(), service, version) {
                debug!(service, version, "using cached artifact");
                return Ok(artifact);
            }
        }

        // Fetch from registry
        self.fetch(service, version).await
    }

    /// Checks if an artifact exists in the registry.
    pub async fn exists(&self, service: &str, version: &str) -> RegistryResult<bool> {
        let reference = ArtifactReference::new(service).with_tag(version);
        let url = self.manifest_url(&reference);

        let response = self
            .http
            .head(&url)
            .headers(self.auth_headers())
            .header(ACCEPT, OCI_MANIFEST_MEDIA_TYPE)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            status => Err(self.handle_error_response(status, response).await),
        }
    }

    /// Lists available versions (tags) for a service.
    pub async fn list_versions(&self, service: &str) -> RegistryResult<Vec<String>> {
        let reference = ArtifactReference::new(service);
        let url = format!(
            "{}/v2/{}/tags/list",
            self.config.base_url(),
            self.repository(&reference)
        );

        let response = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let tags: TagsList = response.json().await?;
                Ok(tags.tags)
            }
            StatusCode::NOT_FOUND => Ok(Vec::new()),
            status => Err(self.handle_error_response(status, response).await),
        }
    }

    /// Deletes an artifact from the registry.
    pub async fn delete(&self, service: &str, version: &str) -> RegistryResult<()> {
        let reference = ArtifactReference::new(service).with_tag(version);

        // Get manifest to find digest
        let manifest = self.get_manifest(&reference).await?;

        // Delete manifest by digest
        let manifest_json = serde_json::to_vec(&manifest)?;
        let digest = format!("sha256:{}", sha256_hex(&manifest_json));
        let url = format!(
            "{}/v2/{}/manifests/{}",
            self.config.base_url(),
            self.repository(&reference),
            digest
        );

        let response = self
            .http
            .delete(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        match response.status() {
            StatusCode::ACCEPTED | StatusCode::OK | StatusCode::NO_CONTENT => {
                // Remove from cache
                if let Some(ref cache) = self.cache {
                    let _ = cache.remove(self.config.namespace.as_deref(), service, version);
                }
                Ok(())
            }
            status => Err(self.handle_error_response(status, response).await),
        }
    }

    /// Returns the cache (if configured).
    #[must_use]
    pub const fn cache(&self) -> Option<&ArtifactCache> {
        self.cache.as_ref()
    }

    // --- Private methods ---

    async fn upload_blob(
        &self,
        reference: &ArtifactReference,
        media_type: &str,
        data: &[u8],
    ) -> RegistryResult<OciDescriptor> {
        let repository = self.repository(reference);

        // Step 1: Start upload session
        let url = format!(
            "{}/v2/{}/blobs/uploads/",
            self.config.base_url(),
            repository
        );
        trace!(%url, "starting blob upload");

        let response = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        if response.status() != StatusCode::ACCEPTED {
            return Err(self
                .handle_error_response(response.status(), response)
                .await);
        }

        // Get upload location
        let upload_url = response
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| RegistryError::InvalidManifest("missing upload location".into()))?
            .to_string();

        // Step 2: Upload content in single PUT
        let digest = format!("sha256:{}", sha256_hex(data));
        let upload_url_with_digest = if upload_url.contains('?') {
            format!("{upload_url}&digest={digest}")
        } else {
            format!("{upload_url}?digest={digest}")
        };

        trace!(url = %upload_url_with_digest, size = data.len(), "uploading blob");

        let response = self
            .http
            .put(&upload_url_with_digest)
            .headers(self.auth_headers())
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await?;

        if response.status() != StatusCode::CREATED {
            return Err(self
                .handle_error_response(response.status(), response)
                .await);
        }

        Ok(descriptor_from_data(media_type, data))
    }

    async fn upload_manifest(
        &self,
        reference: &ArtifactReference,
        manifest: &OciManifest,
    ) -> RegistryResult<()> {
        let url = self.manifest_url(reference);
        let manifest_json = serde_json::to_vec(manifest)?;

        trace!(%url, "uploading manifest");

        let response = self
            .http
            .put(&url)
            .headers(self.auth_headers())
            .header(CONTENT_TYPE, OCI_MANIFEST_MEDIA_TYPE)
            .body(manifest_json)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => Ok(()),
            status => Err(self.handle_error_response(status, response).await),
        }
    }

    async fn get_manifest(&self, reference: &ArtifactReference) -> RegistryResult<OciManifest> {
        let url = self.manifest_url(reference);

        let response = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .header(ACCEPT, OCI_MANIFEST_MEDIA_TYPE)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let manifest: OciManifest = response.json().await?;
                Ok(manifest)
            }
            StatusCode::NOT_FOUND => Err(RegistryError::not_found(
                &reference.service,
                reference.tag_or_latest(),
            )),
            status => Err(self.handle_error_response(status, response).await),
        }
    }

    async fn download_blob(
        &self,
        reference: &ArtifactReference,
        digest: &str,
    ) -> RegistryResult<Vec<u8>> {
        let url = format!(
            "{}/v2/{}/blobs/{}",
            self.config.base_url(),
            self.repository(reference),
            digest
        );

        let response = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.bytes().await?.to_vec()),
            status => Err(self.handle_error_response(status, response).await),
        }
    }

    fn manifest_url(&self, reference: &ArtifactReference) -> String {
        format!(
            "{}/v2/{}/manifests/{}",
            self.config.base_url(),
            self.repository(reference),
            reference.tag_or_latest()
        )
    }

    fn repository(&self, reference: &ArtifactReference) -> String {
        let service = &reference.service;
        match (&self.config.namespace, &reference.namespace) {
            (Some(ns), _) => format!("{ns}/{service}"),
            (_, Some(ns)) => format!("{ns}/{service}"),
            (None, None) => service.clone(),
        }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(ref auth) = self.config.auth {
            if let Ok(value) = HeaderValue::from_str(&auth.authorization_header()) {
                headers.insert(AUTHORIZATION, value);
            }
        }
        headers
    }

    async fn handle_error_response(
        &self,
        status: StatusCode,
        response: reqwest::Response,
    ) -> RegistryError {
        // Try to parse OCI error response
        if let Ok(errors) = response.json::<OciErrors>().await {
            if let Some(error) = errors.errors.first() {
                match error.code.as_str() {
                    "NAME_UNKNOWN" | "MANIFEST_UNKNOWN" => {
                        return RegistryError::NotFound {
                            service: "unknown".into(),
                            version: "unknown".into(),
                        };
                    }
                    "UNAUTHORIZED" => {
                        return RegistryError::auth_failed(&error.message);
                    }
                    "DENIED" => {
                        return RegistryError::authorization_failed(&error.message);
                    }
                    _ => {
                        return RegistryError::registry_error(status.as_u16(), &error.message);
                    }
                }
            }
        }

        match status {
            StatusCode::UNAUTHORIZED => RegistryError::auth_failed("authentication required"),
            StatusCode::FORBIDDEN => RegistryError::authorization_failed("access denied"),
            StatusCode::NOT_FOUND => RegistryError::NotFound {
                service: "unknown".into(),
                version: "unknown".into(),
            },
            StatusCode::TOO_MANY_REQUESTS => RegistryError::RateLimited {
                retry_after_secs: 60,
            },
            _ => RegistryError::registry_error(
                status.as_u16(),
                status.canonical_reason().unwrap_or("unknown"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_mock_server() -> (MockServer, RegistryClient) {
        let server = MockServer::start().await;
        let config = RegistryConfig::new(server.address().to_string())
            .with_https(false)
            .with_namespace("test-org");
        let client = RegistryClient::new(config);
        (server, client)
    }

    #[tokio::test]
    async fn test_list_versions() {
        let (server, client) = setup_mock_server().await;

        Mock::given(method("GET"))
            .and(path("/v2/test-org/users-api/tags/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "test-org/users-api",
                "tags": ["1.0.0", "1.1.0", "2.0.0"]
            })))
            .mount(&server)
            .await;

        let versions = client.list_versions("users-api").await.unwrap();
        assert_eq!(versions, vec!["1.0.0", "1.1.0", "2.0.0"]);
    }

    #[tokio::test]
    async fn test_list_versions_not_found() {
        let (server, client) = setup_mock_server().await;

        Mock::given(method("GET"))
            .and(path("/v2/test-org/nonexistent/tags/list"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let versions = client.list_versions("nonexistent").await.unwrap();
        assert!(versions.is_empty());
    }

    #[tokio::test]
    async fn test_exists() {
        let (server, client) = setup_mock_server().await;

        Mock::given(method("HEAD"))
            .and(path("/v2/test-org/users-api/manifests/1.0.0"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        assert!(client.exists("users-api", "1.0.0").await.unwrap());
    }

    #[tokio::test]
    async fn test_not_exists() {
        let (server, client) = setup_mock_server().await;

        Mock::given(method("HEAD"))
            .and(path("/v2/test-org/users-api/manifests/99.0.0"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        assert!(!client.exists("users-api", "99.0.0").await.unwrap());
    }

    #[tokio::test]
    async fn test_authentication_error() {
        let (server, client) = setup_mock_server().await;

        Mock::given(method("HEAD"))
            .and(path("/v2/test-org/private/manifests/1.0.0"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let result = client.exists("private", "1.0.0").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_auth_error());
    }

    #[test]
    fn test_client_creation() {
        let config = RegistryConfig::new("ghcr.io")
            .with_namespace("my-org")
            .with_token("test-token");

        let client = RegistryClient::new(config);
        // Just verify it doesn't panic
        assert!(client.cache().is_none()); // No cache dir set
    }

    #[test]
    fn test_repository_with_namespace() {
        let config = RegistryConfig::new("ghcr.io").with_namespace("my-org");
        let client = RegistryClient::new(config);
        let reference = ArtifactReference::new("users-api");
        assert_eq!(client.repository(&reference), "my-org/users-api");
    }

    #[test]
    fn test_repository_without_namespace() {
        let config = RegistryConfig::new("ghcr.io");
        let client = RegistryClient::new(config);
        let reference = ArtifactReference::new("users-api");
        assert_eq!(client.repository(&reference), "users-api");
    }
}
