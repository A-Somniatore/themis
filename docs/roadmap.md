# Themis – Development Roadmap

> **Version**: 1.0.0  
> **Created**: 2026-01-04  
> **Target Completion**: Week 14 (MVP)

---

## Overview

Themis is the contract governance toolchain for the Themis Platform. Development runs **in parallel with Eunomia** during the first 12 weeks, with an additional 2 weeks for integration.

### Timeline Summary

| Phase                       | Duration | Weeks | Description                              |
| --------------------------- | -------- | ----- | ---------------------------------------- |
| T0: Shared Types            | 1 week   | 1     | Create `themis-platform-types` crate     |
| T1: Foundation              | 3 weeks  | 2-4   | Core types, OpenAPI parsing, validation  |
| T2: Linting & Compatibility | 2 weeks  | 5-6   | Linting rules, breaking change detection |
| T3: Code Generation         | 4 weeks  | 7-10  | Rust, TypeScript, Python code generation |
| T4: Publishing & Registry   | 2 weeks  | 11-12 | Artifact publishing, registry client     |
| T5: Integration Testing     | 2 weeks  | 13-14 | End-to-end testing with Archimedes/Eunomia |

**Total**: 14 weeks

---

## Phase T0: Shared Platform Types (Week 1) ⭐ NEW

> **Purpose**: Create a shared crate that Themis, Archimedes, and Eunomia all depend on.
> This ensures schema compatibility at compile time and eliminates integration drift.

### Week 1: Create `themis-platform-types`

- [ ] Create new repository `themis-platform-types` (or workspace member)
- [ ] Move/define shared types:
  - [ ] `CallerIdentity` enum (Spiffe, User, ApiKey, Anonymous)
  - [ ] `PolicyInput` struct (caller, service, operation_id, method, path, headers, timestamp, environment, context)
  - [ ] `PolicyDecision` struct (allowed, reason, policy_id, policy_version, evaluation_time_ns)
  - [ ] `ThemisErrorEnvelope` struct (code, message, details, request_id, timestamp, trace_id)
  - [ ] `RequestId` type (UUID v7)
  - [ ] `SemanticVersion` type
  - [ ] Standard error codes enum
- [ ] Add comprehensive documentation
- [ ] Publish to crates.io (or private registry)
- [ ] Create JSON Schema definitions for each type
- [ ] Document in [integration-spec.md](../../docs/integration/integration-spec.md)

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
- [ ] Integrate `themis-platform-types` dependency
- [ ] Refactor error types to use `ThemisErrorEnvelope` from shared crate

### Week 2: Core Types

- [ ] Implement `Contract` model
- [ ] Implement `Operation` model
- [ ] Implement `Schema` types
- [ ] Implement `Version` (semver)
- [ ] Implement error types
- [ ] Write serialization tests

### Week 3: OpenAPI Parser

- [ ] Implement OpenAPI 3.1 parser
- [ ] Handle `$ref` resolution (internal refs)
- [ ] Extract operations with operationId
- [ ] Extract request/response schemas
- [ ] Parse security schemes
- [ ] Test with sample OpenAPI specs

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

## Milestones Summary

| Milestone         | Target  | Criteria                                |
| ----------------- | ------- | --------------------------------------- |
| T1: Parsing       | Week 4  | OpenAPI specs parsed correctly          |
| T2: Compatibility | Week 6  | Breaking changes detected               |
| T3: Code Gen      | Week 10 | Rust, TypeScript, Python code generated |
| T4: Publishing    | Week 12 | Artifacts published to registry         |

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

- `themis-core` - Core types and traits
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

| Dependency | Required For             | Available   |
| ---------- | ------------------------ | ----------- |
| None       | Core development (T1-T4) | Immediately |
| Archimedes | Runtime validation       | Week 12+    |

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
