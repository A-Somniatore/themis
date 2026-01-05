//! Contract artifact creation and loading for Themis.
//!
//! This crate provides types and utilities for creating, loading, and verifying
//! immutable contract artifacts that can be published to a registry.
//!
//! # Overview
//!
//! Artifacts are the publishable form of Themis contracts. They contain:
//! - Parsed contract metadata (service name, version)
//! - All operations with their schemas
//! - A checksum for integrity verification
//! - The raw contract source (base64 encoded)
//!
//! # Example
//!
//! ```ignore
//! use themis_artifact::{ArtifactBuilder, Artifact};
//! use themis_core::Contract;
//!
//! // Build an artifact from a contract
//! let contract: Contract = // ... parse contract
//! let raw_yaml = std::fs::read_to_string("api.yaml")?;
//!
//! let artifact = ArtifactBuilder::new(contract)
//!     .raw_contract(raw_yaml)
//!     .git_commit("abc123")
//!     .git_repository("github.com/example/api")
//!     .owner("platform-team")
//!     .build()?;
//!
//! // Verify integrity
//! artifact.verify_checksum()?;
//!
//! // Serialize for publishing
//! let json = serde_json::to_string(&artifact)?;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod artifact;
mod builder;
mod error;
mod operation;

pub use artifact::{Artifact, ArtifactMetadata, Checksum, ARTIFACT_SCHEMA_VERSION};
pub use builder::ArtifactBuilder;
pub use error::{ArtifactError, ArtifactResult};
pub use operation::{ArtifactOperation, OperationMetadata};
