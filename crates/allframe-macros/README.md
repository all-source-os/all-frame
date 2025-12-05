# allframe-macros

**Procedural macros for the AllFrame framework**

[![Crates.io](https://img.shields.io/crates/v/allframe-macros.svg)](https://crates.io/crates/allframe-macros)
[![Documentation](https://docs.rs/allframe-macros/badge.svg)](https://docs.rs/allframe-macros)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

This crate provides procedural macros used by [allframe-core](https://crates.io/crates/allframe-core).

## Installation

You typically don't need to add this crate directly. It's automatically included when you use features that require macros in `allframe-core`:

```toml
[dependencies]
allframe-core = { version = "0.1", features = ["di", "cqrs", "otel"] }
```

## Macros Provided

### `#[di_container]`
Compile-time dependency injection container.

### `#[api]`
API handler generation and routing.

### `#[command]`, `#[event]`, `#[query]`
CQRS command, event, and query markers.

### `#[instrument]`
OpenTelemetry instrumentation.

## Usage

These macros are used through `allframe-core`. See the [allframe-core documentation](https://docs.rs/allframe-core) for usage examples.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Resources

- **AllFrame Core**: https://docs.rs/allframe-core
- **Repository**: https://github.com/all-source-os/all-frame
- **Issues**: https://github.com/all-source-os/all-frame/issues
