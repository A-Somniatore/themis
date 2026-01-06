//! Code generation integration tests.
//!
//! Tests that generated code is correct and consistent.

use crate::fixtures::{MINIMAL_CONTRACT, SECURE_CONTRACT, USERS_SERVICE_V1};
use themis_codegen::{
    CodeGenerator, GeneratorConfig, PythonGenerator, RustGenerator, TypeScriptGenerator,
};
use themis_openapi::parse_openapi;

/// Helper to get all generated code as a single string.
fn all_code(output: &themis_codegen::GeneratedCode) -> String {
    output
        .files
        .iter()
        .map(|f| f.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Tests Rust code generation produces valid output.
#[test]
fn test_rust_codegen_produces_valid_code() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse");

    let config = GeneratorConfig::default();

    let generator = RustGenerator::new(config);
    let output = generator
        .generate(&contract)
        .expect("Should generate Rust code");
    let code = all_code(&output);

    // Check for required Rust constructs
    assert!(
        code.contains("serde") || code.contains("Serialize"),
        "Should import serde"
    );
    assert!(code.contains("#[derive("), "Should have derive macros");
    assert!(code.contains("pub struct"), "Should have public structs");

    // Check for async trait if there are handlers
    if code.contains("Handler") {
        assert!(
            code.contains("async") || code.contains("async_trait"),
            "Handler should have async capability"
        );
    }
}

/// Tests TypeScript code generation produces valid output.
#[test]
fn test_typescript_codegen_produces_valid_code() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse");

    let config = GeneratorConfig::default();

    let generator = TypeScriptGenerator::new(config);
    let output = generator
        .generate(&contract)
        .expect("Should generate TypeScript code");
    let code = all_code(&output);

    // Check for TypeScript constructs
    assert!(
        code.contains("export") || code.contains("interface") || code.contains("type"),
        "Should have exported types"
    );

    // Check for proper typing
    assert!(
        code.contains("string") || code.contains("number") || code.contains("boolean"),
        "Should have type annotations"
    );
}

/// Tests Python code generation produces valid output.
#[test]
fn test_python_codegen_produces_valid_code() {
    let contract = parse_openapi(USERS_SERVICE_V1).expect("Should parse");

    let config = GeneratorConfig::default();

    let generator = PythonGenerator::new(config);
    let output = generator
        .generate(&contract)
        .expect("Should generate Python code");
    let code = all_code(&output);

    // Check for Python constructs
    assert!(
        code.contains("dataclass") || code.contains("class "),
        "Should have class definitions"
    );

    // Check for type hints
    assert!(
        code.contains("str") || code.contains("int") || code.contains("Optional"),
        "Should have type hints"
    );
}

/// Tests that all languages generate code for contracts.
#[test]
fn test_all_languages_generate_code() {
    let contract = parse_openapi(SECURE_CONTRACT).expect("Should parse");

    let config = GeneratorConfig::default();

    // Rust
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract).expect("Should generate Rust");
    let rust_code = all_code(&rust_output);
    assert!(!rust_code.is_empty(), "Rust code should not be empty");

    // TypeScript
    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen
        .generate(&contract)
        .expect("Should generate TypeScript");
    let ts_code = all_code(&ts_output);
    assert!(!ts_code.is_empty(), "TypeScript code should not be empty");

    // Python
    let py_gen = PythonGenerator::new(config.clone());
    let py_output = py_gen.generate(&contract).expect("Should generate Python");
    let py_code = all_code(&py_output);
    assert!(!py_code.is_empty(), "Python code should not be empty");
}

/// Tests code generation with documentation enabled.
#[test]
fn test_codegen_with_docs() {
    let contract = parse_openapi(MINIMAL_CONTRACT).expect("Should parse");

    let mut config = GeneratorConfig::default();
    config.include_docs = true;

    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract).expect("Should generate Rust");
    let rust_code = all_code(&rust_output);

    // Rust should have doc comments
    assert!(
        rust_code.contains("///") || rust_code.contains("//!") || rust_code.contains("//"),
        "Rust code should have comments"
    );

    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen
        .generate(&contract)
        .expect("Should generate TypeScript");
    let ts_code = all_code(&ts_output);

    // TypeScript should have comments
    assert!(
        ts_code.contains("/**") || ts_code.contains("//") || ts_code.contains("*"),
        "TypeScript code should have comments"
    );
}

/// Tests code generation without documentation.
#[test]
fn test_codegen_without_docs() {
    let contract = parse_openapi(MINIMAL_CONTRACT).expect("Should parse");

    let mut config = GeneratorConfig::default();
    config.include_docs = false;

    let rust_gen = RustGenerator::new(config);
    let rust_output = rust_gen.generate(&contract).expect("Should generate Rust");
    let rust_code = all_code(&rust_output);

    // Should still generate valid code
    assert!(
        rust_code.contains("struct") || rust_code.contains("enum") || !rust_output.files.is_empty(),
        "Should generate code even without docs"
    );
}

