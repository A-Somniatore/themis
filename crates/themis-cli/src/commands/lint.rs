//! Lint command implementation.

use std::path::PathBuf;
use clap::Args;

/// Arguments for the lint command.
#[derive(Args)]
pub struct LintArgs {
    /// Path to the contract file
    #[arg(required = true)]
    pub contract: PathBuf,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Treat warnings as errors
    #[arg(long)]
    pub strict: bool,
}

/// Runs the lint command.
pub fn run(args: LintArgs) -> anyhow::Result<()> {
    println!("Linting contract: {}", args.contract.display());

    // TODO: Implement linting in Week 5
    if !args.contract.exists() {
        anyhow::bail!("Contract file not found: {}", args.contract.display());
    }

    println!("✓ Contract linting not yet implemented");
    Ok(())
}
