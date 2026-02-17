# AllSource-Core Integration Issues

## Status: RESOLVED (v0.7.2)

The `http ^1.2.0` dependency issue has been fixed in allsource-core v0.7.2.

**Fix**: Added explicit `http = "1"` dependency to prevent version conflicts.

AllFrame now supports `cqrs-allsource`, `cqrs-postgres`, and `cqrs-rocksdb` features again.

```toml
[dependencies]
allframe = { version = "0.1", features = ["cqrs-allsource"] }
```

---

## Historical Issues (Now Fixed)

### HTTP Dependency Conflict (Fixed in 0.7.2)

The transitive dependency chain `datafusion 51 → object_store 0.12.4` required `http ^1.2.0` which didn't exist in the registry (only 1.1.0 was available).

**Fix**: allsource-core 0.7.2 added explicit `http = "1"` dependency to ensure compatibility.

---

## Previous Issues (Reference)

The issues below were from an earlier version and may no longer apply:

**Repository**: https://github.com/all-source-os/allsource-monorepo
**Commit**: dd22949a
**AllFrame Feature**: `cqrs-allsource`

---

## Compilation Errors

### 1. Missing Trait Method Implementations

**Error**: `EventStreamRepository` trait has new methods not implemented by backend repositories

**Location**:
- `apps/core/src/infrastructure/repositories/postgres_event_stream_repository.rs:153`
- `apps/core/src/infrastructure/repositories/rocksdb_event_stream_repository.rs:200`

**Missing Methods**:
```rust
async fn get_streams_by_tenant(&self, tenant_id: &crate::domain::value_objects::TenantId) -> Result<Vec<EventStream>>;
async fn count_streams_by_tenant(&self, tenant_id: &crate::domain::value_objects::TenantId) -> Result<usize>;
```

**Affected Implementations**:
- `PostgresEventStreamRepository`
- `RocksDBEventStreamRepository`

**Details**:
```
error[E0046]: not all trait items implemented, missing: `get_streams_by_tenant`, `count_streams_by_tenant`
   --> apps/core/src/infrastructure/repositories/postgres_event_stream_repository.rs:153:1
    |
153 | impl EventStreamRepository for PostgresEventStreamRepository {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    | missing `get_streams_by_tenant`, `count_streams_by_tenant` in implementation
```

**Fix Required**: Implement these two methods in both PostgreSQL and RocksDB repository implementations.

---

### 2. Method Naming Mismatch

**Error**: Code calls `expected_version()` but trait defines `expect_version()`

**Location**: `apps/core/src/infrastructure/repositories/postgres_event_stream_repository.rs:237`

**Current Code**:
```rust
if let Some(expected) = stream.expected_version() {
    // ...
}
```

**Available Method** (from `domain/entities/event_stream.rs:151`):
```rust
pub fn expect_version(&mut self, version: u64) {
    // ...
}
```

**Issue**:
- Calling `expected_version()` (getter) which doesn't exist
- Available method is `expect_version(&mut self, version: u64)` (setter)
- Suggests missing getter method or incorrect usage

**Details**:
```
error[E0599]: no method named `expected_version` found for mutable reference `&'life1 mut EventStream`
   --> apps/core/src/infrastructure/repositories/postgres_event_stream_repository.rs:237:40
    |