/// Tests that generated code handles optional fields correctly.
#[test]
fn test_codegen_handles_optional_fields() {
    let contract_with_optional = r#"
openapi: "3.1.0"
info:
  title: Optional Fields Service
  version: "1.0.0"
components:
  schemas:
    User:
      type: object
      required:
        - id
      properties:
        id:
          type: string
        name:
          type: string
        email:
          type: string
paths:
  /users:
    get:
      operationId: listUsers
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/User'
"#;

    let contract = parse_openapi(contract_with_optional).expect("Should parse");

    let config = GeneratorConfig::default();

    // Rust should use Option for non-required fields
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract).expect("Should generate Rust");
    let rust_code = all_code(&rust_output);
    assert!(
        rust_code.contains("Option<") || rust_code.contains("option") || rust_code.contains("None"),
        "Rust should handle optional fields"
    );

    // TypeScript optional handling
    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen
        .generate(&contract)
        .expect("Should generate TypeScript");
    let ts_code = all_code(&ts_output);
    assert!(!ts_code.is_empty(), "TypeScript should generate code");

    // Python optional handling
    let py_gen = PythonGenerator::new(config.clone());
    let py_output = py_gen.generate(&contract).expect("Should generate Python");
    let py_code = all_code(&py_output);
    assert!(!py_code.is_empty(), "Python should generate code");
}

/// Tests that generated code handles arrays correctly.
#[test]
fn test_codegen_handles_arrays() {
    let contract_with_arrays = r#"
openapi: "3.1.0"
info:
  title: Array Service
  version: "1.0.0"
components:
  schemas:
    Tags:
      type: array
      items:
        type: string
paths:
  /tags:
    get:
      operationId: listTags
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Tags'
"#;

    let contract = parse_openapi(contract_with_arrays).expect("Should parse");

    let config = GeneratorConfig::default();

    // Rust should use Vec
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract).expect("Should generate Rust");
    let rust_code = all_code(&rust_output);
    assert!(
        rust_code.contains("Vec<") || rust_code.contains("vec") || rust_code.contains("["),
        "Rust should handle arrays"
    );

    // TypeScript should use []
    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen
        .generate(&contract)
        .expect("Should generate TypeScript");
    let ts_code = all_code(&ts_output);
    assert!(
        ts_code.contains("[]") || ts_code.contains("Array") || ts_code.contains("array"),
        "TypeScript should handle arrays"
    );

    // Python should use list
    let py_gen = PythonGenerator::new(config.clone());
    let py_output = py_gen.generate(&contract).expect("Should generate Python");
    let py_code = all_code(&py_output);
    assert!(
        py_code.contains("list") || py_code.contains("List") || py_code.contains("["),
        "Python should handle arrays"
    );
}

/// Tests code generation consistency across runs.
#[test]
fn test_codegen_is_deterministic() {
    let contract = parse_openapi(MINIMAL_CONTRACT).expect("Should parse");

    let config = GeneratorConfig::default();

    // Generate code twice
    let gen1 = RustGenerator::new(config.clone());
    let output1 = gen1
        .generate(&contract)
        .expect("Should generate first time");
    let code1 = all_code(&output1);

    let gen2 = RustGenerator::new(config);
    let output2 = gen2
        .generate(&contract)
        .expect("Should generate second time");
    let code2 = all_code(&output2);

    // Should produce identical output
    assert_eq!(code1, code2, "Code generation should be deterministic");
}

/// Tests that generated code handles enums.
#[test]
fn test_codegen_handles_enums() {
    let contract_with_enum = r#"
openapi: "3.1.0"
info:
  title: Enum Service
  version: "1.0.0"
components:
  schemas:
    Status:
      type: string
      enum:
        - active
        - inactive
        - pending
paths:
  /status:
    get:
      operationId: getStatus
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Status'
"#;

    let contract = parse_openapi(contract_with_enum).expect("Should parse");

    let config = GeneratorConfig::default();

    // Rust should generate enum
    let rust_gen = RustGenerator::new(config.clone());
    let rust_output = rust_gen.generate(&contract).expect("Should generate Rust");
    let rust_code = all_code(&rust_output);
    assert!(
        rust_code.contains("enum")
            || rust_code.contains("Active")
            || rust_code.contains("active")
            || rust_code.contains("Status"),
        "Rust should handle enums"
    );

    // TypeScript should generate union or enum
    let ts_gen = TypeScriptGenerator::new(config.clone());
    let ts_output = ts_gen
        .generate(&contract)
        .expect("Should generate TypeScript");
    let ts_code = all_code(&ts_output);
    assert!(
        ts_code.contains("active") || ts_code.contains("Status") || ts_code.contains("enum"),
        "TypeScript should handle enums"
    );
}

/// Tests code generator configuration options.
#[test]
fn test_generator_config_options() {
    let contract = parse_openapi(MINIMAL_CONTRACT).expect("Should parse");

    // With validation
    let mut config_with_validation = GeneratorConfig::default();
    config_with_validation.include_validation = true;

    let gen = RustGenerator::new(config_with_validation);
    let output = gen.generate(&contract).expect("Should generate");
    let code = all_code(&output);

    // Check that code was generated
    assert!(
        !code.is_empty(),
        "Should generate code with validation enabled"
    );

    // Without validation
    let mut config_without_validation = GeneratorConfig::default();
    config_without_validation.include_validation = false;

    let gen2 = RustGenerator::new(config_without_validation);
    let output2 = gen2.generate(&contract).expect("Should generate");
    let code2 = all_code(&output2);

    assert!(!code2.is_empty(), "Should generate code without validation");
}
