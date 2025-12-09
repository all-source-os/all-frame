# AllFrame Ignite Vision Roadmap

> Generative Cloud-Native Microservice Architecture

## Executive Summary

AllFrame Ignite evolves from a simple project scaffolder into a **declarative microservice architecture generator** that produces cloud-ready, serverless-first applications from a single configuration file.

```bash
# The vision
allframe ignite --config architecture.toml

# Generates:
# ├── services/
# │   ├── user-service/          # Stateless microservice
# │   ├── order-service/         # Event-driven with Saga
# │   ├── payment-gateway/       # External integration gateway
# │   ├── web-bff/               # Backend for Frontend
# │   └── notification-service/  # Consumer service
# ├── infrastructure/
# │   ├── terraform/             # Or Pulumi
# │   │   ├── aws/
# │   │   ├── gcp/
# │   │   └── fly/
# │   └── docker-compose.yml     # Local development
# ├── contracts/                 # Pact contract tests
# └── docs/
#     └── architecture.md        # Auto-generated documentation
```

---

## Current State (v0.1.x)

```
allframe ignite my-project
```

- Single service scaffolding
- Clean Architecture structure
- Basic Rust project with tokio

---

## Vision Phases

### Phase 1: Architecture Configuration Schema
**Goal**: Define the declarative configuration format

