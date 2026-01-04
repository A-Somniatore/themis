# Themis Development Guide

This guide covers setting up your local development environment for Themis.

## Prerequisites

### Required Software

| Software | Version | Purpose          |
| -------- | ------- | ---------------- |
| Rust     | 1.75+   | Core development |
| Git      | 2.x     | Version control  |

### Optional Software

| Software | Version | Purpose           |
| -------- | ------- | ----------------- |
| Docker   | 24.x    | Container testing |
| Just     | 1.x     | Task runner       |

## Installation

### 1. Install Rust

Install Rust using rustup (recommended):

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then restart your terminal or run:
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

For Windows, download the installer from [rustup.rs](https://rustup.rs/).

### 2. Clone the Repository

```bash
git clone https://github.com/A-Somniatore/themis.git
cd themis
```

### 3. Build the Project

```bash
# Check that everything compiles
cargo check

# Build all crates
cargo build

# Build release version
cargo build --release
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific crate tests
cargo test -p themis-core
```

## Development Workflow

### Daily Development

```bash
# Format code (do this before every commit)
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Build docs
cargo doc --no-deps --open
```

### Pre-Commit Checklist

Before committing, always run:

```bash
cargo fmt --check      # Check formatting
cargo clippy -- -D warnings  # Linting
cargo test            # All tests pass
cargo doc --no-deps   # Docs build
```

### Running the CLI

```bash
# Build and run
cargo run -p themis-cli -- --help

# Or install locally
cargo install --path crates/themis-cli
themis --help

# Example commands
themis validate ./examples/users-service/v1/openapi.yaml
themis lint ./examples/users-service/v1/openapi.yaml
themis diff ./examples/users-service/v1/openapi.yaml ./examples/users-service/v2/openapi.yaml
```

## Project Structure

```
themis/
├── Cargo.toml                # Workspace root
├── crates/
│   ├── themis-core/          # Core types and traits
│   │   ├── src/
│   │   │   ├── lib.rs        # Crate root
│   │   │   ├── contract.rs   # Contract model
│   │   │   ├── operation.rs  # Operation definitions
│   │   │   ├── schema.rs     # Schema types
│   │   │   ├── version.rs    # Semantic versioning
│   │   │   └── error.rs      # Error types
│   │   └── Cargo.toml
│   ├── themis-openapi/       # OpenAPI 3.1 parser
│   ├── themis-lint/          # Linting rules
│   └── themis-cli/           # CLI application
├── docs/                     # Documentation
├── examples/                 # Example contracts
└── tests/                    # Integration tests
```

## Coding Standards

### Formatting

We use `rustfmt` with default settings:

```bash
cargo fmt
```

### Linting

We use Clippy with strict settings:

```bash
cargo clippy -- -D warnings
```

Our workspace enforces these lints:

- `unsafe_code = "forbid"` - No unsafe code allowed
- `missing_docs = "warn"` - All public items should be documented
- Clippy `pedantic` and `nursery` lints enabled

### Testing

Every change must include tests:

1. **Unit tests** - In the same file as the code
2. **Integration tests** - In `tests/` directory
3. **Doc tests** - In rustdoc examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

## IDE Setup

### VS Code

Recommended extensions:

- `rust-analyzer` - Rust language support
- `Even Better TOML` - TOML file support
- `crates` - Dependency version management

Settings (`.vscode/settings.json`):

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### JetBrains (RustRover/IntelliJ)

- Install the Rust plugin
- Enable "Run rustfmt on save"
- Enable "Run clippy on save"

## Troubleshooting

### "cargo: command not found"

Make sure Rust is installed and the cargo bin directory is in your PATH:

```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```

### "linking with cc failed"

Install Xcode Command Line Tools (macOS):

```bash
xcode-select --install
```

### Compilation Errors

Try cleaning and rebuilding:

```bash
cargo clean
cargo build
```

## Next Steps

See the [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.
