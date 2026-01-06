# Themis – Development Roadmap

> **Version**: 1.13.0  
> **Created**: 2026-01-04  
> **Last Updated**: 2026-01-06  
> **Target Completion**: Week 14 (MVP)
> **Current Progress**: Phase T5 In Progress (Week 13) - Integration Test Suite Complete

---

## ✅ CTO Architecture Review Action Items (Resolved)

> **Source**: [2026-01-04 CTO Architecture Review](../../docs/reviews/2026-01-04-cto-architecture-review.md) > **Status**: ✅ RESOLVED - `themis-platform-types` v0.2.0 addresses all issues

### Resolved Issues (2026-01-05)

1. **JSON Schema vs Rust Implementation Mismatch** - ✅ FIXED

   - [x] Aligned `caller-identity.schema.json` with Rust `identity.rs`
   - [x] Added JSON round-trip tests to themis-platform-types
   - [x] Added schema validation CI script

2. **API Safety Improvements** - ✅ COMPLETE

   - [x] `BuilderError` type replaces `&'static str` errors
   - [x] `build()` deprecated, `try_build()` is the new standard
   - [x] `#[non_exhaustive]` on public enums for future compatibility
   - [x] SemVer pre-release comparison fixed per spec

3. **Schema Evolution** - ✅ COMPLETE
   - [x] Added `schema` module with `Versioned<T>` wrapper
   - [x] Added `CURRENT_SCHEMA_VERSION` constant
   - [x] Documented migration strategy

### Sign-Off Blockers (All Components)

```markdown
✅ All Cargo.toml files reference themis-platform-types
✅ Zero local CallerIdentity/PolicyInput definitions in Archimedes/Eunomia
✅ cargo test passes across all workspaces
✅ JSON round-trip tests pass in each component
□ Control plane crate skeleton exists in Eunomia (Week 13)
```

---

## 🔄 themis-platform-types v0.2.1 Production Readiness (Coming Soon)

> **When**: Before production release
> **Status**: Development Complete - Pending Publish

### New Production Guarantees (v0.2.1)

1. **Thread Safety** - Compile-time `Send + Sync` assertions for all public types
2. **MSRV Testing** - CI validates Rust 1.75 compatibility
3. **Schema Validation** - CI validates JSON schemas match Rust types
4. **Serialization Testing** - Proptest roundtrip tests for all types
5. **Fallible Constructor** - `RequestId::try_new()` for sandboxed environments
6. **Security Lint** - `#[must_use = "security bug"]` on `PolicyDecision::allow/deny`

### Upgrade Path (v0.2.0 → v0.2.1)

- No breaking changes
- Update `Cargo.toml` version when published
- Optionally use `RequestId::try_new()` for extra safety

---

## 🔄 themis-platform-types v0.2.0 Migration (Required)

> **When**: Before next release
> **Breaking Changes**: Yes, see below

### Migration Checklist

- [ ] Update `Cargo.toml` to `themis-platform-types = "0.2.0"`
- [ ] Replace `build()` calls with `try_build().unwrap()` or `try_build()?`
- [ ] Update error handling to use `BuilderError` instead of `&'static str`
- [ ] Add wildcard arms to match statements on `CallerIdentity`, `ErrorCode` (now `#[non_exhaustive]`)
- [ ] Use new re-exports: `SpiffeIdentity`, `UserIdentity`, `ApiKeyIdentity`, `BuilderError`

### Code Changes Required

```rust
// Before (v0.1.0)
let input = PolicyInput::builder()
    .caller(caller)
    .service("my-service")
    .try_build()?; // Returns Result<_, &'static str>

// After (v0.2.0)
use themis_platform_types::BuilderError;
let input = PolicyInput::builder()
    .caller(caller)
    .service("my-service")
    .try_build()?; // Returns Result<_, BuilderError>

// Match statements need wildcard
match error_code {
    ErrorCode::NotFound => ...,
    ErrorCode::InternalError => ...,
    _ => ... // Required for #[non_exhaustive]
}
```

---

## Key Decisions

| Decision                                                                     | Impact                                              |
| ---------------------------------------------------------------------------- | --------------------------------------------------- |
| [ADR-008](../../docs/decisions/008-archimedes-full-framework.md)             | Archimedes is full framework replacement (40 weeks) |
| [ADR-006](../../docs/decisions/006-grpc-post-mvp.md)                         | MVP is OpenAPI 3.x only, gRPC post-MVP              |
| [ADR-005](../../docs/decisions/005-kubernetes-ingress-over-custom-router.md) | No custom router, use K8s Ingress                   |
| [ADR-007](../../docs/decisions/007-apache-2-license.md)                      | Apache 2.0 license                                  |