```toml
# architecture.toml - Complete Reference Example

[architecture]
name = "e-commerce-platform"
version = "1.0.0"
multi_tenant = false

# =============================================================================
# Global Settings
# =============================================================================

[settings]
event_bus = "kafka"              # kafka | rabbitmq | sqs | pubsub | nats
default_database = "postgres"    # postgres | mysql | dynamodb | firestore
observability = "opentelemetry"

# Event schema management
[settings.events]
schema_registry = "confluent"    # confluent | glue | apicurio | none
format = "avro"                  # avro | protobuf | json-schema
compatibility = "backward"       # backward | forward | full | none

# =============================================================================
# Resilience Defaults
# =============================================================================

[resilience]
circuit_breaker = { failure_threshold = 5, success_threshold = 3, timeout_ms = 30000 }
retry = { max_attempts = 3, initial_interval_ms = 100, max_interval_ms = 10000, multiplier = 2.0 }
bulkhead = { max_concurrent = 100, max_queue = 50 }
rate_limit = { rps = 1000, burst = 100 }

# =============================================================================
# Security & Compliance
# =============================================================================

[security]
secrets_manager = "aws-secrets"  # aws-secrets | vault | gcp-secrets | env
mtls = true
api_key_rotation_days = 90

[security.authentication]
provider = "cognito"             # cognito | auth0 | keycloak | custom
jwt_issuer = "${AUTH_ISSUER}"

[compliance]
audit_logging = true
pii_fields = ["email", "phone", "address", "ssn"]
data_retention_days = 365
gdpr_enabled = true
hipaa_enabled = false

# =============================================================================
# Observability
# =============================================================================

[observability]
tracing = "opentelemetry"
propagation = ["traceparent", "b3"]  # W3C Trace Context + Zipkin B3
sampling = { strategy = "probabilistic", rate = 0.1 }
metrics = "prometheus"
logging = "structured"           # structured | json | plain

[observability.baggage]
tenant_id = true
user_id = true
correlation_id = true

# =============================================================================
# Testing
# =============================================================================

[testing]
contract_testing = true
contract_broker = "pactflow"     # pactflow | pact-broker | none
chaos_engineering = false
load_testing = { tool = "k6", baseline_rps = 1000 }

# =============================================================================
# Feature Flags
# =============================================================================

[feature_flags]
provider = "unleash"             # unleash | launchdarkly | flagsmith | env
default_strategy = "gradual-rollout"
sync_interval_seconds = 10

# =============================================================================
# Multi-Tenancy (Optional)
# =============================================================================

[tenant]
enabled = false
isolation = "schema"             # schema | database | row-level
header = "X-Tenant-ID"
routing = "subdomain"            # subdomain | header | path

# =============================================================================
# Cloud Provider Configuration
# =============================================================================

[cloud]
primary = "aws"
regions = ["us-east-1", "eu-west-1"]

[cloud.aws]
account_id = "${AWS_ACCOUNT_ID}"
runtime = "lambda"               # lambda | fargate | ecs | eks

[cloud.gcp]
project_id = "${GCP_PROJECT_ID}"
runtime = "cloud-run"            # cloud-run | cloud-functions | gke

[cloud.fly]
org = "my-org"

[cloud.shuttle]
project = "my-project"

# =============================================================================
# Deployment Strategy
# =============================================================================

[deployment]
strategy = "canary"              # canary | blue-green | rolling | recreate
canary_percentage = 10
canary_duration_minutes = 15
promotion_criteria = { error_rate_max = 0.01, latency_p99_ms = 500 }
rollback_on_failure = true

# =============================================================================
# Service Definitions
# =============================================================================

# -----------------------------------------------------------------------------
# Stateless Microservice
# -----------------------------------------------------------------------------
[[services]]
name = "user-service"
type = "stateless"
responsibility = "User registration, authentication, and profile management"
protocols = ["rest", "grpc"]

[services.domain]
entities = ["User", "Profile", "Session"]
events = ["UserCreated", "UserUpdated", "UserDeleted", "SessionStarted"]

[services.ports]
inbound = ["rest:8080", "grpc:9090"]
outbound = ["postgres", "redis", "event-bus"]

[services.database]
type = "postgres"
migrations = "sqlx"              # sqlx | diesel | sea-orm
strategy = "expand-contract"     # For zero-downtime migrations

[services.idempotency]
enabled = true
key_header = "Idempotency-Key"
ttl_hours = 24
storage = "redis"

[services.messaging]
pattern = "outbox"               # outbox | direct | transactional
outbox_polling_ms = 100

# -----------------------------------------------------------------------------
# Backend for Frontend (BFF)
# -----------------------------------------------------------------------------
[[services]]
name = "web-bff"
type = "bff"
responsibility = "Aggregate APIs for web frontend"
protocols = ["rest", "graphql"]

[services.bff]
aggregates = ["user-service", "order-service", "product-service"]
caching = { ttl_seconds = 60, strategy = "stale-while-revalidate" }
rate_limit = { rps = 100, burst = 20, per = "user" }

[services.graphql]
federation = true
schema_stitching = false

# -----------------------------------------------------------------------------
# Saga Orchestrator
# -----------------------------------------------------------------------------
[[services]]
name = "order-service"
type = "saga-orchestrator"
responsibility = "Order lifecycle management with distributed transactions"
protocols = ["rest", "grpc"]

[services.saga]
name = "CreateOrderSaga"
steps = ["ValidateInventory", "ReservePayment", "CreateShipment"]
compensation = ["ReleaseInventory", "RefundPayment", "CancelShipment"]
timeout_seconds = 300
state_store = "postgres"

[services.idempotency]
enabled = true
key_header = "Idempotency-Key"
ttl_hours = 48

# -----------------------------------------------------------------------------
# Gateway Service (External Integrations)
# -----------------------------------------------------------------------------
[[services]]
name = "payment-gateway"
type = "gateway"
responsibility = "External payment provider integration (Stripe, PayPal)"

[services.integrations]
providers = ["stripe", "paypal"]
circuit_breaker = { failure_threshold = 5, timeout_ms = 30000 }
retry = { max_attempts = 3, idempotent_only = true }

[services.idempotency]
enabled = true
key_header = "Idempotency-Key"
ttl_hours = 72                   # Longer for payment operations

# -----------------------------------------------------------------------------
# Event-Sourced Service
# -----------------------------------------------------------------------------
[[services]]
name = "inventory-service"
type = "event-sourced"
responsibility = "Inventory tracking with full audit trail"

[services.cqrs]
event_store = "postgres"         # postgres | allsource | eventstore
projections = ["InventoryLevel", "StockHistory", "LowStockAlert"]
snapshot_frequency = 100         # Snapshot every N events

[services.messaging]
pattern = "outbox"

# -----------------------------------------------------------------------------
# Consumer Service
# -----------------------------------------------------------------------------
[[services]]
name = "notification-service"
type = "consumer"
responsibility = "Send notifications based on domain events"

[services.consumer]
events = ["OrderCreated", "PaymentSucceeded", "ShipmentDispatched"]
channels = ["email", "sms", "push"]
dlq = true                       # Dead Letter Queue
max_retries = 3
retry_backoff = "exponential"    # exponential | linear | fixed
concurrency = 10

# -----------------------------------------------------------------------------
# WebSocket Gateway
# -----------------------------------------------------------------------------
[[services]]
name = "realtime-gateway"
type = "websocket-gateway"
responsibility = "Real-time event streaming to clients"

[services.websocket]
protocol = "socket.io"           # socket.io | raw | graphql-subscriptions
scaling = "sticky-sessions"      # sticky-sessions | redis-adapter
presence = true
heartbeat_interval_ms = 30000
max_connections_per_instance = 10000

[services.subscriptions]
events = ["OrderStatusChanged", "InventoryUpdated", "ChatMessage"]
authorization = "jwt"

# -----------------------------------------------------------------------------
# Scheduled Service (Cron Jobs)
# -----------------------------------------------------------------------------
[[services]]
name = "report-generator"
type = "scheduled"
responsibility = "Generate and distribute scheduled reports"

[services.schedule]
cron = "0 2 * * *"               # 2 AM daily
timezone = "UTC"
timeout_minutes = 30
overlap_policy = "skip"          # skip | queue | allow

[services.jobs]
daily_sales_report = { cron = "0 6 * * *", timeout_minutes = 15 }
weekly_inventory = { cron = "0 0 * * 0", timeout_minutes = 60 }
monthly_reconciliation = { cron = "0 0 1 * *", timeout_minutes = 120 }

# -----------------------------------------------------------------------------
# Anti-Corruption Layer
# -----------------------------------------------------------------------------
[[services]]
name = "legacy-erp-adapter"
type = "anti-corruption-layer"
responsibility = "Translate legacy ERP API to domain events"

[services.acl]
upstream = "https://legacy-erp.internal"
protocol = "soap"                # soap | rest | grpc | custom
translation_map = "translations/erp.toml"
sync_strategy = "polling"        # polling | webhook | cdc
polling_interval_seconds = 60

[services.circuit_breaker]
failure_threshold = 3
timeout_ms = 60000               # Legacy systems are slow

# -----------------------------------------------------------------------------
# Stream Processor
# -----------------------------------------------------------------------------
[[services]]
name = "analytics-processor"
type = "stream-processor"
responsibility = "Real-time analytics aggregation"

[services.stream]
input_topics = ["orders", "inventory", "user-events"]
output_topics = ["analytics-results"]
windowing = { type = "tumbling", duration_seconds = 60 }
late_arrival_tolerance_seconds = 30

# -----------------------------------------------------------------------------
# Cache Service
# -----------------------------------------------------------------------------
[[services]]
name = "product-cache"
type = "cache"
responsibility = "Distributed cache for product catalog"

[services.cache]
backend = "redis"                # redis | memcached | hazelcast
strategy = "write-through"       # write-through | write-behind | cache-aside
ttl_seconds = 3600
invalidation = ["ProductUpdated", "ProductDeleted"]
```

