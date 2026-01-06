//! Artifact reference parsing and formatting.

use crate::error::{RegistryError, RegistryResult};
use std::fmt;

/// A reference to an artifact in a registry.
///
/// Format: `[registry/][namespace/]service[:version|@digest]`
///
/// # Examples
///
/// - `users-api:1.0.0` - service with version
/// - `my-org/users-api:1.0.0` - with namespace
/// - `ghcr.io/my-org/users-api:1.0.0` - fully qualified
/// - `users-api@sha256:abc...` - with digest
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactReference {
    /// Registry host (e.g., "ghcr.io").
    pub registry: Option<String>,

    /// Namespace/organization (e.g., "my-org").
    pub namespace: Option<String>,

    /// Service name (e.g., "users-api").
    pub service: String,

    /// Version tag (e.g., "1.0.0").
    pub tag: Option<String>,

    /// Content digest (e.g., "sha256:abc123...").
    pub digest: Option<String>,
}

impl ArtifactReference {
    /// Creates a new artifact reference with just the service name.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            registry: None,
            namespace: None,
            service: service.into(),
            tag: None,
            digest: None,
        }
    }

    /// Sets the registry.
    pub fn with_registry(mut self, registry: impl Into<String>) -> Self {
        self.registry = Some(registry.into());
        self
    }

    /// Sets the namespace.
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Sets the version tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self.digest = None; // Tag and digest are mutually exclusive
        self
    }

    /// Sets the content digest.
    pub fn with_digest(mut self, digest: impl Into<String>) -> Self {
        self.digest = Some(digest.into());
        self.tag = None; // Tag and digest are mutually exclusive
        self
    }

    /// Parses an artifact reference from a string.
    ///
    /// # Format
    ///
    /// `[registry/][namespace/]service[:tag|@digest]`
    ///
    /// # Examples
    ///
    /// ```
    /// use themis_registry::ArtifactReference;
    ///
    /// // Simple service:version
    /// let ref1 = ArtifactReference::parse("users-api:1.0.0").unwrap();
    /// assert_eq!(ref1.service, "users-api");
    /// assert_eq!(ref1.tag, Some("1.0.0".to_string()));
    ///
    /// // Fully qualified
    /// let ref2 = ArtifactReference::parse("ghcr.io/my-org/users-api:1.0.0").unwrap();
    /// assert_eq!(ref2.registry, Some("ghcr.io".to_string()));
    /// assert_eq!(ref2.namespace, Some("my-org".to_string()));
    /// assert_eq!(ref2.service, "users-api");
    /// ```
    pub fn parse(reference: &str) -> RegistryResult<Self> {
        if reference.is_empty() {
            return Err(RegistryError::invalid_reference(
                reference,
                "empty reference",
            ));
        }

        // Split off digest first (takes precedence over tag)
        let (name_part, digest) = if let Some(at_pos) = reference.rfind('@') {
            let (name, digest) = reference.split_at(at_pos);
            (name, Some(digest[1..].to_string()))
        } else {
            (reference, None)
        };

        // Split off tag if no digest
        // The tag is after the last colon, but only if it's not part of a port
        let (name_part, tag) = if digest.is_none() {
            if let Some(colon_pos) = name_part.rfind(':') {
                let after_colon = &name_part[colon_pos + 1..];
                // If after colon is all digits, it could be a port - but only if there's no slash after
                // e.g., "localhost:5000/service" - the 5000 is a port
                // e.g., "localhost:5000/service:1.0.0" - the 1.0.0 is a tag
                let is_port = after_colon.chars().all(|c| c.is_ascii_digit())
                    && !name_part[..colon_pos].contains('/');

                if is_port {
                    (name_part, None)
                } else {
                    let (name, tag) = name_part.split_at(colon_pos);
                    (name, Some(tag[1..].to_string()))
                }
            } else {
                (name_part, None)
            }
        } else {
            (name_part, None)
        };

        // Parse the name part into registry/namespace/service
        let parts: Vec<&str> = name_part.split('/').collect();

        // Determine if first part is a registry (contains . or :)
        let first_is_registry = parts
            .first()
            .is_some_and(|p| p.contains('.') || p.contains(':'));

        let (registry, namespace, service) = match (parts.len(), first_is_registry) {
            (1, _) => (None, None, parts[0].to_string()),
            (2, true) => (Some(parts[0].to_string()), None, parts[1].to_string()),
            (2, false) => (None, Some(parts[0].to_string()), parts[1].to_string()),
            (3, _) => (
                Some(parts[0].to_string()),
                Some(parts[1].to_string()),
                parts[2].to_string(),
            ),
            _ => {
                return Err(RegistryError::invalid_reference(
                    reference,
                    "too many path segments",
                ))
            }
        };

        // Validate service name
        if service.is_empty() {
            return Err(RegistryError::invalid_reference(
                reference,
                "empty service name",
            ));
        }

        if !Self::is_valid_name(&service) {
            return Err(RegistryError::invalid_reference(
                reference,
                "invalid service name (must be lowercase alphanumeric with hyphens)",
            ));
        }

        Ok(Self {
            registry,
            namespace,
            service,
            tag,
            digest,
        })
    }

    /// Validates a name component (service or namespace).
    fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    /// Returns the repository path (namespace/service).
    #[must_use]
    pub fn repository(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{ns}/{}", self.service),
            None => self.service.clone(),
        }
    }

    /// Returns the reference string (tag or digest).
    #[must_use]
    pub fn reference(&self) -> Option<&str> {
        self.tag.as_deref().or(self.digest.as_deref())
    }

    /// Returns true if this reference specifies a version.
    #[must_use]
    pub const fn has_version(&self) -> bool {
        self.tag.is_some() || self.digest.is_some()
    }

    /// Returns the tag or "latest" if not specified.
    #[must_use]
    pub fn tag_or_latest(&self) -> &str {
        self.tag.as_deref().unwrap_or("latest")
    }
}

