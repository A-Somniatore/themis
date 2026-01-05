//! Local artifact caching.

use crate::error::{RegistryError, RegistryResult};
use std::path::{Path, PathBuf};
use themis_artifact::Artifact;
use tracing::{debug, trace};

/// Cache configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache directory.
    pub dir: PathBuf,

    /// Maximum cache size in bytes (0 = unlimited).
    pub max_size: u64,

    /// Time-to-live for cache entries in seconds (0 = forever).
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dir: dirs::home_dir().map_or_else(
                || PathBuf::from(".themis-cache"),
                |h| h.join(".themis").join("cache"),
            ),
            max_size: 0,
            ttl_secs: 0,
        }
    }
}

impl CacheConfig {
    /// Creates a new cache config with the given directory.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            ..Default::default()
        }
    }

    /// Sets the maximum cache size.
    #[must_use]
    pub const fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = max_size;
        self
    }

    /// Sets the time-to-live for cache entries.
    #[must_use]
    pub const fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }
}

/// Local artifact cache.
///
/// Caches artifacts locally to reduce network requests. Cache entries are
/// stored as JSON files in a directory structure:
///
/// ```text
/// cache/
/// ├── my-org/
/// │   └── users-api/
/// │       ├── 1.0.0.json
/// │       └── 1.1.0.json
/// └── other-org/
///     └── orders-api/
///         └── 2.0.0.json
/// ```
pub struct ArtifactCache {
    config: CacheConfig,
}