### Phase 2: Service Archetypes
**Goal**: Implement code generators for each service pattern

#### Archetype: Stateless Microservice
```
services/user-service/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point with graceful shutdown
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entities/
│   │   │   └── user.rs          # User aggregate
│   │   ├── events/
│   │   │   └── user_events.rs   # Domain events
│   │   └── repositories/
│   │       └── user_repository.rs
│   ├── application/
│   │   ├── mod.rs
│   │   ├── commands/
│   │   │   └── create_user.rs
│   │   ├── queries/
│   │   │   └── get_user.rs
│   │   └── services/
│   │       └── user_service.rs
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── persistence/
│   │   │   └── postgres_user_repo.rs
│   │   ├── messaging/
│   │   │   ├── outbox.rs        # Outbox pattern implementation
│   │   │   └── kafka_publisher.rs
│   │   └── idempotency/
│   │       └── redis_idempotency.rs
│   └── presentation/
│       ├── mod.rs
│       ├── rest/
│       │   └── user_handlers.rs
│       └── grpc/
│           └── user_service.rs
├── proto/
│   └── user.proto
├── migrations/
│   └── 001_create_users.sql
└── tests/
    ├── integration/
    └── contract/
        └── pacts/
```

#### Archetype: Backend for Frontend (BFF)
```
services/web-bff/
├── src/
│   ├── main.rs
│   ├── aggregators/
│   │   ├── mod.rs
│   │   ├── dashboard.rs         # Aggregates multiple services
│   │   └── checkout.rs
│   ├── cache/
│   │   └── response_cache.rs
│   ├── graphql/
│   │   ├── schema.rs
│   │   ├── resolvers/
│   │   └── dataloaders/         # N+1 prevention
│   └── clients/
│       ├── user_client.rs
│       ├── order_client.rs
│       └── product_client.rs
```

