//! Pack command for creating artifacts.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use themis_artifact::{Artifact, ArtifactBuilder};
use themis_openapi::parse_openapi;

/// Create a packaged artifact from a contract.
#[derive(Args, Debug)]
pub struct PackArgs {
    /// Path to the contract file
    #[arg(value_name = "CONTRACT")]
    pub contract: PathBuf,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Override the service name
    #[arg(long)]
    pub service: Option<String>,

    /// Override the version
    #[arg(long)]
    pub version: Option<String>,

    /// Set the owner
    #[arg(long)]
    pub owner: Option<String>,

    /// Git commit SHA
    #[arg(long)]
    pub git_commit: Option<String>,

    /// Git repository URL
    #[arg(long)]
    pub git_repository: Option<String>,

    /// Include raw contract in artifact
    #[arg(long, default_value = "true")]
    pub include_raw: bool,

    /// Verify the artifact after creation
    #[arg(long)]
    pub verify: bool,
}

/// Runs the pack command.
pub fn run(args: &PackArgs) -> Result<()> {
    // Read the contract file
    let contract_content = std::fs::read_to_string(&args.contract)
        .with_context(|| format!("Failed to read contract file: {}", args.contract.display()))?;

    // Parse the contract
    let contract = parse_openapi(&contract_content)
        .with_context(|| format!("Failed to parse contract: {}", args.contract.display()))?;

    // Build the artifact
    let mut builder = ArtifactBuilder::from_contract(&contract);

    // Apply overrides
    if let Some(service) = &args.service {
        builder = builder.service(service);
    }

    if let Some(version) = &args.version {
        builder = builder.version(version);
    }

    if let Some(owner) = &args.owner {
        builder = builder.owner(owner);
    }

    if let Some(commit) = &args.git_commit {
        builder = builder.git_commit(commit);
    }

    if let Some(repo) = &args.git_repository {
        builder = builder.git_repository(repo);
    }

    // Include raw contract if requested
    if args.include_raw {
        builder = builder.raw_contract(contract_content.as_bytes().to_vec());
    }

    // Build the artifact
    let artifact = builder.build()?;

    // Verify checksum if requested
    if args.verify {
        artifact
            .verify_checksum()
            .context("Checksum verification failed")?;
        println!("✓ Artifact checksum verified");
    }

    // Determine output path
    let output_path = args.output.clone().unwrap_or_else(|| {
        let stem = args
            .contract
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact");
        PathBuf::from(format!("{stem}.artifact.json"))
    });

    // Write the artifact
    artifact
        .to_file(&output_path)
        .with_context(|| format!("Failed to write artifact: {}", output_path.display()))?;

    // Print summary
    println!("Created artifact: {}", output_path.display());
    println!("  Service: {}", artifact.service);
    println!("  Version: {}", artifact.version);
    println!("  Format: {} {}", artifact.format, artifact.format_version);
    println!("  Operations: {}", artifact.operations.len());
    println!("  Schemas: {}", artifact.schemas.len());
    println!(
        "  Checksum: {}...{}",
        &artifact.checksum.value[..8],
        &artifact.checksum.value[artifact.checksum.value.len() - 8..]
    );

    Ok(())
}

/// Inspect an existing artifact.
#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Path to the artifact file
    #[arg(value_name = "ARTIFACT")]
    pub artifact: PathBuf,

    /// Verify the artifact checksum
    #[arg(long)]
    pub verify: bool,

    /// Output format
    #[arg(long, default_value = "text")]
    pub format: InspectFormat,
}

/// Output format for inspect command.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum InspectFormat {
    #[default]
    Text,
    Json,
}

/// Runs the inspect command.
pub fn run_inspect(args: &InspectArgs) -> Result<()> {
    // Load the artifact
    let artifact = Artifact::from_file(&args.artifact)
        .with_context(|| format!("Failed to load artifact: {}", args.artifact.display()))?;

    // Verify checksum if requested
    if args.verify {
        artifact
            .verify_checksum()
            .context("Checksum verification failed")?;
        println!("✓ Artifact checksum verified");
    }

    match args.format {
        InspectFormat::Text => {
            println!("Artifact: {}", artifact.id());
            println!("  Service: {}", artifact.service);
            println!("  Version: {}", artifact.version);
            println!("  Format: {} {}", artifact.format, artifact.format_version);
            println!("  Schema: {}", artifact.schema);
            println!();
            println!("Metadata:");
            println!("  Created: {}", artifact.metadata.created_at);
            if let Some(owner) = &artifact.metadata.owner {
                println!("  Owner: {owner}");
            }
            if let Some(commit) = &artifact.metadata.git_commit {
                println!("  Git Commit: {commit}");
            }
            if let Some(repo) = &artifact.metadata.git_repository {
                println!("  Git Repository: {repo}");
            }
            println!();
            println!("Checksum:");
            println!("  Algorithm: {}", artifact.checksum.algorithm);
            println!("  Value: {}", artifact.checksum.value);
            println!();
            println!("Operations ({}):", artifact.operations.len());
            for op in &artifact.operations {
                println!("  {} {} {}", op.method, op.path, op.id);
            }
            if !artifact.schemas.is_empty() {
                println!();
                println!("Schemas ({}):", artifact.schemas.len());
                for name in artifact.schemas.keys() {
                    println!("  {name}");
                }
            }
        }
        InspectFormat::Json => {
            let json = artifact.to_json()?;
            println!("{json}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_args_defaults() {
        let args = PackArgs {
            contract: PathBuf::from("api.yaml"),
            output: None,
            service: None,
            version: None,
            owner: None,
            git_commit: None,
            git_repository: None,
            include_raw: true,
            verify: false,
        };

        assert!(args.output.is_none());
        assert!(args.include_raw);
    }

    #[test]
    fn test_inspect_format_default() {
        let format = InspectFormat::default();
        assert!(matches!(format, InspectFormat::Text));
    }
}
