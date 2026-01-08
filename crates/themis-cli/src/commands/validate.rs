//! Validate command implementation.
//!
//! Validates contracts for syntax, schema compliance, and Themis rules.

use anyhow::Context;
use clap::{Args, ValueEnum};
use std::path::{Path, PathBuf};
use themis_openapi::{validate_openapi, ValidationResult};

/// Supported contract formats.
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ContractFormat {
    /// `OpenAPI` 3.x specification
    #[default]
    Openapi,
    /// Protocol Buffers v3
    Protobuf,
    /// GraphQL SDL
    Graphql,
    /// `AsyncAPI` 3.0 specification
    Asyncapi,
}

impl std::fmt::Display for ContractFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Openapi => write!(f, "openapi"),
            Self::Protobuf => write!(f, "protobuf"),
            Self::Graphql => write!(f, "graphql"),
            Self::Asyncapi => write!(f, "asyncapi"),
        }
    }
}

/// Arguments for the validate command.
#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the contract file
    #[arg(required = true)]
    pub contract: PathBuf,

    /// Contract format (auto-detected from extension if not specified)
    #[arg(short = 'F', long, value_enum)]
    pub format_type: Option<ContractFormat>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Treat warnings as errors
    #[arg(short = 'W', long)]
    pub warnings_as_errors: bool,

    /// Service name (required for protobuf and graphql)
    #[arg(short, long)]
    pub service_name: Option<String>,
}

/// Detects the contract format from file extension.
fn detect_format(path: &Path) -> ContractFormat {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "proto" => ContractFormat::Protobuf,
        "graphql" | "gql" => ContractFormat::Graphql,
        _ => {
            // Check filename patterns for asyncapi
            let filename = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("")
                .to_lowercase();
            if filename.contains("asyncapi") {
                ContractFormat::Asyncapi
            } else {
                ContractFormat::Openapi
            }
        }
    }
}

/// Runs the validate command.
pub fn run(args: &ValidateArgs) -> anyhow::Result<()> {
    // Check file exists
    if !args.contract.exists() {
        anyhow::bail!("Contract file not found: {}", args.contract.display());
    }

    // Detect or use specified format
    let format = args
        .format_type
        .clone()
        .unwrap_or_else(|| detect_format(&args.contract));

    // Read the contract file
    let content = std::fs::read_to_string(&args.contract)
        .with_context(|| format!("Failed to read contract file: {}", args.contract.display()))?;

    // Validate based on format
    match format {
        ContractFormat::Openapi => {
            let result = validate_openapi(&content)
                .with_context(|| format!("Failed to parse contract: {}", args.contract.display()))?;

            // Output results
            match args.format.as_str() {
                "json" => output_json(&result, args),
                _ => output_text(&result, args, &format),
            }

            // Determine exit status
            let has_errors = !result.is_valid();
            let warnings_are_errors = args.warnings_as_errors && !result.warnings.is_empty();

            if has_errors || warnings_are_errors {
                anyhow::bail!("Validation failed");
            }
        }
        ContractFormat::Protobuf => {
            let service_name = args
                .service_name
                .as_deref()
                .unwrap_or("service");
            
            // Try to parse - if it succeeds, it's valid
            themis_protobuf::parse(&content, service_name)
                .with_context(|| format!("Failed to parse protobuf: {}", args.contract.display()))?;

            println!(
                "Validating ({}): {}",
                format,
                args.contract
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            println!();
            println!("✅ Contract is valid");
        }
        ContractFormat::Graphql => {
            let service_name = args
                .service_name
                .as_deref()
                .unwrap_or("service");
            
            // Try to parse - if it succeeds, it's valid
            themis_graphql::parse(&content, service_name)
                .with_context(|| format!("Failed to parse graphql: {}", args.contract.display()))?;

            println!(
                "Validating ({}): {}",
                format,
                args.contract
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            println!();
            println!("✅ Contract is valid");
        }
        ContractFormat::Asyncapi => {
            // Try to parse - if it succeeds, it's valid
            themis_asyncapi::parse(&content)
                .with_context(|| format!("Failed to parse asyncapi: {}", args.contract.display()))?;

            println!(
                "Validating ({}): {}",
                format,
                args.contract
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            println!();
            println!("✅ Contract is valid");
        }
    }

    Ok(())
}

/// Outputs validation results as human-readable text.
fn output_text(result: &ValidationResult, args: &ValidateArgs, format: &ContractFormat) {
    println!(
        "Validating ({}): {}",
        format,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_openapi_yaml() {
        let path = PathBuf::from("api.yaml");
        assert!(matches!(detect_format(&path), ContractFormat::Openapi));
    }

    #[test]
    fn test_detect_format_openapi_json() {
        let path = PathBuf::from("api.json");
        assert!(matches!(detect_format(&path), ContractFormat::Openapi));
    }

    #[test]
    fn test_detect_format_protobuf() {
        let path = PathBuf::from("service.proto");
        assert!(matches!(detect_format(&path), ContractFormat::Protobuf));
    }

    #[test]
    fn test_detect_format_graphql() {
        let path = PathBuf::from("schema.graphql");
        assert!(matches!(detect_format(&path), ContractFormat::Graphql));
    }

    #[test]
    fn test_detect_format_graphql_gql() {
        let path = PathBuf::from("schema.gql");
        assert!(matches!(detect_format(&path), ContractFormat::Graphql));
    }

    #[test]
    fn test_detect_format_asyncapi() {
        let path = PathBuf::from("asyncapi.yaml");
        assert!(matches!(detect_format(&path), ContractFormat::Asyncapi));
    }

    #[test]
    fn test_contract_format_display() {
        assert_eq!(format!("{}", ContractFormat::Openapi), "openapi");
        assert_eq!(format!("{}", ContractFormat::Protobuf), "protobuf");
        assert_eq!(format!("{}", ContractFormat::Graphql), "graphql");
        assert_eq!(format!("{}", ContractFormat::Asyncapi), "asyncapi");
    }

    #[test]
    fn test_validate_args_parsing() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            validate: ValidateArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml"]);
        assert_eq!(cli.validate.contract, PathBuf::from("api.yaml"));
        assert!(cli.validate.format_type.is_none());
        assert_eq!(cli.validate.format, "text");
        assert!(!cli.validate.warnings_as_errors);
    }

    #[test]
    fn test_validate_args_with_format_type() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            validate: ValidateArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "-F", "protobuf"]);
        assert!(matches!(
            cli.validate.format_type,
            Some(ContractFormat::Protobuf)
        ));
    }

    #[test]
    fn test_validate_args_with_service_name() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            validate: ValidateArgs,
        }

        let cli = TestCli::parse_from(["test", "api.proto", "-s", "my-service"]);
        assert_eq!(cli.validate.service_name, Some("my-service".to_string()));
    }
}
