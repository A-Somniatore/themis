# Themis – Implementation Design Document

> **Version**: 1.1.0-draft  
> **Status**: Design Phase  
> **Repository**: `github.com/A-Somniatore/themis` (to be created)  
> **Last Updated**: 2026-01-04

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Goals & Non-Goals](#2-goals--non-goals)
3. [Architecture Overview](#3-architecture-overview)
4. [Repository Structure](#4-repository-structure)
5. [Contract Formats](#5-contract-formats)
6. [Async Event Contracts](#5b-async-event-contracts)
7. [GraphQL Contracts](#5c-graphql-contracts)
8. [Contract Lifecycle](#6-contract-lifecycle)
9. [Versioning & Compatibility](#7-versioning--compatibility)
10. [CI Pipeline](#8-ci-pipeline)
11. [Code Generation](#9-code-generation)
12. [Artifact Publishing](#10-artifact-publishing)
13. [Contract Registry](#11-contract-registry)
14. [CLI Design](#12-cli-design)
15. [Error Model](#13-error-model)
16. [Integration Points](#14-integration-points)
17. [Testing Strategy](#15-testing-strategy)
18. [Open Questions](#16-open-questions)
19. [Implementation Phases](#17-implementation-phases)

---

## 1. Executive Summary

Themis is the **contract and schema governance system** for the platform. It provides:

- **Contract-first development** – APIs are defined before implementation
- **Schema governance** – OpenAPI 3.1, Protobuf v3, GraphQL, and AsyncAPI as sources of truth
- **Version management** – Semantic versioning with compatibility enforcement
- **CI enforcement** – Non-compliant contracts cannot be merged or deployed
- **Code generation** – Strongly typed models and handler interfaces
- **Artifact publishing** – Immutable, content-addressed contract artifacts

Themis is a **toolchain and workflow**, not a runtime service. It consists of:

- CLI tools for validation and code generation
- CI actions for automated enforcement
- Contract registry for artifact storage and discovery

---

## 2. Goals & Non-Goals

### Goals

- ✅ Define canonical API contract formats (OpenAPI 3.1, Protobuf v3, **GraphQL**, **AsyncAPI**)
- ✅ Enforce contract-first development workflow
- ✅ Validate contracts for correctness and consistency
- ✅ Detect breaking changes between versions
- ✅ Generate code artifacts (types, SDKs, handler interfaces) for **multiple languages**
- ✅ Publish immutable contract artifacts
- ✅ Integrate with CI/CD pipelines
- ✅ Provide standardized error model
- ✅ Support async/event-driven contracts (Kafka, RabbitMQ, etc.)
- ✅ Support GraphQL schema governance

### Supported Contract Formats

| Format           | Use Case                         | Status |
| ---------------- | -------------------------------- | ------ |
| **OpenAPI 3.1**  | REST/HTTP APIs                   | V1     |
| **Protobuf v3**  | gRPC services                    | V1     |
| **GraphQL SDL**  | GraphQL APIs                     | V1     |
| **AsyncAPI 3.0** | Event-driven (Kafka, AMQP, etc.) | V1     |

### Supported Languages

Themis generates type-safe code for services written in any of these languages:

| Language               | Client SDK                     | Server Handlers             | Status       |
| ---------------------- | ------------------------------ | --------------------------- | ------------ |
| **Rust**               | ✅ Types, client               | ✅ Archimedes handlers      | V1           |
| **TypeScript/Node.js** | ✅ Types, fetch client         | ✅ Express/Fastify handlers | V1           |
| **Python**             | ✅ Dataclasses, httpx client   | ✅ FastAPI handlers         | V1           |
| **C++**                | ✅ Structs, cpr/libcurl client | ✅ Header interfaces        | V1           |
| **Go**                 | ✅ Structs, net/http client    | ✅ net/http handlers        | V2 (planned) |

All generated code includes:

- Strongly typed request/response models
- Error type definitions
- Client SDK for calling the service
- Server handler interfaces for implementing the service
- JSON serialization/deserialization
- Validation helpers

### Non-Goals (V1)

- ❌ Runtime contract enforcement (Archimedes responsibility)
- ❌ Authorization policy definition (Eunomia responsibility)
- ❌ UI-based contract editing (Git is source of truth)
- ❌ Schema inference from code

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              THEMIS ECOSYSTEM                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         Contract Repositories                            ││
│  │                                                                          ││
│  │   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           ││
│  │   │ users-service  │  │ orders-service │  │ payments-svc   │   ...     ││
│  │   │ /contracts/    │  │ /contracts/    │  │ /contracts/    │           ││
│  │   │   v1/          │  │   v1/          │  │   v1/          │           ││
│  │   │   v2/          │  │   v2/          │  │                │           ││
│  │   └────────────────┘  └────────────────┘  └────────────────┘           ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│                                    │ git push                                │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                           CI Pipeline                                    ││
│  │                                                                          ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐ ││
│  │  │  Lint    │→ │  Validate │→ │  Compat  │→ │  CodeGen │→ │  Publish  │ ││
│  │  │          │  │  Schema   │  │  Check   │  │          │  │  Artifact │ ││
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └───────────┘ ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│                                    │ publish                                 │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                       Contract Registry                                  ││
│  │                                                                          ││
│  │  ┌─────────────────────────────────────────────────────────────────┐   ││
│  │  │  users-service/v1.0.0.artifact.json                              │   ││
│  │  │  users-service/v1.1.0.artifact.json                              │   ││
│  │  │  users-service/v2.0.0.artifact.json                              │   ││
│  │  │  orders-service/v1.0.0.artifact.json                             │   ││
│  │  │  ...                                                              │   ││
│  │  └─────────────────────────────────────────────────────────────────┘   ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│              ┌─────────────────────┼─────────────────────┐                  │
│              │                     │                     │                   │
│              ▼                     ▼                     ▼                   │
│  ┌───────────────────┐ ┌───────────────────┐ ┌───────────────────┐         │
│  │    Archimedes     │ │      Eunomia      │ │       Stoa        │         │
│  │    (Runtime)      │ │   (Policy Auth)   │ │       (UI)        │         │
│  │                   │ │                   │ │                   │         │
│  │  Loads artifact   │ │  Reads operation  │ │  Displays         │         │
│  │  for validation   │ │  metadata for     │ │  contracts,       │         │
│  │                   │ │  policy context   │ │  versions, diffs  │         │
│  └───────────────────┘ └───────────────────┘ └───────────────────┘         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Repository Structure

### 4.1 Themis Toolchain Repository

```
themis/
├── Cargo.toml                    # Workspace root
├── README.md
├── LICENSE
│
├── crates/
│   ├── themis-core/              # Core types and traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── contract.rs       # Contract model
│   │       ├── version.rs        # Semantic versioning
│   │       ├── operation.rs      # Operation definitions
│   │       └── error.rs          # Error model
│   │
│   ├── themis-openapi/           # OpenAPI 3.1 support
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # OpenAPI parsing
│   │       ├── validator.rs      # Schema validation
│   │       └── normalizer.rs     # Normalize to internal model
│   │
│   ├── themis-protobuf/          # Protobuf v3 support
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # Protobuf parsing
│   │       ├── validator.rs      # Schema validation
│   │       └── normalizer.rs     # Normalize to internal model
│   │
│   ├── themis-graphql/           # GraphQL SDL support
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # GraphQL schema parsing
│   │       ├── validator.rs      # Schema validation
│   │       └── normalizer.rs     # Normalize to internal model
│   │
│   ├── themis-asyncapi/          # AsyncAPI 3.0 support
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # AsyncAPI parsing
│   │       ├── validator.rs      # Schema validation
│   │       ├── normalizer.rs     # Normalize to internal model
│   │       └── channels.rs       # Channel/topic mapping
│   │
│   ├── themis-lint/              # Contract linting
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rules/            # Lint rules
│   │       │   ├── naming.rs
│   │       │   ├── versioning.rs
│   │       │   ├── security.rs
│   │       │   └── documentation.rs
│   │       └── reporter.rs       # Issue reporting
│   │
│   ├── themis-compat/            # Compatibility checking
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── differ.rs         # Schema diffing
│   │       ├── analyzer.rs       # Breaking change detection
│   │       └── report.rs         # Compatibility report
│   │
│   ├── themis-codegen/           # Code generation (multi-language)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs         # Common codegen traits
│   │       ├── rust/             # Rust code generation
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── handlers.rs
│   │       │   ├── graphql.rs    # GraphQL resolvers
│   │       │   └── errors.rs
│   │       ├── typescript/       # TypeScript/Node.js code generation
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── client.rs
│   │       │   ├── graphql.rs    # GraphQL resolvers
│   │       │   └── server.rs     # Express/Fastify handlers
│   │       ├── python/           # Python code generation
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── client.rs
│   │       │   └── server.rs     # FastAPI/Flask handlers
│   │       ├── cpp/              # C++ code generation
│   │       │   ├── mod.rs
│   │       │   ├── types.rs      # Structs with nlohmann/json
│   │       │   ├── client.rs     # libcurl/cpr client
│   │       │   └── headers.rs    # Header file generation
│   │       └── go/               # Go code generation (future)
│   │           ├── mod.rs
│   │           ├── types.rs
│   │           ├── client.rs
│   │           └── server.rs     # net/http handlers
│   │
│   ├── themis-artifact/          # Artifact creation & loading
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builder.rs        # Artifact creation
│   │       ├── loader.rs         # Artifact loading
│   │       └── checksum.rs       # Integrity verification
│   │
│   └── themis-registry/          # Registry client (OCI)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── client.rs         # Registry API client
│           ├── publish.rs        # Artifact publishing
│           └── fetch.rs          # Artifact fetching
│
├── themis-cli/                   # CLI application
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── commands/
│           ├── lint.rs
│           ├── validate.rs
│           ├── diff.rs
│           ├── codegen.rs
│           ├── publish.rs
│           └── fetch.rs
│
├── themis-action/                # GitHub Action
│   ├── action.yml
│   ├── Dockerfile
│   └── entrypoint.sh
│
├── schemas/                      # JSON Schemas for validation
│   ├── themis-contract.schema.json
│   ├── themis-artifact.schema.json
│   └── themis-error.schema.json
│
└── examples/
    ├── users-service/            # Example OpenAPI contract
    └── products-service/         # Example Protobuf contract
```

### 4.2 Service Contract Repository Structure

Each service maintains its contracts in a standardized structure:

```
my-service/
├── contracts/
│   ├── v1/
│   │   ├── openapi.yaml          # OpenAPI 3.1 contract
│   │   └── README.md             # Version changelog
│   ├── v2/
│   │   ├── openapi.yaml
│   │   └── README.md
│   └── themis.yaml               # Contract metadata
├── src/                          # Service implementation
└── ...
```

**`themis.yaml`** - Contract metadata:

```yaml
# themis.yaml
service:
  name: users-service
  description: User management service
  owner: platform-team

contracts:
  format: openapi # or "protobuf"
  current_version: v2

metadata:
  repository: github.com/somniatore/users-service
  documentation: https://docs.somniatore.com/users-service
```

---

## 5. Contract Formats

### 5.1 OpenAPI 3.1 Requirements

Themis requires specific OpenAPI 3.1 features:

```yaml
# contracts/v1/openapi.yaml
openapi: "3.1.0"
info:
  title: Users Service
  version: "1.0.0"
  description: User management API

  # REQUIRED: Themis extension for service metadata
  x-themis:
    service: users-service
    owner: platform-team

# REQUIRED: Security schemes must be defined
components:
  securitySchemes:
    spiffe:
      type: mutualTLS
      description: Internal service-to-service authentication
    bearer:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: External user authentication

paths:
  /users/{userId}:
    get:
      # REQUIRED: Unique operation identifier
      operationId: getUser
      summary: Get user by ID

      # REQUIRED: Security requirements
      security:
        - spiffe: []
        - bearer: []

      # REQUIRED: Themis extensions for operation metadata
      x-themis:
        # Rate limit intent (actual values from runtime config)
        rate-limit-tier: standard
        # Timeout intent
        timeout-tier: fast
        # Idempotency
        idempotent: true

      parameters:
        - name: userId
          in: path
          required: true
          schema:
            type: string
            format: uuid

      responses:
        "200":
          description: User found
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"

        # REQUIRED: All error responses must be declared
        "404":
          description: User not found
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/UserNotFoundError"

        "401":
          $ref: "#/components/responses/Unauthorized"

        "403":
          $ref: "#/components/responses/Forbidden"

components:
  schemas:
    User:
      type: object
      required:
        - id
        - email
        - name
      properties:
        id:
          type: string
          format: uuid
        email:
          type: string
          format: email
        name:
          type: string
          minLength: 1
          maxLength: 100

    # REQUIRED: Error schemas follow Themis error model
    UserNotFoundError:
      allOf:
        - $ref: "#/components/schemas/ThemisError"
        - type: object
          properties:
            code:
              const: USER_NOT_FOUND
            details:
              type: object
              properties:
                userId:
                  type: string

    # Standard Themis error base
    ThemisError:
      type: object
      required:
        - code
        - message
      properties:
        code:
          type: string
          description: Machine-readable error code
        message:
          type: string
          description: Human-readable error message
        request_id:
          type: string
        trace_id:
          type: string
        details:
          type: object
          additionalProperties: true

  responses:
    Unauthorized:
      description: Authentication required
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ThemisError"

    Forbidden:
      description: Authorization denied
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ThemisError"
```

### 5.2 Protobuf v3 Requirements

```protobuf
// contracts/v1/service.proto
syntax = "proto3";

package somniatore.users.v1;

import "google/api/annotations.proto";
import "themis/extensions.proto";

option go_package = "github.com/somniatore/users-service/proto/v1";

// Service definition
service UsersService {
  option (themis.service) = {
    name: "users-service"
    owner: "platform-team"
  };

  // Get user by ID
  rpc GetUser(GetUserRequest) returns (GetUserResponse) {
    option (google.api.http) = {
      get: "/v1/users/{user_id}"
    };
    option (themis.operation) = {
      operation_id: "getUser"
      rate_limit_tier: STANDARD
      timeout_tier: FAST
      idempotent: true
      security: [SPIFFE, BEARER]
    };
  }

  // Create new user
  rpc CreateUser(CreateUserRequest) returns (CreateUserResponse) {
    option (google.api.http) = {
      post: "/v1/users"
      body: "*"
    };
    option (themis.operation) = {
      operation_id: "createUser"
      rate_limit_tier: STRICT
      timeout_tier: STANDARD
      idempotent: false
      security: [BEARER]
    };
  }
}

message GetUserRequest {
  string user_id = 1 [(themis.field) = {
    format: UUID
    description: "Unique user identifier"
  }];
}

message GetUserResponse {
  User user = 1;
}

message User {
  string id = 1;
  string email = 2 [(themis.field) = { format: EMAIL }];
  string name = 3;
  google.protobuf.Timestamp created_at = 4;
}

// Error types
message UserNotFoundError {
  option (themis.error) = {
    code: "USER_NOT_FOUND"
    http_status: 404
  };

  string user_id = 1;
}
```

---

## 5.3 AsyncAPI 3.0 (Event-Driven Contracts)

Themis supports **AsyncAPI 3.0** for event-driven architectures (Kafka, RabbitMQ, etc.).

```yaml
# contracts/v1/asyncapi.yaml
asyncapi: "3.0.0"
info:
  title: Users Service Events
  version: "1.0.0"
  description: Event contracts for Users Service

  # REQUIRED: Themis extension
  x-themis:
    service: users-service
    owner: platform-team

servers:
  production:
    host: kafka.somniatore.com:9092
    protocol: kafka
    description: Production Kafka cluster

channels:
  user/created:
    address: user.created
    messages:
      userCreated:
        $ref: "#/components/messages/UserCreatedEvent"

    # REQUIRED: Themis metadata
    x-themis:
      event_id: userCreated
      retention: 7d
      partitioning_key: "$.userId"

  user/updated:
    address: user.updated
    messages:
      userUpdated:
        $ref: "#/components/messages/UserUpdatedEvent"
    x-themis:
      event_id: userUpdated
      retention: 7d
      partitioning_key: "$.userId"

  user/deleted:
    address: user.deleted
    messages:
      userDeleted:
        $ref: "#/components/messages/UserDeletedEvent"
    x-themis:
      event_id: userDeleted
      retention: 30d # Longer retention for audit
      partitioning_key: "$.userId"

operations:
  publishUserCreated:
    action: send
    channel:
      $ref: "#/channels/user~1created"
    summary: Publish when a new user is created

  consumeUserCreated:
    action: receive
    channel:
      $ref: "#/channels/user~1created"
    summary: Consume user creation events

components:
  messages:
    UserCreatedEvent:
      name: UserCreatedEvent
      contentType: application/json
      headers:
        type: object
        properties:
          correlationId:
            type: string
            format: uuid
          timestamp:
            type: string
            format: date-time
      payload:
        $ref: "#/components/schemas/UserCreatedPayload"

    UserUpdatedEvent:
      name: UserUpdatedEvent
      contentType: application/json
      payload:
        $ref: "#/components/schemas/UserUpdatedPayload"

    UserDeletedEvent:
      name: UserDeletedEvent
      contentType: application/json
      payload:
        $ref: "#/components/schemas/UserDeletedPayload"

  schemas:
    UserCreatedPayload:
      type: object
      required:
        - eventId
        - eventType
        - timestamp
        - userId
        - email
      properties:
        eventId:
          type: string
          format: uuid
        eventType:
          type: string
          const: "user.created"
        timestamp:
          type: string
          format: date-time
        userId:
          type: string
          format: uuid
        email:
          type: string
          format: email
        name:
          type: string

    UserUpdatedPayload:
      type: object
      required:
        - eventId
        - eventType
        - timestamp
        - userId
        - changes
      properties:
        eventId:
          type: string
          format: uuid
        eventType:
          type: string
          const: "user.updated"
        timestamp:
          type: string
          format: date-time
        userId:
          type: string
          format: uuid
        changes:
          type: object
          additionalProperties: true

    UserDeletedPayload:
      type: object
      required:
        - eventId
        - eventType
        - timestamp
        - userId
      properties:
        eventId:
          type: string
          format: uuid
        eventType:
          type: string
          const: "user.deleted"
        timestamp:
          type: string
          format: date-time
        userId:
          type: string
          format: uuid
        reason:
          type: string
```

### AsyncAPI Code Generation

**Rust Producer:**

```rust
// Generated by themis generate rust --async
use crate::events::{UserCreatedPayload, UserCreatedEvent};

#[async_trait]
pub trait UserEventsProducer: Send + Sync {
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), EventError>;
    async fn publish_user_updated(&self, event: UserUpdatedEvent) -> Result<(), EventError>;
    async fn publish_user_deleted(&self, event: UserDeletedEvent) -> Result<(), EventError>;
}

// Kafka implementation provided by archimedes-kafka crate
```

**Rust Consumer:**

```rust
// Generated consumer trait
#[async_trait]
pub trait UserEventsConsumer: Send + Sync {
    async fn on_user_created(&self, event: UserCreatedEvent) -> Result<(), ConsumerError>;
    async fn on_user_updated(&self, event: UserUpdatedEvent) -> Result<(), ConsumerError>;
    async fn on_user_deleted(&self, event: UserDeletedEvent) -> Result<(), ConsumerError>;
}
```

---

## 5.4 GraphQL Schema Contracts

Themis supports **GraphQL SDL** for GraphQL APIs.

```graphql
# contracts/v1/schema.graphql

# REQUIRED: Themis directive definitions
directive @themis(service: String!, owner: String!) on SCHEMA

directive @operation(
  operationId: String!
  rateLimitTier: RateLimitTier = STANDARD
  timeoutTier: TimeoutTier = STANDARD
  security: [SecurityScheme!]!
) on FIELD_DEFINITION

directive @deprecated(
  reason: String!
  sunset: String # ISO date when field will be removed
) on FIELD_DEFINITION | ENUM_VALUE

enum RateLimitTier {
  UNLIMITED
  HIGH
  STANDARD
  STRICT
  AUTH
}

enum TimeoutTier {
  INSTANT
  FAST
  STANDARD
  SLOW
}

enum SecurityScheme {
  SPIFFE
  BEARER
  API_KEY
  PUBLIC
}

# Schema with Themis metadata
schema @themis(service: "users-service", owner: "platform-team") {
  query: Query
  mutation: Mutation
  subscription: Subscription
}

type Query {
  """
  Get a user by their unique identifier
  """
  user(id: ID!): User
    @operation(
      operationId: "getUser"
      rateLimitTier: STANDARD
      timeoutTier: FAST
      security: [SPIFFE, BEARER]
    )

  """
  List all users with pagination
  """
  users(first: Int = 20, after: String, filter: UserFilter): UserConnection!
    @operation(
      operationId: "listUsers"
      rateLimitTier: STANDARD
      timeoutTier: STANDARD
      security: [SPIFFE, BEARER]
    )

  """
  Search users by query string
  """
  searchUsers(query: String!, first: Int = 20): UserConnection!
    @operation(
      operationId: "searchUsers"
      rateLimitTier: STRICT # Expensive operation
      timeoutTier: SLOW
      security: [BEARER]
    )
}

type Mutation {
  """
  Create a new user account
  """
  createUser(input: CreateUserInput!): CreateUserPayload!
    @operation(
      operationId: "createUser"
      rateLimitTier: STRICT
      timeoutTier: STANDARD
      security: [BEARER]
    )

  """
  Update an existing user
  """
  updateUser(id: ID!, input: UpdateUserInput!): UpdateUserPayload!
    @operation(
      operationId: "updateUser"
      rateLimitTier: STANDARD
      timeoutTier: STANDARD
      security: [BEARER]
    )

  """
  Delete a user account
  """
  deleteUser(id: ID!): DeleteUserPayload!
    @operation(
      operationId: "deleteUser"
      rateLimitTier: STRICT
      timeoutTier: STANDARD
      security: [BEARER]
    )
}

type Subscription {
  """
  Subscribe to user updates
  """
  userUpdated(userId: ID!): User!
    @operation(
      operationId: "subscribeUserUpdates"
      rateLimitTier: STANDARD
      timeoutTier: SLOW
      security: [BEARER]
    )
}

# Types
type User {
  id: ID!
  email: String!
  name: String!
  createdAt: DateTime!
  updatedAt: DateTime!

  # Nested resolver with its own operation
  posts(first: Int = 10): PostConnection!
    @operation(
      operationId: "getUserPosts"
      rateLimitTier: STANDARD
      timeoutTier: STANDARD
      security: [SPIFFE, BEARER]
    )
}

type UserConnection {
  edges: [UserEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type UserEdge {
  cursor: String!
  node: User!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# Inputs
input CreateUserInput {
  email: String!
  name: String!
  password: String!
}

input UpdateUserInput {
  email: String
  name: String
}

input UserFilter {
  email: String
  nameContains: String
  createdAfter: DateTime
  createdBefore: DateTime
}

# Payloads (following Relay conventions)
type CreateUserPayload {
  user: User
  errors: [UserError!]!
}

type UpdateUserPayload {
  user: User
  errors: [UserError!]!
}

type DeleteUserPayload {
  deletedUserId: ID
  errors: [UserError!]!
}

# Error types
type UserError {
  field: String
  code: UserErrorCode!
  message: String!
}

enum UserErrorCode {
  NOT_FOUND
  INVALID_EMAIL
  EMAIL_TAKEN
  UNAUTHORIZED
  VALIDATION_FAILED
}

# Custom scalars
scalar DateTime
scalar Email
```

### GraphQL Code Generation

**Rust Resolvers (async-graphql):**

```rust
// Generated by themis generate rust --graphql
use async_graphql::{Context, Object, Result};

// Generated resolver trait - implement this
#[async_trait]
pub trait UserResolvers: Send + Sync {
    async fn get_user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<User>>;
    async fn list_users(&self, ctx: &Context<'_>, first: Option<i32>, after: Option<String>, filter: Option<UserFilter>) -> Result<UserConnection>;
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<CreateUserPayload>;
    // ... other resolvers
}

// Generated types
#[derive(SimpleObject)]
pub struct User {
    pub id: ID,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub password: String,
}
```

**TypeScript Resolvers:**

```typescript
// Generated by themis generate typescript --graphql
import {
  Resolvers,
  User,
  CreateUserInput,
  CreateUserPayload,
} from "./generated/types";

// Implement this interface
export interface UserResolvers {
  Query: {
    user: (
      parent: unknown,
      args: { id: string },
      ctx: Context
    ) => Promise<User | null>;
    users: (
      parent: unknown,
      args: UsersArgs,
      ctx: Context
    ) => Promise<UserConnection>;
  };
  Mutation: {
    createUser: (
      parent: unknown,
      args: { input: CreateUserInput },
      ctx: Context
    ) => Promise<CreateUserPayload>;
  };
}
```

---

## 6. Contract Lifecycle

### 6.1 Workflow States

```
┌─────────────┐    PR Created    ┌─────────────┐
│   Draft     │ ───────────────► │  Proposed   │
│             │                  │             │
└─────────────┘                  └──────┬──────┘
                                        │
                                        │ CI Passes
                                        ▼
                                 ┌─────────────┐
                                 │  Validated  │
                                 │             │
                                 └──────┬──────┘
                                        │
                                        │ PR Merged
                                        ▼
                                 ┌─────────────┐
                                 │  Published  │
                                 │             │
                                 └──────┬──────┘
                                        │
                                        │ Deployed
                                        ▼
                                 ┌─────────────┐
                                 │   Active    │
                                 │             │
                                 └──────┬──────┘
                                        │
                                        │ New version replaces
                                        ▼
                                 ┌─────────────┐
                                 │ Deprecated  │
                                 │             │
                                 └──────┬──────┘
                                        │
                                        │ Sunset period ends
                                        ▼
                                 ┌─────────────┐
                                 │  Retired    │
                                 │             │
                                 └─────────────┘
```

### 6.2 Change Workflow

1. **Developer creates contract change** in feature branch
2. **Open PR** triggers CI pipeline
3. **CI validates**:
   - Schema correctness
   - Lint rules pass
   - Compatibility check (within major version)
   - Code generation succeeds
4. **Review** by contract owner and platform team
5. **Merge** publishes artifact to registry
6. **Downstream consumers** update to new version

---

## 7. Versioning & Compatibility

### 7.1 Semantic Versioning Rules

| Change Type                  | Version Bump | Allowed in Minor/Patch? |
| ---------------------------- | ------------ | ----------------------- |
| Add optional field           | Minor        | ✅ Yes                  |
| Add new endpoint             | Minor        | ✅ Yes                  |
| Add new error variant        | Minor        | ✅ Yes                  |
| Fix typo in description      | Patch        | ✅ Yes                  |
| Remove field                 | Major        | ❌ No                   |
| Change field type            | Major        | ❌ No                   |
| Make optional field required | Major        | ❌ No                   |
| Remove endpoint              | Major        | ❌ No                   |
| Change endpoint path         | Major        | ❌ No                   |
| Change response structure    | Major        | ❌ No                   |

### 7.2 Compatibility Checking Algorithm

```rust
pub struct CompatibilityChecker {
    old_contract: Contract,
    new_contract: Contract,
}

impl CompatibilityChecker {
    pub fn check(&self) -> CompatibilityReport {
        let mut breaking_changes = Vec::new();
        let mut additions = Vec::new();
        let mut modifications = Vec::new();

        // Check each operation in old contract still exists
        for (op_id, old_op) in &self.old_contract.operations {
            match self.new_contract.operations.get(op_id) {
                None => {
                    breaking_changes.push(BreakingChange::OperationRemoved {
                        operation_id: op_id.clone(),
                    });
                }
                Some(new_op) => {
                    // Check path unchanged
                    if old_op.path != new_op.path {
                        breaking_changes.push(BreakingChange::PathChanged {
                            operation_id: op_id.clone(),
                            old_path: old_op.path.clone(),
                            new_path: new_op.path.clone(),
                        });
                    }

                    // Check request schema compatibility
                    self.check_request_compat(op_id, old_op, new_op, &mut breaking_changes);

                    // Check response schema compatibility
                    self.check_response_compat(op_id, old_op, new_op, &mut breaking_changes);
                }
            }
        }

        // Detect additions (new operations)
        for (op_id, new_op) in &self.new_contract.operations {
            if !self.old_contract.operations.contains_key(op_id) {
                additions.push(Addition::OperationAdded {
                    operation_id: op_id.clone(),
                });
            }
        }

        CompatibilityReport {
            is_compatible: breaking_changes.is_empty(),
            breaking_changes,
            additions,
            modifications,
        }
    }

    fn check_request_compat(&self, op_id: &str, old_op: &Operation, new_op: &Operation, breaking: &mut Vec<BreakingChange>) {
        let old_schema = &old_op.request_schema;
        let new_schema = &new_op.request_schema;

        // Check required fields not added (would break existing clients)
        for (field, new_field_schema) in &new_schema.properties {
            if new_schema.required.contains(field) {
                if !old_schema.properties.contains_key(field) {
                    breaking.push(BreakingChange::RequiredFieldAdded {
                        operation_id: op_id.to_string(),
                        location: "request".to_string(),
                        field: field.clone(),
                    });
                }
            }
        }

        // Check field types unchanged
        for (field, old_field_schema) in &old_schema.properties {
            if let Some(new_field_schema) = new_schema.properties.get(field) {
                if !self.types_compatible(old_field_schema, new_field_schema) {
                    breaking.push(BreakingChange::FieldTypeChanged {
                        operation_id: op_id.to_string(),
                        location: "request".to_string(),
                        field: field.clone(),
                        old_type: old_field_schema.type_name(),
                        new_type: new_field_schema.type_name(),
                    });
                }
            }
        }
    }
}
```

### 7.3 Compatibility Report Format

```json
{
  "service": "users-service",
  "old_version": "1.2.0",
  "new_version": "1.3.0",
  "is_compatible": true,
  "breaking_changes": [],
  "additions": [
    {
      "type": "operation_added",
      "operation_id": "listUserGroups",
      "path": "/users/{userId}/groups"
    }
  ],
  "modifications": [
    {
      "type": "field_description_changed",
      "operation_id": "getUser",
      "field": "email",
      "old": "User email",
      "new": "User email address"
    }
  ]
}
```

---

## 8. CI Pipeline

### 8.1 GitHub Action

```yaml
# .github/workflows/contract-ci.yml
name: Contract CI

on:
  push:
    paths:
      - "contracts/**"
  pull_request:
    paths:
      - "contracts/**"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # Full history for compatibility checking

      - name: Themis Validate
        uses: somniatore/themis-action@v1
        with:
          contract-path: ./contracts

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Themis Lint
        uses: somniatore/themis-action@v1
        with:
          command: lint
          contract-path: ./contracts
          config: .themis-lint.yaml

  compatibility:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Themis Compatibility Check
        uses: somniatore/themis-action@v1
        with:
          command: diff
          base-ref: ${{ github.base_ref }}
          head-ref: ${{ github.head_ref }}
          fail-on-breaking: true

  codegen:
    runs-on: ubuntu-latest
    needs: [validate, lint]
    steps:
      - uses: actions/checkout@v4

      - name: Generate Code
        uses: somniatore/themis-action@v1
        with:
          command: codegen
          contract-path: ./contracts
          output-path: ./src/generated
          targets: rust,typescript

      - name: Verify Generated Code Compiles
        run: cargo check

  publish:
    runs-on: ubuntu-latest
    needs: [validate, lint, compatibility, codegen]
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Publish Artifact
        uses: somniatore/themis-action@v1
        with:
          command: publish
          contract-path: ./contracts
          registry-url: ${{ secrets.THEMIS_REGISTRY_URL }}
          registry-token: ${{ secrets.THEMIS_REGISTRY_TOKEN }}
```

### 8.2 Lint Configuration

```yaml
# .themis-lint.yaml
rules:
  # Naming conventions
  naming/operation-id-format:
    enabled: true
    pattern: "^[a-z][a-zA-Z0-9]*$" # camelCase

  naming/path-format:
    enabled: true
    pattern: "^/[a-z][a-z0-9-/{}]*$" # kebab-case

  # Documentation requirements
  docs/operation-summary:
    enabled: true
    required: true

  docs/operation-description:
    enabled: true
    required: false

  docs/schema-description:
    enabled: true
    required: true
    for: [request, response]

  # Security requirements
  security/operation-auth:
    enabled: true
    require-auth: true
    allow-anonymous: [healthCheck, readiness]

  security/https-only:
    enabled: true

  # Versioning
  versioning/path-version:
    enabled: false # We use major version directories

  # Error handling
  errors/declare-all:
    enabled: true
    require: [400, 401, 403, 404, 500]

  errors/use-themis-envelope:
    enabled: true
```

### 8.3 Validation Rules (Implemented)

Themis enforces the following validation rules for OpenAPI contracts:

#### Required Rules (Errors)

| Rule Code   | Rule Name                  | Description                                    |
| ----------- | -------------------------- | ---------------------------------------------- |
| `THEMIS001` | Missing Operation ID       | Every operation MUST have an `operationId`     |
| `THEMIS002` | Duplicate Operation ID     | `operationId` MUST be unique across operations |
| `THEMIS003` | Undefined Security Scheme  | Referenced security schemes MUST be defined    |
| `THEMIS007` | Invalid Version            | API version MUST be valid semantic version     |

#### Recommended Rules (Warnings)

| Rule Code   | Rule Name                     | Description                                                |
| ----------- | ----------------------------- | ---------------------------------------------------------- |
| `THEMIS004` | Missing Error Responses       | Operations SHOULD declare error responses (4xx/5xx)        |
| `THEMIS005` | Missing Operation Description | Operations SHOULD have descriptions                        |
| `THEMIS006` | Missing Schema Description    | Schemas SHOULD have descriptions                           |
| `THEMIS008` | No Security Defined           | Operations SHOULD have security requirements               |
| `THEMIS009` | Missing Response Schema       | Success responses (2xx except 204) SHOULD have schemas     |

#### CLI Usage

```bash
# Validate a contract
themis validate api.yaml

# JSON output for CI integration
themis validate --format json api.yaml

# Treat warnings as errors (strict mode)
themis validate --warnings-as-errors api.yaml
```

### 8.4 Validation vs Linting

Themis separates **validation** from **linting**:

| Aspect       | Validation (`themis validate`)              | Linting (`themis lint`)                          |
| ------------ | ------------------------------------------- | ------------------------------------------------ |
| **Purpose**  | Ensure contract is structurally correct     | Enforce style conventions and best practices     |
| **Blocking** | Errors block CI/deployment                  | Configurable (warnings or errors)                |
| **Rules**    | Fixed rules (THEMIS001-009)                 | Configurable via `.themis-lint.yaml`             |
| **Scope**    | Schema correctness, required fields         | Naming conventions, documentation, patterns      |

**Validation** checks are mandatory and cannot be disabled. They ensure the contract is valid and can be processed.

**Lint** rules are configurable and can be customized per project. They enforce team conventions and best practices.

### 8.5 Lint Rules (Configurable)

| Rule                       | Default  | Description                                      |
| -------------------------- | -------- | ------------------------------------------------ |
| `naming/operation-id`      | warn     | operationId should be camelCase                  |
| `naming/path-format`       | warn     | Paths should be kebab-case                       |
| `naming/schema-name`       | warn     | Schema names should be PascalCase                |
| `docs/operation-summary`   | warn     | Operations should have summaries                 |
| `security/require-auth`    | off      | All operations must have security (except allow list) |
| `versioning/path-version`  | off      | Paths should include version prefix              |

### 8.6 Breaking Change Detection

The `themis diff` command compares two contract versions and detects breaking changes.

#### Change Categories

| Category   | Description                                    | Semver Impact |
| ---------- | ---------------------------------------------- | ------------- |
| Breaking   | Changes that break existing clients            | Major bump    |
| Addition   | Backwards-compatible new features              | Minor bump    |
| Modification | Non-functional changes (docs, descriptions) | Patch bump    |

#### Breaking Change Rules

| Rule Code  | Change Type                    | Severity |
| ---------- | ------------------------------ | -------- |
| BREAK001   | Operation removed              | Breaking |
| BREAK002   | Operation path changed         | Breaking |
| BREAK003   | Operation method changed       | Breaking |
| BREAK004   | Required field added to request| Breaking |
| BREAK005   | Field removed from response    | Breaking |
| BREAK006   | Field type changed             | Breaking |
| BREAK007   | Field became required          | Breaking |
| BREAK008   | Enum value removed             | Breaking |
| BREAK009   | Security scheme removed        | Breaking |

#### Addition Rules (Non-Breaking)

| Rule Code  | Change Type                    | Semver   |
| ---------- | ------------------------------ | -------- |
| ADD001     | Operation added                | Minor    |
| ADD002     | Optional field added to request| Minor    |
| ADD003     | Field added to response        | Minor    |
| ADD004     | Enum value added               | Minor    |
| ADD005     | Security scheme added          | Minor    |

---

## 9. Code Generation

### 9.1 Rust Code Generation

**Generated types** (`src/generated/types.rs`):

```rust
// Auto-generated by themis-codegen. DO NOT EDIT.
// Contract: users-service v1.2.0
// Generated: 2026-01-04T12:00:00Z

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserResponse {
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user: User,
}
```

**Generated errors** (`src/generated/errors.rs`):

```rust
// Auto-generated by themis-codegen. DO NOT EDIT.

use archimedes::prelude::ThemisError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UserNotFoundError {
    pub user_id: String,
}

impl From<UserNotFoundError> for ThemisError {
    fn from(e: UserNotFoundError) -> Self {
        ThemisError {
            code: "USER_NOT_FOUND".to_string(),
            message: format!("User '{}' not found", e.user_id),
            http_status: http::StatusCode::NOT_FOUND,
            details: Some(serde_json::to_value(&e).unwrap()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailAlreadyExistsError {
    pub email: String,
}

impl From<EmailAlreadyExistsError> for ThemisError {
    fn from(e: EmailAlreadyExistsError) -> Self {
        ThemisError {
            code: "EMAIL_ALREADY_EXISTS".to_string(),
            message: format!("Email '{}' is already registered", e.email),
            http_status: http::StatusCode::CONFLICT,
            details: Some(serde_json::to_value(&e).unwrap()),
        }
    }
}
```

**Generated handler interfaces** (`src/generated/handlers.rs`):

```rust
// Auto-generated by themis-codegen. DO NOT EDIT.

use archimedes::prelude::*;
use super::types::*;
use super::errors::*;

/// Handler for getUser operation
#[async_trait]
pub trait GetUserHandler: Send + Sync + 'static {
    async fn handle(
        &self,
        ctx: &RequestContext,
        request: GetUserRequest,
    ) -> Result<GetUserResponse, ThemisError>;
}

/// Handler for createUser operation
#[async_trait]
pub trait CreateUserHandler: Send + Sync + 'static {
    async fn handle(
        &self,
        ctx: &RequestContext,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, ThemisError>;
}

/// All operations for users-service
pub struct UsersServiceHandlers<G, C>
where
    G: GetUserHandler,
    C: CreateUserHandler,
{
    pub get_user: G,
    pub create_user: C,
}
```

### 9.2 TypeScript Code Generation

```typescript
// Auto-generated by themis-codegen. DO NOT EDIT.
// Contract: users-service v1.2.0

export interface User {
  id: string;
  email: string;
  name: string;
  createdAt?: string;
}

export interface GetUserRequest {
  userId: string;
}

export interface GetUserResponse {
  user: User;
}

export interface CreateUserRequest {
  email: string;
  name: string;
}

export interface CreateUserResponse {
  user: User;
}

// Error types
export interface UserNotFoundError {
  code: "USER_NOT_FOUND";
  message: string;
  userId: string;
}

export interface EmailAlreadyExistsError {
  code: "EMAIL_ALREADY_EXISTS";
  message: string;
  email: string;
}

export type UsersServiceError = UserNotFoundError | EmailAlreadyExistsError;

// Client interface
export interface UsersServiceClient {
  getUser(request: GetUserRequest): Promise<GetUserResponse>;
  createUser(request: CreateUserRequest): Promise<CreateUserResponse>;
}
```

### 9.3 Python Code Generation

```python
# Auto-generated by themis-codegen. DO NOT EDIT.
# Contract: users-service v1.2.0

from dataclasses import dataclass
from datetime import datetime
from typing import Optional
from uuid import UUID

@dataclass
class User:
    id: UUID
    email: str
    name: str
    created_at: Optional[datetime] = None

@dataclass
class GetUserRequest:
    user_id: UUID

@dataclass
class GetUserResponse:
    user: User

@dataclass
class CreateUserRequest:
    email: str
    name: str

@dataclass
class CreateUserResponse:
    user: User

# Error types
@dataclass
class UserNotFoundError:
    code: str = "USER_NOT_FOUND"
    message: str = ""
    user_id: str = ""

@dataclass
class EmailAlreadyExistsError:
    code: str = "EMAIL_ALREADY_EXISTS"
    message: str = ""
    email: str = ""
```

### 9.4 C++ Code Generation

**Generated header** (`users_service_types.hpp`):

```cpp
// Auto-generated by themis-codegen. DO NOT EDIT.
// Contract: users-service v1.2.0

#pragma once

#include <string>
#include <optional>
#include <chrono>
#include <nlohmann/json.hpp>

namespace users_service {

struct User {
    std::string id;  // UUID as string
    std::string email;
    std::string name;
    std::optional<std::chrono::system_clock::time_point> created_at;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE_WITH_DEFAULT(User, id, email, name, created_at)
};

struct GetUserRequest {
    std::string user_id;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE(GetUserRequest, user_id)
};

struct GetUserResponse {
    User user;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE(GetUserResponse, user)
};

struct CreateUserRequest {
    std::string email;
    std::string name;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE(CreateUserRequest, email, name)
};

struct CreateUserResponse {
    User user;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE(CreateUserResponse, user)
};

// Error types
struct UserNotFoundError {
    static constexpr const char* code = "USER_NOT_FOUND";
    std::string message;
    std::string user_id;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE(UserNotFoundError, message, user_id)
};

struct EmailAlreadyExistsError {
    static constexpr const char* code = "EMAIL_ALREADY_EXISTS";
    std::string message;
    std::string email;

    NLOHMANN_DEFINE_TYPE_INTRUSIVE(EmailAlreadyExistsError, message, email)
};

}  // namespace users_service
```

**Generated client** (`users_service_client.hpp`):

```cpp
// Auto-generated by themis-codegen. DO NOT EDIT.

#pragma once

#include "users_service_types.hpp"
#include <cpr/cpr.h>
#include <expected>

namespace users_service {

class UsersServiceClient {
public:
    explicit UsersServiceClient(std::string base_url)
        : base_url_(std::move(base_url)) {}

    std::expected<GetUserResponse, std::string> get_user(const GetUserRequest& request) {
        auto response = cpr::Get(
            cpr::Url{base_url_ + "/users/" + request.user_id},
            cpr::Header{{"Content-Type", "application/json"}}
        );

        if (response.status_code == 200) {
            auto json = nlohmann::json::parse(response.text);
            return json.get<GetUserResponse>();
        }
        return std::unexpected(response.text);
    }

    std::expected<CreateUserResponse, std::string> create_user(const CreateUserRequest& request) {
        nlohmann::json body = request;
        auto response = cpr::Post(
            cpr::Url{base_url_ + "/users"},
            cpr::Header{{"Content-Type", "application/json"}},
            cpr::Body{body.dump()}
        );

        if (response.status_code == 201) {
            auto json = nlohmann::json::parse(response.text);
            return json.get<CreateUserResponse>();
        }
        return std::unexpected(response.text);
    }

private:
    std::string base_url_;
};

}  // namespace users_service
```

### 9.5 Go Code Generation (Future - V2)

```go
// Auto-generated by themis-codegen. DO NOT EDIT.
// Contract: users-service v1.2.0

package usersservice

import (
	"time"
)

type User struct {
	ID        string     `json:"id"`
	Email     string     `json:"email"`
	Name      string     `json:"name"`
	CreatedAt *time.Time `json:"createdAt,omitempty"`
}

type GetUserRequest struct {
	UserID string `json:"userId"`
}

type GetUserResponse struct {
	User User `json:"user"`
}

type CreateUserRequest struct {
	Email string `json:"email"`
	Name  string `json:"name"`
}

type CreateUserResponse struct {
	User User `json:"user"`
}

// Error types
type UserNotFoundError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	UserID  string `json:"userId"`
}

func (e UserNotFoundError) Error() string {
	return e.Message
}

type EmailAlreadyExistsError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	Email   string `json:"email"`
}

func (e EmailAlreadyExistsError) Error() string {
	return e.Message
}
```

### 9.6 Node.js Server Handlers (Express)

```typescript
// Auto-generated by themis-codegen. DO NOT EDIT.
// Contract: users-service v1.2.0

import { Router, Request, Response, NextFunction } from "express";
import {
  GetUserRequest,
  GetUserResponse,
  CreateUserRequest,
  CreateUserResponse,
} from "./types";

export interface UsersServiceHandlers {
  getUser(req: GetUserRequest): Promise<GetUserResponse>;
  createUser(req: CreateUserRequest): Promise<CreateUserResponse>;
}

export function createUsersServiceRouter(
  handlers: UsersServiceHandlers
): Router {
  const router = Router();

  router.get(
    "/users/:userId",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const request: GetUserRequest = { userId: req.params.userId };
        const response = await handlers.getUser(request);
        res.json(response);
      } catch (error) {
        next(error);
      }
    }
  );

  router.post(
    "/users",
    async (req: Request, res: Response, next: NextFunction) => {
      try {
        const request: CreateUserRequest = req.body;
        const response = await handlers.createUser(request);
        res.status(201).json(response);
      } catch (error) {
        next(error);
      }
    }
  );

  return router;
}
```

---

## 10. Artifact Publishing

### 10.1 Artifact Format

```json
{
  "$schema": "https://themis.somniatore.com/schemas/artifact.v1.json",
  "version": "1.2.0",
  "service": "users-service",
  "format": "openapi",
  "format_version": "3.1.0",

  "metadata": {
    "created_at": "2026-01-04T12:00:00Z",
    "git_commit": "abc123def456...",
    "git_repository": "github.com/somniatore/users-service",
    "owner": "platform-team"
  },

  "checksum": {
    "algorithm": "sha256",
    "value": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  },

  "operations": [
    {
      "id": "getUser",
      "method": "GET",
      "path": "/users/{userId}",
      "summary": "Get user by ID",
      "security": ["spiffe", "bearer"],
      "request_schema": { ... },
      "response_schemas": {
        "200": { ... },
        "404": { ... }
      },
      "metadata": {
        "rate_limit_tier": "standard",
        "timeout_tier": "fast",
        "idempotent": true
      }
    }
  ],

  "schemas": {
    "User": { ... },
    "UserNotFoundError": { ... }
  },

  "raw_contract": "base64-encoded-original-openapi-yaml"
}
```

### 10.2 Artifact Storage

Artifacts are stored in an OCI-compatible registry:

```
registry.somniatore.com/
  contracts/
    users-service/
      v1.0.0/
        artifact.json
        artifact.json.sig  # Optional signature
      v1.1.0/
        artifact.json
      v2.0.0/
        artifact.json
```

### 10.3 Artifact Fetching

```rust
// In archimedes service startup
let artifact = themis_registry::fetch(
    "users-service",
    "1.2.0",
    &registry_config,
).await?;

// Verify integrity
artifact.verify_checksum()?;

// Load into sentinel
let sentinel = ThemisSentinel::from_artifact(artifact)?;
```

---

## 11. Contract Registry

### 11.1 Registry API

```yaml
# Registry OpenAPI (meta - contracts for the contract registry!)
openapi: "3.1.0"
info:
  title: Themis Contract Registry
  version: "1.0.0"

paths:
  /v1/contracts:
    get:
      operationId: listContracts
      summary: List all contracts
      parameters:
        - name: service
          in: query
          schema:
            type: string
      responses:
        "200":
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/ContractSummary"

  /v1/contracts/{service}/versions:
    get:
      operationId: listVersions
      summary: List versions of a contract
      parameters:
        - name: service
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/VersionInfo"

  /v1/contracts/{service}/{version}:
    get:
      operationId: getArtifact
      summary: Get contract artifact
      parameters:
        - name: service
          in: path
          required: true
          schema:
            type: string
        - name: version
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Artifact"

    put:
      operationId: publishArtifact
      summary: Publish new artifact
      security:
        - bearer: []
      requestBody:
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/Artifact"
      responses:
        "201":
          description: Published
        "409":
          description: Version already exists
```

### 11.2 Registry Storage Backend

Options:

- **OCI Registry** (Harbor, GHCR) - artifacts as OCI layers
- **S3-compatible** storage with metadata in PostgreSQL
- **Git-based** (simple, but less scalable)

Recommended: **OCI Registry** for production, **Git-based** for small teams.

---

## 12. CLI Design

### 12.1 Command Structure

```bash
themis <command> [options]

Commands:
  lint        Lint contracts for style and correctness
  validate    Validate contract schema
  diff        Compare two contract versions
  codegen     Generate code from contracts
  publish     Publish artifact to registry
  fetch       Fetch artifact from registry
  init        Initialize new contract repository

Options:
  --config    Path to themis config file
  --verbose   Enable verbose output
  --json      Output in JSON format
```

### 12.2 Command Examples

```bash
# Lint contracts
themis lint ./contracts

# Validate specific version
themis validate ./contracts/v1/openapi.yaml

# Check compatibility between versions
themis diff ./contracts/v1 ./contracts/v2
themis diff --base v1.0.0 --head v1.1.0 --service users-service

# Generate code
themis codegen \
  --contract ./contracts/v1/openapi.yaml \
  --output ./src/generated \
  --target rust

# Publish to registry
themis publish \
  --contract ./contracts/v1/openapi.yaml \
  --registry https://registry.somniatore.com \
  --version 1.2.0

# Fetch artifact
themis fetch users-service@1.2.0 \
  --registry https://registry.somniatore.com \
  --output ./contracts/users-service.artifact.json

# Initialize new contract
themis init \
  --service my-new-service \
  --format openapi \
  --output ./contracts
```

---

## 13. Error Model

### 13.1 Standard Error Envelope

All Themis-governed services use a consistent error format:

```json
{
  "code": "USER_NOT_FOUND",
  "message": "User with ID '123' not found",
  "request_id": "01941234-5678-7abc-def0-123456789abc",
  "trace_id": "abc123def456...",
  "operation_id": "getUser",
  "details": {
    "user_id": "123"
  }
}
```

### 13.2 Standard Error Codes

| Code                      | HTTP Status | Description                         |
| ------------------------- | ----------- | ----------------------------------- |
| `VALIDATION_ERROR`        | 400         | Request failed validation           |
| `AUTHENTICATION_REQUIRED` | 401         | No valid credentials                |
| `AUTHORIZATION_DENIED`    | 403         | Valid credentials but not permitted |
| `NOT_FOUND`               | 404         | Generic resource not found          |
| `CONFLICT`                | 409         | Resource state conflict             |
| `RATE_LIMITED`            | 429         | Too many requests                   |
| `INTERNAL_ERROR`          | 500         | Unexpected server error             |

### 13.3 Error Schema (JSON Schema)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://themis.somniatore.com/schemas/error.v1.json",
  "type": "object",
  "required": ["code", "message"],
  "properties": {
    "code": {
      "type": "string",
      "pattern": "^[A-Z][A-Z0-9_]*$",
      "description": "Machine-readable error code"
    },
    "message": {
      "type": "string",
      "description": "Human-readable error message"
    },
    "request_id": {
      "type": "string",
      "format": "uuid",
      "description": "Request correlation ID"
    },
    "trace_id": {
      "type": "string",
      "description": "Distributed trace ID"
    },
    "operation_id": {
      "type": "string",
      "description": "Operation that failed"
    },
    "details": {
      "type": "object",
      "additionalProperties": true,
      "description": "Additional structured details"
    }
  }
}
```

---

## 14. Integration Points

### 14.1 Archimedes Integration

```
┌─────────────────┐         ┌─────────────────┐
│    Themis       │         │   Archimedes    │
│   (Toolchain)   │         │    (Runtime)    │
│                 │         │                 │
│  ┌───────────┐  │         │  ┌───────────┐  │
│  │  Artifact │──┼─────────┼─►│  Sentinel │  │
│  │  Builder  │  │ publish │  │  (Loader) │  │
│  └───────────┘  │         │  └───────────┘  │
│                 │         │                 │
│  ┌───────────┐  │         │  ┌───────────┐  │
│  │  CodeGen  │──┼─────────┼─►│  Generated│  │
│  │           │  │  types  │  │   Types   │  │
│  └───────────┘  │         │  └───────────┘  │
└─────────────────┘         └─────────────────┘
```

### 14.2 Eunomia Integration

Eunomia uses contract metadata for policy context:

```rego
# Eunomia policy using Themis operation metadata
package authz

allow {
    input.operation.idempotent == true
    input.caller.role == "reader"
}

allow {
    input.operation.id == "createUser"
    input.caller.role == "admin"
}
```

### 14.3 Stoa Integration

Stoa displays:

- Contract browser (operations, schemas)
- Version history and diffs
- Compliance status (lint results)
- Live validation metrics from Archimedes

---

## 15. Testing Strategy

### 15.1 Unit Tests

```rust
// themis-compat/src/analyzer.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_removed_field_as_breaking() {
        let old = schema! {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "name": { "type": "string" }
            }
        };

        let new = schema! {
            "type": "object",
            "properties": {
                "id": { "type": "string" }
            }
        };

        let result = check_schema_compatibility(&old, &new);

        assert!(result.has_breaking_changes());
        assert!(result.breaking_changes.contains(&BreakingChange::FieldRemoved {
            field: "name".to_string()
        }));
    }

    #[test]
    fn allows_adding_optional_field() {
        let old = schema! {
            "type": "object",
            "required": ["id"],
            "properties": {
                "id": { "type": "string" }
            }
        };

        let new = schema! {
            "type": "object",
            "required": ["id"],
            "properties": {
                "id": { "type": "string" },
                "name": { "type": "string" }
            }
        };

        let result = check_schema_compatibility(&old, &new);

        assert!(!result.has_breaking_changes());
    }
}
```

### 15.2 Integration Tests

```rust
// tests/integration/codegen.rs
#[test]
fn generated_rust_code_compiles() {
    let contract = load_test_contract("users-service-v1.yaml");
    let generated = themis_codegen::rust::generate(&contract).unwrap();

    // Write to temp file and compile
    let temp_dir = tempdir().unwrap();
    let lib_path = temp_dir.path().join("lib.rs");
    fs::write(&lib_path, &generated).unwrap();

    let output = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type", "lib"])
        .arg(&lib_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Generated code failed to compile");
}
```

### 15.3 Contract Conformance Tests

Test that real services match their contracts:

```rust
// tests/conformance/users_service.rs
#[tokio::test]
async fn get_user_matches_contract() {
    let artifact = load_artifact("users-service", "1.0.0");
    let client = TestClient::new("http://localhost:8080");

    // Make real request
    let response = client.get("/users/123").await;

    // Validate response against contract
    let validation = artifact.validate_response(
        "getUser",
        response.status(),
        response.body(),
    );

    assert!(validation.is_valid(), "Response does not match contract: {:?}", validation.errors);
}
```

---

## 16. Open Questions

### Resolved

- ✅ **Primary format**: OpenAPI 3.1 for HTTP, Protobuf v3 for gRPC
- ✅ **Versioning scheme**: Directory-based major versions, semver for minors
- ✅ **Breaking change policy**: Block in CI, require major version bump
- ✅ **Contract ownership**: Team-based ownership via CODEOWNERS
- ✅ **Generated SDK distribution**: Inline generation for MVP, package publishing in V1.1

### Under Discussion

- 🟡 **Contract inheritance**: Allow services to extend base contracts? (Defer to V1.1)
- 🟡 **Deprecation workflow**: 90-day sunset period with warnings
- 🟡 **Multi-service contracts**: One contract per service (shared schemas via $ref)

### Resolved (Post-Review)

- ✅ **Private vs public contracts**: All contracts are internal by default. Public contracts (external API) require explicit annotation (`x-themis-visibility: public`) and additional review. This is a governance concern, not a technical one — all contracts are validated the same way.

---

## 17. Implementation Phases

### Phase 1: Core Toolchain (Weeks 1-4)

- [x] Set up repository structure
  > **Completed 2026-01-04**: Cargo workspace with themis-core, themis-openapi, themis-lint, themis-cli
- [x] Implement `themis-core` (contract model)
  > **Completed 2026-01-04**: Contract, Operation, Schema, Version, Error types with full test coverage (29 tests)
- [x] Implement `themis-openapi` (OpenAPI parsing)
  > **Completed 2026-01-04**: Full parser using openapiv3 crate. Supports all schema types, security schemes, Themis extensions.
- [x] Basic CLI scaffold
  > **Completed 2026-01-04**: CLI with validate, lint, diff commands (stub implementations)

**Deliverable**: Parse and model OpenAPI contracts ✅

### Phase 2: Validation & Linting (Weeks 5-8)

- [ ] Implement `themis-lint` (lint rules)
- [ ] Implement `themis-compat` (compatibility checking)
- [ ] CLI commands: `lint`, `validate`, `diff`

**Deliverable**: Validate contracts and detect breaking changes

### Phase 3: Code Generation (Weeks 9-12)

- [ ] Implement `themis-codegen` (Rust target)
- [ ] TypeScript target
- [ ] Python target
- [ ] CLI command: `codegen`

**Deliverable**: Generate typed code from contracts

### Phase 4: Artifacts & Registry (Weeks 13-16)

- [ ] Implement `themis-artifact` (artifact format)
- [ ] Implement `themis-registry` (registry client)
- [ ] Set up registry infrastructure
- [ ] CLI commands: `publish`, `fetch`

**Deliverable**: Publish and fetch contract artifacts

### Phase 5: CI Integration (Weeks 17-20)

- [ ] GitHub Action implementation
- [ ] GitLab CI template
- [ ] Documentation and examples
- [ ] Migration guides

**Deliverable**: Production-ready CI integration

### Phase 6: Protobuf Support (Weeks 21-24)

- [ ] Implement `themis-protobuf`
- [ ] gRPC code generation
- [ ] Protobuf compatibility checking

**Deliverable**: Full Protobuf/gRPC support

---

## Appendix A: Example Contract Repository

```
users-service/
├── .github/
│   └── workflows/
│       └── contract-ci.yml
├── contracts/
│   ├── v1/
│   │   ├── openapi.yaml
│   │   └── CHANGELOG.md
│   ├── v2/
│   │   ├── openapi.yaml
│   │   └── CHANGELOG.md
│   └── themis.yaml
├── src/
│   ├── generated/          # Auto-generated, gitignored
│   │   ├── types.rs
│   │   ├── errors.rs
│   │   └── handlers.rs
│   ├── handlers/
│   │   ├── get_user.rs
│   │   └── create_user.rs
│   └── main.rs
├── .themis-lint.yaml
├── Cargo.toml
└── archimedes.toml
```

---

_End of Themis Design Document_
