# AllFrame Quality Gates

This document describes the quality gates and development workflows for AllFrame.

**Status**: All quality gates integrated and operational.

## Tools

AllFrame uses standard Rust ecosystem tools for quality assurance:

- **rustfmt** - Code formatting
- **clippy** - Linting and code quality
- **cargo-sort** - Dependency organization
- **cargo-make** - Task runner (optional)

## Installation

```bash
# Install required tools
cargo install cargo-sort
cargo install cargo-bloat  # For binary size monitoring

# Optional: Install cargo-make for task automation
cargo install cargo-make
```

## Usage

### Option 1: Cargo Aliases (Built-in)

The simplest way - uses built-in Cargo alias feature:

```bash
# Linting
cargo lint              # Run clippy with -D warnings
cargo lint-check        # Check code formatting
cargo lint-sort         # Check dependency sorting

# Formatting
cargo format-code       # Format all Rust code
cargo format-sort       # Sort dependencies in Cargo.toml
```

### Option 2: cargo-make (Recommended)

More powerful task runner with dependencies:

```bash
# Show all available tasks
cargo make help

# Quality Gates
cargo make lint         # Check formatting + run clippy
cargo make lint-check   # Check code formatting
cargo make lint-sort    # Check Cargo.toml sorting
cargo make format       # Format all code
cargo make format-sort  # Sort Cargo.toml

# Testing
cargo make test         # Run tests with main features
cargo make test-all     # Run tests with all features
cargo make test-minimal # Run tests with no features

# Building
cargo make build         # Debug build
cargo make build-release # Release build
cargo make clean         # Clean artifacts

# Size Monitoring
cargo make size-minimal  # Measure minimal binary
cargo make size-default  # Measure default features
cargo make size-all      # Measure all features

# CI/CD
cargo make ci           # Run all CI checks (lint + test + build)
```

### Option 3: Direct cargo commands

Run commands directly:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --features "di,openapi,router,cqrs,otel,mcp" -- -D warnings

# Sort dependencies
cargo sort -w

# Check if sorted
cargo sort --check

# Run tests
cargo test --features "di,openapi,router,cqrs"
```

## Offline Quality Gates

AllFrame supports offline-first and offline-only desktop/embedded deployments ([Issue #36](https://github.com/all-source-os/all-frame/issues/36)). These quality gates ensure the offline story remains clean.

### Quick Commands

```bash
# Run offline quality gate tests
cargo make test-offline

# Run allframe-tauri tests
cargo make test-tauri

# Verify no network deps in offline builds
cargo make check-offline-deps

# Run all offline CI checks
cargo make ci-offline
```

### Offline Feature Profiles

| Profile | Features | Use Case |
|---------|----------|----------|
| `offline-only-minimal` | `cqrs,di` | Air-gapped, smallest footprint |
| `offline-first-desktop` | `cqrs,di,resilience,security` | Tauri desktop apps |
| `offline-with-router` | `router,cqrs,di` | Tauri IPC + event sourcing |
| `offline-with-auth` | `cqrs,di,auth,auth-jwt,security` | Offline with local JWT |
| `offline-with-cache` | `cqrs,di,cache-memory` | Offline with in-memory cache |

### Network Dependency Boundaries

**Network-free features** (safe for offline-only):
- `cqrs` — InMemoryBackend, no external store
- `di` — Compile-time dependency injection
- `router` — In-memory handler registry (no HTTP)
- `openapi` — Spec generation only
- `resilience` — Local retry/circuit breaker
- `security` — Logging utilities
- `auth` — Trait definitions only
- `auth-jwt` — Local token validation
- `cache-memory` — In-process cache
- `rate-limit` — Local rate limiting

**Network-dependent features** (exclude for offline-only):
- `health` → hyper
- `otel-otlp` → opentelemetry-otlp
- `http-client` → reqwest
- `cache-redis` → redis
- `resilience-redis` → redis
- `router-grpc` → tonic
- `router-grpc-tls` → rustls
- `auth-axum` → hyper
- `auth-tonic` → tonic

### CI Enforcement

The `offline-quality-gates.yml` workflow validates:
1. All offline feature profiles compile on Linux and macOS
2. `allframe-tauri` builds, tests, and passes clippy
3. No network dependencies (`reqwest`, `redis`, `hyper`, `tonic`, `opentelemetry-otlp`) in offline builds
4. Quality gate integration tests pass (`tests/09_offline_quality_gates.rs`)
5. Binary size stays under 5MB for offline profile

## Configuration Files

Quality gates are configured through standard Rust configuration files:

### `.clippy.toml`
```toml
msrv = "1.70.0"
warn-on-all-wildcard-imports = true
```

### `rustfmt.toml`
```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
reorder_imports = true
reorder_modules = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### `cargo-sort.toml`
```toml
sort = true
expand = true
group_imports = true
```

