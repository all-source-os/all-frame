# Changelog

All notable changes to AllFrame will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.6] - 2025-12-06

### Changed
- **allsource-core upgraded to 0.7.0** - Updated CQRS backend to use latest allsource-core API
  - Migrated to new Event API with `Event::from_strings()` for validated event creation
  - Updated field names (`data` → `payload`)
  - Converted async methods to sync where appropriate
  - Updated `StoreStats` fields (total_entities, total_event_types)

### Fixed
- **Event trait bounds** - Added `Serialize + DeserializeOwned` bounds for proper event serialization

---

## [0.1.5] - 2025-12-06

### Added
- **Zero-warning templates** - `allframe ignite` now generates projects that compile with zero warnings
- **Working Clean Architecture example** - Generated projects include functional Greeter example
  - `Greeter` trait in domain layer
  - `GreetingService` in application layer
  - `ConsoleGreeter` in infrastructure layer
- **Unit tests in templates** - Generated projects include passing tests demonstrating mocking
- **New integration test** - `ignite_creates_project_with_zero_warnings` verifies builds with `-D warnings`

### Fixed
- **Clippy warnings** - Fixed derive for Default, collapsible str::replace calls
- **Template unused imports** - Removed unused `pub use` re-exports from module files

---

## [0.1.4] - 2025-12-06

### Added
- **CLI binary in root crate** - `cargo install allframe` now works correctly
  - Added `allframe-forge` as dependency to root crate
  - Created `src/bin/allframe.rs` binary wrapper
  - Exposed `run()` function from allframe-forge library

### Fixed
- **Graceful shutdown utilities** - Added `GracefulShutdown` and `ShutdownSignal` types
- **`#[derive(GrpcError)]` macro** - Automatic tonic::Status conversion
- **Enhanced `#[traced]` macro** - Configuration options (name, skip, ret, err, level)

---

## [0.1.3] - 2025-12-05

### Added
- Initial crates.io publishing
- allframe-forge CLI for project scaffolding
- allframe-mcp for MCP server integration

---

## [Unreleased]

### Added - Scalar Integration (2025-12-01)

#### Beautiful API Documentation
- **Scalar UI Integration** - Modern OpenAPI 3.1 documentation interface
  - <50KB bundle size (10x smaller than Swagger UI!)
  - Dark mode by default with custom theming support
  - Interactive "Try It" functionality for all endpoints
  - Mobile-friendly responsive design

#### Core Features
- `OpenApiServer` struct for server configuration
- `OpenApiGenerator::with_server()` method for adding server endpoints
- `ScalarConfig` for comprehensive UI customization
  - CDN version pinning for stability
  - SRI (Subresource Integrity) hash support for security
  - Fallback CDN configuration for resilience
  - CORS proxy support for browser compatibility
  - Custom CSS and theming
- `scalar_html()` function for generating documentation HTML

#### Examples
- New `scalar_docs.rs` example (175 lines)
  - Demonstrates all Scalar features
  - Shows 6 REST endpoints with full documentation
  - Includes 3 server configurations
  - Framework integration guide for Axum/Actix/Rocket
- Updated `default_features.rs` to showcase server configuration
- Updated `all_features.rs` to demonstrate complete Scalar integration

#### Documentation
- Comprehensive Scalar Documentation Guide (`docs/guides/SCALAR_DOCUMENTATION.md`) - 500+ lines
  - Quick start guide (4 steps)
  - Complete configuration reference
  - Framework integration examples
  - Advanced usage patterns
  - Troubleshooting section
  - Best practices
- Scalar Integration Complete report (`docs/phases/SCALAR_INTEGRATION_COMPLETE.md`)
- Updated Examples documentation (`docs/phases/EXAMPLES_UPDATED.md`)

#### Testing
- 42 tests for Scalar and OpenAPI features
- 25 Scalar-specific tests covering:
  - Configuration defaults and builder pattern
  - JSON generation
  - HTML generation
  - CDN features
  - SRI hashes
  - Fallback handling
  - Proxy configuration
  - Integration scenarios
- 17 OpenAPI tests covering:
  - Basic generation
  - Route handling
  - Server configuration
  - Schema generation
  - JSON validation

### Added - Binary Size Monitoring (2025-12-01)

#### Automated Monitoring
- **GitHub Actions CI/CD Workflow** for binary size tracking
  - Automated builds for 3 configurations
  - Hard limit enforcement (fails on exceeding targets)
  - Detailed size reporting in CI

#### Local Development Tools
- `scripts/check_size.sh` for local size verification
- `cargo-make` task integration (`cargo make check-size`)
- `cargo-bloat` analysis for size optimization

#### Results
- **All binaries under 2MB** (target was 2-8MB)
  - Minimal config: 1.89MB (target: <2MB) ✅
  - Default features: 1.89MB (target: <5MB) ✅
  - All features: 1.89MB (target: <8MB) ✅
- Exceeded all targets with significant headroom
- Zero-cost abstractions working perfectly

#### Documentation
- Binary Size Monitoring Complete report (`docs/phases/BINARY_SIZE_MONITORING_COMPLETE.md`)

### Updated

#### README.md
- Added Scalar Integration announcement in "What is AllFrame?" section
- Updated "Current Status" to reflect Scalar completion (133 tests passing)
- Added comprehensive Scalar code example in "Core Features"
- Added "Scalar API Documentation" section to documentation index
- Updated roadmap to show Track A (Scalar) and Track B (Binary Size) as complete
- Removed generic Swagger UI references in favor of Scalar

#### Examples
- `default_features.rs`: Now demonstrates server configuration and output
- `all_features.rs`: Now showcases complete Scalar integration with all features

---

## Previous Work (Before Changelog)

### Completed - CQRS Infrastructure (Nov 2025)
- **Phases 1-5** complete with 85% average boilerplate reduction
- CommandBus (90% reduction)
- ProjectionRegistry (90% reduction)
- Event Versioning with auto-upcasting (95% reduction)
- Saga Orchestration with automatic compensation (75% reduction)
- AllSource backend integration

### Completed - Core Features (2024-2025)
- **v0.3**: OpenTelemetry tracing support
- **v0.2**: Compile-time DI + Auto OpenAPI 3.1
- **v0.1**: `allframe ignite` CLI + project scaffolding
- **v0.0**: Repository setup, documentation migration

---

## Statistics

### Test Coverage
- **Total Tests**: 133 passing
- **CQRS Tests**: 72
- **Scalar/OpenAPI Tests**: 42
- **Other Tests**: 19
- **Coverage**: 100% (TDD-enforced)

### Binary Sizes (Release builds)
- Minimal (no features): 1.89MB
- Default features: 1.89MB
- All features: 1.89MB

### Documentation
- **Total Documentation**: 2,500+ lines
- Scalar guides: 675+ lines
- CQRS guides: 1,000+ lines
- Project documentation: 800+ lines

---

## Links

- **Repository**: https://github.com/all-source-os/all-frame
- **Documentation**: https://github.com/all-source-os/all-frame/tree/main/docs
- **Twitter Thread**: See `docs/announcements/TWITTER_THREAD_2025_12_01.md`

---

**AllFrame. One frame. Infinite transformations.**
