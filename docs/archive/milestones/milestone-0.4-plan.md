# Milestone 0.4: OTEL + CQRS + Clean Architecture Enforcement

**Status**: Planning
**Target**: 100% test coverage, TDD-first approach
**Feature flag**: `arch`, `cqrs`, `otel`

## Overview

Milestone 0.4 adds three critical production features:
1. **Clean Architecture Enforcement** - Compile-time layer boundary validation
2. **CQRS + Event Sourcing** - Command/Query separation with event sourcing
3. **OpenTelemetry** - Auto-instrumentation for observability

## Implementation Order

### Phase 1: Clean Architecture Enforcement (Week 1)
**Feature flag**: `arch`
**Tests**: `tests/05_arch_*.rs`

#### Architecture Layers

```rust
// Layer 1: Domain (no dependencies)
#[domain]
struct User {
    id: UserId,
    email: Email,
}

// Layer 2: Repository (depends on domain)
#[repository]
trait UserRepository {
    async fn save(&self, user: User);
}

// Layer 3: Use Case (depends on domain + repository)
#[use_case]
struct CreateUserUseCase {
    repo: Arc<dyn UserRepository>,
}

// Layer 4: Handler (depends on use case)
#[handler]
async fn create_user_handler(
    use_case: Arc<CreateUserUseCase>,
) -> Result<User> {
    use_case.execute().await
}
```

#### Test Plan (RED Phase)

**File**: `tests/05_arch_layers.rs`
1. `test_domain_has_no_dependencies` - Domain entities compile without any dependencies
2. `test_repository_depends_on_domain` - Repository can use domain types
3. `test_use_case_depends_on_repository` - Use case can inject repository
4. `test_handler_depends_on_use_case` - Handler can inject use case
5. `test_handler_cannot_depend_on_repository` - **Must fail to compile**

**File**: `tests/05_arch_violations.rs`
1. `test_domain_cannot_depend_on_repository` - **Compile error**
2. `test_domain_cannot_depend_on_use_case` - **Compile error**
3. `test_repository_cannot_depend_on_use_case` - **Compile error**
4. `test_handler_cannot_skip_use_case_layer` - **Compile error**
5. `test_circular_dependency_prevented` - **Compile error**

**File**: `tests/05_arch_integration.rs`
1. `test_full_clean_architecture_flow` - End-to-end with all layers
2. `test_multiple_use_cases_share_repository` - DI integration
3. `test_handler_can_have_multiple_use_cases` - Multiple dependencies
4. `test_layer_metadata_available_at_runtime` - Reflection/debugging
5. `test_architecture_diagram_generation` - Auto-generate mermaid diagram

#### Implementation Tasks

1. **Proc Macros** (`crates/allframe-macros/src/arch/`)
   - `#[domain]` - Marks domain entities (Layer 1)
   - `#[repository]` - Marks repositories (Layer 2)
   - `#[use_case]` - Marks use cases (Layer 3)
   - `#[handler]` - Marks handlers (Layer 4)

2. **Layer Validation** (`crates/allframe-core/src/arch/`)
   - `LayerValidator` - Compile-time dependency checker
   - `DependencyGraph` - Tracks layer relationships
   - `ViolationReporter` - Clear error messages

3. **Runtime Metadata** (for tooling)
   - `LayerMetadata` - Store layer information
   - `ArchitectureInspector` - Query architecture at runtime
   - `DiagramGenerator` - Generate architecture diagrams

#### Acceptance Criteria

- [ ] 15/15 tests passing
- [ ] All architecture violations fail to compile
- [ ] Clear error messages for violations
- [ ] Works with existing DI system
- [ ] Documentation with examples
- [ ] Feature flag `arch` controls compilation

---

### Phase 2: CQRS + Event Sourcing (Week 2)
**Feature flag**: `cqrs`
**Tests**: `tests/06_cqrs_*.rs`

#### Core Concepts

