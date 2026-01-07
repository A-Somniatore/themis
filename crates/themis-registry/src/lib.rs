//! OCI registry client for Themis contract artifacts.
//!
//! This crate provides a client for publishing and fetching Themis contract
//! artifacts to/from OCI-compatible registries.
//!
//! # Overview
//!
//! The registry client follows the [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec)
//! to store contract artifacts as OCI artifacts. Each artifact is stored as:
//!
//! - **Media type**: `application/vnd.themis.artifact.v1+json`
//! - **Repository**: `{registry}/{namespace}/{service}`
//! - **Tag**: Contract version (e.g., `1.0.0`)
//!
//! # Example
//!
//! ```ignore
//! use themis_registry::{RegistryClient, RegistryConfig};
//! use themis_artifact::Artifact;
//!
//! // Create a client
//! let config = RegistryConfig::new("registry.example.com")
//!     .with_namespace("my-org")
//!     .with_auth_token("xxx");
//! let client = RegistryClient::new(config)?;
//!
//! // Publish an artifact
//! let artifact: Artifact = // ... build artifact
//! client.publish(&artifact).await?;
//!

// Allow missing docs for errors and panics during development
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::self_named_constructors)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::option_map_or_none)]
//! // Fetch an artifact
//! let artifact = client.fetch("users-api", "1.0.0").await?;
//!
//! // List versions
//! let versions = client.list_versions("users-api").await?;
//! ```
//!
//! # Caching
//!
//! The client supports local caching to reduce network requests. Cached
//! artifacts are stored in `~/.themis/cache/` by default.
//!
//! ```ignore
//! let config = RegistryConfig::new("registry.example.com")
//!     .with_cache_dir("~/.themis/cache");
//! let client = RegistryClient::new(config)?;
//!
//! // Will use cache if available
//! let artifact = client.fetch_cached("users-api", "1.0.0").await?;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cache;
mod client;
mod config;
mod error;
mod oci;
mod reference;

pub use cache::{ArtifactCache, CacheConfig};
pub use client::RegistryClient;
pub use config::{AuthConfig, RegistryConfig};
pub use error::{RegistryError, RegistryResult};
pub use oci::{MediaType, OciDescriptor, OciManifest};
pub use reference::ArtifactReference;