**MVP Contract Support:**

- ✅ OpenAPI 3.0 / 3.1 (REST APIs)
- ❌ Protobuf/gRPC (post-MVP)
- ❌ GraphQL (post-MVP)
- ❌ AsyncAPI (post-MVP)

**Contract Registry:** OCI-compatible registry (see [Infrastructure Decisions](../../docs/architecture/infrastructure-decisions.md))

---

## Overview

Themis is the contract governance toolchain for the Themis Platform. Development runs **in parallel with Eunomia** during the first 12 weeks, with an additional 2 weeks for integration.

### Timeline Summary

| Phase                       | Duration | Weeks | Description                                |
| --------------------------- | -------- | ----- | ------------------------------------------ |
| T0: Shared Types            | 1 week   | 1     | Create `themis-platform-types` crate       |
| T1: Foundation              | 3 weeks  | 2-4   | Core types, OpenAPI parsing, validation    |
| T2: Linting & Compatibility | 2 weeks  | 5-6   | Linting rules, breaking change detection   |
| T3: Code Generation         | 4 weeks  | 7-10  | Rust, TypeScript, Python code generation   |
| T4: Publishing & Registry   | 2 weeks  | 11-12 | Artifact publishing, registry client       |
| T5: Integration Testing     | 2 weeks  | 13-14 | End-to-end testing with Archimedes/Eunomia |

**Total**: 14 weeks

### Cross-Component Timeline Alignment

**MVP Timeline (Weeks 1-20):**

```
         Week: 1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20
 Themis:      [T0][---T1---][--T2--][------T3------][--T4--][--T5--]
 Eunomia:     [E0][---E1---][------E2------][------E3------]        (gap)        [------E4------]
 Archimedes:  [A0][---A1---][----------A2----------][----------A3----------][A4][------A5------]
```

**Full Framework Timeline (Weeks 21-40):**

```
         Week: 21  22  23  24  25  26  27  28  29  30  31  32  33  34  35  36  37  38  39  40
 Archimedes:  [------A6------][------A7------][------A8------][------A9------][-----A10------]
                  Router         FastAPI        WebSocket         CLI        Multi-Lang
                Extractors       Parity          SSE/Tasks       DevExp       SDKs
```

> **Note**: Archimedes is evolving from a governance layer to a **full framework replacement** for Axum, FastAPI, and Boost.Beast. See [ADR-008](../../docs/decisions/008-archimedes-full-framework.md).

**Key Coordination Points**:

- Week 1: Themis creates `themis-platform-types` crate (all components depend on this)
- Week 12: Themis artifacts available for Archimedes integration (T4 complete)
- Week 14: Themis MVP complete, supporting Archimedes A5 integration
- Weeks 17-20: Platform MVP integration (Themis team supports Archimedes/Eunomia)
- Weeks 37-40: Themis codegen updated to target Archimedes multi-language SDKs

---

## Phase T0: Shared Platform Types (Week 1) ⭐ NEW

> **Purpose**: Create a shared crate that Themis, Archimedes, and Eunomia all depend on.
> This ensures schema compatibility at compile time and eliminates integration drift.

### Week 1: Create `themis-platform-types`

- [x] Create new repository `themis-platform-types` (or workspace member)
  > ✅ **Completed 2026-01-04**: Created `themis-platform-types` crate at platform root
- [x] Move/define shared types:
  - [x] `CallerIdentity` enum (Spiffe, User, ApiKey, Anonymous)
  - [x] `PolicyInput` struct (caller, service, operation_id, method, path, headers, timestamp, environment, context)
  - [x] `PolicyDecision` struct (allowed, reason, policy_id, policy_version, evaluation_time_ns)
  - [x] `ThemisErrorEnvelope` struct (code, message, details, request_id, timestamp, trace_id)
  - [x] `RequestId` type (UUID v7)
  - [x] `SemanticVersion` type
  - [x] Standard error codes enum
    > ✅ **Completed 2026-01-04**: All core types implemented in `src/identity.rs`, `src/policy.rs`, `src/error.rs`, `src/request.rs`, `src/version.rs`. ErrorCode enum in error.rs.
- [x] Add comprehensive documentation
  > ✅ **Completed 2026-01-04**: Rustdoc with examples in lib.rs and README.md
