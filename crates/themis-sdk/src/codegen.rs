//! Code generation functionality.
//!
//! This module provides functions for generating code from contracts.

use std::path::Path;

use themis_codegen::{
    CodeGenerator, CppGenerator, GeneratedCode, GeneratorConfig, GoGenerator,
    JsonSchemaGenerator, PythonGenerator, RustGenerator, TypeScriptGenerator,
};
use themis_core::Contract;

use crate::error::{SdkError, SdkResult};
use crate::Language;

/// Generate code from a contract.
///
/// # Arguments
///
/// * `contract` - The contract to generate code from
/// * `language` - The target language for code generation
///
/// # Returns
///
/// The generated code as a collection of files
///
/// # Errors
///
/// Returns an error if code generation fails
///
/// # Examples
///
/// ```ignore
/// use themis_sdk::codegen::generate;
/// use themis_sdk::parse::parse_string;
/// use themis_sdk::Language;
///
/// let contract = parse_string(yaml)?;
/// let code = generate(&contract, Language::Rust)?;
/// for file in &code.files {
///     println!("Generated: {} ({} bytes)", file.path, file.content.len());
/// }
/// ```
pub fn generate(contract: &Contract, language: Language) -> SdkResult<GeneratedCode> {
    let config = GeneratorConfig::default();
    generate_with_config(contract, language, &config)
}

/// Generate code from a contract with custom configuration.
///
/// # Arguments
///
/// * `contract` - The contract to generate code from
/// * `language` - The target language for code generation
/// * `config` - The code generation configuration
///
/// # Returns
///
/// The generated code as a collection of files
///
/// # Errors
///
/// Returns an error if code generation fails
pub fn generate_with_config(
    contract: &Contract,
    language: Language,
    config: &GeneratorConfig,
) -> SdkResult<GeneratedCode> {
    match language {
        Language::Rust => {
            let generator = RustGenerator::new(config.clone());
            generator.generate(contract).map_err(|e| SdkError::CodeGen {
                message: e.to_string(),
            })
        }
        Language::TypeScript => {
            let generator = TypeScriptGenerator::new(config.clone());
            generator.generate(contract).map_err(|e| SdkError::CodeGen {
                message: e.to_string(),
            })
        }
        Language::Python => {
            let generator = PythonGenerator::new(config.clone());
            generator.generate(contract).map_err(|e| SdkError::CodeGen {
                message: e.to_string(),
            })
        }
        Language::Go => {
            let generator = GoGenerator::new(config.clone());
            generator.generate(contract).map_err(|e| SdkError::CodeGen {
                message: e.to_string(),
            })
        }
        Language::Cpp => {
            let generator = CppGenerator::new(config.clone());
            generator.generate(contract).map_err(|e| SdkError::CodeGen {
                message: e.to_string(),
            })
        }
        Language::JsonSchema => {
            let generator = JsonSchemaGenerator::new(config.clone());
            generator.generate(contract).map_err(|e| SdkError::CodeGen {
                message: e.to_string(),
            })
        }
    }
}

/// Generate code from a contract file.
///
/// # Arguments
///
/// * `contract_path` - Path to the contract file
/// * `language` - The target language for code generation
///
/// # Returns
///
/// The generated code as a collection of files
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The contract cannot be parsed
/// - Code generation fails
pub fn generate_from_file<P: AsRef<Path>>(
    contract_path: P,
    language: Language,
) -> SdkResult<GeneratedCode> {
    let contract = crate::parse::parse_file(contract_path)?;
    generate(&contract, language)
}

/// Generate code from a contract file and write to an output directory.
///
/// # Arguments
///
/// * `contract_path` - Path to the contract file
/// * `language` - The target language for code generation
/// * `output_dir` - Directory to write generated files to
///
/// # Returns
///
/// The list of generated file paths
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The contract cannot be parsed
/// - Code generation fails
/// - Files cannot be written
pub fn generate_to_directory<P: AsRef<Path>, Q: AsRef<Path>>(
    contract_path: P,
    language: Language,
    output_dir: Q,
) -> SdkResult<Vec<std::path::PathBuf>> {
    let code = generate_from_file(contract_path, language)?;
    write_generated_code(&code, output_dir)
}

/// Write generated code to a directory.
///
/// # Arguments
///
/// * `code` - The generated code to write
/// * `output_dir` - Directory to write generated files to
///
/// # Returns
///
/// The list of generated file paths
///
/// # Errors
///
/// Returns an error if files cannot be written
pub fn write_generated_code<P: AsRef<Path>>(
    code: &GeneratedCode,
    output_dir: P,
) -> SdkResult<Vec<std::path::PathBuf>> {
    let output_dir = output_dir.as_ref();
    let mut paths = Vec::new();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir).map_err(|e| SdkError::FileWrite {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    for file in &code.files {
        let file_path = output_dir.join(&file.path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SdkError::FileWrite {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        std::fs::write(&file_path, &file.content).map_err(|e| SdkError::FileWrite {
            path: file_path.clone(),
            source: e,
        })?;

        paths.push(file_path);
    }

    Ok(paths)
}

/// Get a list of supported languages for code generation.
#[must_use]
pub fn supported_languages() -> Vec<Language> {
    vec![
        Language::Rust,
        Language::TypeScript,
        Language::Python,
        Language::Go,
        Language::Cpp,
        Language::JsonSchema,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_languages() {
        let languages = supported_languages();
        assert!(!languages.is_empty());
        assert!(languages.contains(&Language::Rust));
        assert!(languages.contains(&Language::TypeScript));
        assert!(languages.contains(&Language::Python));
        assert!(languages.contains(&Language::Go));
        assert!(languages.contains(&Language::Cpp));
        assert!(languages.contains(&Language::JsonSchema));
    }
}
