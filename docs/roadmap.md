# Themis – Development Roadmap

> **Version**: 1.1.0  
> **Created**: 2026-01-04  
> **Last Updated**: 2026-01-04  
> **Target Completion**: Week 14 (MVP)

---

## Key Decisions

| Decision                                                                     | Impact                                 |
| ---------------------------------------------------------------------------- | -------------------------------------- |
| [ADR-006](../../docs/decisions/006-grpc-post-mvp.md)                         | MVP is OpenAPI 3.x only, gRPC post-MVP |
| [ADR-005](../../docs/decisions/005-kubernetes-ingress-over-custom-router.md) | No custom router, use K8s Ingress      |
| [ADR-007](../../docs/decisions/007-apache-2-license.md)                      | Apache 2.0 license                     |

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

```
         Week: 1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20
 Themis:      [T0][---T1---][--T2--][------T3------][--T4--][--T5--]
 Eunomia:     [E0][---E1---][------E2------][------E3------]        (gap)        [------E4------]
 Archimedes:  [A0][---A1---][----------A2----------][----------A3----------][A4][------A5------]
```

**Key Coordination Points**:

- Week 1: Themis creates `themis-platform-types` crate (all components depend on this)
- Week 12: Themis artifacts available for Archimedes integration (T4 complete)
- Week 14: Themis MVP complete, supporting Archimedes A5 integration
- Weeks 17-20: Full platform integration (Themis team supports Archimedes/Eunomia)

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
- [x] Support Themis extensions (x-themis-*)
  > **Completed 2026-01-04**: Extracts x-themis-rate-limit-tier, x-themis-timeout-tier, x-themis-idempotent

### Week 4: Contract Validation

- [ ] Implement schema validation
- [ ] Check required operationId on all operations
- [ ] Validate error response declarations
- [ ] Check security scheme declarations
- [ ] Implement custom rule framework
- [ ] Add `themis validate` CLI command

### Phase T1 Milestone

**Criteria**: OpenAPI specs can be parsed, validated, and loaded

---

## Phase T2: Linting & Compatibility (Weeks 5-6)

### Week 5: Linting & Compatibility

- [ ] Implement naming convention checks
- [ ] Add versioning rules
- [ ] Implement schema comparison (diffing)
- [ ] Detect added/removed fields
- [ ] Detect type changes
- [ ] Add `themis lint` CLI command

### Week 6: Breaking Change Detection

- [ ] Define breaking change rules
- [ ] Implement compatibility analyzer
- [ ] Add semver validation
- [ ] Generate compatibility report
- [ ] Block breaking changes in minor/patch
- [ ] Add `themis diff` CLI command

### Phase T2 Milestone

**Criteria**: Breaking changes are detected, contracts can be linted

---

## Phase T3: Code Generation (Weeks 7-10)

### Week 7: Rust Types Generation

- [ ] Create `themis-codegen` crate
- [ ] Generate Rust structs from schemas
- [ ] Handle nested types
- [ ] Add serde derives
- [ ] Generate validation derives
- [ ] Test generated code compiles

### Week 8: Rust Handlers & CLI

- [ ] Generate handler trait definitions
- [ ] Generate request/response types per operation
- [ ] Generate error enum
- [ ] Add `themis codegen --language rust` CLI command
- [ ] Document generated code usage

### Week 9: TypeScript Generation

- [ ] Generate TypeScript interfaces
- [ ] Generate fetch client
- [ ] Generate Express/Fastify handler types
- [ ] Add JSDoc comments
- [ ] Create npm package output
- [ ] Add `themis codegen --language typescript` CLI command

### Week 10: Python Generation

- [ ] Generate Python dataclasses
- [ ] Generate httpx client
- [ ] Generate FastAPI handler signatures
- [ ] Add type hints
- [ ] Create package output
- [ ] Add `themis codegen --language python` CLI command

### Phase T3 Milestone

**Criteria**: Rust, TypeScript, and Python code can be generated from contracts

---

## Phase T4: Publishing & Registry (Weeks 11-12)

### Week 11: Artifact Publishing

- [ ] Create `themis-artifact` crate
- [ ] Design artifact format
- [ ] Implement artifact builder
- [ ] Add content-addressable storage (checksum)
- [ ] Implement checksum verification
- [ ] Add artifact versioning

### Week 12: Registry Client

- [ ] Create `themis-registry` crate
- [ ] Implement OCI registry client
- [ ] Add `themis publish` command
- [ ] Add `themis fetch` command
- [ ] Implement caching
- [ ] Test registry operations

### Phase T4 Milestone

**Criteria**: Artifacts can be published to and fetched from registry

---

## Phase T5: Integration Testing (Weeks 13-14) ⭐ NEW

> **Purpose**: Validate that Themis artifacts work correctly with Archimedes and Eunomia.

### Week 13: Archimedes Integration

- [ ] Test artifact loading in Archimedes
- [ ] Verify operation → handler mapping
- [ ] Test request validation with generated schemas
- [ ] Test response validation
- [ ] Verify error envelope format matches shared types
- [ ] Document integration patterns

### Week 14: End-to-End Testing

- [ ] Create integration test suite
- [ ] Test full workflow: contract → artifact → Archimedes validation
- [ ] Test policy context includes correct operation metadata
- [ ] Verify `PolicyInput.operation_id` matches Themis operationId
- [ ] Performance testing with large contracts
- [ ] Write integration documentation

### Phase T5 Milestone

**Criteria**: Themis artifacts work seamlessly with Archimedes runtime

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
