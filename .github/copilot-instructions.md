# Copilot Instructions for Themis

## Project Overview

**Themis** is the contract and schema governance system for the Themis Platform. It provides contract-first API development, schema validation, compatibility checking, and code generation for multiple languages.

## Technology Stack

| Area          | Technology                 |
| ------------- | -------------------------- |
| Language      | Rust (latest stable)       |
| Async Runtime | Tokio                      |
| Serialization | Serde                      |
| CLI           | Clap                       |
| OpenAPI       | Custom parser + validation |
| Protobuf      | prost                      |
| GraphQL       | async-graphql              |
| Testing       | Built-in + proptest        |

## Supported Contract Formats

| Format       | Use Case       | Status |
| ------------ | -------------- | ------ |
| OpenAPI 3.1  | REST/HTTP APIs | V1     |
| Protobuf v3  | gRPC services  | V1     |
| GraphQL SDL  | GraphQL APIs   | V1     |
| AsyncAPI 3.0 | Event-driven   | V1     |

## Development Guidelines

### Code Formatting

- Use `rustfmt` with default settings
- Run `cargo fmt` before every commit
- Use `cargo clippy` and fix all warnings (treat warnings as errors in CI)

### Linting Rules

```toml
# Cargo.toml or .cargo/config.toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

### Naming Conventions

- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Crates: `kebab-case` (e.g., `themis-core`)

## Testing Requirements

### CRITICAL: Test-Driven Development

**Every change MUST include tests.** This is non-negotiable.

1. **Before writing code** → Write a failing test first (TDD)
2. **New features** → Add unit tests + integration tests
3. **Bug fixes** → Add regression test that fails before fix
4. **Refactors** → Ensure existing tests still pass
5. **New files** → Every new `.rs` file needs corresponding tests

### Test Structure

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}

// Integration tests in tests/ directory
// tests/integration_test.rs
```

### Contract Parsing Tests

For contract parsing:

```rust
#[test]
fn test_parse_openapi_with_all_operations() {
    let yaml = include_str!("fixtures/complete-api.yaml");
    let contract = parse_openapi(yaml).unwrap();

    assert_eq!(contract.operations.len(), 5);
    assert!(contract.operations.contains_key("getUser"));
}

#[test]
fn test_detect_breaking_change() {
    let old = parse_openapi(include_str!("fixtures/v1.yaml")).unwrap();
    let new = parse_openapi(include_str!("fixtures/v2.yaml")).unwrap();

    let changes = diff_contracts(&old, &new);
    assert!(changes.has_breaking_changes());
}
```

### Run Tests Before Every Push

```bash
# Run ALL of these before pushing
cargo test                        # Unit + integration tests
cargo clippy -- -D warnings       # Linting (fail on warnings)
cargo fmt --check                 # Formatting check
cargo doc --no-deps               # Ensure docs build
```

## Documentation Requirements

### CRITICAL: Document After Every Change

**After writing tests and implementing a change, you MUST add documentation.** This is mandatory, not optional.

#### Development Workflow: Test → Implement → Document

1. Write failing test
2. Implement the change
3. Verify tests pass
4. **Add in-code documentation (rustdoc)**
5. **Update `docs/` for significant changes**

### In-Code Documentation (Always Required)

1. **New function** → Add rustdoc with examples
2. **New module** → Add module-level documentation
3. **New crate** → Update crate-level docs and README
4. **API changes** → Update all affected rustdoc immediately
5. **New types** → Document all public structs, enums, traits

### Documentation in `docs/` (Required for Significant Changes)

Update the `docs/` folder when changes affect:

1. **Contracts & Constraints**

   - New validation rules → Update `docs/spec.md`
   - Changed parsing behavior → Update `docs/design.md`
   - New contract format support → Update both

2. **Breaking Changes**

   - Any breaking change → Document in `docs/` with migration guide
   - Changed error types → Update error documentation

3. **New Features**

   - New CLI commands → Update `docs/roadmap.md` and README
   - New code generation targets → Update `docs/design.md`
   - New linting rules → Document in `docs/spec.md`

4. **Architecture Changes**
   - New crates → Update `docs/design.md`
   - Changed data flow → Update architecture diagrams