#### Archetype: Saga Orchestrator
```
services/order-service/
├── src/
│   ├── domain/
│   │   └── sagas/
│   │       ├── mod.rs
│   │       ├── create_order_saga.rs
│   │       └── cancel_order_saga.rs
│   ├── application/
│   │   ├── saga_orchestrator.rs
│   │   └── compensation_handler.rs
│   └── infrastructure/
│       ├── saga_state_store.rs
│       └── saga_log.rs          # For debugging/replay
```

#### Archetype: Event-Sourced Service
```
services/inventory-service/
├── src/
│   ├── domain/
│   │   ├── aggregates/
│   │   │   └── inventory_item.rs
│   │   └── events/
│   │       └── inventory_events.rs
│   ├── application/
│   │   ├── command_handlers/
│   │   └── projections/
│   │       ├── inventory_level.rs
│   │       ├── stock_history.rs
│   │       └── low_stock_alert.rs
│   └── infrastructure/
│       ├── event_store/
│       │   └── postgres_event_store.rs
│       ├── snapshots/
│       │   └── snapshot_store.rs
│       └── outbox/
│           └── event_outbox.rs
```

#### Archetype: Consumer Service with DLQ
```
services/notification-service/
├── src/
│   ├── domain/
│   │   └── channels/
│   │       ├── email.rs
│   │       ├── sms.rs
│   │       └── push.rs
│   ├── application/
│   │   └── event_handlers/
│   │       ├── mod.rs
│   │       ├── order_created_handler.rs
│   │       └── payment_succeeded_handler.rs
│   └── infrastructure/
│       └── consumers/
│           ├── kafka_consumer.rs
│           ├── dlq_handler.rs   # Dead letter queue processing
│           └── retry_policy.rs
```

#### Archetype: WebSocket Gateway
```
services/realtime-gateway/
├── src/
│   ├── main.rs
│   ├── connections/
│   │   ├── manager.rs
│   │   └── presence.rs
│   ├── subscriptions/
│   │   ├── router.rs
│   │   └── authorization.rs
│   └── adapters/
│       ├── socket_io.rs
│       └── redis_pubsub.rs      # For scaling
```

#### Archetype: Scheduled Service
```
services/report-generator/
├── src/
│   ├── main.rs
│   ├── jobs/
│   │   ├── mod.rs
│   │   ├── daily_sales.rs
│   │   ├── weekly_inventory.rs
│   │   └── monthly_reconciliation.rs
│   ├── scheduler/
│   │   ├── cron_parser.rs
│   │   └── job_runner.rs
│   └── outputs/
│       ├── s3_storage.rs
│       └── email_sender.rs
```

#### Archetype: Anti-Corruption Layer
```
services/legacy-erp-adapter/
├── src/
│   ├── main.rs
│   ├── upstream/
│   │   ├── soap_client.rs
│   │   └── response_parser.rs
│   ├── translation/
│   │   ├── mod.rs
│   │   ├── order_translator.rs
│   │   └── product_translator.rs
│   └── downstream/
│       └── event_publisher.rs
├── translations/
│   └── erp.toml                 # Mapping definitions
```

### Phase 3: Infrastructure as Code Generation
**Goal**: Generate cloud-specific deployment configurations

#### Terraform Modules

