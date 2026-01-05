//! Diff command implementation.
//!
//! Compares two contract versions and detects breaking changes,
//! additions, and modifications.

use anyhow::Context;
use clap::Args;
use std::path::{Path, PathBuf};
use themis_compat::{check_compatibility, CompatibilityReport, SuggestedBump};
use themis_openapi::parse_openapi;

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

    /// Validate that version bump matches detected changes
    #[arg(long)]
    pub validate_version: bool,
}

/// Runs the diff command.
pub fn run(args: &DiffArgs) -> anyhow::Result<()> {
    // Check files exist
    if !args.old.exists() {
        anyhow::bail!("Old contract file not found: {}", args.old.display());
    }
    if !args.new.exists() {
        anyhow::bail!("New contract file not found: {}", args.new.display());
    }

    // Read contract files
    let old_content = std::fs::read_to_string(&args.old)
        .with_context(|| format!("Failed to read old contract: {}", args.old.display()))?;
    let new_content = std::fs::read_to_string(&args.new)
        .with_context(|| format!("Failed to read new contract: {}", args.new.display()))?;

    // Parse contracts
    let old_contract = parse_openapi(&old_content)
        .with_context(|| format!("Failed to parse old contract: {}", args.old.display()))?;
    let new_contract = parse_openapi(&new_content)
        .with_context(|| format!("Failed to parse new contract: {}", args.new.display()))?;

    // Run compatibility check
    let report = check_compatibility(&old_contract, &new_contract);

    // Output results
    match args.format.as_str() {
        "json" => output_json(&report)?,
        _ => output_text(&report, &args.old, &args.new),
    }

    // Validate version bump if requested
    if args.validate_version {
        validate_version_bump(&report)?;
    }

    // Check for breaking changes exit condition
    if args.fail_on_breaking && !report.is_compatible {
        anyhow::bail!(
            "Breaking changes detected ({} breaking change{})",
            report.breaking_changes.len(),
            if report.breaking_changes.len() == 1 {
                ""
            } else {
                "s"
            }
        );
    }

    Ok(())
}

/// Outputs the report in text format.
fn output_text(report: &CompatibilityReport, old_path: &Path, new_path: &Path) {
    println!("Contract Comparison");
    println!("===================");
    println!();
    println!(
        "  Old: {} (v{})",
        old_path.display(),
        report.old_version.as_deref().unwrap_or("?")
    );
    println!(
        "  New: {} (v{})",
        new_path.display(),
        report.new_version.as_deref().unwrap_or("?")
    );
    println!();

    // Summary
    if report.is_compatible {
        println!("✓ Contracts are backward compatible");
    } else {
        println!("✗ Breaking changes detected");
    }
    println!();
    println!("  {}", report.summary());
    println!(
        "  Suggested version bump: {}",
        suggested_bump_display(report.suggested_bump)
    );

    // Breaking changes
    if !report.breaking_changes.is_empty() {
        println!();
        println!("Breaking Changes ({}):", report.breaking_changes.len());
        for change in &report.breaking_changes {
            println!("  ✗ {change}");
        }
    }

    // Additions
    if !report.additions.is_empty() {
        println!();
        println!("Additions ({}):", report.additions.len());
        for addition in &report.additions {
            println!("  + {addition}");
        }
    }

    // Modifications
    if !report.modifications.is_empty() {
        println!();
        println!("Modifications ({}):", report.modifications.len());
        for modification in &report.modifications {
            println!("  ~ {modification}");
        }
    }

    println!();
}

/// Outputs the report in JSON format.
fn output_json(report: &CompatibilityReport) -> anyhow::Result<()> {
    let json =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;
    println!("{json}");
    Ok(())
}

/// Validates that the version bump matches detected changes.
fn validate_version_bump(report: &CompatibilityReport) -> anyhow::Result<()> {
    let (Some(old_ver), Some(new_ver)) = (&report.old_version, &report.new_version) else {
        return Ok(()); // Can't validate without versions
    };

    let old = semver::Version::parse(old_ver)
        .with_context(|| format!("Invalid old version: {old_ver}"))?;
    let new = semver::Version::parse(new_ver)
        .with_context(|| format!("Invalid new version: {new_ver}"))?;

    let actual_bump = if new.major > old.major {
        SuggestedBump::Major
    } else if new.minor > old.minor {
        SuggestedBump::Minor
    } else if new.patch > old.patch {
        SuggestedBump::Patch
    } else {
        SuggestedBump::None
    };

    let required = report.suggested_bump;

    // Check if bump is sufficient
    let is_sufficient = match required {
        SuggestedBump::Major => actual_bump == SuggestedBump::Major,
        SuggestedBump::Minor => {
            matches!(actual_bump, SuggestedBump::Major | SuggestedBump::Minor)
        }
        SuggestedBump::Patch => {
            matches!(
                actual_bump,
                SuggestedBump::Major | SuggestedBump::Minor | SuggestedBump::Patch
            )
        }
        SuggestedBump::None => true,
    };

    if !is_sufficient {
        anyhow::bail!(
            "Version bump from {} to {} ({}) is insufficient; a {} bump is required",
            old_ver,
            new_ver,
            suggested_bump_display(actual_bump),
            suggested_bump_display(required)
        );
    }

    Ok(())
}

/// Returns a display string for the suggested bump.
const fn suggested_bump_display(bump: SuggestedBump) -> &'static str {
    match bump {
        SuggestedBump::Major => "major",
        SuggestedBump::Minor => "minor",
        SuggestedBump::Patch => "patch",
        SuggestedBump::None => "none",
    }
}
