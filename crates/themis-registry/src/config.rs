//! Registry configuration.

use std::path::PathBuf;

/// Registry client configuration.
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Registry URL (e.g., "registry.example.com" or "ghcr.io").
    pub registry: String,

    /// Namespace/organization for artifacts (e.g., "my-org").
    pub namespace: Option<String>,

    /// Authentication configuration.
    pub auth: Option<AuthConfig>,

    /// Whether to use HTTPS (default: true).
    pub use_https: bool,

    /// Request timeout in seconds (default: 30).
    pub timeout_secs: u64,

    /// Number of retries for failed requests (default: 3).
    pub max_retries: u32,

    /// Cache directory (default: ~/.themis/cache).
    pub cache_dir: Option<PathBuf>,

    /// Whether to verify artifact checksums (default: true).
    pub verify_checksums: bool,
}

/// Authentication configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Authentication method.
    pub method: AuthMethod,
}

/// Authentication method.
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Bearer token authentication.
    Token(String),

    /// Basic authentication (username:password).
    Basic {
        /// Username.
        username: String,
        /// Password.
        password: String,
    },

    /// `OAuth2` token.
    OAuth2 {
        /// Access token.
        access_token: String,
        /// Token type (default: "Bearer").
        token_type: String,
    },
}

impl RegistryConfig {
    /// Creates a new configuration with the given registry URL.
    ///
    /// # Example
    ///
    /// ```
    /// use themis_registry::RegistryConfig;
    ///
    /// let config = RegistryConfig::new("ghcr.io");
    /// assert_eq!(config.registry, "ghcr.io");
    /// assert!(config.use_https);
    /// ```
    pub fn new(registry: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            namespace: None,
            auth: None,
            use_https: true,
            timeout_secs: 30,
            max_retries: 3,
            cache_dir: None,
            verify_checksums: true,
        }
    }

    /// Sets the namespace for artifacts.
    ///
    /// # Example
    ///
    /// ```
    /// use themis_registry::RegistryConfig;
    ///
    /// let config = RegistryConfig::new("ghcr.io")
    ///     .with_namespace("my-org");
    /// assert_eq!(config.namespace, Some("my-org".to_string()));
    /// ```
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Sets a bearer token for authentication.
    ///
    /// # Example
    ///
    /// ```
    /// use themis_registry::RegistryConfig;
    ///
    /// let config = RegistryConfig::new("ghcr.io")
    ///     .with_token("ghp_xxx");
    /// assert!(config.auth.is_some());
    /// ```
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.auth = Some(AuthConfig {
            method: AuthMethod::Token(token.into()),
        });
        self
    }

    /// Sets basic authentication credentials.
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.auth = Some(AuthConfig {
            method: AuthMethod::Basic {
                username: username.into(),
                password: password.into(),
            },
        });
        self
    }

    /// Sets `OAuth2` authentication.
    pub fn with_oauth2(mut self, access_token: impl Into<String>) -> Self {
        self.auth = Some(AuthConfig {
            method: AuthMethod::OAuth2 {
                access_token: access_token.into(),
                token_type: "Bearer".to_string(),
            },
        });
        self
    }

    /// Sets whether to use HTTPS.
    #[must_use]
    pub const fn with_https(mut self, use_https: bool) -> Self {
        self.use_https = use_https;
        self
    }

    /// Sets the request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Sets the maximum number of retries.
    #[must_use]
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the cache directory.
    pub fn with_cache_dir(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    /// Disables checksum verification.
    ///
    /// **Warning**: This should only be used in controlled environments.
    #[must_use]
    pub const fn without_checksum_verification(mut self) -> Self {
        self.verify_checksums = false;
        self
    }

    /// Returns the base URL for the registry.
    #[must_use]
    pub fn base_url(&self) -> String {
        let scheme = if self.use_https { "https" } else { "http" };
        format!("{scheme}://{}", self.registry)
    }

    /// Returns the full repository path for a service.
    #[must_use]
    pub fn repository_path(&self, service: &str) -> String {
        match &self.namespace {
            Some(ns) => format!("{ns}/{service}"),
            None => service.to_string(),
        }
    }

    /// Returns the default cache directory.
    #[must_use]
    pub fn default_cache_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".themis").join("cache"))
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self::new("localhost:5000").with_https(false)
    }
}

impl AuthConfig {
    /// Returns the authorization header value.
    #[must_use]
    pub fn authorization_header(&self) -> String {
        match &self.method {
            AuthMethod::Token(token) => format!("Bearer {token}"),
            AuthMethod::Basic { username, password } => {
                let credentials = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    format!("{username}:{password}"),
                );
                format!("Basic {credentials}")
            }
            AuthMethod::OAuth2 {
                access_token,
                token_type,
            } => {
                format!("{token_type} {access_token}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = RegistryConfig::new("ghcr.io");
        assert_eq!(config.registry, "ghcr.io");
        assert!(config.use_https);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.verify_checksums);
    }

    #[test]
    fn test_config_builder() {
        let config = RegistryConfig::new("registry.example.com")
            .with_namespace("my-org")
            .with_token("secret-token")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.namespace, Some("my-org".to_string()));
        assert!(config.auth.is_some());
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_base_url() {
        let https_config = RegistryConfig::new("ghcr.io");
        assert_eq!(https_config.base_url(), "https://ghcr.io");

        let http_config = RegistryConfig::new("localhost:5000").with_https(false);
        assert_eq!(http_config.base_url(), "http://localhost:5000");
    }

    #[test]
    fn test_repository_path() {
        let config_with_ns = RegistryConfig::new("ghcr.io").with_namespace("my-org");
        assert_eq!(
            config_with_ns.repository_path("users-api"),
            "my-org/users-api"
        );

        let config_without_ns = RegistryConfig::new("ghcr.io");
        assert_eq!(config_without_ns.repository_path("users-api"), "users-api");
    }

    #[test]
    fn test_token_auth_header() {
        let auth = AuthConfig {
            method: AuthMethod::Token("my-token".to_string()),
        };
        assert_eq!(auth.authorization_header(), "Bearer my-token");
    }

    #[test]
    fn test_basic_auth_header() {
        let auth = AuthConfig {
            method: AuthMethod::Basic {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
        };
        let header = auth.authorization_header();
        assert!(header.starts_with("Basic "));
        // Decode and verify
        let encoded = header.strip_prefix("Basic ").unwrap();
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .expect("valid base64");
        let decoded_str = String::from_utf8(decoded).expect("valid utf8");
        assert_eq!(decoded_str, "user:pass");
    }

    #[test]
    fn test_oauth2_auth_header() {
        let auth = AuthConfig {
            method: AuthMethod::OAuth2 {
                access_token: "oauth-token".to_string(),
                token_type: "Bearer".to_string(),
            },
        };
        assert_eq!(auth.authorization_header(), "Bearer oauth-token");
    }

    #[test]
    fn test_default_config() {
        let config = RegistryConfig::default();
        assert_eq!(config.registry, "localhost:5000");
        assert!(!config.use_https);
    }
}
