# Themis Contract Governance Action

A GitHub Action for validating, linting, and checking API contract compatibility using Themis.

## Features

- **Validate** - Verify contract schema correctness (OpenAPI, Protobuf, GraphQL, AsyncAPI)
- **Lint** - Check contracts against best practices and governance rules
- **Compat** - Detect breaking changes between contract versions
- **Codegen** - Generate code from contracts (Rust, TypeScript, Python, Go, C++, JSON Schema)

## Quick Start

### Validate an OpenAPI Contract

```yaml
- uses: A-Somniatore/themis/.github/actions/themis-action@main
  with:
    command: validate
    contract: "./api/openapi.yaml"
```

### Lint with All Rules

```yaml
- uses: A-Somniatore/themis/.github/actions/themis-action@main
  with:
    command: lint
    contract: "./api/openapi.yaml"
    fail-on-warnings: "true"
```

### Check Breaking Changes

```yaml
- uses: A-Somniatore/themis/.github/actions/themis-action@main
  with:
    command: compat
    old-contract: "./api/v1/openapi.yaml"
    new-contract: "./api/v2/openapi.yaml"
```

### Generate TypeScript Client

```yaml
- uses: A-Somniatore/themis/.github/actions/themis-action@main
  with:
    command: codegen
    contract: "./api/openapi.yaml"
    language: typescript
    output: "./src/generated"
```

## Inputs

| Input               | Description                                                    | Required | Default       |
| ------------------- | -------------------------------------------------------------- | -------- | ------------- |
| `command`           | Themis command (`validate`, `lint`, `compat`, `codegen`)       | Yes      | `validate`    |
| `contract`          | Path to the contract file                                      | No\*     | -             |
| `old-contract`      | Path to baseline contract (for `compat`)                       | No       | -             |
| `new-contract`      | Path to new contract (for `compat`)                            | No       | -             |
| `format`            | Contract format (`openapi`, `protobuf`, `graphql`, `asyncapi`) | No       | auto-detect   |
| `language`          | Target language for code generation                            | No\*\*   | -             |
| `output`            | Output directory for generated code                            | No       | `./generated` |
| `config`            | Path to Themis configuration file                              | No       | -             |
| `fail-on-warnings`  | Treat warnings as errors                                       | No       | `false`       |
| `rules`             | Comma-separated lint rules or `all`                            | No       | `all`         |
| `allow-breaking`    | Allow breaking changes in compat check                         | No       | `false`       |
| `working-directory` | Working directory for commands                                 | No       | `.`           |
| `version`           | Themis CLI version to use                                      | No       | `latest`      |

\* Required for `validate`, `lint`, and `codegen` commands  
\*\* Required for `codegen` command

## Outputs

| Output             | Description                                          |
| ------------------ | ---------------------------------------------------- |
| `result`           | Command result (`success` or `failure`)              |
| `issues-count`     | Number of issues found (lint command)                |
| `breaking-changes` | Number of breaking changes detected (compat command) |
| `generated-files`  | List of generated files (codegen command)            |

## Complete Workflow Examples

### PR Contract Validation

Validate contracts on every pull request:

```yaml
name: Contract Validation

on:
  pull_request:
    paths:
      - "api/**"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Validate OpenAPI Contract
        uses: A-Somniatore/themis/.github/actions/themis-action@main
        with:
          command: validate
          contract: "./api/openapi.yaml"

      - name: Lint Contract
        uses: A-Somniatore/themis/.github/actions/themis-action@main
        with:
          command: lint
          contract: "./api/openapi.yaml"
          fail-on-warnings: "true"
```

### Breaking Change Detection

Detect breaking changes when contracts are modified:

```yaml
name: Breaking Change Detection

on:
  pull_request:
    paths:
      - "api/**"

jobs:
  compat-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # Need full history for comparison

      - name: Get base branch contract
        run: |
          git show origin/${{ github.base_ref }}:api/openapi.yaml > /tmp/base-contract.yaml

      - name: Check for Breaking Changes
        id: compat
        uses: A-Somniatore/themis/.github/actions/themis-action@main
        with:
          command: compat
          old-contract: "/tmp/base-contract.yaml"
          new-contract: "./api/openapi.yaml"

      - name: Report Breaking Changes
        if: failure()
        run: |
          echo "⚠️ Breaking changes detected!"
          echo "Number of breaking changes: ${{ steps.compat.outputs.breaking-changes }}"
```