- [ ] Publish to crates.io (or private registry)
- [x] Create JSON Schema definitions for each type
  > ✅ **Completed 2026-01-04**: JSON Schemas in `schemas/` directory for CallerIdentity, PolicyInput, PolicyDecision, ThemisErrorEnvelope
- [x] Add GitHub Actions CI workflow
  > ✅ **Completed 2026-01-04**: CI workflow with check, test, fmt, clippy, docs, schema feature testing
- [x] Document in [integration-spec.md](../../docs/integration/integration-spec.md)
  > ✅ **Completed 2026-01-04**: Integration spec has authoritative type definitions

### Shared Types Location Strategy

```
themis-platform/
├── themis-platform-types/     # NEW: Shared types crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── identity.rs        # CallerIdentity
│       ├── policy.rs          # PolicyInput, PolicyDecision
│       ├── error.rs           # ThemisErrorEnvelope, error codes
│       ├── request.rs         # RequestId
│       └── version.rs         # SemanticVersion
├── themis/                    # Depends on themis-platform-types
├── archimedes/                # Depends on themis-platform-types
└── eunomia/                   # Depends on themis-platform-types
```

### Phase T0 Milestone

**Criteria**: Shared types crate published, all three components can depend on it

---

## Phase T1: Foundation (Weeks 2-4)

### Week 2: Project Setup & Core Types

- [x] Create `themis` repository structure
  > **Completed 2026-01-04**: Created full workspace structure with all planned crates
- [x] Set up Cargo workspace:
  ```
  crates/
  ├── themis-core/      # Core types
  ├── themis-openapi/   # OpenAPI parser
  ├── themis-lint/      # Linting rules
  └── themis-cli/       # CLI application
  ```
  > **Completed 2026-01-04**: Workspace configured with shared dependencies, lints, and profiles
- [x] Configure CI pipeline (GitHub Actions)
  > **Completed 2026-01-04**: Created ci.yml (check, test, fmt, clippy, docs, audit) and release.yml (multi-platform builds, GitHub releases, crates.io publishing)
- [x] Define core data models
  > **Completed 2026-01-04**: Implemented Contract, Operation, Schema, Version, and Error types in themis-core with full test coverage
- [x] Write initial documentation
  > **Completed 2026-01-04**: Created development.md guide, updated README with Quick Start, added users-service example contract
- [x] Integrate `themis-platform-types` dependency
  > ✅ **Completed 2026-01-04**: Added as workspace dependency, re-exported shared types in themis-core
- [x] Refactor error types to use `ThemisErrorEnvelope` from shared crate
  > ✅ **Completed 2026-01-04**: Re-export ErrorCode, FieldError, ThemisErrorEnvelope from themis-platform-types

### Week 3: OpenAPI Parser

- [x] Implement OpenAPI 3.1 parser
  > **Completed 2026-01-04**: Full parser in themis-openapi using openapiv3 crate. Parses YAML/JSON, converts to themis-core Contract type.
- [x] Handle `$ref` resolution (internal refs)
  > **Completed 2026-01-04**: RefSchema support for $ref pointers, preserves reference strings for later resolution
- [x] Extract operations with operationId
  > **Completed 2026-01-04**: Extracts all operations from paths, requires operationId (returns error if missing)
- [x] Extract request/response schemas
  > **Completed 2026-01-04**: Full schema conversion including string, number, integer, boolean, array, object, enum, oneOf, allOf, anyOf
- [x] Parse security schemes
  > **Completed 2026-01-04**: Supports HTTP, ApiKey, OAuth2, OpenIdConnect security scheme types
- [x] Test with sample OpenAPI specs
  > **Completed 2026-01-04**: 8 unit tests covering parsing, schema conversion, security schemes, extensions
- [x] Support Themis extensions (x-themis-\*)
  > **Completed 2026-01-04**: Extracts x-themis-rate-limit-tier, x-themis-timeout-tier, x-themis-idempotent

### Week 4: Contract Validation

- [x] Implement schema validation
  > **Completed 2026-01-05**: Response schema validation (THEMIS009), checks 2xx responses have content
- [x] Check required operationId on all operations
  > **Completed 2026-01-04**: Parser rejects specs without operationId (ThemisError::MissingField)
- [x] Validate error response declarations
  > **Completed 2026-01-05**: Warns for missing 4xx/5xx responses (THEMIS004), 401 for secured ops