impl fmt::Display for ArtifactReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref registry) = self.registry {
            write!(f, "{registry}/")?;
        }
        if let Some(ref namespace) = self.namespace {
            write!(f, "{namespace}/")?;
        }
        write!(f, "{}", self.service)?;
        if let Some(ref tag) = self.tag {
            write!(f, ":{tag}")?;
        }
        if let Some(ref digest) = self.digest {
            write!(f, "@{digest}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for ArtifactReference {
    type Err = RegistryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let ref1 = ArtifactReference::parse("users-api").unwrap();
        assert_eq!(ref1.service, "users-api");
        assert!(ref1.registry.is_none());
        assert!(ref1.namespace.is_none());
        assert!(ref1.tag.is_none());
        assert!(ref1.digest.is_none());
    }

    #[test]
    fn test_parse_with_tag() {
        let ref1 = ArtifactReference::parse("users-api:1.0.0").unwrap();
        assert_eq!(ref1.service, "users-api");
        assert_eq!(ref1.tag, Some("1.0.0".to_string()));
        assert!(ref1.digest.is_none());
    }

    #[test]
    fn test_parse_with_namespace() {
        let ref1 = ArtifactReference::parse("my-org/users-api:1.0.0").unwrap();
        assert_eq!(ref1.namespace, Some("my-org".to_string()));
        assert_eq!(ref1.service, "users-api");
        assert_eq!(ref1.tag, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_fully_qualified() {
        let ref1 = ArtifactReference::parse("ghcr.io/my-org/users-api:1.0.0").unwrap();
        assert_eq!(ref1.registry, Some("ghcr.io".to_string()));
        assert_eq!(ref1.namespace, Some("my-org".to_string()));
        assert_eq!(ref1.service, "users-api");
        assert_eq!(ref1.tag, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_with_digest() {
        let ref1 = ArtifactReference::parse("users-api@sha256:abc123").unwrap();
        assert_eq!(ref1.service, "users-api");
        assert!(ref1.tag.is_none());
        assert_eq!(ref1.digest, Some("sha256:abc123".to_string()));
    }

    #[test]
    fn test_parse_with_port() {
        // Registry with port shouldn't be confused with tag
        let ref1 = ArtifactReference::parse("localhost:5000/users-api:1.0.0").unwrap();
        assert_eq!(ref1.registry, Some("localhost:5000".to_string()));
        assert_eq!(ref1.service, "users-api");
        assert_eq!(ref1.tag, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_invalid_empty() {
        let result = ArtifactReference::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_too_many_segments() {
        let result = ArtifactReference::parse("a/b/c/d/service");
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let ref1 = ArtifactReference::new("users-api")
            .with_registry("ghcr.io")
            .with_namespace("my-org")
            .with_tag("1.0.0");
        assert_eq!(ref1.to_string(), "ghcr.io/my-org/users-api:1.0.0");
    }

    #[test]
    fn test_display_with_digest() {
        let ref1 = ArtifactReference::new("users-api").with_digest("sha256:abc123");
        assert_eq!(ref1.to_string(), "users-api@sha256:abc123");
    }

    #[test]
    fn test_repository_path() {
        let ref1 = ArtifactReference::new("users-api").with_namespace("my-org");
        assert_eq!(ref1.repository(), "my-org/users-api");

        let ref2 = ArtifactReference::new("users-api");
        assert_eq!(ref2.repository(), "users-api");
    }

    #[test]
    fn test_tag_or_latest() {
        let ref1 = ArtifactReference::new("users-api").with_tag("1.0.0");
        assert_eq!(ref1.tag_or_latest(), "1.0.0");

        let ref2 = ArtifactReference::new("users-api");
        assert_eq!(ref2.tag_or_latest(), "latest");
    }

    #[test]
    fn test_builder_pattern() {
        let reference = ArtifactReference::new("service")
            .with_registry("ghcr.io")
            .with_namespace("org")
            .with_tag("v1");

        assert_eq!(reference.registry, Some("ghcr.io".to_string()));
        assert_eq!(reference.namespace, Some("org".to_string()));
        assert_eq!(reference.service, "service");
        assert_eq!(reference.tag, Some("v1".to_string()));
    }

    #[test]
    fn test_from_str() {
        let reference: ArtifactReference = "users-api:1.0.0".parse().unwrap();
        assert_eq!(reference.service, "users-api");
        assert_eq!(reference.tag, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_has_version() {
        let with_tag = ArtifactReference::new("service").with_tag("1.0.0");
        assert!(with_tag.has_version());

        let with_digest = ArtifactReference::new("service").with_digest("sha256:abc");
        assert!(with_digest.has_version());

        let without = ArtifactReference::new("service");
        assert!(!without.has_version());
    }
}
