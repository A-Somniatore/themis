# Themis – Contracts & Schema Governance Specification (V1)

## Purpose

Themis is the contract and schema governance layer of the platform. It defines the authoritative shape, behavior, compatibility rules, and guarantees of all APIs. Contracts governed by Themis are the law: implementations must conform exactly, and all enforcement flows downstream from these definitions.

This document is developer-ready and intended to be used directly to implement Themis tooling, CI checks, and runtime validation.

---

## 1. Responsibilities

Themis is responsible for:

- Defining API contracts (HTTP and gRPC)
- Enforcing contract-first development
- Governing versioning and backward compatibility
- Declaring authentication requirements
- Declaring rate-limit and timeout intent
- Providing canonical error definitions
- Powering validation, code generation, and documentation

Themis is explicitly **not** responsible for:

- Runtime authorization decisions (Eunomia)
- Traffic routing or exposure (Kratos / ingress)
- Business logic implementation

---

## 2. Supported Contract Formats

### 2.1 HTTP APIs

- OpenAPI **3.1** is mandatory
- JSON Schema dialect must be OpenAPI 3.1 compliant

### 2.2 gRPC APIs

- Protobuf v3
- gRPC service definitions

### 2.3 GraphQL APIs

- GraphQL SDL (Schema Definition Language)
- Schema-first approach

### 2.4 Event-Driven APIs

- AsyncAPI 3.0
- Supports Kafka, AMQP, WebSocket channels

Both formats are first-class citizens and must follow the same governance rules.

---

## 3. Contract-First Development Model

- Contracts must exist **before** implementation
- Implementations must conform to published contracts
- Runtime behavior is validated against contracts
- CI blocks publishing and deployment if conformance fails

No code-first or schema-inferred contracts are allowed for Themis-native services.

---

## 4. Contract Repository Structure

Contracts are stored in Git as the source of truth.

Directory layout:

```
<service-name>/
  v1/
    openapi.yaml | service.proto
  v2/
    openapi.yaml | service.proto
```

Rules:

- Each directory represents a **major version**
- Only one contract file per major version
- Minor and patch changes are tracked via Git history and artifact metadata

---

## 5. Versioning & Compatibility

### 5.1 Semantic Versioning

- MAJOR: breaking changes
- MINOR: backward-compatible additions
- PATCH: backward-compatible fixes

### 5.2 Compatibility Rules

Within a major version:

- Removing fields is forbidden
- Changing field types is forbidden
- Making optional fields required is forbidden
- Changing semantics of existing behavior is forbidden

Allowed within a major version:

- Adding optional fields
- Adding new endpoints
- Adding new error variants

### 5.3 Breaking Changes

Breaking changes are only allowed by:

- Creating a new major version directory
- Explicitly publishing a new major artifact

CI must fail any breaking diff detected within the same major version.

---

## 6. Authentication Declaration

- Authentication requirements are declared **only** in contracts
- Themis defines _what_ authentication is required, not _how_ it is enforced

### 6.1 HTTP (OpenAPI)

- Use OpenAPI `securitySchemes`
- Operations may declare:
  - No auth
  - Required auth scheme(s)

### 6.2 gRPC (Protobuf)

- Auth requirements declared via standard or custom annotations

Themis runtime validation ensures that required identity context is present.

---

## 7. Rate Limits & Timeouts

### 7.1 Intent Declaration

Contracts may declare intent metadata such as:

- Latency sensitivity
- Expected request frequency
- Critical vs non-critical operations

### 7.2 Enforcement

- Contracts declare intent only
- Concrete values are supplied via runtime configuration
- Stoa displays effective values per environment

---

## 8. Error Model

### 8.1 Standard Error Envelope

All errors must conform to a single, canonical error schema defined by Themis.

Characteristics:

- Machine-readable error code
- Human-readable message
- Optional structured details
- Stable across versions

### 8.2 Error Declaration

- All possible error variants must be declared in the contract
- HTTP status codes and gRPC statuses must be specified

Undeclared errors are forbidden.

---

## 9. Validation & Enforcement

### 9.1 CI Enforcement

CI must perform:

- Schema linting
- Backward compatibility diffing
- Error envelope validation
- Auth declaration validation

Failures block:

- Merge
- Artifact publishing
- Deployment

### 9.2 Runtime Validation

- Requests and responses are validated against contracts
- Validation failures produce standard Themis errors
- Legacy services may run in monitor-only mode

---

## 10. Artifact Publishing

- Git is the source of truth
- CI produces immutable contract artifacts
- Artifacts are versioned and content-addressed

Consumers:

- Archimedes
- Eunomia (for policy context)
- Stoa
- Client code generators

---

## 11. Code Generation

Contracts drive generation of:

- Strongly typed request/response models
- Error types
- Client SDKs
- Server handler interfaces

Generated code must never be edited manually.

---

## 12. Observability Integration

Contracts provide metadata used by observability systems:

- Operation identifiers
- Error taxonomy
- Latency intent

This metadata is consumed by Archimedes and visualized in Stoa.

---

## 13. Testing Strategy

### 13.1 Contract Tests

- Schema validity
- Example payload validation
- Error envelope validation

### 13.2 Compatibility Tests

- Diff-based breaking change detection
- Major-version enforcement

### 13.3 Conformance Tests

- Runtime request/response validation
- Error behavior verification

---

## 14. Non-Goals (V1)

- Runtime authorization logic
- Policy definition
- UI editing of contracts
- Schema inference from code
