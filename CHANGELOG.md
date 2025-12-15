# Changelog

All notable changes to AllFrame will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.12] - 2025-12-15

### Added
- **MCP Server Debuggability** - Production-ready stdio transport with comprehensive debugging
  - `StdioTransport` - New production-ready stdio transport with graceful shutdown
  - `StdioConfig` - Builder-style configuration for MCP servers
  - Built-in `allframe/debug` tool - Claude can call to get server diagnostics (uptime, request count, tool count, PID)
  - Request/response tracing for debugging MCP protocol issues
  - Graceful shutdown handling (SIGTERM/SIGINT on Unix, Ctrl+C on Windows)
  - Notification handling (`initialized`, `notifications/cancelled`)
  - Optional `tracing` feature for structured logging with tracing-subscriber

- **Environment Variables for MCP Debugging**
  - `ALLFRAME_MCP_DEBUG=1` - Enable debug output to stderr
  - `ALLFRAME_MCP_LOG_FILE=/path/to/file` - Write logs to file instead of stderr
  - `RUST_LOG=debug` - Log level control (when using `tracing` feature)

- **Claude Code Setup Guide** - Comprehensive documentation for Claude Code CLI
  - Step-by-step `.mcp.json` configuration
  - `.claude/settings.local.json` setup with `enabledMcpjsonServers`
  - Troubleshooting guide for connection issues
  - Manual testing commands for debugging

### Fixed
- **AllSource Core Re-enabled** - `cqrs-allsource` features now work
  - Upstream fix in allsource-core 0.7.2 resolved `http ^1.2.0` dependency conflict
  - `cqrs-allsource`, `cqrs-postgres`, `cqrs-rocksdb` features are functional again
  - Updated to allsource-core 0.7.2

### Documentation
- Added "Quick Start with Claude Code (CLI)" section to MCP README
- Added debugging section with environment variables table
- Added `StdioTransport` and `StdioConfig` to API Overview
- Updated `ALLSOURCE_CORE_ISSUES.md` to mark issue as resolved
- Added social media content for MCP launch (`docs/social/`)

---

## [0.1.9] - 2025-12-13

### Added
- **Protocol-Agnostic Routing Complete** - Full multi-protocol support
  - REST adapter with path parameters and HTTP methods
  - GraphQL adapter with queries, mutations, and schema generation
  - gRPC adapter with all streaming modes and proto generation
  - Single handler exposed via multiple protocols
  - Automatic schema generation (OpenAPI, GraphQL SDL, .proto)
  - Protocol-specific error handling
  - 78 tests across 5 routing phases

- **MCP Server (Zero-Bloat)** - Separate crate for LLM tool integration
  - Auto-discovery: Handlers automatically become MCP tools
  - JSON Schema generation and validation
  - Type coercion (string → number, boolean)
  - Tool listing and invocation
  - Claude Desktop integration ready
  - 37 tests in allframe-mcp crate

- **`KeyedCircuitBreaker<K>`** - Generic keyed circuit breaker for per-resource isolation
  - Independent circuit breaker state per key (e.g., per exchange, per endpoint)
  - Failures in one resource don't affect others
  - Same API as `KeyedRateLimiter<K>` for consistency
  - Full statistics and reset capabilities per key

- **`resilience-redis` Feature** - Redis-backed distributed rate limiting
  - `RedisRateLimiter` with sliding window algorithm
  - `KeyedRedisRateLimiter` with per-key configuration
  - Atomic operations via Lua scripts
  - Works across multiple instances for distributed deployments
  - Auto-cleanup of expired entries

- **Layered Authentication** - Zero-bloat, feature-gated auth infrastructure
  - `auth`: Core traits only (`Authenticator`, `AuthContext`, `AuthError`) - zero deps
  - `auth-jwt`: JWT validation with HS256/RS256/EdDSA support via `jsonwebtoken`
  - `auth-axum`: Axum extractors (`AuthenticatedUser<C>`) and middleware (`AuthLayer`)
  - `auth-tonic`: gRPC interceptors (`AuthInterceptor`) for tonic services
  - Protocol-agnostic design - same claims work across REST/GraphQL/gRPC
  - Environment-based configuration (`JwtConfig::from_env()`)

### Changed
- Updated workspace version to 0.1.9
- Total test count now 455+ (was 361+)
- **Upgraded `thiserror` from 1.0 to 2.0** - Resolves version conflicts with downstream crates

---

## [0.1.8] - 2025-12-09

### Added
- **Graceful Shutdown Utilities** - Production-ready shutdown handling
  - `ShutdownAwareTaskSpawner` for named tasks with automatic cancellation
  - `spawn()` - Spawn tasks that respond to shutdown signals
  - `spawn_background()` - Background tasks (non-blocking)
  - `spawn_with_result()` - Tasks that return values
  - `GracefulShutdownExt` trait for cleanup orchestration
  - `ShutdownExt` trait for making any future cancellable
  - 17 tests for shutdown utilities