```hcl
# infrastructure/terraform/aws/main.tf (generated)

# Dead Letter Queue for failed events
resource "aws_sqs_queue" "dlq" {
  name = "${var.service_name}-dlq"
  message_retention_seconds = 1209600  # 14 days
}

# Main queue with DLQ redrive policy
resource "aws_sqs_queue" "main" {
  name = "${var.service_name}-queue"

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = 3
  })
}

module "user_service" {
  source = "./modules/lambda-service"

  name        = "user-service"
  runtime     = "provided.al2"
  memory      = 256
  timeout     = 30

  environment = {
    DATABASE_URL     = aws_ssm_parameter.db_url.value
    KAFKA_BROKERS    = aws_msk_cluster.main.bootstrap_brokers
    REDIS_URL        = aws_elasticache_cluster.main.cache_nodes[0].address
    IDEMPOTENCY_TTL  = "86400"
    OTEL_EXPORTER_OTLP_ENDPOINT = var.otel_endpoint
  }

  vpc_config = {
    subnet_ids         = module.vpc.private_subnets
    security_group_ids = [aws_security_group.lambda.id]
  }
}

# Canary deployment
module "canary" {
  source = "./modules/canary-deployment"

  service_name     = "user-service"
  canary_percent   = 10
  promotion_criteria = {
    error_rate_threshold = 0.01
    latency_p99_threshold = 500
  }
}

# Schema Registry (Glue)
resource "aws_glue_registry" "events" {
  registry_name = "${var.project_name}-events"
}

resource "aws_glue_schema" "user_created" {
  schema_name       = "UserCreated"
  registry_arn      = aws_glue_registry.events.arn
  data_format       = "AVRO"
  compatibility     = "BACKWARD"
  schema_definition = file("${path.module}/schemas/user_created.avsc")
}
```

#### Cloud-Specific Modules

```
infrastructure/
├── terraform/
│   ├── modules/
│   │   ├── lambda-service/       # AWS Lambda + API Gateway
│   │   ├── fargate-service/      # AWS Fargate
│   │   ├── cloud-run-service/    # GCP Cloud Run
│   │   ├── fly-service/          # Fly.io Machine
│   │   ├── shuttle-service/      # Shuttle deployment
│   │   ├── canary-deployment/    # Canary/Blue-Green
│   │   ├── dead-letter-queue/    # DLQ setup
│   │   ├── schema-registry/      # Event schema management
│   │   ├── secrets-manager/      # Secrets rotation
│   │   └── feature-flags/        # Feature flag infra
│   ├── aws/
│   │   ├── main.tf
│   │   ├── networking.tf
│   │   ├── database.tf
│   │   ├── messaging.tf          # MSK/SQS/EventBridge + DLQ
│   │   ├── observability.tf      # CloudWatch/X-Ray
│   │   ├── secrets.tf            # Secrets Manager
│   │   └── waf.tf                # Web Application Firewall
│   ├── gcp/
│   │   ├── main.tf
│   │   ├── networking.tf
│   │   ├── database.tf           # Cloud SQL/Firestore
│   │   ├── messaging.tf          # Pub/Sub + DLQ
│   │   └── observability.tf      # Cloud Trace
│   └── fly/
│       └── fly.toml
├── pulumi/
│   ├── aws/
│   ├── gcp/
│   └── multi-cloud/
├── docker/
│   ├── docker-compose.yml        # Local development
│   ├── docker-compose.test.yml   # Integration tests
│   └── services/
│       └── */Dockerfile
└── k8s/                          # Kubernetes manifests (optional)
    ├── base/
    └── overlays/
        ├── dev/
        ├── staging/
        └── production/
```

### Phase 4: Demo Architecture Templates
**Goal**: Pre-built architecture templates for common patterns

#### Template: E-Commerce Platform
```bash
allframe ignite --template ecommerce --cloud aws
```

Generates:
- User Service (stateless + idempotency)
- Product Catalog (event-sourced + cache)
- Order Service (saga orchestrator)
- Payment Gateway (with circuit breaker)
- Inventory Service (event-sourced + outbox)
- Notification Service (consumer + DLQ)
- Web BFF (GraphQL federation)
- Realtime Gateway (WebSocket)
- API Gateway (Kong/AWS API Gateway)