- [x] Check security scheme declarations
  > **Completed 2026-01-05**: Errors for undefined schemes (THEMIS003), warns for no security (THEMIS008)
- [x] Implement custom rule framework
  > **Completed 2026-01-05**: Rule codes THEMIS001-THEMIS009, ValidationResult with errors/warnings
- [x] Add `themis validate` CLI command
  > **Completed 2026-01-05**: Supports text/JSON output, --warnings-as-errors flag

### Phase T1 Milestone

**Criteria**: OpenAPI specs can be parsed, validated, and loaded

> **✅ COMPLETE 2026-01-05**: All Week 4 validation features implemented. 50 tests passing.

---

## Phase T2: Linting & Compatibility (Weeks 5-6)

### Week 5: Linting Rules ✅

- [x] Define lint rule trait and configuration
- [x] Implement naming convention checks:
  - [x] `naming/operation-id`: operationId should be camelCase
  - [x] `naming/path-format`: paths should be kebab-case
  - [x] `naming/schema-name`: schema names should be PascalCase
- [x] Implement documentation checks:
  - [x] `docs/operation-summary`: operations should have summaries
  - [x] `docs/operation-description`: operations should have descriptions
  - [x] `docs/schema-description`: schemas should have descriptions
- [x] Add `.themis-lint.yaml` configuration file support
  > **Completed 2026-01-06**: LintConfigFile with extends (default/strict/relaxed), per-rule overrides, auto-detection
- [x] Add `themis lint` CLI command
- [x] Add lint rule tests (32 tests → 50 tests with config tests)

### Week 6: Breaking Change Detection ✅

- [x] Implement schema comparison (diffing)
  > **Completed 2026-01-05**: Created themis-compat crate with diff_contracts() function
- [x] Detect added/removed fields
  > **Completed 2026-01-05**: Detects required/optional field additions and removals
- [x] Detect type changes
  > **Completed 2026-01-05**: Detects field type changes (BREAK006)
- [x] Define breaking change rules
  > **Completed 2026-01-05**: BREAK001-010, ADD001-006, MOD001-004 documented in design.md
- [x] Implement compatibility analyzer
  > **Completed 2026-01-05**: CompatibilityChecker with configurable validation
- [x] Add semver validation
  > **Completed 2026-01-05**: Validates version bump matches detected changes
- [x] Generate compatibility report
  > **Completed 2026-01-05**: CompatibilityReport with JSON/text output, suggested_bump
- [x] Add `themis diff` CLI command
  > **Completed 2026-01-05**: Supports --format text/json, --fail-on-breaking, --validate-version
- [x] Add themis-compat tests (35 tests)

### Phase T2 Milestone

**Criteria**: Breaking changes are detected, contracts can be linted

> **✅ COMPLETE 2026-01-05**: All Week 5 & 6 features implemented. 116 tests total.

---

## Phase T3: Code Generation (Weeks 7-10)

### Week 7: Rust Types Generation ✅

- [x] Create `themis-codegen` crate
  > **Completed 2026-01-06**: Created crate with CodeGenerator trait, GeneratorConfig, error types
- [x] Generate Rust structs from schemas
  > **Completed 2026-01-06**: RustTypeGenerator with generate_struct, handles properties, required fields
- [x] Handle nested types
  > **Completed 2026-01-06**: Supports oneOf, allOf, anyOf composition schemas
- [x] Add serde derives
  > **Completed 2026-01-06**: Adds Serialize, Deserialize, rename_all, skip_serializing_if
- [x] Generate validation derives
  > **Completed 2026-01-06**: Maps OpenAPI constraints to validator crate attributes (length, range, email, url, regex)
- [x] Test generated code compiles
  > **Completed 2026-01-06**: 30 tests covering all type generation and validation scenarios

### Week 8: Rust Handlers & CLI ✅

- [x] Generate handler trait definitions
  > **Completed 2026-01-06**: Handler traits with async fn handle(), Send + Sync + 'static bounds
- [x] Generate request/response types per operation
  > **Completed 2026-01-06**: Typed request structs with path/query params, response enums with status variants
- [x] Generate error enum
  > **Completed 2026-01-06**: ServiceError with BadRequest, Unauthorized, NotFound, etc. + status_code() method
- [x] Add RequestContext parameter
  > **Completed 2026-01-06**: RequestContext placeholder for Archimedes integration (request_id, user_id, headers)
- [x] Add `themis codegen --language rust` CLI command
  > **Completed 2026-01-06**: CLI with --output, --include-docs, --include-validation, --force, --dry-run options