### Documentation
- Added `graceful_shutdown.rs` example
- Added `shutdown_patterns.rs` with 5 common patterns

---

## [0.1.7] - 2025-12-08

### Added
- **Comprehensive Resilience Module** (`resilience` feature) - ~1,000+ lines replacing kraken-gateway code
  - `RetryExecutor` - Async retry with exponential backoff and jitter
  - `RetryConfig` - Configurable retry behavior (max_retries, intervals, randomization)
  - `RetryPolicy` trait - Custom retry decision logic
  - `RetryBudget` - System-wide retry token management to prevent retry storms
  - `AdaptiveRetry` - Adjusts retry behavior based on success/failure rates
  - `RateLimiter` - Token bucket rate limiting with burst support
  - `AdaptiveRateLimiter` - Backs off when receiving external 429 responses
  - `KeyedRateLimiter<K>` - Per-key rate limiting (per-endpoint, per-user)
  - `CircuitBreaker` - Fail-fast pattern with Closed/Open/HalfOpen states
  - `CircuitBreakerConfig` - Configurable thresholds and timeouts
  - `CircuitBreakerManager` - Manages multiple circuit breakers by name

- **Security Module** (`security` feature) - Safe logging utilities
  - `obfuscate_url()` - Strips credentials, path, and query from URLs
  - `obfuscate_redis_url()` - Preserves host/port, hides auth
  - `obfuscate_api_key()` - Shows prefix/suffix only (e.g., "sk_l***mnop")
  - `obfuscate_header()` - Smart header obfuscation (Authorization, Cookie, etc.)
  - `Obfuscate` trait - Custom obfuscation for user types
  - `Sensitive<T>` wrapper - Debug/Display always shows "***"
  - `#[derive(Obfuscate)]` macro - Auto-generate Obfuscate impl with `#[sensitive]` fields

- **New Procedural Macros** (allframe-macros)
  - `#[derive(Obfuscate)]` - Auto-generate safe logging with `#[sensitive]` field attribute
  - `#[retry(max_retries = 3)]` - Wrap async functions with exponential backoff
  - `#[circuit_breaker(failure_threshold = 5)]` - Fail-fast pattern for functions
  - `#[rate_limited(rps = 100, burst = 10)]` - Token bucket rate limiting

- **New Feature Flags for Reduced Dependencies** - Modular feature flags
  - `router-grpc-tls` - TLS/mTLS support for gRPC (tonic/tls-ring, rustls-pemfile, tokio-rustls)
  - `http-client` - Re-exports reqwest for HTTP client functionality
  - `otel-otlp` - Full OpenTelemetry stack with OTLP exporter
  - `metrics` - Prometheus metrics support
  - `cache-memory` - In-memory caching with moka and dashmap
  - `cache-redis` - Redis client for distributed caching
  - `rate-limit` - Basic governor re-export
  - `resilience` - Full resilience module (retry, circuit breaker, rate limiting)
  - `security` - URL/credential obfuscation utilities
  - `utils` - Common utilities bundle (chrono, url, parking_lot, rand)

### Changed
- **`health` feature now optional** - `hyper` and `hyper-util` gated behind `health` feature (still in default)
- **CQRS feature flags** - Added `cqrs-allsource`, `cqrs-postgres`, `cqrs-rocksdb` for AllSource backends
- **Legacy alias** - `grpc-tls` is now deprecated, use `router-grpc-tls` instead

### Documentation
- Updated FEATURE_FLAGS.md with comprehensive documentation for all new features

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
- **Total Tests**: 455+ passing
- **CQRS Tests**: 72
- **Router/Protocol Tests**: 78
- **Scalar/OpenAPI Tests**: 42
- **MCP Tests**: 37
- **Resilience/Security Tests**: 55
- **Graceful Shutdown Tests**: 17
- **Other Tests**: 154
- **Coverage**: 100% (TDD-enforced)

### Binary Sizes (Release builds)
- Minimal (no features): 1.89MB
- Default features: 1.89MB
- All features: 1.89MB

### Documentation
- **Total Documentation**: 4,000+ lines
- Scalar guides: 675+ lines
- GraphQL guides: 600+ lines
- CQRS guides: 1,000+ lines
- Routing guides: 500+ lines
- Project documentation: 1,200+ lines

---

## Links

- **Repository**: https://github.com/all-source-os/all-frame
- **Documentation**: https://github.com/all-source-os/all-frame/tree/main/docs
- **Twitter Thread**: See `docs/announcements/TWITTER_THREAD_2025_12_01.md`

---

**AllFrame. One frame. Infinite transformations.**