```rust
// Commands (write operations)
#[command]
struct CreateUserCommand {
    email: String,
    name: String,
}

// Events (what happened)
#[event]
struct UserCreatedEvent {
    user_id: UserId,
    email: String,
    timestamp: DateTime,
}

// Command Handler
#[command_handler]
async fn handle_create_user(
    cmd: CreateUserCommand,
    event_store: Arc<EventStore>,
) -> Result<Vec<Event>> {
    // Validate, create events
    Ok(vec![UserCreatedEvent { ... }])
}

// Query (read operations)
#[query]
struct GetUserQuery {
    user_id: UserId,
}

// Projection (read model)
#[projection]
struct UserProjection {
    users: HashMap<UserId, User>,
}

impl Projection for UserProjection {
    fn apply(&mut self, event: &Event) {
        match event {
            UserCreatedEvent { user_id, .. } => {
                self.users.insert(*user_id, ...);
            }
        }
    }
}
```

#### Test Plan (RED Phase)

**File**: `tests/06_cqrs_commands.rs`
1. `test_command_handler_execution` - Execute command, produce events
2. `test_command_validation` - Invalid commands rejected
3. `test_command_handler_composition` - Multiple handlers
4. `test_command_idempotency` - Same command produces same events
5. `test_command_ordering` - Commands execute in order

**File**: `tests/06_cqrs_events.rs`
1. `test_event_store_append` - Store events
2. `test_event_store_replay` - Rebuild state from events
3. `test_event_versioning` - Handle event schema evolution
4. `test_event_serialization` - Events persist correctly
5. `test_event_stream_subscribe` - Real-time event streaming

**File**: `tests/06_cqrs_queries.rs`
1. `test_query_handler_execution` - Execute query, return data
2. `test_query_projection_update` - Projection updates from events
3. `test_query_eventual_consistency` - Read after write consistency
4. `test_multiple_projections` - Same events, different read models
5. `test_projection_rebuild` - Replay all events to rebuild projection

**File**: `tests/06_cqrs_integration.rs`
1. `test_full_cqrs_flow` - Command → Event → Projection → Query
2. `test_cqrs_with_clean_architecture` - CQRS uses layer enforcement
3. `test_event_sourcing_aggregate` - Aggregate root from events
4. `test_snapshot_optimization` - Snapshots for performance
5. `test_saga_coordination` - Multi-aggregate transactions

**File**: `tests/06_cqrs_property.rs` (Property-based testing)
1. `proptest_command_event_invariants` - Commands always produce valid events
2. `proptest_projection_consistency` - Same events = same state
3. `proptest_event_replay_deterministic` - Replay is deterministic
4. `proptest_concurrent_commands` - Race condition handling
5. `proptest_event_store_integrity` - No data loss

#### Implementation Tasks

1. **Command System**
   - `Command` trait
   - `CommandHandler` trait
   - `CommandBus` for routing
   - Validation pipeline

2. **Event Store**
   - `EventStore` trait
   - In-memory implementation
   - Event serialization
   - Event streaming

3. **Query System**
   - `Query` trait
   - `QueryHandler` trait
   - `Projection` trait
   - Projection rebuilding

4. **Integration**
   - CQRS + Clean Architecture
   - CQRS + DI
   - Event-driven handler invocation

#### Acceptance Criteria

- [ ] 25/25 tests passing (including property tests)
- [ ] Event store with replay capability
- [ ] Projections update automatically
- [ ] Property tests verify invariants
- [ ] Works with architecture enforcement
- [ ] Feature flag `cqrs` controls compilation

---

### Phase 3: OpenTelemetry Auto-Instrumentation (Week 3)
**Feature flag**: `otel`
**Tests**: `tests/07_otel_*.rs`

#### Auto-Instrumentation

