//! Publish and fetch commands for registry operations.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use themis_artifact::Artifact;
use themis_registry::{RegistryClient, RegistryConfig};

/// Publish an artifact to the registry.
#[derive(Args, Debug)]
pub struct PublishArgs {
    /// Path to the artifact file
    #[arg(value_name = "ARTIFACT")]
    pub artifact: PathBuf,

    /// Registry URL
    #[arg(short, long, env = "THEMIS_REGISTRY_URL")]
    pub registry: Option<String>,

    /// Namespace/organization
    #[arg(short, long, env = "THEMIS_NAMESPACE")]
    pub namespace: Option<String>,

    /// Authentication token
    #[arg(long, env = "THEMIS_REGISTRY_TOKEN")]
    pub token: Option<String>,

    /// Verify artifact before publishing
    #[arg(long, default_value = "true")]
    pub verify: bool,

    /// Skip if version already exists
    #[arg(long)]
    pub skip_existing: bool,
}

/// Runs the publish command.
pub fn run_publish(args: &PublishArgs) -> Result<()> {
    // Load the artifact
    let artifact = Artifact::from_file(&args.artifact)
        .with_context(|| format!("Failed to load artifact: {}", args.artifact.display()))?;

    // Verify checksum if requested
    if args.verify {
        artifact
            .verify_checksum()
            .context("Artifact checksum verification failed")?;
    }

    // Build registry config
    let registry_url = args.registry.clone().unwrap_or_else(|| {
        std::env::var("THEMIS_REGISTRY_URL")
            .unwrap_or_else(|_| "https://registry.themis.io".to_string())
    });

    let mut config = RegistryConfig::new(&registry_url);

    if let Some(namespace) = &args.namespace {
        config = config.with_namespace(namespace);
    }

    if let Some(token) = &args.token {
        config = config.with_token(token);
    }

    // Create client
    let client = RegistryClient::new(config);

    // Run async publish
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        // Check if version exists
        if args.skip_existing {
            match client.exists(&artifact.service, &artifact.version).await {
                Ok(true) => {
                    println!(
                        "✓ Version {}@{} already exists, skipping",
                        artifact.service, artifact.version
                    );
                    return Ok(());
                }
                Ok(false) => {}
                Err(e) => {
                    // Ignore existence check errors, proceed with publish
                    tracing::debug!("Existence check failed: {}", e);
                }
            }
        }

        // Publish the artifact
        client
            .publish(&artifact)
            .await
            .context("Failed to publish artifact")?;

        println!("✓ Published {}@{}", artifact.service, artifact.version);
        println!("  Registry: {}", registry_url);
        if let Some(ns) = &args.namespace {
            println!("  Namespace: {}", ns);
        }

        Ok(())
    })
}

/// Fetch an artifact from the registry.
#[derive(Args, Debug)]
pub struct FetchArgs {
    /// Service name
    #[arg(value_name = "SERVICE")]
    pub service: String,

    /// Version to fetch (or "latest")
    #[arg(value_name = "VERSION", default_value = "latest")]
    pub version: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Registry URL
    #[arg(short, long, env = "THEMIS_REGISTRY_URL")]
    pub registry: Option<String>,

    /// Namespace/organization
    #[arg(short, long, env = "THEMIS_NAMESPACE")]
    pub namespace: Option<String>,

    /// Authentication token
    #[arg(long, env = "THEMIS_REGISTRY_TOKEN")]
    pub token: Option<String>,

    /// Use cached version if available
    #[arg(long)]
    pub cache: bool,

    /// Verify artifact after fetching
    #[arg(long, default_value = "true")]
    pub verify: bool,
}

