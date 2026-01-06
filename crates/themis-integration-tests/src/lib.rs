//! # Themis Integration Tests
//!
//! End-to-end integration tests for the Themis contract governance toolchain.
//!
//! These tests verify that all components work together correctly:
//!
//! - Contract parsing (themis-openapi)
//! - Linting (themis-lint)
//! - Compatibility checking (themis-compat)
//! - Code generation (themis-codegen)
//! - Artifact creation (themis-artifact)
//!
//! ## Test Categories
//!
//! - `workflow_tests`: Full workflow from contract to artifact
//! - `validation_tests`: Contract validation across components
//! - `codegen_tests`: Code generation and compilation
//! - `artifact_tests`: Artifact creation and verification

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod artifact_tests;
pub mod codegen_tests;
pub mod validation_tests;
pub mod workflow_tests;

/// Test fixtures and helpers
pub mod fixtures {
    /// Sample OpenAPI contract for testing
    pub const USERS_SERVICE_V1: &str = include_str!("../../../examples/users-service/v1/openapi.yaml");
    
    /// Sample OpenAPI contract v2 for compatibility testing
    pub const USERS_SERVICE_V2: &str = include_str!("../../../examples/users-service/v2/openapi.yaml");
    
    /// Minimal valid OpenAPI contract
    pub const MINIMAL_CONTRACT: &str = r#"
openapi: "3.1.0"
info:
  title: Minimal Service
  version: "1.0.0"
paths:
  /health:
    get:
      operationId: getHealth
      summary: Health check
      responses:
        "200":
          description: OK
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
"#;

    /// Contract with security schemes
    pub const SECURE_CONTRACT: &str = r#"
openapi: "3.1.0"
info:
  title: Secure Service
  version: "1.0.0"
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      security:
        - bearerAuth: []
      responses:
        "200":
          description: Success
          content:
            application/json:
              schema:
                type: array
                items:
                  type: object
                  properties:
                    id:
                      type: string
        "401":
          description: Unauthorized
"#;

    /// Contract with breaking changes (for compat testing)
    pub const BREAKING_CHANGE_CONTRACT: &str = r#"
openapi: "3.1.0"
info:
  title: Minimal Service
  version: "2.0.0"
paths:
  /health:
    get:
      operationId: checkHealth
      summary: Health check (renamed)
      responses:
        "200":
          description: OK
          content:
            application/json:
              schema:
                type: object
                properties:
                  healthy:
                    type: boolean
"#;
}
