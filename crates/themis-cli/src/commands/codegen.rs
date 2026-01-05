//! Code generation command implementation.
//!
//! Generates typed code from contracts for use in services.

use anyhow::Context;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
use themis_codegen::{CodeGenerator, GeneratorConfig, PythonGenerator, RustGenerator, TypeScriptGenerator};
use themis_openapi::parse_openapi;

/// Supported target languages for code generation.
#[derive(Debug, Clone, ValueEnum)]
pub enum Language {
    /// Generate Rust code
    Rust,
    /// Generate TypeScript code
    Typescript,
    /// Generate Python code
    Python,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Typescript => write!(f, "typescript"),
            Self::Python => write!(f, "python"),
        }
    }
}

/// Arguments for the codegen command.
#[derive(Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct CodegenArgs {
    /// Path to the contract file
    #[arg(required = true)]
    pub contract: PathBuf,

    /// Output directory for generated code
    #[arg(short, long, default_value = "generated")]
    pub output: PathBuf,

    /// Target language
    #[arg(short, long, value_enum, default_value = "rust")]
    pub language: Language,

    /// Include documentation comments in generated code
    #[arg(long, default_value = "true")]
    pub include_docs: bool,

    /// Include validation derives (e.g., validator crate)
    #[arg(long)]
    pub include_validation: bool,

    /// Overwrite existing files
    #[arg(long)]
    pub force: bool,

    /// Dry run - show what would be generated without writing
    #[arg(long)]
    pub dry_run: bool,
}

/// Runs the codegen command.
pub fn run(args: &CodegenArgs) -> anyhow::Result<()> {
    // Check file exists
    if !args.contract.exists() {
        anyhow::bail!("Contract file not found: {}", args.contract.display());
    }

    // Read the contract file
    let content = std::fs::read_to_string(&args.contract)
        .with_context(|| format!("Failed to read contract file: {}", args.contract.display()))?;

    // Parse the contract
    let contract = parse_openapi(&content)
        .with_context(|| format!("Failed to parse contract: {}", args.contract.display()))?;

    // Create generator config
    let mut config = GeneratorConfig::new();
    config.include_docs = args.include_docs;
    config.include_validation = args.include_validation;

    // Create generator for target language
    let generated = match args.language {
        Language::Rust => {
            let generator = RustGenerator::new(config);
            generator
                .generate(&contract)
                .with_context(|| "Failed to generate code")?
        }
        Language::Typescript => {
            let generator = TypeScriptGenerator::new(config);
            generator
                .generate(&contract)
                .with_context(|| "Failed to generate code")?
        }
        Language::Python => {
            let generator = PythonGenerator::new(config);
            generator
                .generate(&contract)
                .with_context(|| "Failed to generate code")?
        }
    };

    // Print summary
    println!(
        "📦 Generating {} code from: {}",
        args.language,
        args.contract
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    println!();

    // Handle warnings
    for warning in &generated.warnings {
        println!("⚠️  Warning: {warning}");
    }

    // Print or write files
    if args.dry_run {
        println!("🔍 Dry run mode - showing what would be generated:");
        println!();
        for file in &generated.files {
            println!("  📄 {}", file.path);
            println!("     {} bytes", file.content.len());
        }
    } else {
        // Create output directory
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                args.output.display()
            )
        })?;

        // Write files
        for file in &generated.files {
            let output_path = args.output.join(&file.path);

            // Check if file exists
            if output_path.exists() && !args.force && !file.overwrite {
                anyhow::bail!(
                    "File already exists: {} (use --force to overwrite)",
                    output_path.display()
                );
            }

            std::fs::write(&output_path, &file.content)
                .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

            println!("  ✅ Generated: {}", output_path.display());
        }
    }

    println!();
    println!("✨ Generated {} files", generated.files.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_args_defaults() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml"]);
        assert_eq!(cli.codegen.contract, PathBuf::from("api.yaml"));
        assert_eq!(cli.codegen.output, PathBuf::from("generated"));
        assert!(matches!(cli.codegen.language, Language::Rust));
        assert!(cli.codegen.include_docs);
        assert!(!cli.codegen.include_validation);
        assert!(!cli.codegen.force);
        assert!(!cli.codegen.dry_run);
    }

    #[test]
    fn test_codegen_with_validation() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "--include-validation"]);
        assert!(cli.codegen.include_validation);
    }

    #[test]
    fn test_codegen_output_directory() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "-o", "src/gen"]);
        assert_eq!(cli.codegen.output, PathBuf::from("src/gen"));
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::Rust), "rust");
    }
}