impl ArtifactCache {
    /// Creates a new cache with the given configuration.
    pub fn new(config: CacheConfig) -> RegistryResult<Self> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&config.dir).map_err(|e| {
            RegistryError::io_error(
                &config.dir,
                format!("failed to create cache directory: {e}"),
            )
        })?;

        Ok(Self { config })
    }

    /// Creates a cache with default configuration.
    pub fn default_cache() -> RegistryResult<Self> {
        Self::new(CacheConfig::default())
    }

    /// Gets an artifact from the cache.
    ///
    /// Returns `None` if the artifact is not cached or has expired.
    pub fn get(&self, namespace: Option<&str>, service: &str, version: &str) -> Option<Artifact> {
        let path = self.artifact_path(namespace, service, version);
        trace!(?path, "checking cache");

        if !path.exists() {
            debug!(?path, "cache miss: file not found");
            return None;
        }

        // Check TTL if configured
        if self.config.ttl_secs > 0 {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > self.config.ttl_secs {
                            debug!(?path, "cache miss: entry expired");
                            return None;
                        }
                    }
                }
            }
        }

        // Read and parse
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<Artifact>(&json) {
                Ok(artifact) => {
                    debug!(?path, "cache hit");
                    Some(artifact)
                }
                Err(e) => {
                    debug!(?path, error = %e, "cache miss: parse error");
                    // Remove corrupted entry
                    let _ = std::fs::remove_file(&path);
                    None
                }
            },
            Err(e) => {
                debug!(?path, error = %e, "cache miss: read error");
                None
            }
        }
    }

    /// Stores an artifact in the cache.
    pub fn put(
        &self,
        namespace: Option<&str>,
        service: &str,
        version: &str,
        artifact: &Artifact,
    ) -> RegistryResult<()> {
        let path = self.artifact_path(namespace, service, version);

        // Ensure parent directories exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RegistryError::io_error(parent, format!("failed to create cache directory: {e}"))
            })?;
        }

        let json = serde_json::to_string_pretty(artifact)?;

        std::fs::write(&path, json)
            .map_err(|e| RegistryError::io_error(&path, format!("failed to write cache: {e}")))?;

        debug!(?path, "cached artifact");
        Ok(())
    }

    /// Removes an artifact from the cache.
    pub fn remove(
        &self,
        namespace: Option<&str>,
        service: &str,
        version: &str,
    ) -> RegistryResult<()> {
        let path = self.artifact_path(namespace, service, version);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                RegistryError::io_error(&path, format!("failed to remove cache: {e}"))
            })?;
            debug!(?path, "removed from cache");
        }
        Ok(())
    }

    /// Clears all cached artifacts for a service.
    pub fn clear_service(&self, namespace: Option<&str>, service: &str) -> RegistryResult<()> {
        let dir = self.service_dir(namespace, service);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| {
                RegistryError::io_error(&dir, format!("failed to clear cache: {e}"))
            })?;
            debug!(?dir, "cleared service cache");
        }
        Ok(())
    }

    /// Clears all cached artifacts.
    pub fn clear_all(&self) -> RegistryResult<()> {
        if self.config.dir.exists() {
            for entry in std::fs::read_dir(&self.config.dir)
                .map_err(|e| RegistryError::io_error(&self.config.dir, e.to_string()))?
            {
                let entry = entry.map_err(|e| RegistryError::CacheError(e.to_string()))?;
                let path = entry.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                        .map_err(|e| RegistryError::io_error(&path, e.to_string()))?;
                } else {
                    std::fs::remove_file(&path)
                        .map_err(|e| RegistryError::io_error(&path, e.to_string()))?;
                }
            }
            debug!("cleared all cache");
        }
        Ok(())
    }

    /// Lists cached versions for a service.
    pub fn list_versions(
        &self,
        namespace: Option<&str>,
        service: &str,
    ) -> RegistryResult<Vec<String>> {
        let dir = self.service_dir(namespace, service);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in
            std::fs::read_dir(&dir).map_err(|e| RegistryError::io_error(&dir, e.to_string()))?
        {
            let entry = entry.map_err(|e| RegistryError::CacheError(e.to_string()))?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    versions.push(stem.to_string());
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Returns the cache directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.config.dir
    }

    /// Returns the total size of the cache in bytes.
    pub fn size(&self) -> RegistryResult<u64> {
        Self::dir_size(&self.config.dir)
    }

    fn dir_size(dir: &Path) -> RegistryResult<u64> {
        if !dir.exists() {
            return Ok(0);
        }

        let mut size = 0;
        for entry in
            std::fs::read_dir(dir).map_err(|e| RegistryError::io_error(dir, e.to_string()))?
        {
            let entry = entry.map_err(|e| RegistryError::CacheError(e.to_string()))?;
            let path = entry.path();
            let metadata = std::fs::metadata(&path)
                .map_err(|e| RegistryError::io_error(&path, e.to_string()))?;

            if metadata.is_dir() {
                size += Self::dir_size(&path)?;
            } else {
                size += metadata.len();
            }
        }
        Ok(size)
    }

    fn service_dir(&self, namespace: Option<&str>, service: &str) -> PathBuf {
        match namespace {
            Some(ns) => self.config.dir.join(ns).join(service),
            None => self.config.dir.join(service),
        }
    }

    fn artifact_path(&self, namespace: Option<&str>, service: &str, version: &str) -> PathBuf {
        self.service_dir(namespace, service)
            .join(format!("{version}.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use themis_artifact::{Artifact, ArtifactMetadata, Checksum};

    fn temp_cache() -> (TempDir, ArtifactCache) {
        let dir = TempDir::new().unwrap();
        let cache = ArtifactCache::new(CacheConfig::new(dir.path())).unwrap();
        (dir, cache)
    }

    fn sample_artifact() -> Artifact {
        Artifact {
            schema: "test".to_string(),
            version: "1.0.0".to_string(),
            service: "test-service".to_string(),
            format: "openapi".to_string(),
            format_version: "3.1.0".to_string(),
            metadata: ArtifactMetadata::default(),
            checksum: Checksum {
                algorithm: "sha256".to_string(),
                value: "abc123".to_string(),
            },
            operations: vec![],
            schemas: std::collections::HashMap::new(),
            raw_contract: None,
        }
    }

    #[test]
    fn test_cache_put_and_get() {
        let (_dir, cache) = temp_cache();
        let artifact = sample_artifact();

        // Put
        cache
            .put(Some("my-org"), "test-service", "1.0.0", &artifact)
            .unwrap();

        // Get
        let cached = cache.get(Some("my-org"), "test-service", "1.0.0");
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.version, "1.0.0");
        assert_eq!(cached.service, "test-service");
    }

    #[test]
    fn test_cache_miss() {
        let (_dir, cache) = temp_cache();
        let cached = cache.get(Some("my-org"), "nonexistent", "1.0.0");
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_without_namespace() {
        let (_dir, cache) = temp_cache();
        let artifact = sample_artifact();

        cache.put(None, "test-service", "1.0.0", &artifact).unwrap();

        let cached = cache.get(None, "test-service", "1.0.0");
        assert!(cached.is_some());
    }

    #[test]
    fn test_cache_remove() {
        let (_dir, cache) = temp_cache();
        let artifact = sample_artifact();

        cache.put(Some("org"), "svc", "1.0.0", &artifact).unwrap();
        assert!(cache.get(Some("org"), "svc", "1.0.0").is_some());

        cache.remove(Some("org"), "svc", "1.0.0").unwrap();
        assert!(cache.get(Some("org"), "svc", "1.0.0").is_none());
    }

    #[test]
    fn test_cache_clear_service() {
        let (_dir, cache) = temp_cache();
        let artifact = sample_artifact();

        cache.put(Some("org"), "svc", "1.0.0", &artifact).unwrap();
        cache.put(Some("org"), "svc", "2.0.0", &artifact).unwrap();

        cache.clear_service(Some("org"), "svc").unwrap();

        assert!(cache.get(Some("org"), "svc", "1.0.0").is_none());
        assert!(cache.get(Some("org"), "svc", "2.0.0").is_none());
    }

    #[test]
    fn test_cache_list_versions() {
        let (_dir, cache) = temp_cache();
        let artifact = sample_artifact();

        cache.put(Some("org"), "svc", "1.0.0", &artifact).unwrap();
        cache.put(Some("org"), "svc", "2.0.0", &artifact).unwrap();
        cache.put(Some("org"), "svc", "1.1.0", &artifact).unwrap();

        let versions = cache.list_versions(Some("org"), "svc").unwrap();
        assert_eq!(versions, vec!["1.0.0", "1.1.0", "2.0.0"]);
    }

    #[test]
    fn test_cache_size() {
        let (_dir, cache) = temp_cache();
        let artifact = sample_artifact();

        cache.put(Some("org"), "svc", "1.0.0", &artifact).unwrap();

        let size = cache.size().unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_cache_config_builder() {
        let config = CacheConfig::new("/tmp/test")
            .with_max_size(1024 * 1024)
            .with_ttl(3600);

        assert_eq!(config.dir, PathBuf::from("/tmp/test"));
        assert_eq!(config.max_size, 1024 * 1024);
        assert_eq!(config.ttl_secs, 3600);
    }
}