#### Template: Event-Driven Data Pipeline
```bash
allframe ignite --template data-pipeline --cloud gcp
```

Generates:
- Ingestion Service (producer)
- Transform Service (stream processor)
- Storage Service (consumer + DLQ)
- Query Service (read model)
- Schema Registry setup

#### Template: Real-Time Collaboration
```bash
allframe ignite --template collaboration --cloud fly
```

Generates:
- Auth Service (stateless)
- Document Service (event-sourced + CRDT)
- Presence Service (WebSocket gateway)
- Sync Service (consumer)
- Cache Service

#### Template: SaaS Multi-Tenant
```bash
allframe ignite --template saas --cloud aws --multi-tenant
```

Generates:
- Tenant Service (tenant management)
- User Service (per-tenant isolation)
- Billing Service (Stripe integration)
- Admin BFF
- Tenant BFF

---

## Configuration Schema Reference

### Service Types

| Type | Description | Use Case |
|------|-------------|----------|
| `stateless` | Request/response, no local state | CRUD APIs, validation |
| `event-sourced` | Full event history, CQRS | Audit trails, complex domains |
| `saga-orchestrator` | Distributed transaction coordination | Multi-service workflows |
| `gateway` | External system integration | Payment providers, APIs |
| `consumer` | Event-driven processing | Notifications, analytics |
| `producer` | Event generation | Data ingestion, sensors |
| `stream-processor` | Real-time event transformation | ETL, aggregations |
| `bff` | Backend for Frontend | API aggregation, caching |
| `websocket-gateway` | Real-time bidirectional | Live updates, chat |
| `scheduled` | Cron/scheduled jobs | Reports, cleanup |
| `anti-corruption-layer` | Legacy system adapter | Migration, integration |
| `cache` | Distributed caching | Performance, read models |

### Messaging Patterns

| Pattern | Description | When to Use |
|---------|-------------|-------------|
| `outbox` | Transactional outbox | Guaranteed delivery with DB transaction |
| `direct` | Direct publish | Fire-and-forget, idempotent consumers |
| `transactional` | 2PC (if supported) | Strong consistency requirements |

### Cloud Runtimes

| Cloud | Serverless | Container | Kubernetes |
|-------|------------|-----------|------------|
| AWS | Lambda | Fargate | EKS |
| GCP | Cloud Functions | Cloud Run | GKE |
| Azure | Functions | Container Apps | AKS |
| Fly.io | - | Machines | - |
| Shuttle | Shuttle | - | - |

### Event Bus Options

| Option | Managed AWS | Managed GCP | Self-Hosted | DLQ Support |
|--------|-------------|-------------|-------------|-------------|
| Kafka | MSK | Confluent | Strimzi | Yes |
| RabbitMQ | AmazonMQ | - | CloudAMQP | Yes |
| SQS/SNS | Native | - | - | Yes |
| Pub/Sub | - | Native | - | Yes |
| NATS | - | - | Synadia | JetStream |
| EventBridge | Native | - | - | Yes |

### Schema Registry Options

| Option | Cloud | Format Support |
|--------|-------|----------------|
| Confluent | Any | Avro, Protobuf, JSON Schema |
| AWS Glue | AWS | Avro, JSON Schema |
| Apicurio | Self-hosted | Avro, Protobuf, JSON Schema, OpenAPI |

---

## Implementation Milestones

### Milestone 1: Schema & Parser (v0.2.0)
- [ ] Define TOML/YAML schema for architecture configuration
- [ ] Implement configuration parser with validation
- [ ] Add schema documentation and examples
- [ ] Variable interpolation (environment variables)

### Milestone 2: Core Archetypes (v0.3.0)
- [ ] Stateless microservice generator
- [ ] Event-sourced service generator
- [ ] Outbox pattern implementation
- [ ] Idempotency middleware
- [ ] Integrate with allframe-core CQRS module

### Milestone 3: Advanced Patterns (v0.4.0)
- [ ] Saga orchestrator generator
- [ ] Gateway service generator
- [ ] Consumer/Producer generators
- [ ] Dead Letter Queue handling
- [ ] BFF service generator

