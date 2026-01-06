//! # Themis CLI
//!
//! Command-line interface for Themis contract governance.
//!
//! ## Commands
//!
//! - `themis validate` - Validate contract syntax and schema
//! - `themis lint` - Run linting rules
//! - `themis diff` - Compare two contract versions
//! - `themis codegen` - Generate code from contracts
//! - `themis pack` - Create artifact from contract
//! - `themis inspect` - Inspect an artifact
//! - `themis publish` - Publish artifact to registry
//! - `themis fetch` - Fetch artifact from registry
//! - `themis versions` - List versions of a service

// CLI args commonly have many bools
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::uninlined_format_args)]

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

    /// Generate code from a contract
    Codegen(commands::codegen::CodegenArgs),

    /// Create an artifact from a contract
    Pack(commands::pack::PackArgs),

    /// Inspect an artifact
    Inspect(commands::pack::InspectArgs),

    /// Publish an artifact to the registry
    Publish(commands::registry::PublishArgs),

    /// Fetch an artifact from the registry
    Fetch(commands::registry::FetchArgs),

    /// List versions of a service in the registry
    Versions(commands::registry::ListVersionsArgs),
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
        Commands::Codegen(args) => commands::codegen::run(&args),
        Commands::Pack(args) => commands::pack::run(&args),
        Commands::Inspect(args) => commands::pack::run_inspect(&args),
        Commands::Publish(args) => commands::registry::run_publish(&args),
        Commands::Fetch(args) => commands::registry::run_fetch(&args),
        Commands::Versions(args) => commands::registry::run_list_versions(&args),
    }
}