- [x] Add handler generation tests (12 tests)
  > **Completed 2026-01-06**: Tests for RequestContext, error types, request/response types, service struct

### Week 9: TypeScript Generation

- [x] Generate TypeScript interfaces
  > **Completed 2026-01-06**: TypeScriptTypeGenerator produces interfaces from OpenAPI schemas
- [x] Generate fetch client
  > **Completed 2026-01-06**: Fetch client with Result pattern, typed requests/responses
- [x] Generate Express/Fastify handler types
  > **Completed 2026-01-06**: Express router factory with RequestContext support
- [x] Add JSDoc comments
  > **Completed 2026-01-06**: JSDoc generated when include_docs is enabled
- [ ] Create npm package output (optional enhancement)
- [x] Add `themis codegen --language typescript` CLI command
  > **Completed 2026-01-06**: CLI supports --language typescript

### Week 10: Python Generation

- [x] Generate Python dataclasses
  > **Completed 2026-01-06**: PythonTypeGenerator produces @dataclass from OpenAPI schemas
- [x] Generate httpx client
  > **Completed 2026-01-06**: httpx client with Result pattern, context manager support
- [x] Generate FastAPI handler signatures
  > **Completed 2026-01-06**: FastAPI router factory with RequestContext and Protocol-based handlers
- [x] Add type hints
  > **Completed 2026-01-06**: Full type hints including Optional, Union, list[], dict[]
- [ ] Create package output (optional enhancement)
- [x] Add `themis codegen --language python` CLI command
  > **Completed 2026-01-06**: CLI supports --language python

### Phase T3 Milestone ✅

**Criteria**: Rust, TypeScript, and Python code can be generated from contracts
**Status**: COMPLETE - All three language generators implemented with full test coverage

---

## Phase T4: Publishing & Registry (Weeks 11-12)

### Week 11: Artifact Publishing

- [x] Create `themis-artifact` crate
  > **Completed 2026-01-06**: Created crate with Artifact, ArtifactBuilder, ArtifactOperation types
- [x] Design artifact format
  > **Completed 2026-01-06**: JSON format with schema, version, service, checksum, operations, schemas, raw_contract
- [x] Implement artifact builder
  > **Completed 2026-01-06**: Fluent ArtifactBuilder with from_contract() helper
- [x] Add content-addressable storage (checksum)
  > **Completed 2026-01-06**: SHA256 checksum over deterministic JSON representation
- [x] Implement checksum verification
  > **Completed 2026-01-06**: verify_checksum() method with ArtifactError on mismatch
- [x] Add artifact versioning
  > **Completed 2026-01-06**: Artifacts include $schema URL and format version
- [x] Add `themis pack` CLI command
  > **Completed 2026-01-06**: Pack and inspect commands for creating and viewing artifacts
- [x] Add 23 artifact tests
  > **Completed 2026-01-06**: Full test coverage for artifact, builder, operation, and error types

### Week 12: Registry Client

- [x] Create `themis-registry` crate
  > **Completed 2026-01-06**: Created crate with OCI registry support
- [x] Implement OCI registry client
  > **Completed 2026-01-06**: RegistryClient with publish, fetch, exists, list_versions, delete
- [x] Add `themis publish` command
  > **Completed 2026-01-06**: Publish command with verification, skip-existing, token auth
- [x] Add `themis fetch` command
  > **Completed 2026-01-06**: Fetch command with caching, latest version support
- [x] Implement caching
  > **Completed 2026-01-06**: ArtifactCache with disk-based caching by namespace/service/version
- [x] Test registry operations
  > **Completed 2026-01-06**: 54 tests for client, cache, config, reference, OCI types

### Phase T4 Milestone ✅

**Criteria**: Artifacts can be published to and fetched from registry
**Status**: COMPLETE - All registry commands implemented

---

## Phase T5: Integration Testing (Weeks 13-14) ⭐ IN PROGRESS

> **Purpose**: Validate that Themis artifacts work correctly with Archimedes and Eunomia.

### Week 13: Integration Test Suite ✅

- [x] Create `themis-integration-tests` crate
  > **Completed 2026-01-06**: Created crate with 35 integration tests
- [x] Implement workflow tests
  > **Completed 2026-01-06**: Full workflow, version comparison, breaking change detection
- [x] Implement validation tests
  > **Completed 2026-01-06**: Contract validation, lint configs (default/strict/relaxed)
- [x] Implement codegen tests
  > **Completed 2026-01-06**: Rust, TypeScript, Python generation tests
