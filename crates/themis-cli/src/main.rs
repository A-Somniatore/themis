//! # Themis CLI
//!
//! Command-line interface for Themis contract governance.
//!
//! ## Commands
//!
//! - `themis validate` - Validate contract syntax and schema
//! - `themis lint` - Run linting rules
//! - `themis diff` - Compare two contract versions
//! - `themis codegen` - Generate code (future)
//! - `themis publish` - Publish artifact (future)
//! - `themis fetch` - Fetch artifact (future)

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;

/// Themis - Contract and Schema Governance CLI
#[derive(Parser)]
#[command(name = "themis")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a contract file
    Validate(commands::validate::ValidateArgs),

    /// Lint a contract file
    Lint(commands::lint::LintArgs),

    /// Compare two contract versions
    Diff(commands::diff::DiffArgs),
}

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate(args) => commands::validate::run(&args),
        Commands::Lint(args) => commands::lint::run(&args),
        Commands::Diff(args) => commands::diff::run(&args),
    }
}
