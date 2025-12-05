# allframe-forge

**AllFrame CLI - Project scaffolding and code generation**

[![Crates.io](https://img.shields.io/crates/v/allframe-forge.svg)](https://crates.io/crates/allframe-forge)
[![Documentation](https://docs.rs/allframe-forge/badge.svg)](https://docs.rs/allframe-forge)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

The AllFrame CLI tool for creating and managing AllFrame projects.

## Installation

```bash
cargo install allframe-forge
```

## Usage

### Create New Project

```bash
# Create a new AllFrame project
allframe ignite my-api

# Navigate to the project
cd my-api

# Run the API
cargo run
```

### Project Structure

The generated project includes:

```
my-api/
├── Cargo.toml
├── src/
│   ├── main.rs           # Application entry point
│   ├── handlers/         # API handlers
│   ├── domain/           # Domain models
│   └── infrastructure/   # Database, config, etc.
├── tests/                # Integration tests
└── examples/             # Usage examples
```

### Features

- ✅ **Clean Architecture** - Proper separation of concerns
- ✅ **Protocol-Agnostic** - REST, GraphQL, gRPC support
- ✅ **CQRS Ready** - Command/Query separation
- ✅ **Test Setup** - Pre-configured test infrastructure
- ✅ **Best Practices** - Industry-standard project structure

## Commands

```bash
# Create new project
allframe ignite <name>

# Generate handler
allframe generate handler <name>

# Generate command (CQRS)
allframe generate command <name>

# Generate query (CQRS)
allframe generate query <name>
```

## Project Templates

### Minimal
```bash
allframe ignite my-api --template minimal
# Simple REST API with basic routing
```

### Full
```bash
allframe ignite my-api --template full
# Complete setup with REST, GraphQL, gRPC, and CQRS
```

### CQRS
```bash
allframe ignite my-api --template cqrs
# CQRS-focused architecture with event sourcing
```

## Configuration

Generated projects use `allframe-core` with sensible defaults:

```toml
[dependencies]
allframe-core = { version = "0.1", features = [
    "di",
    "openapi",
    "router",
    "otel"
] }
```

## Development

After creating a project:

```bash
# Run the API
cargo run

# Run tests
cargo test

# Run with specific features
cargo run --features graphql,grpc

# View API documentation
# Visit http://localhost:8080/docs after starting
```

## Examples

See the [examples/](../../examples/) directory in the AllFrame repository for complete project examples.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Resources

- **AllFrame Core**: https://crates.io/crates/allframe-core
- **Documentation**: https://docs.rs/allframe-forge
- **Repository**: https://github.com/all-source-os/all-frame
- **Issues**: https://github.com/all-source-os/all-frame/issues