## CI/CD Integration

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# AllFrame pre-commit hook

echo "Running quality gates..."

# Format check
cargo fmt --check || {
    echo "❌ Code formatting check failed. Run: cargo fmt"
    exit 1
}

# Clippy
cargo clippy --features "di,openapi,router,cqrs,otel,mcp" -- -D warnings || {
    echo "❌ Clippy check failed. Fix warnings and try again."
    exit 1
}

# Dependency sorting
cargo sort --check || {
    echo "❌ Dependencies not sorted. Run: cargo sort -w"
    exit 1
}

echo "✅ All quality gates passed!"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

### GitHub Actions

Example workflow (`.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Install tools
        run: cargo install cargo-sort

      - name: Format check
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy --features "di,openapi,router,cqrs,otel,mcp" -- -D warnings

      - name: Sort check
        run: cargo sort --check

      - name: Tests
        run: cargo test --features "di,openapi,router,cqrs"

      - name: Build
        run: cargo build --release
```

## Quality Standards

### Code Formatting
- **Line width**: 100 characters
- **Indentation**: 4 spaces (no tabs)
- **Newlines**: Unix (LF)
- **Import organization**: Grouped by std/external/crate

### Linting
- **Zero warnings**: All clippy warnings must be addressed
- **MSRV**: Minimum Rust version 1.70.0
- **No wildcard imports**: Use explicit imports

### Testing
- **Coverage**: Aim for 100% test coverage
- **Feature testing**: Test with different feature combinations
- **TDD**: Write tests first (RED-GREEN-REFACTOR)

### Dependencies
- **Sorted**: Dependencies must be alphabetically sorted
- **Minimal**: Only include necessary dependencies
- **Version control**: Pin versions in Cargo.lock

## Troubleshooting

### "cargo: command not found"
```bash
cargo install cargo-make
```

### "cargo-sort: command not found"
```bash
cargo install cargo-sort
```

### Clippy errors
Fix all warnings before committing:
```bash
cargo clippy --fix --allow-dirty --allow-staged
```

### Format issues
Auto-format all code:
```bash
cargo fmt
```

### Unsorted dependencies
Auto-sort dependencies:
```bash
cargo sort -w
```

## Best Practices

1. **Always run `cargo make lint` before committing**
2. **Run `cargo make test` to ensure tests pass**
3. **Use `cargo make ci` to simulate CI/CD locally**
4. **Keep dependencies sorted** with `cargo sort -w`
5. **Format code automatically** with `cargo fmt`

## Integration with IDEs

### VS Code
Install extensions:
- `rust-analyzer` - Language server
- `Even Better TOML` - TOML support
- `crates` - Dependency management

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": [
    "--features", "di,openapi,router,cqrs,otel,mcp",
    "--", "-D", "warnings"
  ],
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ IDEA / RustRover
- Enable "Rustfmt" in Settings → Languages & Frameworks → Rust
- Enable "External Linter" → Clippy
- Set "Run clippy on save"

---

**AllFrame. One frame. Infinite transformations.**
*Built with quality, from day zero.* 🦀
