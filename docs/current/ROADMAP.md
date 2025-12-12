# AllFrame Roadmap

> From Framework to Platform: The Journey to Cloud-Native Microservice Generation

**Version**: 2.0 (December 2025)
**Status**: Active Development

---

## Vision Statement

AllFrame evolves from a **composable Rust API framework** into a **cloud-native microservice architecture generator** that produces production-ready, serverless-first applications from declarative configuration.

```bash
# Today
allframe ignite my-project

# Tomorrow
allframe ignite --config architecture.toml
# Generates complete microservice architecture with IaC
```

---

## Completed Work (v0.1.x)

### Foundation Layer ✅

| Component | Status | Achievement |
|-----------|--------|-------------|
| **Project Scaffolding** | ✅ Complete | Clean Architecture project generation |
| **Compile-time DI** | ✅ Complete | Zero-overhead dependency injection |
| **OpenTelemetry** | ✅ Complete | Distributed tracing support |
| **Auto OpenAPI 3.1** | ✅ Complete | Automatic documentation generation |

### CQRS Infrastructure ✅ (Phases 1-5)

| Phase | Feature | Boilerplate Reduction |
|-------|---------|----------------------|
| 1 | AllSource Integration | Pluggable event store backends |
| 2 | CommandBus | 90% (30-40 → 3 lines) |
| 3 | ProjectionRegistry | 90% (50+ → 5 lines) |
| 4 | Event Versioning | 95% (30-40 → 5 lines) |
| 5 | Saga Orchestration | 75% (100+ → 20 lines) |
| **Average** | **All CQRS** | **85% reduction** |

**Tests**: 72 tests across all CQRS components

### Router & Documentation ✅ (Phase 6)

| Deliverable | Status | Impact |
|-------------|--------|--------|
| Protocol-Agnostic Routing | ✅ Complete | Write once, expose via REST/GraphQL/gRPC |
| Scalar API Docs | ✅ Complete | <50KB bundle (10x smaller than Swagger) |
| GraphiQL Playground | ✅ Complete | Modern GraphQL documentation |
| gRPC Explorer | ✅ Complete | First Rust framework with web-based gRPC docs |
| Contract Testing | ✅ Complete | Automatic test generation |

**Tests**: 78 tests across 5 phases

### Resilience & Security ✅ (v0.1.7)

| Feature | Components |
|---------|------------|
| **Resilience** | RetryExecutor, CircuitBreaker, RateLimiter, AdaptiveRetry, RetryBudget |
| **Security** | URL obfuscation, Sensitive<T> wrapper, `#[derive(Obfuscate)]` |
| **Macros** | `#[retry]`, `#[circuit_breaker]`, `#[rate_limited]` |

**Tests**: 55 tests (43 resilience + 12 security)

### Graceful Shutdown ✅ (v0.1.8)

| Feature | Description |
|---------|-------------|
| `ShutdownAwareTaskSpawner` | Named tasks with automatic cancellation |
| `GracefulShutdownExt` | Cleanup orchestration with error handling |
| `spawn_with_result` | Tasks that return values on completion |

**Tests**: 17 shutdown tests

---

## Current Status (v0.1.8)

```
┌─────────────────────────────────────────────────────────────────┐
│                     ALLFRAME v0.1.8                              │
├─────────────────────────────────────────────────────────────────┤
│  ✅ Foundation: DI, OpenAPI, OTel                               │
│  ✅ CQRS: CommandBus, Projections, Sagas, Versioning            │
│  ✅ Router: Protocol-agnostic, Scalar, GraphiQL, gRPC Explorer  │
│  ✅ Resilience: Retry, CircuitBreaker, RateLimiter              │
│  ✅ Security: Obfuscation, Safe Logging                         │
│  ✅ Shutdown: Graceful shutdown utilities                       │
├─────────────────────────────────────────────────────────────────┤
│  Tests: 361+ passing | Binary: <2MB | MSRV: 1.86               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Roadmap: AllFrame Ignite Evolution

### Phase 7: Architecture Configuration (v0.2.0)

**Goal**: Declarative multi-service architecture definition

**Deliverables**:
- [ ] TOML/YAML configuration schema
- [ ] Configuration parser with validation
- [ ] Variable interpolation (environment variables)
- [ ] Service dependency resolution
- [ ] Schema documentation

**Configuration Preview**:
```toml
[architecture]
name = "my-platform"
version = "1.0.0"

[[services]]
name = "user-service"
type = "stateless"
responsibility = "User management"
protocols = ["rest", "grpc"]