### Milestone 4: Real-Time & Scheduling (v0.4.5)
- [ ] WebSocket gateway generator
- [ ] Scheduled service generator
- [ ] Anti-corruption layer generator
- [ ] Stream processor generator

### Milestone 5: AWS Infrastructure (v0.5.0)
- [ ] Lambda + API Gateway module
- [ ] Fargate module
- [ ] MSK/SQS integration with DLQ
- [ ] RDS/DynamoDB setup
- [ ] Glue Schema Registry
- [ ] Secrets Manager rotation

### Milestone 6: Multi-Cloud (v0.6.0)
- [ ] GCP Cloud Run module
- [ ] Fly.io integration
- [ ] Shuttle integration
- [ ] Multi-cloud deployment strategies
- [ ] Canary/Blue-Green deployments

### Milestone 7: Demo Templates (v0.7.0)
- [ ] E-Commerce template
- [ ] Data Pipeline template
- [ ] Collaboration template
- [ ] SaaS Multi-Tenant template
- [ ] Custom template system

### Milestone 8: Testing & Quality (v0.8.0)
- [ ] Contract testing generation (Pact)
- [ ] Integration test scaffolding
- [ ] Load testing setup (k6)
- [ ] Chaos engineering hooks

### Milestone 9: Production Readiness (v1.0.0)
- [ ] Observability integration (OTel)
- [ ] Security best practices (secrets, IAM, mTLS)
- [ ] CI/CD pipeline generation
- [ ] Documentation generation
- [ ] Feature flags integration
- [ ] Multi-tenancy support

---

## Technical Architecture

### Generator Pipeline

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Configuration  │────▶│    Resolver     │────▶│   Validators    │
│   (TOML/YAML)   │     │  (Variables,    │     │  (Schema,       │
│                 │     │   Defaults)     │     │   Dependencies) │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    Emitter      │◀────│   Assembler     │◀────│   Archetypes    │
│  (File Writer)  │     │  (Combines      │     │  (Templates +   │
│                 │     │   Components)   │     │   Logic)        │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│  Post-Process   │────▶│   Verification  │
│  (Format, Lint) │     │  (Compile, Test)│
└─────────────────┘     └─────────────────┘
```

### Code Structure

```
crates/allframe-forge/
├── src/
│   ├── lib.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── schema.rs            # Configuration types
│   │   ├── parser.rs            # TOML/YAML parsing
│   │   ├── validator.rs         # Schema validation
│   │   └── resolver.rs          # Variable interpolation
│   ├── archetypes/
│   │   ├── mod.rs
│   │   ├── stateless.rs
│   │   ├── event_sourced.rs
│   │   ├── saga.rs
│   │   ├── gateway.rs
│   │   ├── consumer.rs
│   │   ├── bff.rs
│   │   ├── websocket.rs
│   │   ├── scheduled.rs
│   │   ├── acl.rs               # Anti-corruption layer
│   │   └── cache.rs
│   ├── patterns/
│   │   ├── mod.rs
│   │   ├── outbox.rs
│   │   ├── idempotency.rs
│   │   ├── dlq.rs
│   │   └── circuit_breaker.rs
│   ├── generators/
│   │   ├── mod.rs
│   │   ├── rust/                # Rust code generation
│   │   │   ├── domain.rs
│   │   │   ├── application.rs
│   │   │   ├── infrastructure.rs
│   │   │   └── presentation.rs
│   │   ├── infrastructure/      # IaC generation
│   │   │   ├── terraform/
│   │   │   └── pulumi/
│   │   └── testing/             # Test generation
│   │       ├── contract.rs
│   │       └── integration.rs
│   ├── clouds/
│   │   ├── mod.rs
│   │   ├── aws.rs
│   │   ├── gcp.rs
│   │   ├── fly.rs
│   │   └── shuttle.rs
│   └── templates/
│       ├── mod.rs
│       ├── ecommerce/
│       ├── data_pipeline/
│       ├── collaboration/
│       └── saas/
```

---

## Example: Generated User Service with Outbox Pattern

```rust
// services/user-service/src/main.rs (generated)