/// Runs the fetch command.
pub fn run_fetch(args: &FetchArgs) -> Result<()> {
    // Build registry config
    let registry_url = args.registry.clone().unwrap_or_else(|| {
        std::env::var("THEMIS_REGISTRY_URL")
            .unwrap_or_else(|_| "https://registry.themis.io".to_string())
    });

    let mut config = RegistryConfig::new(&registry_url);

    if let Some(namespace) = &args.namespace {
        config = config.with_namespace(namespace);
    }

    if let Some(token) = &args.token {
        config = config.with_token(token);
    }

    // Create client
    let client = RegistryClient::new(config);

    // Run async fetch
    let runtime = tokio::runtime::Runtime::new()?;
    let artifact = runtime.block_on(async {
        let version = if args.version == "latest" {
            // Get latest version
            let versions = client
                .list_versions(&args.service)
                .await
                .context("Failed to list versions")?;

            versions
                .into_iter()
                .max()
                .ok_or_else(|| anyhow::anyhow!("No versions found for service: {}", args.service))?
        } else {
            args.version.clone()
        };

        // Fetch the artifact
        let artifact = if args.cache {
            client
                .fetch_cached(&args.service, &version)
                .await
                .context("Failed to fetch artifact")?
        } else {
            client
                .fetch(&args.service, &version)
                .await
                .context("Failed to fetch artifact")?
        };

        Ok::<_, anyhow::Error>(artifact)
    })?;

    // Verify checksum if requested
    if args.verify {
        artifact
            .verify_checksum()
            .context("Artifact checksum verification failed")?;
    }

    // Determine output path
    let output_path = args.output.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            "{}-{}.artifact.json",
            artifact.service, artifact.version
        ))
    });

    // Write the artifact
    artifact
        .to_file(&output_path)
        .with_context(|| format!("Failed to write artifact: {}", output_path.display()))?;

    println!("✓ Fetched {}@{}", artifact.service, artifact.version);
    println!("  Saved to: {}", output_path.display());
    if args.verify {
        println!("  Checksum: verified");
    }

    Ok(())
}

/// List versions of a service in the registry.
#[derive(Args, Debug)]
pub struct ListVersionsArgs {
    /// Service name
    #[arg(value_name = "SERVICE")]
    pub service: String,

    /// Registry URL
    #[arg(short, long, env = "THEMIS_REGISTRY_URL")]
    pub registry: Option<String>,

    /// Namespace/organization
    #[arg(short, long, env = "THEMIS_NAMESPACE")]
    pub namespace: Option<String>,

    /// Authentication token
    #[arg(long, env = "THEMIS_REGISTRY_TOKEN")]
    pub token: Option<String>,
}

/// Runs the list-versions command.
pub fn run_list_versions(args: &ListVersionsArgs) -> Result<()> {
    // Build registry config
    let registry_url = args.registry.clone().unwrap_or_else(|| {
        std::env::var("THEMIS_REGISTRY_URL")
            .unwrap_or_else(|_| "https://registry.themis.io".to_string())
    });

    let mut config = RegistryConfig::new(&registry_url);

    if let Some(namespace) = &args.namespace {
        config = config.with_namespace(namespace);
    }

    if let Some(token) = &args.token {
        config = config.with_token(token);
    }

    // Create client
    let client = RegistryClient::new(config);

    // Run async list
    let runtime = tokio::runtime::Runtime::new()?;
    let versions = runtime.block_on(async {
        client
            .list_versions(&args.service)
            .await
            .context("Failed to list versions")
    })?;

    if versions.is_empty() {
        println!("No versions found for service: {}", args.service);
    } else {
        println!("Versions for {}:", args.service);
        for version in &versions {
            println!("  {}", version);
        }
        println!("\n{} version(s) found", versions.len());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_args_defaults() {
        let args = PublishArgs {
            artifact: PathBuf::from("test.artifact.json"),
            registry: None,
            namespace: None,
            token: None,
            verify: true,
            skip_existing: false,
        };

        assert!(args.verify);
        assert!(!args.skip_existing);
    }

    #[test]
    fn test_fetch_args_defaults() {
        let args = FetchArgs {
            service: "test-service".to_string(),
            version: "latest".to_string(),
            output: None,
            registry: None,
            namespace: None,
            token: None,
            cache: false,
            verify: true,
        };

        assert_eq!(args.version, "latest");
        assert!(args.verify);
    }
}