### What Counts as "Significant"?

- Changes to public API surface
- New constraints or validation rules
- Behavior changes that affect users
- New capabilities or features
- Deprecations or removals

### Rustdoc Standards

````rust
/// Brief one-line description.
///
/// Longer description if needed, explaining the purpose
/// and any important details.
///
/// # Arguments
///
/// * `contract_path` - Path to the OpenAPI/Protobuf file
///
/// # Returns
///
/// The parsed and validated contract
///
/// # Errors
///
/// Returns `ParseError` if:
/// - File cannot be read
/// - YAML/JSON syntax is invalid
/// - Contract schema validation fails
///
/// # Examples
///
/// ```
/// use themis_openapi::parse;
///
/// let contract = parse("api.yaml")?;
/// println!("Found {} operations", contract.operations.len());
/// ```
pub fn parse(contract_path: &Path) -> Result<Contract, ParseError> {
    // implementation
}
````

## Git Practices

### Commit Often, Push Frequently

- Make small, focused commits
- Each commit should be a logical unit of work
- Push at least at end of each work session
- Never leave work only on local machine

### Commit Message Format

```
type(scope): short description

- Detail 1
- Detail 2

Refs: #issue-number (if applicable)
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code change that neither fixes bug nor adds feature
- `test`: Adding or updating tests
- `chore`: Build, CI, tooling changes

**Examples:**

```
feat(openapi): add $ref resolution for nested schemas

- Resolve internal $ref pointers
- Support external file references
- Handle circular reference detection

test(codegen): add Rust type generation tests

- Test struct generation from schemas
- Test enum generation
- Test nested type handling
```

### Branch Strategy

- `main` – Always deployable, protected
- `feat/description` – Feature branches
- `fix/description` – Bug fix branches
- `docs/description` – Documentation branches

## Dependency Management

### Use Latest Stable Versions

- Always use the latest stable version of dependencies
- Run `cargo update` regularly
- Check for security advisories with `cargo audit`

### Minimize Dependencies

- Prefer well-maintained, audited crates
- Evaluate necessity before adding new dependencies
- Pin versions in `Cargo.lock`

## Error Handling

- Use `Result<T, E>` for fallible operations
- Use `thiserror` for defining error types
- **No `.unwrap()` in library code** (only tests/examples)
- Provide context with error chaining

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to read contract from {path}: {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid OpenAPI schema: {reason}")]
    SchemaError { reason: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },
}
```

## Security Practices

- Never commit secrets or credentials
- Use environment variables for configuration
- All external input is untrusted (validate everything)
- Follow principle of least privilege
- Log security-relevant events

## Performance Considerations

- Profile before optimizing
- Document performance-critical paths
- Add benchmarks for hot paths (`cargo bench`)
- Consider memory allocation patterns
- Code generation should be fast (< 1s for typical contracts)

## Project Structure

```
themis/
├── .github/
│   └── copilot-instructions.md   # This file
├── docs/
│   ├── design.md                 # Implementation design
│   ├── spec.md                   # Specification
│   └── roadmap.md                # Development roadmap
├── README.md
└── CONTRIBUTING.md
```

## Key Reminders

1. **Test everything** – If it's not tested, it's broken
2. **Document immediately** – Don't defer documentation
3. **Format and lint** – Run `cargo fmt` and `cargo clippy` always
4. **Small commits** – Atomic, focused changes
5. **Latest versions** – Keep dependencies up to date
6. **No unsafe code** – Unless absolutely necessary and documented
7. **Error handling** – No `.unwrap()` in production code
8. **Contract-first** – APIs are defined in contracts, not inferred from code

## Terminal Command Guidelines

When running terminal commands:

- **Do NOT use `2>&1`** for stderr redirection – let errors display naturally
- Use `&&` to chain dependent commands
- Prefer absolute paths for file operations
- Run one command at a time for long-running tasks
- For background processes (servers, watch mode), set `isBackground: true`

## CI Checklist

Before creating a PR, ensure:

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] Docs build (`cargo doc --no-deps`)
- [ ] No security vulnerabilities (`cargo audit`)
- [ ] Documentation updated for changes
- [ ] Commit messages follow format