- [x] Implement artifact tests
  > **Completed 2026-01-06**: Artifact creation, round-trip, checksums

### Week 14: Archimedes Integration (Pending)

- [ ] Test artifact loading in Archimedes
- [ ] Verify operation → handler mapping
- [ ] Test request validation with generated schemas
- [ ] Test response validation
- [ ] Verify error envelope format matches shared types
- [ ] Document integration patterns

### Week 14: End-to-End Testing

- [ ] Test full workflow: contract → artifact → Archimedes validation
- [ ] Test policy context includes correct operation metadata
- [ ] Verify `PolicyInput.operation_id` matches Themis operationId
- [ ] Performance testing with large contracts
- [ ] Write integration documentation

### Phase T5 Milestone

**Criteria**: Themis artifacts work seamlessly with Archimedes runtime
**Current Status**: Integration test suite complete (35 tests), Archimedes integration pending

---

## Milestones Summary

| Milestone         | Target  | Criteria                                |
| ----------------- | ------- | --------------------------------------- |
| T0: Shared Types  | Week 1  | Platform types crate published          |
| T1: Parsing       | Week 4  | OpenAPI specs parsed correctly          |
| T2: Compatibility | Week 6  | Breaking changes detected               |
| T3: Code Gen      | Week 10 | Rust, TypeScript, Python code generated |
| T4: Publishing    | Week 12 | Artifacts published to registry         |
| T5: Integration   | Week 14 | End-to-end testing with Archimedes      |

---

## Deliverables

### CLI Commands

- `themis validate` - Validate contract syntax and schema
- `themis lint` - Run linting rules
- `themis diff` - Compare two contract versions
- `themis codegen` - Generate code (Rust, TypeScript, Python)
- `themis publish` - Publish artifact to registry
- `themis fetch` - Fetch artifact from registry

### Crates

- `themis-platform-types` - **Shared types** (CallerIdentity, PolicyInput, ThemisErrorEnvelope)
- `themis-core` - Core types and traits (depends on `themis-platform-types`)
- `themis-openapi` - OpenAPI 3.1 parser
- `themis-protobuf` - Protobuf parser (future)
- `themis-graphql` - GraphQL parser (future)
- `themis-asyncapi` - AsyncAPI parser (future)
- `themis-lint` - Linting rules
- `themis-compat` - Compatibility checking
- `themis-codegen` - Code generation
- `themis-artifact` - Artifact creation
- `themis-registry` - Registry client

---

## Dependencies on Other Components

| Dependency              | Required For              | Available   |
| ----------------------- | ------------------------- | ----------- |
| None                    | Core development (T1-T4)  | Immediately |
| `themis-platform-types` | Shared types (T0)         | Week 1      |
| Archimedes              | Integration testing (T5)  | Week 13+    |
| Eunomia                 | Policy context validation | Week 13+    |

---

## Integration Contracts

> **See**: [integration-spec.md](../../docs/integration/integration-spec.md) for authoritative schema definitions.

Themis MUST produce artifacts that:

1. Use `ThemisErrorEnvelope` from `themis-platform-types` for all error responses
2. Include `operationId` for every operation (used by Eunomia policies)
3. Follow the artifact format defined in integration-spec.md
4. Include JSON schemas compatible with Archimedes validation

---

## Future Work (Post-MVP)

### Protobuf Support (T5)

- [ ] Protobuf v3 parser
- [ ] gRPC service extraction
- [ ] Protobuf-specific linting
- [ ] Rust tonic code generation

### GraphQL Support (T6)

- [ ] GraphQL SDL parser
- [ ] Schema validation
- [ ] Resolver code generation

### AsyncAPI Support (T7)

- [ ] AsyncAPI 3.0 parser
- [ ] Channel/topic validation
- [ ] Event handler code generation

### Additional Languages (T8)

- [ ] C++ code generation
- [ ] Go code generation

---

## Risk Mitigation

### Technical Risks

1. **OpenAPI Complexity**
   - _Mitigation_: Start with subset, expand coverage iteratively
2. **$ref Resolution**

   - _Mitigation_: Handle internal refs first, external refs later

3. **Code Generation Quality**
   - _Mitigation_: Test generated code compiles and runs

### Schedule Risks

1. **Feature Creep**

   - _Mitigation_: Strict MVP scope, defer extras to post-MVP

2. **Testing Overhead**
   - _Mitigation_: Build test fixtures early, reuse across crates
