//! Code generation command implementation.
//!
//! Generates typed code from contracts for use in services.

use anyhow::Context;
use clap::{Args, ValueEnum};
use std::path::{Path, PathBuf};
use themis_codegen::{
    CodeGenerator, CppGenerator, GeneratorConfig, GoGenerator, JsonSchemaGenerator,
    PythonGenerator, RustGenerator, TypeScriptGenerator,
};
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
    /// Generate C++ code
    Cpp,
    /// Generate Go code
    Go,
    /// Generate JSON Schema files (for use with quicktype, etc.)
    JsonSchema,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Typescript => write!(f, "typescript"),
            Self::Python => write!(f, "python"),
            Self::Cpp => write!(f, "cpp"),
            Self::Go => write!(f, "go"),
            Self::JsonSchema => write!(f, "json-schema"),
        }
    }
}

/// Supported contract formats for code generation.
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

    /// Contract format (auto-detected from extension if not specified)
    #[arg(short = 'F', long, value_enum)]
    pub format: Option<ContractFormat>,

    /// Service name (required for protobuf and graphql)
    #[arg(short, long)]
    pub service_name: Option<String>,

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

/// Runs the codegen command.
#[allow(clippy::too_many_lines)]
pub fn run(args: &CodegenArgs) -> anyhow::Result<()> {
    // Check file exists
    if !args.contract.exists() {
        anyhow::bail!("Contract file not found: {}", args.contract.display());
    }

    // Detect or use specified format
    let format = args
        .format
        .clone()
        .unwrap_or_else(|| detect_format(&args.contract));

    // Read the contract file
    let content = std::fs::read_to_string(&args.contract)
        .with_context(|| format!("Failed to read contract file: {}", args.contract.display()))?;

    // Parse the contract based on format
    let contract = match format {
        ContractFormat::Openapi => parse_openapi(&content)
            .with_context(|| format!("Failed to parse OpenAPI: {}", args.contract.display()))?,
        ContractFormat::Protobuf => {
            let service_name = args.service_name.as_deref().unwrap_or("service");
            themis_protobuf::parse(&content, service_name)
                .with_context(|| format!("Failed to parse Protobuf: {}", args.contract.display()))?
        }
        ContractFormat::Graphql => {
            let service_name = args.service_name.as_deref().unwrap_or("service");
            themis_graphql::parse(&content, service_name)
                .with_context(|| format!("Failed to parse GraphQL: {}", args.contract.display()))?
        }
        ContractFormat::Asyncapi => themis_asyncapi::parse(&content)
            .with_context(|| format!("Failed to parse AsyncAPI: {}", args.contract.display()))?,
    };

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
        Language::Cpp => {
            let generator = CppGenerator::new(config);
            generator
                .generate(&contract)
                .with_context(|| "Failed to generate code")?
        }
        Language::Go => {
            let generator = GoGenerator::new(config);
            generator
                .generate(&contract)
                .with_context(|| "Failed to generate code")?
        }
        Language::JsonSchema => {
            let generator = JsonSchemaGenerator::new(config);
            generator
                .generate(&contract)
                .with_context(|| "Failed to generate JSON Schema")?
        }
    };

    // Print summary
    println!(
        "📦 Generating {} code from {} contract: {}",
        args.language,
        format,
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
        assert_eq!(format!("{}", Language::Typescript), "typescript");
        assert_eq!(format!("{}", Language::Python), "python");
        assert_eq!(format!("{}", Language::Cpp), "cpp");
        assert_eq!(format!("{}", Language::Go), "go");
        assert_eq!(format!("{}", Language::JsonSchema), "json-schema");
    }

    #[test]
    fn test_codegen_cpp_language() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "-l", "cpp"]);
        assert!(matches!(cli.codegen.language, Language::Cpp));
    }

    #[test]
    fn test_codegen_go_language() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "-l", "go"]);
        assert!(matches!(cli.codegen.language, Language::Go));
    }

    #[test]
    fn test_detect_format_openapi() {
        let path = PathBuf::from("api.yaml");
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
    fn test_codegen_with_format() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "-F", "protobuf"]);
        assert!(matches!(cli.codegen.format, Some(ContractFormat::Protobuf)));
    }

    #[test]
    fn test_codegen_with_service_name() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.proto", "-s", "my-service"]);
        assert_eq!(cli.codegen.service_name, Some("my-service".to_string()));
    }

    #[test]
    fn test_codegen_json_schema_language() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            codegen: CodegenArgs,
        }

        let cli = TestCli::parse_from(["test", "api.yaml", "-l", "json-schema"]);
        assert!(matches!(cli.codegen.language, Language::JsonSchema));
    }
}