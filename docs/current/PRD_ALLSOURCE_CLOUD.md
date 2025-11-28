# PRD: AllSource Cloud-Ready Evolution

**Title**: AllSource - Cloud-Native Event Store for Serverless and Multi-Tenant Architectures
**Version**: 1.0
**Status**: Draft
**Priority**: P0
**Author**: AllFrame Team
**Created**: 2025-11-27
**Last Updated**: 2025-11-27

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current State Analysis](#2-current-state-analysis)
3. [Goals & Non-Goals](#3-goals--non-goals)
4. [Market Context](#4-market-context)
5. [Technical Requirements](#5-technical-requirements)
6. [Architecture](#6-architecture)
7. [Cloud Backend Implementations](#7-cloud-backend-implementations)
8. [Multi-Tenancy Design](#8-multi-tenancy-design)
9. [Serverless Optimization](#9-serverless-optimization)
10. [API Design](#10-api-design)
11. [Migration Path](#11-migration-path)
12. [Timeline & Milestones](#12-timeline--milestones)
13. [Success Criteria](#13-success-criteria)
14. [Risks & Mitigations](#14-risks--mitigations)
15. [Appendix](#appendix)

---

## 1. Executive Summary

AllSource is the embedded event store powering AllFrame's CQRS+ES infrastructure. This PRD defines the evolution from a local-first embedded database to a **cloud-native, serverless-ready, multi-tenant event store** that can run on AWS, GCP, and self-hosted environments.

### Vision

**AllSource becomes the unified event store abstraction** that:
- Runs embedded for local development (current capability)
- Deploys to DynamoDB/Firestore for serverless workloads
- Scales horizontally for multi-tenant SaaS
- Maintains the same API regardless of backend

### Key Outcomes

| Outcome | Metric |
|---------|--------|
| Cloud-native deployment | Works on Lambda + DynamoDB |
| Multi-tenant support | Row-level isolation, per-tenant queries |
| Zero code changes | Same EventStore API across all backends |
| Sub-10ms latency | Matches current AllSource performance |
| Cost efficiency | Pay-per-request pricing on serverless |

---

## 2. Current State Analysis

### 2.1 What AllSource Has Today

| Feature | Status | Notes |
|---------|--------|-------|
| Event ingestion | ✅ | 469K events/sec |
| Event querying | ✅ | 11.9μs p99 latency |
| Parquet storage | ✅ | Columnar format |
| Write-Ahead Log | ✅ | Durability guarantee |
| Snapshots | ✅ | Aggregate state caching |
| PostgreSQL backend | ⚠️ | Compilation errors |
| RocksDB backend | ⚠️ | Compilation errors |
| Multi-tenant queries | ⚠️ | Trait defined, not implemented |

### 2.2 Current Issues (Blocking)

From `docs/current/ALLSOURCE_CORE_ISSUES.md`:

1. **Missing trait implementations**:
   - `get_streams_by_tenant()` not implemented
   - `count_streams_by_tenant()` not implemented

2. **API inconsistencies**:
   - `expected_version()` getter missing from EventStream

3. **Error handling gaps**:
   - `From<sqlx::Error>` not implemented for AllSourceError

### 2.3 What's Missing for Cloud

| Gap | Impact |
|-----|--------|
| DynamoDB backend | Cannot deploy on AWS serverless |
| Firestore backend | Cannot deploy on GCP serverless |
| S3/GCS archival | No cold storage tier |
| Multi-tenant isolation | SaaS platforms blocked |
| Connection pooling | Lambda cold start overhead |
| Serverless-optimized queries | Scan-heavy, expensive |

---

## 3. Goals & Non-Goals

### Goals

| Goal | Success Metric |
|------|----------------|
| Fix compilation errors | `cargo build --all-features` succeeds |
| DynamoDB backend | Full EventStoreBackend implementation |
| Firestore backend | Full EventStoreBackend implementation |
| Multi-tenant support | Tenant isolation in all backends |
| S3/GCS archival | Cold storage for old events |
| Serverless optimization | < 50ms cold start contribution |
| Connection reuse | Pool connections across invocations |

### Non-Goals

- **Global replication**: Multi-region is future phase
- **Real-time streaming**: EventBridge/Pub-Sub handles this
- **GraphQL API**: AllFrame Router provides this
- **Custom query language**: Use backend-native queries
- **ACID across aggregates**: Event sourcing is eventually consistent

---

## 4. Market Context

### 4.1 Event Store Landscape 2025

| Solution | Type | Serverless | Multi-Tenant | Rust SDK |
|----------|------|------------|--------------|----------|
| EventStoreDB | Dedicated | ❌ | ❌ | ❌ |
| DynamoDB | Managed | ✅ | ✅ | ✅ |
| Firestore | Managed | ✅ | ✅ | ⚠️ |
| Kafka | Streaming | ❌ | ✅ | ✅ |
| **AllSource** | Embedded + Cloud | ✅ | ✅ | ✅ |

Sources: [AWS CQRS Event Store](https://aws.amazon.com/blogs/database/build-a-cqrs-event-store-with-amazon-dynamodb/), [EventStoreDB Cloud](https://aws.amazon.com/marketplace/pp/prodview-kxo6grvoovk2y)

### 4.2 Why DynamoDB for Serverless Event Sourcing

> "DynamoDB offers performance benefits associated with NoSQL technology. An event store design is not characterized by joins between tables or relations, which makes it an ideal candidate for DynamoDB." — [AWS Database Blog](https://aws.amazon.com/blogs/database/build-a-cqrs-event-store-with-amazon-dynamodb/)

> "MongoDB needs to keep its working set in RAM for acceptable performance, which makes it too expensive for event sourcing. In contrast, DynamoDB's great performance is not dependent on the amount of data stored." — [The Mill Adventure](https://aws.amazon.com/blogs/architecture/how-the-mill-adventure-implemented-event-sourcing-at-scale-using-dynamodb/)

### 4.3 Multi-Tenant Patterns

> "For pooled or bridge models, the most battle-tested pattern is Postgres with Row Level Security and a strict tenant context." — [Multi-Tenant SaaS Architecture 2025](https://isitdev.com/multi-tenant-saas-architecture-cloud-2025/)

AllSource should support:
- **Pooled**: Single table, tenant_id column, RLS
- **Siloed**: Separate tables per tenant
- **Bridge**: Shared infrastructure, isolated data

---

## 5. Technical Requirements

### TR-1: Fix Existing Compilation Errors

**Priority**: P0 (Blocking)

**Requirements**:
1. Implement `get_streams_by_tenant()` for PostgreSQL and RocksDB backends
2. Implement `count_streams_by_tenant()` for PostgreSQL and RocksDB backends
3. Add `expected_version()` getter to EventStream
4. Implement `From<sqlx::Error>` for AllSourceError

**Acceptance Criteria**:
- `cargo build --all-features` succeeds
- All existing tests pass
- No new warnings

### TR-2: DynamoDB Backend

**Priority**: P0

**Requirements**:
1. Implement `EventStoreBackend` for DynamoDB
2. Single-table design with PK (stream_id) and SK (version)
3. Optimistic concurrency via conditional writes
4. DynamoDB Streams integration for projections
5. Auto-scaling configuration
6. GSI for tenant-based queries

**Schema**:
```
Table: {prefix}_events
├── PK (String): {tenant_id}#{stream_id}
├── SK (String): {version:020}  # Zero-padded for sorting
├── event_type (String): Discriminator
├── payload (String): JSON event data
├── timestamp (String): ISO 8601
├── metadata (Map): Custom metadata
└── ttl (Number): Optional expiration

GSI: by_tenant
├── PK: tenant_id
├── SK: timestamp
└── Projects: stream_id, event_type
```

### TR-3: Firestore Backend

**Priority**: P1

**Requirements**:
1. Implement `EventStoreBackend` for Firestore
2. Collection hierarchy: tenants/{tenant}/streams/{stream}/events/{version}
3. Firestore transactions for optimistic concurrency
4. Firestore triggers for projections
5. Security rules for tenant isolation

**Schema**:
```
Collection: tenants
└── Document: {tenant_id}
    └── Collection: streams
        └── Document: {stream_id}
            ├── current_version: number
            ├── created_at: timestamp
            └── Collection: events
                └── Document: {version}
                    ├── event_type: string
                    ├── payload: map
                    ├── timestamp: timestamp
                    └── metadata: map
```

### TR-4: S3/GCS Archival Backend

**Priority**: P1

**Requirements**:
1. Archive old events to object storage
2. Configurable retention policies
3. Lazy loading for archived events
4. Parquet format for analytics
5. Lifecycle rules integration

**Archive Strategy**:
```
Hot tier (DynamoDB/Firestore): Last 30 days
Warm tier (S3/GCS): 30 days - 1 year
Cold tier (S3 Glacier/GCS Archive): > 1 year
```

### TR-5: Multi-Tenant Support

**Priority**: P0

**Requirements**:
1. Tenant context injection at request level
2. Automatic tenant_id in all queries
3. Cross-tenant query prevention
4. Per-tenant statistics
5. Tenant-based rate limiting (optional)

**API**:
```rust
// Tenant context
let store = EventStore::with_tenant("tenant-123");

// All operations scoped to tenant
store.append("order-1", events).await?;  // Stored as tenant-123#order-1
store.get_events("order-1").await?;      // Only returns tenant-123's events

// Admin operations (cross-tenant)
let admin_store = EventStore::admin();
admin_store.get_streams_by_tenant("tenant-123").await?;
admin_store.count_streams_by_tenant("tenant-123").await?;
```

### TR-6: Connection Pooling for Lambda

**Priority**: P0

**Requirements**:
1. Reuse connections across Lambda invocations
2. Lazy initialization (don't connect until needed)
3. Connection keep-alive configuration
4. Graceful connection recovery
5. Metrics for connection reuse rate

**Implementation**:
```rust
// Static connection pool (lives across invocations)
static POOL: OnceCell<DynamoDBPool> = OnceCell::new();

pub async fn get_connection() -> &'static DynamoDBPool {
    POOL.get_or_init(|| async {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        DynamoDBPool::new(&config)
    }).await
}
```

### TR-7: Serverless Query Optimization

**Priority**: P1

**Requirements**:
1. Avoid full table scans
2. Use GSI for common access patterns
3. Projection expressions (return only needed fields)
4. Pagination with consistent reads
5. Query cost estimation

**Access Patterns**:
| Pattern | Solution |
|---------|----------|
| Get events for stream | Query by PK |
| Get events after version | Query by PK + SK >= version |
| Get streams for tenant | GSI query by tenant_id |
| Count streams for tenant | GSI count (or cached) |
| Get recent events | GSI by timestamp |

---

## 6. Architecture

### 6.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         AllSource Cloud Architecture                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────┐                                                    │
│  │   AllFrame      │                                                    │
│  │   Application   │                                                    │
│  └────────┬────────┘                                                    │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────┐                                                    │
│  │  EventStore<E>  │  Unified API                                       │
│  │  with_backend() │                                                    │
│  └────────┬────────┘                                                    │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    EventStoreBackend Trait                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│           │                                                              │
│     ┌─────┼─────┬─────────┬─────────┬─────────┬─────────┐              │
│     ▼     ▼     ▼         ▼         ▼         ▼         ▼              │
│  ┌─────┐┌─────┐┌───────┐┌───────┐┌───────┐┌───────┐┌───────┐          │
│  │InMem││AllSrc││DynamoDB││Firestore││Postgres││RocksDB││ S3/GCS │     │
│  │     ││Core ││       ││        ││       ││      ││Archive│          │
│  └─────┘└─────┘└───────┘└────────┘└───────┘└──────┘└───────┘          │
│     │      │       │         │         │        │        │              │
│     ▼      ▼       ▼         ▼         ▼        ▼        ▼              │
│  ┌─────┐┌─────┐┌───────┐┌───────┐┌───────┐┌──────┐┌───────┐           │
│  │ RAM ││Parquet││ AWS  ││  GCP  ││  SQL  ││ LSM  ││Object │           │
│  │     ││+ WAL ││      ││       ││       ││ Tree ││Storage│           │
│  └─────┘└─────┘└───────┘└───────┘└───────┘└──────┘└───────┘           │
│                                                                          │
│  Local Development  │  Serverless Cloud  │  Self-Hosted  │  Archival   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Crate Structure

```
allsource/
├── allsource-core/                 # Current - Fix and enhance
│   ├── src/
│   │   ├── lib.rs
│   │   ├── domain/
│   │   │   ├── entities/
│   │   │   │   └── event_stream.rs  # Add expected_version() getter
│   │   │   └── value_objects/
│   │   │       └── tenant_id.rs
│   │   ├── infrastructure/
│   │   │   └── repositories/
│   │   │       ├── postgres_event_stream_repository.rs  # Fix implementations
│   │   │       └── rocksdb_event_stream_repository.rs   # Fix implementations
│   │   └── error.rs                 # Add From<sqlx::Error>
│   └── Cargo.toml
│
├── allsource-dynamodb/             # NEW - AWS DynamoDB backend
│   ├── src/
│   │   ├── lib.rs
│   │   ├── backend.rs              # EventStoreBackend impl
│   │   ├── schema.rs               # Table schema definitions
│   │   ├── queries.rs              # Optimized query builders
│   │   └── streams.rs              # DynamoDB Streams integration
│   └── Cargo.toml
│
├── allsource-firestore/            # NEW - GCP Firestore backend
│   ├── src/
│   │   ├── lib.rs
│   │   ├── backend.rs              # EventStoreBackend impl
│   │   ├── schema.rs               # Collection schema
│   │   └── triggers.rs             # Firestore triggers
│   └── Cargo.toml
│
├── allsource-archive/              # NEW - S3/GCS archival
│   ├── src/
│   │   ├── lib.rs
│   │   ├── s3.rs                   # S3 backend
│   │   ├── gcs.rs                  # GCS backend
│   │   ├── parquet.rs              # Parquet serialization
│   │   └── lifecycle.rs            # Retention policies
│   └── Cargo.toml
│
└── allsource-tenant/               # NEW - Multi-tenancy support
    ├── src/
    │   ├── lib.rs
    │   ├── context.rs              # Tenant context
    │   ├── isolation.rs            # Isolation strategies
    │   └── metrics.rs              # Per-tenant metrics
    └── Cargo.toml
```

### 6.3 Feature Flags

```toml
# allframe-core/Cargo.toml
[features]
# Current
cqrs = ["allframe-macros"]
cqrs-allsource = ["cqrs", "allsource-core"]
cqrs-postgres = ["cqrs-allsource", "allsource-core/postgres"]
cqrs-rocksdb = ["cqrs-allsource", "allsource-core/rocksdb"]

# New cloud backends
cqrs-dynamodb = ["cqrs", "allsource-dynamodb"]
cqrs-firestore = ["cqrs", "allsource-firestore"]

# Archival
cqrs-archive-s3 = ["cqrs", "allsource-archive/s3"]
cqrs-archive-gcs = ["cqrs", "allsource-archive/gcs"]

# Multi-tenancy
cqrs-tenant = ["cqrs", "allsource-tenant"]

# Convenience bundles
cqrs-aws = ["cqrs-dynamodb", "cqrs-archive-s3"]
cqrs-gcp = ["cqrs-firestore", "cqrs-archive-gcs"]
cqrs-cloud = ["cqrs-aws", "cqrs-gcp"]
```

---

## 7. Cloud Backend Implementations

### 7.1 DynamoDB Backend

```rust
// allsource-dynamodb/src/backend.rs

use allframe_core::cqrs::{Event, EventStoreBackend, BackendError, BackendStats};
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use async_trait::async_trait;

pub struct DynamoDBBackend<E> {
    client: Client,
    table_name: String,
    tenant_id: Option<String>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E> DynamoDBBackend<E> {
    /// Create backend for single-tenant use
    pub async fn new(table_name: impl Into<String>) -> Result<Self, BackendError> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        Ok(Self {
            client: Client::new(&config),
            table_name: table_name.into(),
            tenant_id: None,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create backend scoped to a specific tenant
    pub async fn with_tenant(
        table_name: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Result<Self, BackendError> {
        let mut backend = Self::new(table_name).await?;
        backend.tenant_id = Some(tenant_id.into());
        Ok(backend)
    }

    /// Build partition key with optional tenant prefix
    fn build_pk(&self, stream_id: &str) -> String {
        match &self.tenant_id {
            Some(tenant) => format!("{}#{}", tenant, stream_id),
            None => stream_id.to_string(),
        }
    }

    /// Build sort key from version
    fn build_sk(version: u64) -> String {
        format!("{:020}", version)
    }
}

#[async_trait]
impl<E> EventStoreBackend<E> for DynamoDBBackend<E>
where
    E: Event + serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    async fn append(
        &self,
        stream_id: &str,
        events: Vec<E>,
        expected_version: Option<u64>,
    ) -> Result<u64, BackendError> {
        let pk = self.build_pk(stream_id);

        // Get current version
        let current_version = self.get_current_version(stream_id).await?;

        // Optimistic concurrency check
        if let Some(expected) = expected_version {
            if current_version != expected {
                return Err(BackendError::ConcurrencyConflict {
                    expected,
                    actual: current_version,
                });
            }
        }

        // Batch write events
        let mut version = current_version;
        for event in events {
            version += 1;
            let sk = Self::build_sk(version);

            let payload = serde_json::to_string(&event)
                .map_err(|e| BackendError::Serialization(e.to_string()))?;

            self.client
                .put_item()
                .table_name(&self.table_name)
                .item("PK", AttributeValue::S(pk.clone()))
                .item("SK", AttributeValue::S(sk))
                .item("payload", AttributeValue::S(payload))
                .item("timestamp", AttributeValue::S(chrono::Utc::now().to_rfc3339()))
                .item("event_type", AttributeValue::S(std::any::type_name::<E>().to_string()))
                // Conditional write for optimistic concurrency
                .condition_expression("attribute_not_exists(PK) AND attribute_not_exists(SK)")
                .send()
                .await
                .map_err(|e| BackendError::Storage(e.to_string()))?;
        }

        Ok(version)
    }

    async fn get_events(&self, stream_id: &str) -> Result<Vec<E>, BackendError> {
        let pk = self.build_pk(stream_id);

        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk")
            .expression_attribute_values(":pk", AttributeValue::S(pk))
            .send()
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;

        let events = result
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                item.get("payload")
                    .and_then(|v| v.as_s().ok())
                    .and_then(|s| serde_json::from_str(s).ok())
            })
            .collect();

        Ok(events)
    }

    async fn get_events_after(
        &self,
        stream_id: &str,
        after_version: u64,
    ) -> Result<Vec<E>, BackendError> {
        let pk = self.build_pk(stream_id);
        let sk_start = Self::build_sk(after_version + 1);

        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND SK >= :sk")
            .expression_attribute_values(":pk", AttributeValue::S(pk))
            .expression_attribute_values(":sk", AttributeValue::S(sk_start))
            .send()
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;

        let events = result
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                item.get("payload")
                    .and_then(|v| v.as_s().ok())
                    .and_then(|s| serde_json::from_str(s).ok())
            })
            .collect();

        Ok(events)
    }

    async fn get_all_events(&self) -> Result<Vec<E>, BackendError> {
        // For tenant-scoped backend, scan with tenant prefix
        // For global backend, full scan (expensive, use sparingly)
        let mut events = Vec::new();
        let mut last_key = None;

        loop {
            let mut scan = self.client.scan().table_name(&self.table_name);

            if let Some(tenant) = &self.tenant_id {
                scan = scan
                    .filter_expression("begins_with(PK, :tenant)")
                    .expression_attribute_values(":tenant", AttributeValue::S(format!("{}#", tenant)));
            }

            if let Some(key) = last_key {
                scan = scan.set_exclusive_start_key(Some(key));
            }

            let result = scan.send().await.map_err(|e| BackendError::Storage(e.to_string()))?;

            for item in result.items.unwrap_or_default() {
                if let Some(payload) = item.get("payload").and_then(|v| v.as_s().ok()) {
                    if let Ok(event) = serde_json::from_str(payload) {
                        events.push(event);
                    }
                }
            }

            last_key = result.last_evaluated_key;
            if last_key.is_none() {
                break;
            }
        }

        Ok(events)
    }

    async fn flush(&self) -> Result<(), BackendError> {
        // DynamoDB is immediately consistent, no flush needed
        Ok(())
    }

    async fn stats(&self) -> BackendStats {
        BackendStats {
            total_events: 0, // Would require scan
            total_aggregates: 0,
            total_snapshots: 0,
            backend_specific: [
                ("backend_type".to_string(), "dynamodb".to_string()),
                ("table_name".to_string(), self.table_name.clone()),
            ]
            .into(),
        }
    }
}
```

### 7.2 Multi-Tenant Queries

```rust
// allsource-dynamodb/src/tenant.rs

impl<E> DynamoDBBackend<E>
where
    E: Event + serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    /// Get all streams for a tenant (admin operation)
    pub async fn get_streams_by_tenant(&self, tenant_id: &str) -> Result<Vec<String>, BackendError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("by_tenant")
            .key_condition_expression("tenant_id = :tenant")
            .expression_attribute_values(":tenant", AttributeValue::S(tenant_id.to_string()))
            .projection_expression("stream_id")
            .send()
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;

        let streams: Vec<String> = result
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                item.get("stream_id")
                    .and_then(|v| v.as_s().ok())
                    .map(|s| s.to_string())
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Ok(streams)
    }

    /// Count streams for a tenant (admin operation)
    pub async fn count_streams_by_tenant(&self, tenant_id: &str) -> Result<usize, BackendError> {
        let streams = self.get_streams_by_tenant(tenant_id).await?;
        Ok(streams.len())
    }
}
```

---

## 8. Multi-Tenancy Design

### 8.1 Isolation Strategies

| Strategy | Use Case | Pros | Cons |
|----------|----------|------|------|
| **Pooled** | SaaS, cost-sensitive | Single table, easy ops | Noisy neighbor risk |
| **Siloed** | Enterprise, regulated | Full isolation | Higher ops cost |
| **Bridge** | Hybrid | Flexible | Complex |

### 8.2 Pooled Multi-Tenancy (Recommended)

```rust
// Partition key includes tenant
PK: "{tenant_id}#{stream_id}"
SK: "{version:020}"

// GSI for tenant queries
GSI: by_tenant
  PK: tenant_id
  SK: created_at

// Access pattern: Always scope by tenant
let store = EventStore::with_tenant("tenant-123");
store.get_events("order-1").await?;  // Queries "tenant-123#order-1"
```

### 8.3 Tenant Context Middleware

```rust
// allsource-tenant/src/context.rs

use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-local tenant context for request scoping
#[derive(Clone)]
pub struct TenantContext {
    tenant_id: Arc<RwLock<Option<String>>>,
}

impl TenantContext {
    pub fn new() -> Self {
        Self {
            tenant_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Set tenant for current request
    pub async fn set_tenant(&self, tenant_id: impl Into<String>) {
        *self.tenant_id.write().await = Some(tenant_id.into());
    }

    /// Get current tenant
    pub async fn get_tenant(&self) -> Option<String> {
        self.tenant_id.read().await.clone()
    }

    /// Clear tenant (end of request)
    pub async fn clear(&self) {
        *self.tenant_id.write().await = None;
    }
}

/// Middleware for Lambda/Cloud Run
pub async fn tenant_middleware<F, Fut, R>(
    tenant_id: &str,
    ctx: TenantContext,
    f: F,
) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    ctx.set_tenant(tenant_id).await;
    let result = f().await;
    ctx.clear().await;
    result
}
```

### 8.4 Row-Level Security (PostgreSQL)

```sql
-- Enable RLS
ALTER TABLE events ENABLE ROW LEVEL SECURITY;

-- Create policy
CREATE POLICY tenant_isolation ON events
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

-- Set tenant context
SET app.tenant_id = 'tenant-123';

-- All queries now filtered by tenant
SELECT * FROM events WHERE stream_id = 'order-1';
-- Automatically adds: AND tenant_id = 'tenant-123'
```

---

## 9. Serverless Optimization

### 9.1 Cold Start Optimization

| Technique | Impact | Implementation |
|-----------|--------|----------------|
| Lazy init | -50ms | Don't connect until first query |
| Static pool | -30ms | Reuse connections across invocations |
| ARM64 | -20ms | Use Graviton processors |
| Minimal deps | -10ms | Feature flags, no unused code |

### 9.2 Connection Reuse Pattern

```rust
// Static connection pool for Lambda
use once_cell::sync::OnceCell;
use tokio::sync::OnceCell as AsyncOnceCell;

static CLIENT: AsyncOnceCell<Client> = AsyncOnceCell::const_new();

async fn get_client() -> &'static Client {
    CLIENT
        .get_or_init(|| async {
            let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            Client::new(&config)
        })
        .await
}
```

### 9.3 Query Cost Optimization

```rust
// Projection expressions - return only needed fields
.projection_expression("payload, timestamp")

// Consistent vs eventual reads
.consistent_read(false)  // Faster, cheaper for most cases

// Pagination for large result sets
.limit(100)
.exclusive_start_key(last_key)
```

### 9.4 Binary Size Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Size optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Remove symbols
panic = "abort"      # Smaller panic handling

[profile.release.package."*"]
opt-level = "z"
```

**Target**: < 5 MB for Lambda deployment

---

## 10. API Design

### 10.1 Unified EventStore API

```rust
/// EventStore with pluggable backends and multi-tenancy
pub struct EventStore<E: Event, B: EventStoreBackend<E>> {
    backend: Arc<B>,
    tenant_id: Option<String>,
}

impl<E: Event, B: EventStoreBackend<E>> EventStore<E, B> {
    /// Create with backend
    pub fn with_backend(backend: B) -> Self;

    /// Create with backend and tenant scope
    pub fn with_tenant(backend: B, tenant_id: impl Into<String>) -> Self;

    /// Append events to stream
    pub async fn append(&self, stream_id: &str, events: Vec<E>) -> Result<u64, StoreError>;

    /// Append with optimistic concurrency
    pub async fn append_with_version(
        &self,
        stream_id: &str,
        events: Vec<E>,
        expected_version: u64,
    ) -> Result<u64, StoreError>;

    /// Get all events for stream
    pub async fn get_events(&self, stream_id: &str) -> Result<Vec<E>, StoreError>;

    /// Get events after version
    pub async fn get_events_after(&self, stream_id: &str, version: u64) -> Result<Vec<E>, StoreError>;

    /// Save snapshot
    pub async fn save_snapshot<A: Aggregate<Event = E>>(
        &self,
        aggregate_id: &str,
        aggregate: &A,
        version: u64,
    ) -> Result<(), StoreError>;

    /// Load snapshot
    pub async fn get_snapshot<A: Aggregate<Event = E>>(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<(A, u64)>, StoreError>;

    /// Get backend statistics
    pub async fn stats(&self) -> BackendStats;

    /// Flush pending writes (no-op for DynamoDB)
    pub async fn flush(&self) -> Result<(), StoreError>;
}
```

### 10.2 Backend Selection

```rust
// Local development - InMemory
let store = EventStore::new();

// Local development - AllSource embedded
let store = EventStore::with_backend(AllSourceBackend::new()?);

// Production - DynamoDB
let store = EventStore::with_backend(
    DynamoDBBackend::with_tenant("events-table", "tenant-123").await?
);

// Production - Firestore
let store = EventStore::with_backend(
    FirestoreBackend::with_tenant("my-project", "tenant-123").await?
);

// Self-hosted - PostgreSQL
let store = EventStore::with_backend(
    PostgresBackend::new("postgres://localhost/events").await?
);
```

---

## 11. Migration Path

### Phase 1: Fix Current Issues (Week 1)

1. Fix compilation errors in allsource-core
2. Implement missing trait methods
3. Add error conversions
4. All tests passing

### Phase 2: DynamoDB Backend (Weeks 2-4)

1. Create allsource-dynamodb crate
2. Implement EventStoreBackend
3. Add multi-tenant support
4. Integration tests with DynamoDB Local

### Phase 3: Multi-Tenancy (Weeks 5-6)

1. Create allsource-tenant crate
2. Tenant context middleware
3. Per-tenant metrics
4. Documentation

### Phase 4: Firestore Backend (Weeks 7-8)

1. Create allsource-firestore crate
2. Implement EventStoreBackend
3. Firestore triggers for projections
4. GCP integration tests

### Phase 5: Archival (Weeks 9-10)

1. Create allsource-archive crate
2. S3 backend
3. GCS backend
4. Lifecycle policies

### Phase 6: Documentation & Polish (Weeks 11-12)

1. Comprehensive documentation
2. Example applications
3. Performance benchmarks
4. Migration guides

---

## 12. Timeline & Milestones

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Fix Issues | 1 week | Compilation succeeds |
| 2. DynamoDB | 3 weeks | Full EventStoreBackend |
| 3. Multi-Tenancy | 2 weeks | Tenant isolation |
| 4. Firestore | 2 weeks | GCP support |
| 5. Archival | 2 weeks | Cold storage |
| 6. Polish | 2 weeks | Docs, examples |
| **Total** | **12 weeks** | |

---

## 13. Success Criteria

### Functional

- [ ] `cargo build --all-features` succeeds
- [ ] DynamoDB backend passes all EventStoreBackend tests
- [ ] Multi-tenant queries work with isolation
- [ ] Firestore backend works on GCP
- [ ] S3/GCS archival stores and retrieves events

### Performance

| Metric | Target |
|--------|--------|
| DynamoDB append latency | < 10ms p99 |
| DynamoDB query latency | < 20ms p99 |
| Lambda cold start contribution | < 50ms |
| Binary size | < 5 MB |

### Quality

- [ ] 80%+ code coverage
- [ ] Zero unsafe code in new backends
- [ ] All public APIs documented
- [ ] No new Clippy warnings

---

## 14. Risks & Mitigations

### Risk 1: DynamoDB Costs

**Risk**: High-volume event sourcing may be expensive.

**Mitigation**:
- Use on-demand billing
- Implement archival to S3
- Document cost estimation
- Consider reserved capacity for predictable workloads

### Risk 2: Cross-Tenant Data Leak

**Risk**: Multi-tenant isolation failure.

**Mitigation**:
- Tenant ID in partition key (can't query without it)
- Integration tests for isolation
- Security review before release
- Documentation for proper usage

### Risk 3: Eventual Consistency

**Risk**: DynamoDB eventual consistency surprises users.

**Mitigation**:
- Document consistency model
- Provide consistent read option
- Use DynamoDB Streams for projections
- Strong consistency for critical reads

### Risk 4: Migration Complexity

**Risk**: Migrating from AllSource embedded to DynamoDB is hard.

**Mitigation**:
- Provide migration tool
- Document migration path
- Support dual-write during migration
- Rollback procedures

---

## Appendix

### A. DynamoDB Table Terraform

```hcl
resource "aws_dynamodb_table" "events" {
  name         = "${var.prefix}_events"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "PK"
  range_key    = "SK"

  attribute {
    name = "PK"
    type = "S"
  }

  attribute {
    name = "SK"
    type = "S"
  }

  attribute {
    name = "tenant_id"
    type = "S"
  }

  attribute {
    name = "created_at"
    type = "S"
  }

  global_secondary_index {
    name            = "by_tenant"
    hash_key        = "tenant_id"
    range_key       = "created_at"
    projection_type = "KEYS_ONLY"
  }

  stream_enabled   = true
  stream_view_type = "NEW_AND_OLD_IMAGES"

  ttl {
    attribute_name = "ttl"
    enabled        = true
  }

  tags = {
    Environment = var.environment
    Application = "allsource"
  }
}
```

### B. Firestore Security Rules

```javascript
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    // Tenant isolation
    match /tenants/{tenantId}/{document=**} {
      allow read, write: if request.auth != null
        && request.auth.token.tenant_id == tenantId;
    }

    // Admin access
    match /{document=**} {
      allow read, write: if request.auth != null
        && request.auth.token.admin == true;
    }
  }
}
```

### C. Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| aws-sdk-dynamodb | 1.55+ | DynamoDB client |
| aws-config | 1.5+ | AWS configuration |
| google-cloud-firestore | 0.4+ | Firestore client |
| aws-sdk-s3 | 1.65+ | S3 archival |
| google-cloud-storage | 0.20+ | GCS archival |
| chrono | 0.4+ | Timestamps |
| serde | 1.0+ | Serialization |
| tokio | 1.0+ | Async runtime |

### D. References

- [AWS CQRS Event Store with DynamoDB](https://aws.amazon.com/blogs/database/build-a-cqrs-event-store-with-amazon-dynamodb/)
- [The Mill Adventure Event Sourcing](https://aws.amazon.com/blogs/architecture/how-the-mill-adventure-implemented-event-sourcing-at-scale-using-dynamodb/)
- [Multi-Tenant SaaS Architecture 2025](https://isitdev.com/multi-tenant-saas-architecture-cloud-2025/)
- [Serverless Event Sourcing with AWS](https://dev.to/slsbytheodo/serverless-event-sourcing-with-aws-state-of-the-art-data-synchronization-4mog)
- [Event Sourcing Pattern - AWS](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/event-sourcing.html)
- [redb - Embedded Database](https://github.com/cberner/redb)

---

**AllSource. Events without boundaries.**

*Cloud-native. Multi-tenant. Serverless-ready.*