237 |         if let Some(expected) = stream.expected_version() {
    |                                        ^^^^^^^^^^^^^^^^ private field, not a method
```

**Fix Required**: Either add `expected_version()` getter method to `EventStream` or refactor code to use correct API.

---

### 3. Missing Error Conversion Trait

**Error**: `AllSourceError` doesn't implement `From<sqlx::Error>`

**Location**: Multiple locations in `postgres_event_stream_repository.rs`

**Affected Code** (lines 328-333, 372-375):
```rust
let partition_id: i32 = row.try_get("partition_id")?;
let current_version: i64 = row.try_get("current_version")?;
let watermark: i64 = row.try_get("watermark")?;
let created_at: DateTime<Utc> = row.try_get("created_at")?;
let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
```

**Current Implementation** (`apps/core/src/error.rs:6`):
```rust
pub enum AllSourceError {
    // Has From implementations for:
    // - arrow::error::ArrowError
    // - parquet::errors::ParquetError
    // - serde_json::Error
    // Missing: sqlx::Error
}
```

**Details**:
```
error[E0277]: `?` couldn't convert the error to `AllSourceError`
   --> apps/core/src/infrastructure/repositories/postgres_event_stream_repository.rs:328:64
    |
328 |             let partition_id: i32 = row.try_get("partition_id")?;
    |                                         -----------------------^
    |                                         the trait `std::convert::From<sqlx::Error>`
    |                                         is not implemented for `AllSourceError`
```

**Fix Required**: Add `From<sqlx::Error>` implementation:
```rust
impl From<sqlx::Error> for AllSourceError {
    fn from(err: sqlx::Error) -> Self {
        AllSourceError::DatabaseError(err.to_string())
    }
}
```

---

## Impact on AllFrame

### Blocked Features
- ❌ `cqrs-allsource` - AllSource embedded database backend
- ❌ `--all-features` builds
- ❌ Full CI/CD testing matrix

### Available Workarounds
- ✅ Default features work: `cargo test`
- ✅ Individual features: `cargo test --features "di,openapi,router,cqrs,otel"`
- ✅ PostgreSQL/RocksDB backends can be excluded

### AllFrame Usage Impact
AllFrame integrates AllSource as an optional CQRS event store backend. Without these fixes, users cannot:
1. Use AllSource embedded database for local development
2. Deploy single-binary applications with built-in event storage
3. Test AllFrame with all backend implementations

---

## Recommended Fixes (Priority Order)

### High Priority
1. **Implement missing trait methods** (breaks compilation)
   - Add `get_streams_by_tenant()` to PostgresEventStreamRepository
   - Add `count_streams_by_tenant()` to PostgresEventStreamRepository
   - Add `get_streams_by_tenant()` to RocksDBEventStreamRepository
   - Add `count_streams_by_tenant()` to RocksDBEventStreamRepository

2. **Add sqlx error conversion** (breaks compilation)
   - Implement `From<sqlx::Error>` for `AllSourceError`

### Medium Priority
3. **Fix method naming issue** (API design)
   - Add `expected_version()` getter to `EventStream`
   - Or refactor callers to use correct API

---

## Temporary AllFrame Solution

Until upstream fixes are available, AllFrame will:
1. Keep `cqrs-allsource` as optional feature
2. Document limitation in README
3. Skip AllSource tests in CI/CD
4. Provide alternative backends (in-memory, PostgreSQL, RocksDB)

---

## Use Cases Blocked

### 1. Embedded Event Store Development
**User Story**: As a developer, I want to run AllFrame locally with an embedded database for rapid prototyping without external dependencies.

**Status**: ❌ Blocked - AllSource cannot compile

**Alternative**: Use in-memory event store (data lost on restart)

### 2. Single-Binary Deployment
**User Story**: As a DevOps engineer, I want to deploy AllFrame as a single binary with embedded event storage for edge deployments.

**Status**: ❌ Blocked - Cannot build with AllSource backend

**Alternative**: Deploy with PostgreSQL/RocksDB (requires separate database service)

### 3. Multi-Tenant Event Streaming
**User Story**: As a SaaS platform, I want to query event streams by tenant for isolation and compliance.

**Status**: ❌ Blocked - `get_streams_by_tenant()` not implemented

**Alternative**: Manual filtering in application code (performance impact)

### 4. Tenant Usage Metrics
**User Story**: As a SaaS admin, I want to count event streams per tenant for billing and capacity planning.

**Status**: ❌ Blocked - `count_streams_by_tenant()` not implemented

**Alternative**: Query all streams and count manually (performance impact)

### 5. Production Observability
**User Story**: As an SRE, I want proper error handling from database operations for alerting and debugging.

**Status**: ❌ Blocked - sqlx errors cannot be converted to AllSourceError

**Alternative**: Generic error messages without database context

---

## Testing Impact

### Cannot Test
- AllSource backend integration
- Multi-tenant event stream queries
- Tenant-based event stream counting
- Database error propagation from sqlx

### Can Test (Workarounds)
- In-memory event store
- Single-tenant scenarios
- Basic event sourcing patterns
- Other AllFrame features (DI, OpenAPI, Router, etc.)

---

## Upstream Issue Tracking

**Recommended Actions**:
1. Open GitHub issue in allsource-monorepo repository
2. Link to this document for detailed error analysis
3. Provide reproduction steps
4. Suggest fixes with code examples

**Issue Title**: "Missing trait implementations and error conversions in EventStreamRepository"

**Labels**: `bug`, `compilation-error`, `api-design`

---

## Version Information

- **AllSource Core**: Commit dd22949a from allsource-monorepo
- **AllFrame**: v0.1.0
- **Rust**: 1.80+
- **sqlx**: 0.7.4
- **Build Date**: 2025-11-27

---

## Contact

For AllFrame-specific questions:
- Repository: https://github.com/all-source-os/all-frame
- Issues: https://github.com/all-source-os/all-frame/issues

For AllSource-Core fixes:
- Repository: https://github.com/all-source-os/allsource-monorepo
- Path: `apps/core/`
