# Themis

**Contract and Schema Governance System for the Themis Platform**

Themis is the contract-first API governance toolchain that provides:

- 📝 **Contract Definition** – OpenAPI 3.1, Protobuf v3, GraphQL SDL, AsyncAPI 3.0
- ✅ **Validation** – Schema validation and linting
- 🔄 **Compatibility** – Breaking change detection
- ⚡ **Code Generation** – Types, clients, handlers for Rust, TypeScript, Python, C++
- 📦 **Artifact Publishing** – Immutable contract artifacts

## Quick Links

- [Design Document](docs/design.md)
- [Specification](docs/spec.md)
- [Roadmap](docs/roadmap.md)
- [Contributing](CONTRIBUTING.md)
- [Integration Specification](../docs/integration/integration-spec.md) – Shared schemas with Archimedes/Eunomia
- [Themis Platform](../)

## Quick Start

### Prerequisites

- Rust 1.75+ ([install](https://rustup.rs/))

### Installation

```bash
# Clone the repository
git clone https://github.com/A-Somniatore/themis.git
cd themis

# Build
cargo build --release

# Install CLI
cargo install --path crates/themis-cli

# Verify
themis --help
```

### Usage

```bash
# Validate a contract
themis validate ./examples/users-service/v1/openapi.yaml

# Lint a contract
themis lint ./examples/users-service/v1/openapi.yaml

# Compare versions for breaking changes
themis diff ./examples/users-service/v1/openapi.yaml ./examples/users-service/v2/openapi.yaml

# Generate Rust code from a contract
themis codegen ./examples/users-service/v1/openapi.yaml -o ./generated

# Generate with validation derives
themis codegen ./api/openapi.yaml --include-validation -o ./src/gen

# Dry run to preview generated files
themis codegen ./api/openapi.yaml --dry-run
```

## Documentation

- [Development Guide](docs/development.md) - Local setup and development workflow
- [Design Document](docs/design.md) - Implementation design
- [Specification](docs/spec.md) - Contract governance specification
- [Roadmap](docs/roadmap.md) - Development roadmap
- [Contributing](CONTRIBUTING.md) - Contribution guidelines

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Themis System                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Contract    │───▶│   Validate   │───▶│   Publish    │       │
│  │   (Git)      │    │   & Lint     │    │  Artifact    │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                             │                    │               │
│                             ▼                    ▼               │
│                      ┌──────────────┐    ┌──────────────┐       │
│                      │   CodeGen    │    │   Registry   │       │
│                      │              │    │              │       │
│                      └──────────────┘    └──────────────┘       │
│                             │                    │               │
│              ┌──────────────┼──────────────┬─────┘               │
│              ▼              ▼              ▼                     │
│       ┌──────────┐   ┌──────────┐   ┌──────────┐               │
│       │   Rust   │   │TypeScript│   │  Python  │               │
│       │  Types   │   │  Client  │   │ Handlers │               │
│       └──────────┘   └──────────┘   └──────────┘               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Features

### Contract Formats

- **OpenAPI 3.1** – Full support with Themis extensions
- **Protobuf v3** – gRPC service definitions
- **GraphQL SDL** – Schema-first GraphQL
- **AsyncAPI 3.0** – Event-driven APIs (Kafka, AMQP)

### Validation & Linting

- Schema correctness validation
- Naming convention enforcement
- Security scheme validation
- Documentation completeness checks

### Compatibility Checking

- Breaking change detection
- Semantic versioning enforcement
- Diff reports between versions

### Code Generation

- **Rust**: Structs, handlers, error types
- **TypeScript**: Interfaces, fetch client, Express handlers
- **Python**: Dataclasses, httpx client, FastAPI handlers
- **C++**: Structs, cpr/libcurl client

## Project Structure

```
themis/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                # CI pipeline
│   │   └── release.yml           # Release automation
│   └── copilot-instructions.md   # AI assistant guidelines
├── docs/
│   ├── design.md                 # Implementation design
│   ├── development.md            # Development guide
│   ├── spec.md                   # Specification
│   └── roadmap.md                # Development roadmap
├── crates/
│   ├── themis-core/              # Core types and traits
│   ├── themis-openapi/           # OpenAPI 3.1 parser
│   ├── themis-lint/              # Linting rules
│   └── themis-cli/               # CLI application
├── examples/                     # Example contracts
│   └── users-service/
│       └── v1/
│           └── openapi.yaml
├── Cargo.toml                    # Workspace configuration
├── README.md
└── CONTRIBUTING.md
```

## CLI Commands

| Command           | Description                              | Status       |
| ----------------- | ---------------------------------------- | ------------ |
| `themis validate` | Validate contract syntax and schema      | 🚧 Week 4    |
| `themis lint`     | Run linting rules                        | 🚧 Week 5    |
| `themis diff`     | Compare two contract versions            | 🚧 Week 6    |
| `themis codegen`  | Generate code (Rust, TypeScript, Python) | 🚧 Week 7-10 |
| `themis publish`  | Publish artifact to registry             | 🚧 Week 11   |
| `themis fetch`    | Fetch artifact from registry             | 🚧 Week 12   |

## Related Projects

- **[Eunomia](../eunomia/)** – Authorization policy platform
- **[Archimedes](../docs/components/archimedes-design.md)** – HTTP/gRPC server framework
- **[Stoa](../docs/components/stoa-design.md)** – Web UI for service governance

## License

Apache License 2.0 - See [ADR-007](../docs/decisions/007-apache-2-license.md) for rationale.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.