[services.messaging]
pattern = "outbox"
```

**Timeline**: Q1 2025

---

### Phase 8: Core Service Archetypes (v0.3.0) ✅

**Goal**: Code generators for fundamental service patterns

**Service Types**:
| Type | Description | Generator |
|------|-------------|-----------|
| `stateless` | Request/response CRUD | ✅ Existing (extend) |
| `event-sourced` | CQRS + Event Store | ✅ Existing (extend) |
| `consumer` | Event handler service | ✅ Complete |
| `producer` | Event publishing service | ✅ Complete |

**Patterns Generated**:
- [x] Outbox pattern for reliable messaging
- [x] Idempotency middleware
- [x] Dead Letter Queue handling
- [x] Health checks and readiness probes

**Timeline**: Q1-Q2 2025

---

### Phase 9: Advanced Service Patterns (v0.4.0) ✅

**Goal**: Complex service archetypes

**Service Types**:
| Type | Description | Use Case | Status |
|------|-------------|----------|--------|
| `saga-orchestrator` | Distributed transactions | Multi-service workflows | ✅ Complete |
| `gateway` | External integration | Payment providers, APIs | ✅ Complete |
| `bff` | Backend for Frontend | API aggregation | ✅ Complete |
| `websocket-gateway` | Real-time streaming | Live updates, chat | ✅ Complete |
| `scheduled` | Cron jobs | Reports, cleanup | ✅ Complete |
| `legacy-adapter` | Legacy system adapter | Migration, ACL | ✅ Complete |

**Bonus**: Forge MCP Server for AI-assisted code generation

**Timeline**: Q2 2025 ✅

---

### Phase 10: AWS Infrastructure (v0.5.0)

**Goal**: Terraform/Pulumi generation for AWS

**Modules**:
- [ ] Lambda + API Gateway
- [ ] Fargate for stateful services
- [ ] MSK/SQS with DLQ
- [ ] RDS/DynamoDB
- [ ] Glue Schema Registry
- [ ] Secrets Manager rotation
- [ ] CloudWatch/X-Ray integration

**Generated Structure**:
```
infrastructure/
├── terraform/
│   ├── modules/
│   │   ├── lambda-service/
│   │   ├── fargate-service/
│   │   └── dead-letter-queue/
│   └── aws/
│       ├── main.tf
│       ├── networking.tf
│       └── messaging.tf
└── docker/
    └── docker-compose.yml
```

**Timeline**: Q2-Q3 2025

---

### Phase 11: Multi-Cloud Support (v0.6.0)

**Goal**: Deploy to any cloud with minimal config changes

**Cloud Providers**:
| Provider | Serverless | Container | Status |
|----------|------------|-----------|--------|
| AWS | Lambda | Fargate | Phase 10 |
| GCP | Cloud Run | Cloud Run | 🆕 |
| Fly.io | - | Machines | 🆕 |
| Shuttle | Native | - | 🆕 |

**Features**:
- [ ] GCP Cloud Run module
- [ ] Fly.io integration
- [ ] Shuttle deployment
- [ ] Canary/Blue-Green deployments
- [ ] Multi-region strategies

**Timeline**: Q3 2025

---

### Phase 12: Architecture Templates (v0.7.0)

**Goal**: Pre-built templates for common architectures

**Templates**:

| Template | Services Generated | Use Case |
|----------|-------------------|----------|
| `ecommerce` | User, Order (saga), Payment, Inventory, Notification, BFF | Online stores |
| `data-pipeline` | Ingestion, Transform, Storage, Query | ETL/Analytics |
| `collaboration` | Auth, Document (CRDT), Presence, Sync | Real-time apps |
| `saas` | Tenant, User, Billing, Admin BFF | Multi-tenant SaaS |

**Usage**:
```bash
allframe ignite --template ecommerce --cloud aws
```

**Timeline**: Q3-Q4 2025

---

### Phase 13: Testing & Quality (v0.8.0)

**Goal**: Automated testing and quality assurance

**Features**:
- [ ] Pact contract test generation
- [ ] Integration test scaffolding
- [ ] k6 load testing setup
- [ ] Chaos engineering hooks (Gremlin integration)

**Timeline**: Q4 2025

---

### Phase 14: Production Readiness (v1.0.0)

**Goal**: Enterprise-ready release

**Features**:
- [ ] Security audit and hardening
- [ ] mTLS configuration
- [ ] Feature flags integration (Unleash, LaunchDarkly)
- [ ] Multi-tenancy support
- [ ] CI/CD pipeline generation
- [ ] Architecture documentation generation
- [ ] Performance optimization

**Success Criteria**:
- Generate deployable architecture in < 5 minutes
- Generated code passes `clippy -D warnings`
- 80%+ test coverage out of the box
- Same service runs on AWS, GCP, Fly.io

**Timeline**: Q4 2025

---

## Version Summary

| Version | Milestone | Key Feature |
|---------|-----------|-------------|
| **0.1.x** | Foundation | Framework + CQRS + Router ✅ |
| **0.2.0** | Configuration | Architecture schema |
| **0.3.0** | Core Archetypes | Consumer, Producer ✅ |
| **0.4.0** | Advanced Patterns | Gateway, Saga, BFF, WebSocket, Scheduled, Legacy Adapter ✅ |
| **0.5.0** | AWS | Lambda, Fargate, MSK |
| **0.6.0** | Multi-Cloud | GCP, Fly.io, Shuttle |
| **0.7.0** | Templates | E-commerce, Data Pipeline, SaaS |
| **0.8.0** | Quality | Contract testing, Chaos engineering |
| **1.0.0** | Production | Security audit, Enterprise features |

---

## Technical Foundation

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
```

### Code Structure (Future)

```
crates/allframe-forge/
├── src/
│   ├── config/           # Configuration schema & parsing
│   ├── archetypes/       # Service type generators
│   ├── patterns/         # Cross-cutting patterns (outbox, idempotency)
│   ├── generators/       # Code generation (Rust, Terraform, Pulumi)
│   ├── clouds/           # Cloud-specific modules
│   └── templates/        # Pre-built architecture templates
```

---

## Related Documents

- **[IGNITE_VISION.md](./IGNITE_VISION.md)** - Detailed vision with configuration examples
- **[PROJECT_STATUS.md](../PROJECT_STATUS.md)** - Current development status
- **[FEATURE_FLAGS.md](../guides/FEATURE_FLAGS.md)** - Feature flag reference

---

## Contributing

We welcome contributions! Priority areas:

1. **Configuration Schema** - Help define the architecture TOML/YAML format
2. **Cloud Modules** - Add Terraform/Pulumi modules for your cloud
3. **Service Archetypes** - Create generators for new service patterns
4. **Templates** - Build architecture templates for common use cases

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

**AllFrame Ignite: From configuration to cloud in one command.**

*One frame. Infinite transformations.* 🦀
