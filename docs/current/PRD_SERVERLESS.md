# PRD: AllFrame Serverless (Phase 7)

**Title**: AllFrame Serverless - Cloud-Native Deployment
**Version**: 1.0
**Status**: Draft
**Priority**: P0
**Author**: AllFrame Team
**Created**: 2025-11-27
**Last Updated**: 2025-11-27

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Goals & Non-Goals](#2-goals--non-goals)
3. [Background & Market Context](#3-background--market-context)
4. [User Stories](#4-user-stories)
5. [Technical Requirements](#5-technical-requirements)
6. [Architecture](#6-architecture)
7. [Detailed Design](#7-detailed-design)
8. [API Design](#8-api-design)
9. [Infrastructure-from-Code](#9-infrastructure-from-code)
10. [Timeline & Milestones](#10-timeline--milestones)
11. [Success Criteria](#11-success-criteria)
12. [Risks & Mitigations](#12-risks--mitigations)
13. [Future Considerations](#13-future-considerations)
14. [Appendix](#appendix)

---

## 1. Executive Summary

AllFrame Serverless extends the framework to support deployment on serverless platforms (AWS Lambda, GCP Cloud Run) while preserving full CQRS+ES capability with **zero code changes**. Developers write handlers once and deploy anywhere.

### Key Value Propositions

- **Write Once, Deploy Anywhere**: Same AllFrame handler works on Lambda, Cloud Run, and local development
- **Native CQRS on Serverless**: CommandBus, Projections, and Sagas work seamlessly with cloud event sources
- **Infrastructure-from-Code**: Generate Terraform/IaC from AllFrame annotations—no separate infrastructure files
- **Rust Performance Advantage**: Sub-50ms cold starts, 96% cost reduction vs JVM-based serverless

### Target Platforms

| Platform | Priority | Status |
|----------|----------|--------|
| AWS Lambda | P0 | Primary target |
| GCP Cloud Run | P1 | Secondary target |
| Shuttle.rs | P2 | Future consideration |
| Fly.io | P2 | Future consideration |

### Excluded Platforms

| Platform | Reason |
|----------|--------|
| Azure Functions | SDK in beta, immature |
| Fermyon Spin | No Tokio support, ecosystem incompatible |
| Cloudflare Workers | WASM limitations, no async Rust ecosystem |

---

## 2. Goals & Non-Goals

### Goals

| Goal | Success Metric |
|------|----------------|
| Zero-code deployment | Same handler works on Lambda, Cloud Run, local |
| Sub-50ms cold start | Measured via CloudWatch/Cloud Monitoring |
| Full CQRS support | CommandBus, Projections, Sagas work on serverless |
| Infrastructure-from-Code | Generate valid Terraform from annotations |
| Small binary size | < 10 MB release binary (stripped, LTO) |
| DX parity | Local development identical to cloud behavior |

### Non-Goals

- **WASM/Spin support**: Ecosystem too immature, no Tokio compatibility
- **Azure Functions**: SDK in beta after 10 years, avoid
- **Multi-region orchestration**: Future phase (Phase 8+)
- **Kubernetes operator**: Separate phase (Phase 9)
- **Custom VPC networking**: Users configure separately

---

## 3. Background & Market Context

### Market Opportunity

The Rust serverless ecosystem reached a critical inflection point in 2025:

- **AWS Lambda Rust GA** (November 2025): Full SLA, enterprise support
- **GCP Rust SDK GA** (September 2025): Cloud Run viable for Rust
- **Cold start advantage**: Rust 16ms vs Java 410ms (25x faster)
- **Cost efficiency**: Up to 96% reduction vs JVM workloads

### Competitive Landscape

| Framework | CQRS | Serverless | IaC Gen | Production Ready |
|-----------|------|------------|---------|------------------|
| **AllFrame** (proposed) | Native | Multi-cloud | Yes | Target |
| Shuttle.rs | No | Shuttle only | Yes | Yes |
| Axum + cargo-lambda | No | Manual | No | Yes |
| Spring Boot | Yes | Limited | No | Yes |

**Gap**: No framework combines **CQRS+ES + Serverless + IaC Generation**.

### Technical Foundation

AllFrame's existing architecture is serverless-ready:

- **Tower-based router**: Compatible with lambda_http
- **Tokio async runtime**: Full AWS SDK compatibility
- **CQRS infrastructure**: Event-driven by design
- **OpenTelemetry**: Distributed tracing built-in

---

## 4. User Stories

### US-1: Lambda Deployment

> As a developer, I want to deploy my AllFrame handler to Lambda with one command, so that I can ship to production without infrastructure expertise.

**Acceptance Criteria**:
- `cargo lambda build` produces deployable artifact
- `cargo lambda deploy` deploys to AWS
- Handler responds to API Gateway requests
- Cold start < 50ms

### US-2: CQRS on Serverless

> As a developer, I want my CQRS commands to work identically on Lambda and locally, so that I can develop and test without cloud resources.

**Acceptance Criteria**:
- CommandBus dispatches commands on Lambda
- EventBridge events trigger command handlers
- Projections update from DynamoDB Streams
- Local development uses InMemoryBackend

### US-3: Infrastructure Generation

> As a DevOps engineer, I want Terraform generated from AllFrame code annotations, so that infrastructure stays in sync with application code.

**Acceptance Criteria**:
- `allframe infra emit terraform` generates valid HCL
- Lambda functions, DynamoDB tables, SQS queues generated
- IAM roles with least-privilege policies
- Output can be applied with `terraform apply`

### US-4: Event-Driven Integration

> As a developer, I want EventBridge events to dispatch to my CommandBus automatically, so that I can build event-driven architectures.

**Acceptance Criteria**:
- EventBridge rule triggers Lambda
- Event payload deserializes to Command
- CommandBus.dispatch() processes command
- Events stored in DynamoDB event store

### US-5: Cloud Run Deployment

> As a developer using GCP, I want to deploy AllFrame to Cloud Run, so that I can use my preferred cloud provider.

**Acceptance Criteria**:
- Dockerfile generated or provided
- `gcloud run deploy` works
- Axum server runs in container
- Same handlers work as Lambda

---

## 5. Technical Requirements

### TR-1: Lambda Runtime Adapter

**Description**: Wrap AllFrame Router in lambda_http Tower service.

**Requirements**:
- Convert AllFrame Router to Tower Service
- Map Lambda events to HTTP requests
- Inject Lambda context into handlers
- Support API Gateway v1 and v2 payloads
- Support Function URLs

**Dependencies**:
- `lambda_http` 0.14+
- `lambda_runtime` 0.14+
- `tower` 0.4+

### TR-2: Event Source Mappings

**Description**: Map Lambda event sources to AllFrame CQRS primitives.

**Requirements**:

| Event Source | AllFrame Mapping |
|--------------|------------------|
| API Gateway HTTP | Router → Handler |
| EventBridge | CommandBus.dispatch() |
| SQS Message | CommandBus.dispatch() |
| DynamoDB Stream | ProjectionRegistry.update() |
| S3 Event | Custom Event Handler |
| SNS Message | CommandBus.dispatch() |

**Dependencies**:
- `aws_lambda_events` 0.16+

### TR-3: Serverless Event Store Backends

**Description**: Implement EventStoreBackend for serverless databases.

**Requirements**:
- DynamoDB backend with single-table design
- Optimistic concurrency via conditional writes
- S3 backend for snapshots
- Automatic schema creation (optional)
- Zero-config local fallback

**Dependencies**:
- `aws-sdk-dynamodb` 1.55+
- `aws-sdk-s3` 1.65+

### TR-4: Infrastructure Generation

**Description**: Generate Terraform HCL from AllFrame annotations.

**Requirements**:
- Parse infrastructure macros at build time
- Generate HCL via hcl-rs library
- Support AWS provider (Lambda, DynamoDB, SQS, EventBridge)
- Support GCP provider (Cloud Run, Firestore)
- Generate IAM policies with least privilege
- Output valid, formatted Terraform

**Dependencies**:
- `hcl-rs` 0.18+
- `syn` 2.0+ (macro parsing)

### TR-5: Cold Start Optimization

**Description**: Minimize Lambda cold start time.

**Requirements**:
- Lazy initialization of non-critical services
- Connection pooling with keep-alive
- Binary size < 10 MB (release, stripped)
- Profile-guided optimization (optional)
- ARM64 (Graviton) support

**Techniques**:
- `opt-level = "z"` for size
- `lto = true` for link-time optimization
- `strip = true` for symbol removal
- Feature flags to exclude unused code

### TR-6: Cloud Run Adapter

**Description**: Support GCP Cloud Run deployment.

**Requirements**:
- Standard Axum/Hyper server (no special runtime)
- Read PORT from environment
- Health check endpoint
- Graceful shutdown on SIGTERM
- Dockerfile generation

---

## 6. Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         AllFrame Serverless                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │   Event Source  │    │  Runtime Adapter │    │  AllFrame Core  │     │
│  │                 │───▶│                  │───▶│                 │     │
│  │  - API Gateway  │    │  - Lambda        │    │  - Router       │     │
│  │  - EventBridge  │    │  - Cloud Run     │    │  - CommandBus   │     │
│  │  - SQS/SNS      │    │  - Local         │    │  - Projections  │     │
│  │  - DynamoDB     │    │                  │    │  - Sagas        │     │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘     │
│           │                      │                      │               │
│           ▼                      ▼                      ▼               │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │  Event Mapping  │    │     Tower       │    │  Storage Backend│     │
│  │                 │    │    Service      │    │                 │     │
│  │  - Deserialize  │    │  - Middleware   │    │  - DynamoDB     │     │
│  │  - Validate     │    │  - Tracing      │    │  - S3           │     │
│  │  - Route        │    │  - Metrics      │    │  - InMemory     │     │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
allframe/
├── crates/
│   ├── allframe-core/              # Existing - unchanged
│   ├── allframe-macros/            # Existing - extended with infra macros
│   ├── allframe-forge/             # Existing - extended with infra commands
│   │
│   ├── allframe-serverless/        # NEW - Runtime adapters
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── runtime/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── lambda.rs       # AWS Lambda runtime
│   │   │   │   ├── cloud_run.rs    # GCP Cloud Run
│   │   │   │   └── local.rs        # Local development
│   │   │   ├── events/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── api_gateway.rs  # HTTP events
│   │   │   │   ├── eventbridge.rs  # EventBridge → CommandBus
│   │   │   │   ├── sqs.rs          # SQS → CommandBus
│   │   │   │   └── dynamodb.rs     # DynamoDB Streams
│   │   │   ├── backends/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dynamodb.rs     # DynamoDB event store
│   │   │   │   └── s3.rs           # S3 snapshots
│   │   │   └── config/
│   │   │       └── env.rs          # Environment config
│   │   └── Cargo.toml
│   │
│   └── allframe-infra/             # NEW - Infrastructure generation
│       ├── src/
│       │   ├── lib.rs
│       │   ├── ir/
│       │   │   ├── mod.rs          # Infrastructure IR
│       │   │   └── resources.rs    # Resource definitions
│       │   ├── terraform/
│       │   │   ├── mod.rs
│       │   │   ├── aws.rs          # AWS provider
│       │   │   └── gcp.rs          # GCP provider
│       │   └── cli/
│       │       └── mod.rs          # CLI commands
│       └── Cargo.toml
```

### Data Flow: Lambda HTTP Request

```
API Gateway Request
        │
        ▼
┌───────────────────┐
│   lambda_http     │  Deserialize API Gateway event
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  LambdaRuntime    │  AllFrame Lambda adapter
└───────────────────┘
        │
        ▼
┌───────────────────┐
│   Tower Service   │  Middleware (tracing, metrics)
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  AllFrame Router  │  Route to handler
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  Command Handler  │  Business logic
└───────────────────┘
        │
        ▼
┌───────────────────┐
│   CommandBus      │  Dispatch & store events
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  DynamoDB Backend │  Persist events
└───────────────────┘
```

### Data Flow: EventBridge → CQRS

```
EventBridge Event
        │
        ▼
┌───────────────────┐
│  Lambda Trigger   │  Event source mapping
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ EventBridge Adapter│  Deserialize to Command
└───────────────────┘
        │
        ▼
┌───────────────────┐
│   CommandBus      │  dispatch(command)
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Command Handler   │  Process & emit events
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  DynamoDB Backend │  Append events to stream
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ DynamoDB Streams  │  Trigger projection Lambda
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ProjectionRegistry │  Update read models
└───────────────────┘
```

---

## 7. Detailed Design

### 7.1 Lambda Runtime Adapter

```rust
// allframe-serverless/src/runtime/lambda.rs

use lambda_http::{run, service_fn, Body, Error, Request, Response};
use allframe_core::router::Router;
use tower::ServiceExt;
use std::sync::Arc;

/// Lambda runtime wrapper for AllFrame applications
pub struct LambdaRuntime<S> {
    service: Arc<S>,
}

impl<S> LambdaRuntime<S>
where
    S: tower::Service<Request, Response = Response<Body>> + Clone + Send + Sync + 'static,
    S::Future: Send,
    S::Error: std::error::Error + Send + Sync,
{
    /// Create a new Lambda runtime from a Tower service
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    /// Create from AllFrame Router
    pub fn from_router(router: Router) -> Self {
        Self::new(router.into_tower_service())
    }

    /// Run the Lambda runtime
    pub async fn run(self) -> Result<(), Error> {
        let service = self.service;

        run(service_fn(move |request: Request| {
            let svc = service.clone();
            async move {
                svc.as_ref()
                    .clone()
                    .oneshot(request)
                    .await
                    .map_err(|e| Error::from(e.to_string()))
            }
        }))
        .await
    }
}

/// Convenience function to run AllFrame on Lambda
pub async fn run_lambda(router: Router) -> Result<(), Error> {
    LambdaRuntime::from_router(router).run().await
}
```

### 7.2 DynamoDB Event Store Backend

```rust
// allframe-serverless/src/backends/dynamodb.rs

use allframe_core::cqrs::{Event, EventStoreBackend, BackendError, BackendStats};
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::marker::PhantomData;

/// DynamoDB-backed event store for serverless deployments
///
/// Uses single-table design with:
/// - PK: stream_id
/// - SK: version (zero-padded for sorting)
/// - event_type: discriminator for deserialization
/// - payload: JSON event data
/// - timestamp: ISO 8601 timestamp
pub struct DynamoDBEventStore<E> {
    client: Client,
    table_name: String,
    _phantom: PhantomData<E>,
}

impl<E> DynamoDBEventStore<E> {
    /// Create a new DynamoDB event store
    pub async fn new(table_name: impl Into<String>) -> Result<Self, BackendError> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            table_name: table_name.into(),
            _phantom: PhantomData,
        })
    }

    /// Create with custom client (for testing)
    pub fn with_client(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<E> EventStoreBackend<E> for DynamoDBEventStore<E>
where
    E: Event + Serialize + DeserializeOwned + Send + Sync,
{
    async fn append(
        &self,
        stream_id: &str,
        events: Vec<E>,
        expected_version: Option<u64>,
    ) -> Result<u64, BackendError> {
        // Get current version
        let current_version = self.get_stream_version(stream_id).await?;

        // Optimistic concurrency check
        if let Some(expected) = expected_version {
            if current_version != expected {
                return Err(BackendError::ConcurrencyConflict {
                    expected,
                    actual: current_version,
                });
            }
        }

        // Append events with conditional write
        let mut version = current_version;
        for event in events {
            version += 1;
            let item = self.event_to_item(stream_id, version, &event)?;

            self.client
                .put_item()
                .table_name(&self.table_name)
                .set_item(Some(item))
                .condition_expression("attribute_not_exists(PK)")
                .send()
                .await
                .map_err(|e| BackendError::Storage(e.to_string()))?;
        }

        Ok(version)
    }

    async fn read_stream(&self, stream_id: &str) -> Result<Vec<E>, BackendError> {
        self.read_stream_from(stream_id, 0).await
    }

    async fn read_stream_from(&self, stream_id: &str, from_version: u64) -> Result<Vec<E>, BackendError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND SK >= :sk")
            .expression_attribute_values(":pk", AttributeValue::S(stream_id.to_string()))
            .expression_attribute_values(":sk", AttributeValue::S(format!("{:020}", from_version)))
            .send()
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;

        let events = result
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|item| self.item_to_event(item))
            .collect::<Result<Vec<E>, _>>()?;

        Ok(events)
    }

    async fn stats(&self) -> BackendStats {
        // DynamoDB doesn't provide cheap count, return defaults
        BackendStats::default()
    }
}

impl<E> DynamoDBEventStore<E>
where
    E: Serialize + DeserializeOwned,
{
    fn event_to_item(
        &self,
        stream_id: &str,
        version: u64,
        event: &E,
    ) -> Result<std::collections::HashMap<String, AttributeValue>, BackendError> {
        let payload = serde_json::to_string(event)
            .map_err(|e| BackendError::Serialization(e.to_string()))?;

        let mut item = std::collections::HashMap::new();
        item.insert("PK".to_string(), AttributeValue::S(stream_id.to_string()));
        item.insert("SK".to_string(), AttributeValue::S(format!("{:020}", version)));
        item.insert("payload".to_string(), AttributeValue::S(payload));
        item.insert(
            "timestamp".to_string(),
            AttributeValue::S(chrono::Utc::now().to_rfc3339()),
        );

        Ok(item)
    }

    fn item_to_event(
        &self,
        item: std::collections::HashMap<String, AttributeValue>,
    ) -> Result<E, BackendError> {
        let payload = item
            .get("payload")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| BackendError::Storage("Missing payload".to_string()))?;

        serde_json::from_str(payload).map_err(|e| BackendError::Serialization(e.to_string()))
    }

    async fn get_stream_version(&self, stream_id: &str) -> Result<u64, BackendError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk")
            .expression_attribute_values(":pk", AttributeValue::S(stream_id.to_string()))
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;

        let version = result
            .items
            .and_then(|items| items.first().cloned())
            .and_then(|item| item.get("SK").cloned())
            .and_then(|sk| sk.as_s().ok().cloned())
            .and_then(|sk| sk.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(version)
    }
}
```

### 7.3 EventBridge Adapter

```rust
// allframe-serverless/src/events/eventbridge.rs

use allframe_core::cqrs::{Command, CommandBus, Event};
use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Handle EventBridge events by dispatching to CommandBus
pub async fn handle_eventbridge_event<E, C, H>(
    event: EventBridgeEvent<serde_json::Value>,
    command_bus: Arc<CommandBus<E>>,
) -> Result<(), EventBridgeError>
where
    E: Event + Send + Sync + 'static,
    C: Command + DeserializeOwned + Send + Sync + 'static,
    H: allframe_core::cqrs::CommandHandler<C, E> + Send + Sync + 'static,
{
    // Extract command from event detail
    let command: C = serde_json::from_value(event.detail)
        .map_err(|e| EventBridgeError::Deserialization(e.to_string()))?;

    // Dispatch through CommandBus
    command_bus
        .dispatch(command)
        .await
        .map_err(|e| EventBridgeError::CommandFailed(e.to_string()))?;

    Ok(())
}

/// Errors from EventBridge event handling
#[derive(Debug, thiserror::Error)]
pub enum EventBridgeError {
    #[error("Failed to deserialize event: {0}")]
    Deserialization(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}

/// Builder for EventBridge Lambda handlers
pub struct EventBridgeHandler<E> {
    command_bus: Arc<CommandBus<E>>,
}

impl<E: Event + Send + Sync + 'static> EventBridgeHandler<E> {
    pub fn new(command_bus: CommandBus<E>) -> Self {
        Self {
            command_bus: Arc::new(command_bus),
        }
    }

    /// Create a Lambda handler function for a specific command type
    pub fn handler<C, H>(&self) -> impl Fn(EventBridgeEvent<serde_json::Value>) -> futures::future::BoxFuture<'static, Result<(), EventBridgeError>>
    where
        C: Command + DeserializeOwned + Send + Sync + 'static,
        H: allframe_core::cqrs::CommandHandler<C, E> + Send + Sync + 'static,
    {
        let bus = self.command_bus.clone();
        move |event| {
            let bus = bus.clone();
            Box::pin(async move {
                handle_eventbridge_event::<E, C, H>(event, bus).await
            })
        }
    }
}
```

---

## 8. API Design

### 8.1 User-Facing API

```rust
// Example: Complete Lambda application with CQRS

use allframe_core::prelude::*;
use allframe_core::cqrs::{Command, Event, CommandBus, command_handler};
use allframe_serverless::{run_lambda, DynamoDBEventStore};

// Domain events
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
enum OrderEvent {
    Created { order_id: String, customer_id: String },
    ItemAdded { order_id: String, item: String, quantity: u32 },
    Submitted { order_id: String },
}

// Commands
#[derive(Debug, Deserialize, Command)]
struct CreateOrder {
    order_id: String,
    customer_id: String,
}

// Handler
#[command_handler]
async fn handle_create_order(cmd: CreateOrder) -> Result<Vec<OrderEvent>, Error> {
    Ok(vec![OrderEvent::Created {
        order_id: cmd.order_id,
        customer_id: cmd.customer_id,
    }])
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Initialize event store (auto-configures from environment)
    let event_store = DynamoDBEventStore::new("order-events").await?;

    // Build CommandBus
    let command_bus = CommandBus::new(event_store)
        .register::<CreateOrder, _>(handle_create_order);

    // Build router
    let router = Router::new()
        .post("/orders", create_order_handler(command_bus.clone()));

    // Run on Lambda
    run_lambda(router).await
}
```

### 8.2 Infrastructure Annotations

```rust
// Infrastructure-from-Code annotations

use allframe_macros::{lambda_function, event_store, queue};

#[lambda_function(
    memory = 256,
    timeout = 30,
    runtime = "provided.al2023",
    architecture = "arm64",
    environment = {
        "RUST_LOG" = "info"
    }
)]
#[command_handler]
async fn handle_create_order(cmd: CreateOrder) -> Result<Vec<OrderEvent>, Error> {
    // ...
}

#[event_store(
    provider = "dynamodb",
    table = "order-events",
    billing_mode = "pay_per_request",
    stream = true
)]
struct OrderEventStore;

#[queue(
    provider = "sqs",
    visibility_timeout = 30,
    dlq = true,
    dlq_max_receives = 3
)]
struct OrderQueue;

#[event_bus(
    provider = "eventbridge",
    bus_name = "orders"
)]
struct OrderEventBus;
```

### 8.3 CLI Commands

```bash
# Build for Lambda
cargo lambda build --release --arm64

# Deploy to Lambda
cargo lambda deploy --iam-role arn:aws:iam::123456789:role/lambda-role

# Generate Terraform
allframe infra emit terraform --output ./terraform/
allframe infra emit terraform --provider aws --output ./terraform/aws/
allframe infra emit terraform --provider gcp --output ./terraform/gcp/

# Preview infrastructure
allframe infra plan

# Validate generated infrastructure
allframe infra validate

# Local development
allframe dev  # Runs local server with InMemoryBackend
```

---

## 9. Infrastructure-from-Code

### 9.1 Design Philosophy

Infrastructure-from-Code (IfC) derives infrastructure requirements from application code, ensuring:

1. **Single Source of Truth**: Application code defines what infrastructure is needed
2. **Type Safety**: Infrastructure configuration is validated at compile time
3. **Drift Prevention**: Generated infrastructure always matches code
4. **Developer Experience**: No separate IaC files to maintain

### 9.2 Infrastructure IR

```rust
// allframe-infra/src/ir/resources.rs

/// Internal representation of infrastructure resources
#[derive(Debug, Clone, Serialize)]
pub enum InfraResource {
    Lambda(LambdaConfig),
    DynamoDB(DynamoDBConfig),
    SQS(SQSConfig),
    EventBridge(EventBridgeConfig),
    IAMRole(IAMConfig),
    S3(S3Config),
    CloudRun(CloudRunConfig),
}

#[derive(Debug, Clone, Serialize)]
pub struct LambdaConfig {
    pub name: String,
    pub handler: String,
    pub runtime: String,
    pub architecture: String,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub environment: HashMap<String, String>,
    pub event_sources: Vec<EventSourceConfig>,
    pub vpc_config: Option<VPCConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DynamoDBConfig {
    pub table_name: String,
    pub partition_key: KeySchema,
    pub sort_key: Option<KeySchema>,
    pub billing_mode: BillingMode,
    pub stream_enabled: bool,
    pub ttl_attribute: Option<String>,
    pub global_secondary_indexes: Vec<GSIConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SQSConfig {
    pub queue_name: String,
    pub visibility_timeout: u32,
    pub message_retention: u32,
    pub dlq_enabled: bool,
    pub dlq_max_receives: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventBridgeConfig {
    pub bus_name: String,
    pub rules: Vec<EventBridgeRuleConfig>,
}
```

### 9.3 Terraform Generation

```rust
// allframe-infra/src/terraform/aws.rs

use hcl::{Block, Body, Expression};
use crate::ir::*;

pub fn generate_terraform(resources: Vec<InfraResource>) -> String {
    let mut body = Body::builder();

    // Add provider
    body = body.add_block(
        Block::builder("terraform")
            .add_block(
                Block::builder("required_providers")
                    .add_attribute(("aws", hcl::expression!({
                        source = "hashicorp/aws"
                        version = "~> 5.0"
                    })))
                    .build()
            )
            .build()
    );

    body = body.add_block(
        Block::builder("provider")
            .add_label("aws")
            .add_attribute(("region", Expression::from("var.aws_region")))
            .build()
    );

    // Generate resources
    for resource in resources {
        match resource {
            InfraResource::Lambda(config) => {
                body = body.add_block(generate_lambda(config));
            }
            InfraResource::DynamoDB(config) => {
                body = body.add_block(generate_dynamodb(config));
            }
            InfraResource::SQS(config) => {
                body = body.add_block(generate_sqs(config));
            }
            InfraResource::EventBridge(config) => {
                body = body.add_block(generate_eventbridge(config));
            }
            InfraResource::IAMRole(config) => {
                body = body.add_block(generate_iam_role(config));
            }
            _ => {}
        }
    }

    hcl::format::to_string(&body.build()).unwrap()
}

fn generate_lambda(config: LambdaConfig) -> Block {
    Block::builder("resource")
        .add_label("aws_lambda_function")
        .add_label(&config.name)
        .add_attribute(("function_name", config.name.clone()))
        .add_attribute(("runtime", config.runtime))
        .add_attribute(("handler", "bootstrap"))
        .add_attribute(("memory_size", config.memory_mb as i64))
        .add_attribute(("timeout", config.timeout_seconds as i64))
        .add_attribute(("architectures", vec![config.architecture]))
        .add_attribute(("role", Expression::from(format!("aws_iam_role.{}_role.arn", config.name))))
        .add_block(
            Block::builder("environment")
                .add_attribute(("variables", config.environment))
                .build()
        )
        .build()
}

fn generate_dynamodb(config: DynamoDBConfig) -> Block {
    let mut builder = Block::builder("resource")
        .add_label("aws_dynamodb_table")
        .add_label(&config.table_name)
        .add_attribute(("name", config.table_name.clone()))
        .add_attribute(("billing_mode", config.billing_mode.to_string()))
        .add_attribute(("hash_key", config.partition_key.name.clone()))
        .add_block(
            Block::builder("attribute")
                .add_attribute(("name", config.partition_key.name.clone()))
                .add_attribute(("type", config.partition_key.key_type.to_string()))
                .build()
        );

    if let Some(sk) = &config.sort_key {
        builder = builder
            .add_attribute(("range_key", sk.name.clone()))
            .add_block(
                Block::builder("attribute")
                    .add_attribute(("name", sk.name.clone()))
                    .add_attribute(("type", sk.key_type.to_string()))
                    .build()
            );
    }

    if config.stream_enabled {
        builder = builder.add_attribute(("stream_enabled", true))
            .add_attribute(("stream_view_type", "NEW_AND_OLD_IMAGES"));
    }

    builder.build()
}
```

### 9.4 Generated Output Example

```hcl
# Generated by AllFrame - DO NOT EDIT MANUALLY
# Source: src/handlers/orders.rs

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# Lambda Function: create_order
resource "aws_lambda_function" "create_order" {
  function_name = "create_order"
  runtime       = "provided.al2023"
  handler       = "bootstrap"
  memory_size   = 256
  timeout       = 30
  architectures = ["arm64"]
  role          = aws_iam_role.create_order_role.arn

  environment {
    variables = {
      RUST_LOG    = "info"
      EVENT_TABLE = aws_dynamodb_table.order_events.name
    }
  }
}

# DynamoDB Table: order-events
resource "aws_dynamodb_table" "order_events" {
  name         = "order-events"
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

  stream_enabled   = true
  stream_view_type = "NEW_AND_OLD_IMAGES"
}

# SQS Queue: order-queue
resource "aws_sqs_queue" "order_queue" {
  name                       = "order-queue"
  visibility_timeout_seconds = 30
  message_retention_seconds  = 1209600

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.order_queue_dlq.arn
    maxReceiveCount     = 3
  })
}

resource "aws_sqs_queue" "order_queue_dlq" {
  name = "order-queue-dlq"
}

# IAM Role for Lambda
resource "aws_iam_role" "create_order_role" {
  name = "create_order_lambda_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy" "create_order_policy" {
  name = "create_order_policy"
  role = aws_iam_role.create_order_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:PutItem",
          "dynamodb:GetItem",
          "dynamodb:Query"
        ]
        Resource = aws_dynamodb_table.order_events.arn
      },
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}
```

---

## 10. Timeline & Milestones

### Phase 7.1: Lambda Runtime Adapter (2 weeks)

**Deliverables**:
- [ ] `LambdaRuntime` struct wrapping Tower service
- [ ] API Gateway v1/v2 event handling
- [ ] Function URL support
- [ ] Lambda context injection
- [ ] Integration tests with LocalStack

**Tests**:
- Lambda runtime initialization
- HTTP request/response mapping
- Error handling and status codes
- Cold start measurement

### Phase 7.2: DynamoDB Event Store Backend (2 weeks)

**Deliverables**:
- [ ] `DynamoDBEventStore` implementing `EventStoreBackend`
- [ ] Single-table design with PK/SK
- [ ] Optimistic concurrency with conditional writes
- [ ] S3 snapshot backend
- [ ] Auto-fallback to InMemoryBackend locally

**Tests**:
- Event append and read
- Concurrency conflict detection
- Stream version tracking
- Snapshot creation and restore

### Phase 7.3: Event Source Mappings (2 weeks)

**Deliverables**:
- [ ] EventBridge → CommandBus adapter
- [ ] SQS → CommandBus adapter
- [ ] DynamoDB Streams → ProjectionRegistry adapter
- [ ] SNS → CommandBus adapter
- [ ] Batch processing support

**Tests**:
- Event deserialization
- Command dispatch
- Error handling and DLQ
- Batch processing

### Phase 7.4: Infrastructure Generation (3 weeks)

**Deliverables**:
- [ ] Infrastructure IR definitions
- [ ] Terraform HCL generation (hcl-rs)
- [ ] AWS provider (Lambda, DynamoDB, SQS, EventBridge, IAM)
- [ ] Infrastructure macros (`#[lambda_function]`, `#[event_store]`, etc.)
- [ ] `allframe infra` CLI commands

**Tests**:
- IR construction from macros
- Valid HCL output
- Terraform validation (`terraform validate`)
- IAM policy correctness

### Phase 7.5: Cloud Run Adapter (1 week)

**Deliverables**:
- [ ] Standard Axum server configuration
- [ ] PORT environment variable handling
- [ ] Health check endpoint
- [ ] Graceful shutdown
- [ ] Dockerfile template

**Tests**:
- Server startup
- Health check response
- Graceful shutdown on SIGTERM

### Phase 7.6: Documentation & Polish (2 weeks)

**Deliverables**:
- [ ] Getting started guide
- [ ] API documentation
- [ ] Example applications
- [ ] Performance benchmarks
- [ ] Troubleshooting guide

**Total Duration**: 12 weeks

---

## 11. Success Criteria

### Functional Requirements

- [ ] AllFrame handler deploys to Lambda with `cargo lambda deploy`
- [ ] Same handler runs locally with `allframe dev`
- [ ] CQRS CommandBus works with EventBridge trigger
- [ ] Projections update from DynamoDB Streams
- [ ] `allframe infra emit terraform` generates valid HCL
- [ ] Generated Terraform applies successfully
- [ ] Cloud Run deployment works

### Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold start | < 50ms | CloudWatch Logs |
| Warm invocation | < 10ms | CloudWatch Logs |
| Binary size | < 10 MB | `ls -la target/lambda/` |
| Memory usage | < 128 MB | CloudWatch Metrics |

### Quality Requirements

- [ ] All existing allframe-core tests pass
- [ ] 80%+ code coverage on new code
- [ ] Zero `unsafe` code
- [ ] All public APIs documented
- [ ] No new Clippy warnings

---

## 12. Risks & Mitigations

### Risk 1: AWS SDK Size

**Risk**: AWS SDK crates significantly increase binary size.

**Mitigation**:
- Use feature flags to include only needed SDK components
- Evaluate `aws-lite` as lightweight alternative
- Document size optimization techniques

### Risk 2: Cold Start Variability

**Risk**: Cold starts may exceed 50ms under certain conditions.

**Mitigation**:
- Lazy initialization patterns
- Provisioned concurrency documentation
- ARM64 (Graviton) recommendation
- SnapStart evaluation (if supported for custom runtime)

### Risk 3: DynamoDB Costs

**Risk**: High-volume event sourcing may incur unexpected DynamoDB costs.

**Mitigation**:
- Default to PAY_PER_REQUEST billing
- Document capacity planning
- Implement event batching
- S3 archival for old events

### Risk 4: Infrastructure Drift

**Risk**: Generated Terraform may drift from actual infrastructure.

**Mitigation**:
- Version tracking in generated files
- `allframe infra validate` command
- CI/CD integration guide
- Terraform state locking documentation

---

## 13. Future Considerations

### Phase 8: Multi-Region & Advanced Patterns

- Global tables replication
- Cross-region event routing
- Active-active deployment patterns
- Latency-based routing

### Phase 9: Kubernetes Operator

- CRD generation from AllFrame aggregates
- Operator pattern with kube-rs
- GitOps integration
- Helm chart generation

### Phase 10: Observability Platform

- CloudWatch dashboards generation
- X-Ray tracing integration
- Custom metrics from CQRS operations
- Alerting rules generation

### Beyond: Platform Expansion

- Shuttle.rs native integration
- Fly.io Machines API
- Vercel Functions (if Rust support improves)
- Deno Deploy (via wasm, if ecosystem matures)

---

## Appendix

### A. Dependency Versions

| Dependency | Version | Purpose |
|------------|---------|---------|
| lambda_http | 0.14+ | Lambda HTTP runtime |
| lambda_runtime | 0.14+ | Lambda base runtime |
| aws_lambda_events | 0.16+ | Event type definitions |
| aws-sdk-dynamodb | 1.55+ | DynamoDB client |
| aws-sdk-s3 | 1.65+ | S3 client |
| aws-config | 1.5+ | AWS configuration |
| hcl-rs | 0.18+ | Terraform HCL generation |
| tower | 0.4+ | Service abstraction |
| tokio | 1.0+ | Async runtime |

### B. Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AWS_REGION` | AWS region | Required |
| `EVENT_TABLE` | DynamoDB table name | Required |
| `SNAPSHOT_BUCKET` | S3 bucket for snapshots | Optional |
| `RUST_LOG` | Logging level | `info` |
| `PORT` | Server port (Cloud Run) | `8080` |

### C. DynamoDB Table Schema

```
Table: {EVENT_TABLE}
├── PK (String): Stream ID (e.g., "order-123")
├── SK (String): Version (zero-padded, e.g., "00000000000000000001")
├── event_type (String): Event discriminator
├── payload (String): JSON event data
├── timestamp (String): ISO 8601 timestamp
└── metadata (Map): Optional metadata

GSI: by-timestamp
├── PK: event_type
├── SK: timestamp
└── Projects: all attributes
```

### D. References

- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [cargo-lambda](https://www.cargo-lambda.info/)
- [hcl-rs](https://github.com/martinohmann/hcl-rs)
- [AWS Lambda Rust GA Announcement](https://aws.amazon.com/about-aws/whats-new/2025/11/aws-lambda-rust/)
- [Cloud Run Rust Guide](https://cloud.google.com/run/docs/quickstarts/build-and-deploy/deploy-service-other-languages)

---

**AllFrame. One frame. Infinite transformations.**

*Built with TDD, from day zero.*