use allframe_core::prelude::*;
use allframe_core::shutdown::{GracefulShutdown, ShutdownAwareTaskSpawner};
use allframe_core::resilience::{CircuitBreaker, RetryExecutor};
use std::sync::Arc;

mod domain;
mod application;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize observability
    let _guard = allframe_core::otel::init_tracing("user-service")?;

    // Setup graceful shutdown
    let shutdown = Arc::new(GracefulShutdown::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build());
    let spawner = ShutdownAwareTaskSpawner::new(shutdown.clone());

    // Wire dependencies
    let db_pool = infrastructure::create_db_pool().await?;
    let redis = infrastructure::create_redis_client().await?;
    let kafka_producer = infrastructure::create_kafka_producer().await?;

    // Repositories and services
    let user_repo = Arc::new(infrastructure::PostgresUserRepository::new(db_pool.clone()));
    let outbox_repo = Arc::new(infrastructure::PostgresOutboxRepository::new(db_pool.clone()));
    let idempotency = Arc::new(infrastructure::RedisIdempotencyStore::new(redis.clone()));

    let user_service = application::UserService::new(
        user_repo.clone(),
        outbox_repo.clone(),
        idempotency.clone(),
    );

    // Start outbox processor (publishes events from outbox table)
    let outbox_processor = infrastructure::OutboxProcessor::new(
        outbox_repo.clone(),
        kafka_producer.clone(),
        std::time::Duration::from_millis(100),
    );
    spawner.spawn("outbox-processor", || outbox_processor.run());

    // Start HTTP server
    let rest_server = presentation::rest::create_server(user_service.clone());
    spawner.spawn("rest-server", || rest_server.serve());

    // Start gRPC server
    let grpc_server = presentation::grpc::create_server(user_service);
    spawner.spawn("grpc-server", || grpc_server.serve());

    // Wait for shutdown signal
    let signal = shutdown.wait().await;
    tracing::info!("Received {}, starting graceful shutdown", signal);

    // Cleanup
    shutdown.perform_shutdown(|| async {
        // Wait for outbox to drain
        outbox_processor.drain().await?;
        db_pool.close().await;
        Ok::<_, anyhow::Error>(())
    }).await?;

    Ok(())
}
```

```rust
// services/user-service/src/infrastructure/outbox.rs (generated)

use crate::domain::events::UserEvent;
use sqlx::PgPool;

pub struct PostgresOutboxRepository {
    pool: PgPool,
}

impl PostgresOutboxRepository {
    pub async fn store(&self, event: &UserEvent) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
            event.id,
            "User",
            event.aggregate_id,
            event.event_type(),
            serde_json::to_value(event)?,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>, Error> {
        sqlx::query_as!(
            OutboxEntry,
            r#"
            SELECT * FROM outbox
            WHERE published_at IS NULL
            ORDER BY created_at
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_published(&self, ids: &[Uuid]) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE outbox SET published_at = NOW() WHERE id = ANY($1)",
            ids
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

---

## Success Metrics

1. **Time to Production**: Generate a complete, deployable microservice architecture in < 5 minutes
2. **Code Quality**: Generated code passes clippy with `deny(warnings)`, 80%+ test coverage
3. **Cloud Parity**: Same service runs on AWS, GCP, and Fly.io with minimal config changes
4. **Pattern Coverage**: Support 95% of common microservice patterns
5. **Developer Experience**: Zero manual infrastructure code needed for MVP deployments
6. **Reliability**: Built-in resilience patterns (outbox, DLQ, idempotency) by default
7. **Observability**: Full distributed tracing out of the box

---

## Related Documents

- [PROJECT_STATUS.md](../PROJECT_STATUS.md) - Current project status
- [FEATURE_FLAGS.md](../guides/FEATURE_FLAGS.md) - Feature flag reference
- [allframe-core CQRS](../../crates/allframe-core/src/cqrs.rs) - Event sourcing implementation
- [allframe-core Resilience](../../crates/allframe-core/src/resilience.rs) - Circuit breaker, retry
- [allframe-core Shutdown](../../crates/allframe-core/src/shutdown.rs) - Graceful shutdown utilities

---

*AllFrame Ignite: From configuration to cloud in one command.*