```rust
// Handlers automatically traced
#[handler]
#[traced] // Auto-added by framework
async fn get_user(id: UserId) -> Result<User> {
    // Span created automatically:
    // - Name: "handler.get_user"
    // - Attributes: user_id, timestamp
    // - Parent: request span
    // - Duration: measured automatically
}

// Use cases traced with DI context
#[use_case]
struct GetUserUseCase {
    #[trace_inject] // Span context propagated
    repo: Arc<dyn UserRepository>,
}

// Repository calls traced
#[repository]
trait UserRepository {
    #[traced] // DB spans created
    async fn find(&self, id: UserId) -> Option<User>;
}
```

#### Test Plan (RED Phase)

**File**: `tests/07_otel_tracing.rs`
1. `test_handler_auto_traced` - Handler creates span
2. `test_span_attributes` - Span has correct metadata
3. `test_span_hierarchy` - Parent-child relationships correct
4. `test_error_spans` - Errors recorded in spans
5. `test_async_span_propagation` - Spans work across await points

**File**: `tests/07_otel_context.rs`
1. `test_context_propagation_through_di` - Context flows through DI
2. `test_trace_id_consistency` - Same trace ID throughout request
3. `test_baggage_propagation` - Custom context data propagates
4. `test_distributed_tracing` - Cross-service trace correlation
5. `test_context_extraction_from_headers` - HTTP header parsing

**File**: `tests/07_otel_export.rs`
1. `test_export_to_stdout` - Console exporter works
2. `test_export_to_jaeger` - Jaeger exporter works
3. `test_export_to_otlp` - OTLP exporter works
4. `test_batch_export` - Efficient batch export
5. `test_sampling_configuration` - Configurable sampling

**File**: `tests/07_otel_metrics.rs`
1. `test_request_counter` - Count requests automatically
2. `test_request_duration_histogram` - Measure latency
3. `test_custom_metrics` - User-defined metrics
4. `test_metric_labels` - Proper label propagation
5. `test_metric_aggregation` - Correct aggregation

**File**: `tests/07_otel_integration.rs`
1. `test_otel_with_router` - All protocols traced
2. `test_otel_with_clean_arch` - Each layer traced separately
3. `test_otel_with_cqrs` - Commands/Events/Queries traced
4. `test_otel_performance_overhead` - < 5% overhead
5. `test_otel_configuration` - Config-driven setup

#### Implementation Tasks

1. **Tracing Core**
   - `TracedHandler` wrapper
   - Automatic span creation
   - Span attribute extraction
   - Error recording

2. **Context Propagation**
   - DI integration for context
   - Async context management
   - Cross-boundary propagation
   - HTTP header extraction/injection

3. **Exporters**
   - Console/stdout exporter
   - Jaeger exporter
   - OTLP exporter
   - Configurable backends

4. **Metrics**
   - Auto-metrics for handlers
   - Custom metric macros
   - Histogram support
   - Counter support

#### Acceptance Criteria

- [ ] 20/20 tests passing
- [ ] Zero manual instrumentation required
- [ ] < 5% performance overhead
- [ ] Multiple exporter backends
- [ ] Works with all previous features
- [ ] Feature flag `otel` controls compilation

---

## Total Test Count

**Milestone 0.4 Target**: 60 new tests

- Phase 1 (Clean Arch): 15 tests
- Phase 2 (CQRS): 25 tests (including property tests)
- Phase 3 (OTEL): 20 tests

**Running total**: 35 (0.3) + 60 (0.4) = **95 tests**

## Success Criteria

1. ✅ All 60 tests pass with 100% coverage
2. ✅ Architecture violations fail to compile with clear errors
3. ✅ CQRS property tests verify invariants
4. ✅ OTEL tracing works across all features
5. ✅ Feature flags work independently and together
6. ✅ Documentation with examples for each feature
7. ✅ Performance benchmarks show < 5% overhead

## Next Steps

1. Create `tests/05_arch_layers.rs` (RED phase)
2. Implement `#[domain]`, `#[repository]`, `#[use_case]`, `#[handler]` macros
3. Build layer validation in proc macro
4. Convert tests to GREEN
5. Refactor and optimize
6. Move to Phase 2 (CQRS)