### Multi-Format Validation

Validate multiple contract formats:

```yaml
name: Multi-Format Validation

on: [push, pull_request]

jobs:
  validate-contracts:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - contract: "./api/rest/openapi.yaml"
            format: openapi
          - contract: "./api/grpc/service.proto"
            format: protobuf
          - contract: "./api/graphql/schema.graphql"
            format: graphql
          - contract: "./api/events/asyncapi.yaml"
            format: asyncapi

    steps:
      - uses: actions/checkout@v4

      - name: Validate ${{ matrix.format }} Contract
        uses: A-Somniatore/themis/.github/actions/themis-action@main
        with:
          command: validate
          contract: ${{ matrix.contract }}
          format: ${{ matrix.format }}
```

### Code Generation Pipeline

Generate client code when contracts change:

```yaml
name: Generate Client Code

on:
  push:
    branches: [main]
    paths:
      - "api/**"

jobs:
  generate:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        language: [typescript, python, rust]

    steps:
      - uses: actions/checkout@v4

      - name: Generate ${{ matrix.language }} Client
        uses: A-Somniatore/themis/.github/actions/themis-action@main
        with:
          command: codegen
          contract: "./api/openapi.yaml"
          language: ${{ matrix.language }}
          output: "./clients/${{ matrix.language }}"

      - name: Upload Generated Code
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.language }}-client
          path: ./clients/${{ matrix.language }}
```

### Custom Lint Rules

Run specific lint rules:

```yaml
- uses: A-Somniatore/themis/.github/actions/themis-action@main
  with:
    command: lint
    contract: "./api/openapi.yaml"
    rules: "operation-id-required,path-parameters-camel-case,schema-description-required"
```

### Using Configuration File

Use a Themis configuration file:

```yaml
# themis.toml
[lint]
rules = ["all"]
fail_on_warnings = true

[lint.rule.operation-id-required]
enabled = true
severity = "error"

[compat]
allow_experimental_breaking = true
```

```yaml
- uses: A-Somniatore/themis/.github/actions/themis-action@main
  with:
    command: lint
    contract: "./api/openapi.yaml"
    config: "./themis.toml"
```

## Supported Contract Formats

| Format       | Extensions               | Description                |
| ------------ | ------------------------ | -------------------------- |
| OpenAPI 3.x  | `.yaml`, `.yml`, `.json` | REST/HTTP API contracts    |
| Protobuf     | `.proto`                 | gRPC service definitions   |
| GraphQL      | `.graphql`, `.gql`       | GraphQL schemas            |
| AsyncAPI 3.x | `.yaml`, `.yml`, `.json` | Event-driven API contracts |

## Available Lint Rules

### OpenAPI Rules

- `operation-id-required` - All operations must have operationId
- `path-parameters-camel-case` - Path parameters in camelCase
- `schema-description-required` - All schemas need descriptions
- `response-codes-standard` - Use standard HTTP response codes

### Protobuf Rules

- `protobuf-package-name` - Package names follow conventions
- `protobuf-service-name` - Service names in PascalCase

### GraphQL Rules

- `graphql-operation-directive` - Operations have required directives
- `graphql-input-naming` - Input types follow naming conventions

### Security Rules

- `security-schemes-defined` - Security schemes are defined
- `authentication-required` - Endpoints require authentication

### Versioning Rules

- `version-in-path` - API version in path
- `deprecation-planned` - Deprecated items have sunset dates

## Troubleshooting

### Common Issues

**"Contract file not found"**

- Ensure the contract path is relative to the repository root
- Use `working-directory` input if contracts are in a subdirectory

**"Unknown format"**

- Explicitly specify the `format` input
- Check file extension matches supported formats

**"Breaking changes detected"**

- Review the output for specific breaking changes
- Use `allow-breaking: 'true'` if changes are intentional

## License

MIT License - see [LICENSE](../../../LICENSE) for details.
