//! Validate command implementation.
//!
//! Validates `OpenAPI` contracts for syntax, schema compliance, and Themis rules.

use anyhow::Context;
use clap::Args;
use std::path::PathBuf;
use themis_openapi::{validate_openapi, ValidationResult};

/// Arguments for the validate command.
#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the contract file
    #[arg(required = true)]
    pub contract: PathBuf,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Treat warnings as errors
    #[arg(short = 'W', long)]
    pub warnings_as_errors: bool,
}

/// Runs the validate command.
pub fn run(args: &ValidateArgs) -> anyhow::Result<()> {
    // Check file exists
    if !args.contract.exists() {
        anyhow::bail!("Contract file not found: {}", args.contract.display());
    }

    // Read the contract file
    let content = std::fs::read_to_string(&args.contract)
        .with_context(|| format!("Failed to read contract file: {}", args.contract.display()))?;

    // Validate the contract
    let result = validate_openapi(&content)
        .with_context(|| format!("Failed to parse contract: {}", args.contract.display()))?;

    // Output results
    match args.format.as_str() {
        "json" => output_json(&result, args),
        _ => output_text(&result, args),
    }

    // Determine exit status
    let has_errors = !result.is_valid();
    let warnings_are_errors = args.warnings_as_errors && !result.warnings.is_empty();

    if has_errors || warnings_are_errors {
        anyhow::bail!("Validation failed");
    }

    Ok(())
}

/// Outputs validation results as human-readable text.
fn output_text(result: &ValidationResult, args: &ValidateArgs) {
    println!(
        "Validating: {}",
        args.contract
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    println!();

    // Print errors
    for error in &result.errors {
        println!("❌ [{}] {}", error.code, error.message);
        if let Some(path) = &error.path {
            println!("   at: {path}");
        }
    }

    // Print warnings
    for warning in &result.warnings {
        println!("⚠️  [{}] {}", warning.code, warning.message);
        if let Some(path) = &warning.path {
            println!("   at: {path}");
        }
    }

    // Print summary
    println!();
    if result.is_valid() && result.warnings.is_empty() {
        println!("✅ Contract is valid");
    } else if result.is_valid() {
        println!(
            "✅ Contract is valid with {} warning(s)",
            result.warning_count()
        );
    } else {
        println!(
            "❌ Validation failed: {} error(s), {} warning(s)",
            result.error_count(),
            result.warning_count()
        );
    }
}

/// Outputs validation results as JSON.
fn output_json(result: &ValidationResult, _args: &ValidateArgs) {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        valid: bool,
        error_count: usize,
        warning_count: usize,
        errors: Vec<JsonIssue>,
        warnings: Vec<JsonIssue>,
    }

    #[derive(serde::Serialize)]
    struct JsonIssue {
        code: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    }

    let output = JsonOutput {
        valid: result.is_valid(),
        error_count: result.error_count(),
        warning_count: result.warning_count(),
        errors: result
            .errors
            .iter()
            .map(|e| JsonIssue {
                code: e.code.clone(),
                message: e.message.clone(),
                path: e.path.clone(),
            })
            .collect(),
        warnings: result
            .warnings
            .iter()
            .map(|w| JsonIssue {
                code: w.code.clone(),
                message: w.message.clone(),
                path: w.path.clone(),
            })
            .collect(),
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
