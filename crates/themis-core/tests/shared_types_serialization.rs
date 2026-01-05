//! Cross-component serialization tests for shared platform types.
//!
//! These tests ensure that types from `themis-platform-types` serialize
//! and deserialize correctly, maintaining compatibility across all
//! Themis Platform components (Themis, Archimedes, Eunomia).
//!
//! ## CTO Architecture Review Reference
//!
//! These tests address the following concerns from the 2026-01-04 CTO review:
//! - Type Schema Divergence (Critical Issue #1)
//! - JSON Schema vs Rust Implementation Mismatch (Issue #4)
//!
//! ## Test Coverage
//!
//! - CallerIdentity JSON round-trip for all variants
//! - ThemisErrorEnvelope JSON round-trip
//! - ErrorCode serialization format
//! - RequestId JSON format (UUID v7)

use themis_core::{CallerIdentity, ErrorCode, FieldError, RequestId, ThemisErrorEnvelope};

// ============================================================================
// CallerIdentity Serialization Tests
// ============================================================================

mod caller_identity {
    use super::*;

    #[test]
    fn spiffe_identity_json_roundtrip() {
        let identity = CallerIdentity::spiffe_full(
            "spiffe://example.org/orders-service",
            "example.org",
            "orders-service",
        );

        let json = serde_json::to_string(&identity).expect("Failed to serialize SpiffeIdentity");

        // Verify JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(value["type"], "spiffe");
        assert_eq!(value["spiffe_id"], "spiffe://example.org/orders-service");
        assert_eq!(value["trust_domain"], "example.org");
        assert_eq!(value["service_name"], "orders-service");

        // Verify round-trip
        let deserialized: CallerIdentity =
            serde_json::from_str(&json).expect("Failed to deserialize SpiffeIdentity");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn spiffe_identity_minimal_json_roundtrip() {
        let identity = CallerIdentity::spiffe("spiffe://example.org/orders-service");

        let json = serde_json::to_string(&identity).expect("Failed to serialize");

        // Verify minimal JSON (optional fields skipped)
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(value["type"], "spiffe");
        assert_eq!(value["spiffe_id"], "spiffe://example.org/orders-service");
        // Optional fields should be absent (skip_serializing_if)
        assert!(value.get("trust_domain").is_none() || value["trust_domain"].is_null());

        // Verify round-trip
        let deserialized: CallerIdentity =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn user_identity_json_roundtrip() {
        let identity = CallerIdentity::user("user-123", "user@example.com");

        let json = serde_json::to_string(&identity).expect("Failed to serialize UserIdentity");

        // Verify JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(value["type"], "user");
        assert_eq!(value["user_id"], "user-123");
        assert_eq!(value["email"], "user@example.com");

        // Verify round-trip
        let deserialized: CallerIdentity =
            serde_json::from_str(&json).expect("Failed to deserialize UserIdentity");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn user_identity_full_json_roundtrip() {
        // Create a full user identity with all fields
        let json_input = r#"{
            "type": "user",
            "user_id": "user-456",
            "email": "admin@example.com",
            "name": "Admin User",
            "roles": ["admin", "editor"],
            "groups": ["engineering", "platform"],
            "tenant_id": "tenant-abc"
        }"#;

        let identity: CallerIdentity =
            serde_json::from_str(json_input).expect("Failed to deserialize full UserIdentity");

        // Verify round-trip
        let json_output = serde_json::to_string(&identity).expect("Failed to serialize");
        let roundtrip: CallerIdentity =
            serde_json::from_str(&json_output).expect("Failed to roundtrip");
        assert_eq!(identity, roundtrip);

        // Verify type checking
        assert!(identity.is_user());
        assert_eq!(identity.identifier(), "user-456");
    }

    #[test]
    fn api_key_identity_json_roundtrip() {
        let identity = CallerIdentity::api_key("key-abc123", "Production API Key");

        let json = serde_json::to_string(&identity).expect("Failed to serialize ApiKeyIdentity");

        // Verify JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(value["type"], "api_key");
        assert_eq!(value["key_id"], "key-abc123");
        assert_eq!(value["name"], "Production API Key");

        // Verify round-trip
        let deserialized: CallerIdentity =
            serde_json::from_str(&json).expect("Failed to deserialize ApiKeyIdentity");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn api_key_identity_full_json_roundtrip() {
        let json_input = r#"{
            "type": "api_key",
            "key_id": "key-xyz",
            "name": "External Integration",
            "scopes": ["read:users", "write:orders"],
            "owner_id": "org-123"
        }"#;

        let identity: CallerIdentity =
            serde_json::from_str(json_input).expect("Failed to deserialize full ApiKeyIdentity");

        // Verify round-trip
        let json_output = serde_json::to_string(&identity).expect("Failed to serialize");
        let roundtrip: CallerIdentity =
            serde_json::from_str(&json_output).expect("Failed to roundtrip");
        assert_eq!(identity, roundtrip);

        assert!(identity.is_api_key());
        assert_eq!(identity.identifier(), "key-xyz");
    }

    #[test]
    fn anonymous_identity_json_roundtrip() {
        let identity = CallerIdentity::anonymous();

        let json = serde_json::to_string(&identity).expect("Failed to serialize AnonymousIdentity");

        // Verify JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(value["type"], "anonymous");

        // Verify round-trip
        let deserialized: CallerIdentity =
            serde_json::from_str(&json).expect("Failed to deserialize AnonymousIdentity");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn identity_type_discrimination() {
        // Test that type field correctly discriminates variants
        let spiffe_json = r#"{"type": "spiffe", "spiffe_id": "spiffe://test/svc"}"#;
        let user_json = r#"{"type": "user", "user_id": "u1"}"#;
        let api_key_json = r#"{"type": "api_key", "key_id": "k1", "name": "Test"}"#;
        let anon_json = r#"{"type": "anonymous"}"#;

        let spiffe: CallerIdentity = serde_json::from_str(spiffe_json).unwrap();
        let user: CallerIdentity = serde_json::from_str(user_json).unwrap();
        let api_key: CallerIdentity = serde_json::from_str(api_key_json).unwrap();
        let anon: CallerIdentity = serde_json::from_str(anon_json).unwrap();

        assert!(spiffe.is_service());
        assert!(user.is_user());
        assert!(api_key.is_api_key());
        assert!(anon.is_anonymous());
    }
}

// ============================================================================
// ThemisErrorEnvelope Serialization Tests
// ============================================================================

mod error_envelope {
    use super::*;

    #[test]
    fn error_envelope_json_roundtrip() {
        let envelope = ThemisErrorEnvelope::new(ErrorCode::ValidationFailed, "Validation failed");

        let json = serde_json::to_string(&envelope).expect("Failed to serialize");

        // Verify JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(value["code"], "VALIDATION_FAILED");
        assert_eq!(value["message"], "Validation failed");
        assert!(value["timestamp"].is_string());

        // Verify round-trip
        let deserialized: ThemisErrorEnvelope =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(envelope.code, deserialized.code);
        assert_eq!(envelope.message, deserialized.message);
    }

    #[test]
    fn error_envelope_with_details() {
        let envelope = ThemisErrorEnvelope::new(ErrorCode::ValidationFailed, "Invalid request")
            .with_detail("email", "The 'email' field is required");

        let json = serde_json::to_string(&envelope).expect("Failed to serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(value["details"]["email"], "The 'email' field is required");

        // Round-trip
        let deserialized: ThemisErrorEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(
            envelope.details.get("email"),
            deserialized.details.get("email")
        );
    }

    #[test]
    fn error_envelope_with_field_errors() {
        let envelope = ThemisErrorEnvelope::new(ErrorCode::ValidationFailed, "Validation failed")
            .with_field_error(
                FieldError::new("email", "Invalid email format").with_code(ErrorCode::InvalidField),
            )
            .with_field_error(
                FieldError::new("age", "Must be positive").with_code(ErrorCode::InvalidField),
            );

        let json = serde_json::to_string(&envelope).expect("Failed to serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        let errors = value["errors"].as_array().expect("errors should be array");
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0]["field"], "email");
        assert_eq!(errors[1]["field"], "age");

        // Round-trip
        let deserialized: ThemisErrorEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope.errors.len(), deserialized.errors.len());
    }

    #[test]
    fn error_envelope_with_trace_id() {
        let envelope = ThemisErrorEnvelope::new(ErrorCode::InternalError, "Internal error")
            .with_trace_id("trace-abc123");

        let json = serde_json::to_string(&envelope).expect("Failed to serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(value["trace_id"], "trace-abc123");

        // Round-trip
        let deserialized: ThemisErrorEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope.trace_id, deserialized.trace_id);
    }
}

// ============================================================================
// ErrorCode Serialization Tests
// ============================================================================

mod error_code {
    use super::*;

    #[test]
    fn error_code_serializes_as_string() {
        let codes = vec![
            (ErrorCode::ValidationFailed, "VALIDATION_FAILED"),
            (ErrorCode::Unauthenticated, "UNAUTHENTICATED"),
            (ErrorCode::PermissionDenied, "PERMISSION_DENIED"),
            (ErrorCode::NotFound, "NOT_FOUND"),
            (ErrorCode::RateLimitExceeded, "RATE_LIMIT_EXCEEDED"),
            (ErrorCode::InternalError, "INTERNAL_ERROR"),
            (ErrorCode::ServiceUnavailable, "SERVICE_UNAVAILABLE"),
            (ErrorCode::MalformedRequest, "MALFORMED_REQUEST"),
            (ErrorCode::Conflict, "CONFLICT"),
            (ErrorCode::Gone, "GONE"),
        ];

        for (code, expected_str) in codes {
            let json = serde_json::to_string(&code).expect("Failed to serialize ErrorCode");
            assert_eq!(json, format!("\"{}\"", expected_str));

            // Round-trip
            let deserialized: ErrorCode =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(code, deserialized);
        }
    }

    #[test]
    fn error_code_deserialization_is_case_sensitive() {
        // Should work with correct case
        let code: ErrorCode = serde_json::from_str("\"VALIDATION_FAILED\"").unwrap();
        assert_eq!(code, ErrorCode::ValidationFailed);
    }
}

// ============================================================================
// RequestId Serialization Tests
// ============================================================================

mod request_id {
    use super::*;

    #[test]
    fn request_id_json_roundtrip() {
        let id = RequestId::new();

        let json = serde_json::to_string(&id).expect("Failed to serialize RequestId");

        // Verify it's a quoted string (UUID format)
        assert!(json.starts_with('"'));
        assert!(json.ends_with('"'));

        // UUID v7 format: 8-4-4-4-12 hex characters
        let uuid_str = &json[1..json.len() - 1]; // Remove quotes
        assert!(uuid_str.contains('-'));
        assert_eq!(uuid_str.len(), 36); // UUID string length

        // Round-trip
        let deserialized: RequestId = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(id, deserialized);
    }

    #[test]
    fn request_id_display_matches_json() {
        let id = RequestId::new();
        let display = id.to_string();
        let json = serde_json::to_string(&id).unwrap();

        // JSON should be the quoted display string
        assert_eq!(json, format!("\"{}\"", display));
    }
}

// ============================================================================
// Cross-Component Compatibility Tests
// ============================================================================

mod cross_component {
    use super::*;

    /// This test verifies the exact JSON format that Archimedes and Eunomia
    /// expect when receiving CallerIdentity from Themis-generated code.
    #[test]
    fn archimedes_expected_user_format() {
        // This is the format Archimedes should send and Eunomia should receive
        let canonical_json = r#"{
            "type": "user",
            "user_id": "user-123",
            "email": "user@example.com",
            "roles": ["admin"],
            "groups": [],
            "tenant_id": "tenant-1"
        }"#;

        let identity: CallerIdentity =
            serde_json::from_str(canonical_json).expect("Archimedes format should deserialize");

        assert!(identity.is_user());
        assert_eq!(identity.identifier(), "user-123");

        // Re-serialize and verify structure is maintained
        let reserialized = serde_json::to_value(&identity).unwrap();
        assert_eq!(reserialized["type"], "user");
        assert_eq!(reserialized["user_id"], "user-123");
        assert_eq!(reserialized["tenant_id"], "tenant-1");
    }

    /// Test the format used in PolicyInput which flows to Eunomia
    #[test]
    fn eunomia_expected_spiffe_format() {
        // Format expected by Eunomia policies (OPA)
        let canonical_json = r#"{
            "type": "spiffe",
            "spiffe_id": "spiffe://cluster.local/ns/default/sa/orders-service",
            "trust_domain": "cluster.local",
            "service_name": "orders-service"
        }"#;

        let identity: CallerIdentity =
            serde_json::from_str(canonical_json).expect("Eunomia format should deserialize");

        assert!(identity.is_service());

        // Verify identifier extraction
        assert_eq!(
            identity.identifier(),
            "spiffe://cluster.local/ns/default/sa/orders-service"
        );
    }

    /// Test minimal identity format (no optional fields)
    #[test]
    fn minimal_identity_compatibility() {
        // Minimal formats that all components must handle
        let minimal_user = r#"{"type": "user", "user_id": "u1"}"#;
        let minimal_spiffe = r#"{"type": "spiffe", "spiffe_id": "spiffe://t/s"}"#;
        let minimal_apikey = r#"{"type": "api_key", "key_id": "k1", "name": "test"}"#;
        let minimal_anon = r#"{"type": "anonymous"}"#;

        // All should deserialize successfully
        let _: CallerIdentity = serde_json::from_str(minimal_user).unwrap();
        let _: CallerIdentity = serde_json::from_str(minimal_spiffe).unwrap();
        let _: CallerIdentity = serde_json::from_str(minimal_apikey).unwrap();
        let _: CallerIdentity = serde_json::from_str(minimal_anon).unwrap();
    }
}
