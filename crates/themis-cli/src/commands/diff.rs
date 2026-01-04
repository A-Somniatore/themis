//! Diff command implementation.

use std::path::PathBuf;
use clap::Args;

/// Arguments for the diff command.
#[derive(Args)]
pub struct DiffArgs {
    /// Path to the old (base) contract file
    #[arg(required = true)]
    pub old: PathBuf,

    /// Path to the new contract file
    #[arg(required = true)]
    pub new: PathBuf,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Fail if breaking changes are detected
    #[arg(long)]
    pub fail_on_breaking: bool,
}

/// Runs the diff command.
pub fn run(args: DiffArgs) -> anyhow::Result<()> {
    println!("Comparing contracts:");
    println!("  Old: {}", args.old.display());
    println!("  New: {}", args.new.display());

    // TODO: Implement diff in Week 6
    if !args.old.exists() {
        anyhow::bail!("Old contract file not found: {}", args.old.display());
    }
    if !args.new.exists() {
        anyhow::bail!("New contract file not found: {}", args.new.display());
    }

    println!("✓ Contract diff not yet implemented");
    Ok(())
}
