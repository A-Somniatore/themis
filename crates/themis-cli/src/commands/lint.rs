//! Lint command implementation.
//!
//! Runs configurable lint rules against contracts to check for
//! naming conventions, documentation, and best practices.

use anyhow::Context;
use clap::Args;
use std::path::{Path, PathBuf};
use themis_lint::{LintConfig, LintConfigFile, LintReport, LintReporter, Severity};
use themis_openapi::parse_openapi;

/// Arguments for the lint command.
#[derive(Args)]
pub struct LintArgs {
    /// Path to the contract file
    pub contract: Option<PathBuf>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Treat warnings as errors
    #[arg(long)]
    pub strict: bool,

    /// Path to lint configuration file (default: auto-detect .themis-lint.yaml)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Use strict configuration (all rules as errors)
    #[arg(long, conflicts_with = "relaxed")]
    pub strict_config: bool,

    /// Use relaxed configuration (all rules as warnings)
    #[arg(long, conflicts_with = "strict_config")]
    pub relaxed: bool,

    /// List available lint rules
    #[arg(long)]
    pub list_rules: bool,
}

/// Runs the lint command.
pub fn run(args: &LintArgs) -> anyhow::Result<()> {
    // Handle --list-rules flag
    if args.list_rules {
        print_available_rules();
        return Ok(());
    }

    // Get contract path (required if not listing rules)
    let contract_path = args
        .contract
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Contract file path is required"))?;

    // Check file exists
    if !contract_path.exists() {
        anyhow::bail!("Contract file not found: {}", contract_path.display());
    }

    // Read the contract file
    let content = std::fs::read_to_string(contract_path)
        .with_context(|| format!("Failed to read contract file: {}", contract_path.display()))?;

    // Parse the contract
    let contract = parse_openapi(&content)
        .with_context(|| format!("Failed to parse contract: {}", contract_path.display()))?;

    // Load lint configuration
    let config = load_lint_config(args, contract_path)?;
    let linter = LintReporter::new(config);

    // Run linting
    let report = linter.lint(&contract);

    // Output results
    match args.format.as_str() {
        "json" => output_json(&report, contract_path),
        _ => output_text(&report, contract_path),
    }

    // Determine exit status
    let has_errors = report.error_count() > 0;
    let warnings_are_errors = args.strict && report.warning_count() > 0;

    if has_errors || warnings_are_errors {
        anyhow::bail!("Linting failed");
    }

    Ok(())
}

/// Loads lint configuration based on CLI args and config files.
fn load_lint_config(args: &LintArgs, contract_path: &Path) -> anyhow::Result<LintConfig> {
    // Priority:
    // 1. --strict-config flag
    // 2. --relaxed flag
    // 3. --config <file>
    // 4. Auto-detect .themis-lint.yaml
    // 5. Default configuration

    if args.strict_config {
        return Ok(LintConfig::strict());
    }

    if args.relaxed {
        return Ok(LintConfig::relaxed());
    }

    if let Some(config_path) = &args.config {
        if !config_path.exists() {
            anyhow::bail!("Config file not found: {}", config_path.display());
        }
        let file_config = LintConfigFile::from_file(config_path)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
        return file_config
            .to_lint_config()
            .with_context(|| "Invalid lint configuration");
    }

    // Auto-detect config file from contract directory
    let start_dir = contract_path.parent().unwrap_or_else(|| Path::new("."));

    match LintConfigFile::load_or_default(start_dir) {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("Warning: Failed to load config file: {e}");
            Ok(LintConfig::default())
        }
    }
}

/// Prints available lint rules.
fn print_available_rules() {
    let linter = LintReporter::with_defaults();
    let rules = linter.available_rules();

    println!("Available lint rules:\n");

    // Group by category
    let mut naming_rules: Vec<_> = rules
        .iter()
        .filter(|(id, _)| id.starts_with("naming/"))
        .collect();
    let mut docs_rules: Vec<_> = rules
        .iter()
        .filter(|(id, _)| id.starts_with("docs/"))
        .collect();

    naming_rules.sort_by_key(|(id, _)| *id);
    docs_rules.sort_by_key(|(id, _)| *id);

    println!("Naming Convention Rules:");
    for (id, desc) in naming_rules {
        println!("  {id:<30} {desc}");
    }

    println!("\nDocumentation Rules:");
    for (id, desc) in docs_rules {
        println!("  {id:<30} {desc}");
    }

    println!();
}

/// Outputs lint results as human-readable text.
fn output_text(report: &LintReport, contract_path: &Path) {
    println!(
        "Linting: {}",
        contract_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    println!();

    // Print issues grouped by severity
    let errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .collect();
    let infos: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Info)
        .collect();

    // Print errors
    for issue in &errors {
        println!("❌ [{}] {}", issue.rule, issue.message);
        if let Some(loc) = &issue.location {
            println!("   at: {loc}");
        }
    }

    // Print warnings
    for issue in &warnings {
        println!("⚠️  [{}] {}", issue.rule, issue.message);
        if let Some(loc) = &issue.location {
            println!("   at: {loc}");
        }
    }

    // Print info
    for issue in &infos {
        println!("ℹ️  [{}] {}", issue.rule, issue.message);
        if let Some(loc) = &issue.location {
            println!("   at: {loc}");
        }
    }

    // Print summary
    println!();
    if report.is_clean() {
        println!("✅ No lint issues found");
    } else {
        let parts: Vec<String> = [
            (errors.len(), "error"),
            (warnings.len(), "warning"),
            (infos.len(), "info"),
        ]
        .iter()
        .filter(|(count, _)| *count > 0)
        .map(|(count, label)| {
            if *count == 1 {
                format!("{count} {label}")
            } else {
                format!("{count} {label}s")
            }
        })
        .collect();

        println!("📋 Found {}", parts.join(", "));
    }
}

/// Outputs lint results as JSON.
fn output_json(report: &LintReport, _contract_path: &Path) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct JsonReport {
        issues: Vec<JsonIssue>,
        summary: JsonSummary,
    }

    #[derive(Serialize)]
    struct JsonIssue {
        rule: String,
        severity: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        location: Option<String>,
    }

    #[derive(Serialize)]
    struct JsonSummary {
        total: usize,
        errors: usize,
        warnings: usize,
        info: usize,
        clean: bool,
    }

    let json_report = JsonReport {
        issues: report
            .issues
            .iter()
            .map(|i| JsonIssue {
                rule: i.rule.clone(),
                severity: match i.severity {
                    Severity::Error => "error".to_string(),
                    Severity::Warning => "warning".to_string(),
                    Severity::Info => "info".to_string(),
                },
                message: i.message.clone(),
                location: i.location.clone(),
            })
            .collect(),
        summary: JsonSummary {
            total: report.issues.len(),
            errors: report.error_count(),
            warnings: report.warning_count(),
            info: report.info_count(),
            clean: report.is_clean(),
        },
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&json_report).unwrap_or_default()
    );
}
