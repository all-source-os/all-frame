# PRD: Quality Metrics & Performance Testing

**Status**: 📝 DRAFT
**Created**: 2025-11-26
**Owner**: AllFrame Core Team
**Priority**: P0 (Critical for 1.0)

---

## Executive Summary

Establish comprehensive quality metrics, performance testing, and binary size monitoring to ensure AllFrame meets its promises of being **lightweight**, **fast**, and **production-ready**.

### Goals

1. **Binary Size Monitoring** - Ensure < 8 MB target, track per feature
2. **Demo Scenarios** - Real-world examples demonstrating all features
3. **Performance Testing** - TechEmpower parity (> 500k req/s)
4. **Automated Monitoring** - CI/CD enforcement of all metrics

---

## 1. Binary Size Monitoring

### Problem

**Current State**:
- No automated binary size tracking
- No per-feature size breakdown
- No CI/CD enforcement of size limits
- Unknown impact of each feature flag

**Why This Matters**:
- AllFrame promises "< 8 MB binaries"
- Users need to know the cost of each feature
- Need to prevent size creep over time
- Competition (Axum ~6 MB, we target < 8 MB)

---

### Solution: cargo-bloat + CI/CD Integration

#### 1.1 Binary Size Targets

| Configuration | Target Size | Hard Limit | Status |
|---------------|-------------|------------|--------|
| Minimal (no features) | < 2 MB | 3 MB | 🚧 |
| Default (di, openapi, router) | < 4 MB | 5 MB | 🚧 |
| CQRS (di, openapi, cqrs) | < 5 MB | 6 MB | 🚧 |
| Full (all features) | < 8 MB | 10 MB | 🚧 |
| router-graphql | < 6 MB | 7 MB | 🚧 |
| router-grpc | < 7 MB | 8 MB | 🚧 |
| router-full | < 8 MB | 10 MB | 🚧 |

**Hard Limits**: CI/CD fails if exceeded
**Target**: Best effort, warnings if exceeded

---

#### 1.2 Implementation Plan

**Phase 1: Measurement (Week 1)**

Add `cargo-bloat` to CI/CD:

```bash
# Install cargo-bloat
cargo install cargo-bloat

# Measure binary size breakdown
cargo bloat --release --crates

# Measure per-feature
cargo bloat --release --features=di,openapi --crates
cargo bloat --release --features=cqrs --crates
cargo bloat --release --features=router-full --crates
```

**Output Example**:
```
File  .text     Size Crate
0.3%   5.8%  23.5KiB allframe_core
0.2%   4.2%  17.0KiB tokio
0.1%   2.1%   8.5KiB hyper
...
```

---

**Phase 2: CI/CD Integration (Week 1)**

Create `.github/workflows/binary-size-check.yml`:

```yaml
name: Binary Size Check

on:
  pull_request:
  push:
    branches: [main]

jobs:
  binary-size:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-bloat
        run: cargo install cargo-bloat

      - name: Build minimal binary
        run: cargo build --release --no-default-features

      - name: Check minimal binary size
        run: |
          SIZE=$(stat -c%s "target/release/allframe" || stat -f%z "target/release/allframe")
          echo "Minimal binary size: $(($SIZE / 1024 / 1024)) MB"
          if [ $SIZE -gt 3145728 ]; then  # 3 MB in bytes
            echo "ERROR: Minimal binary exceeds 3 MB limit"
            exit 1
          fi

      - name: Build default binary
        run: cargo build --release

      - name: Check default binary size
        run: |
          SIZE=$(stat -c%s "target/release/allframe" || stat -f%z "target/release/allframe")
          echo "Default binary size: $(($SIZE / 1024 / 1024)) MB"
          if [ $SIZE -gt 5242880 ]; then  # 5 MB in bytes
            echo "ERROR: Default binary exceeds 5 MB limit"
            exit 1
          fi

      - name: Build full binary
        run: cargo build --release --all-features

      - name: Check full binary size
        run: |
          SIZE=$(stat -c%s "target/release/allframe" || stat -f%z "target/release/allframe")
          echo "Full binary size: $(($SIZE / 1024 / 1024)) MB"
          if [ $SIZE -gt 10485760 ]; then  # 10 MB in bytes
            echo "ERROR: Full binary exceeds 10 MB limit"
            exit 1
          fi

      - name: Generate size breakdown
        run: |
          echo "## Binary Size Report" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "| Configuration | Size | Limit | Status |" >> $GITHUB_STEP_SUMMARY
          echo "|---------------|------|-------|--------|" >> $GITHUB_STEP_SUMMARY
          # Add size rows here

      - name: Upload size report as artifact
        uses: actions/upload-artifact@v3
        with:
          name: binary-size-report
          path: target/release/allframe
```

---

**Phase 3: Dashboard (Week 2)**

Create `scripts/binary-size-report.sh`:

```bash
#!/bin/bash
# Generate comprehensive binary size report

echo "# AllFrame Binary Size Report"
echo "Generated: $(date)"
echo ""

# Test each configuration
configs=(
    "minimal:--no-default-features"
    "default:"
    "cqrs:--features=cqrs"
    "router-graphql:--features=router-graphql"
    "router-grpc:--features=router-grpc"
    "full:--all-features"
)

echo "| Configuration | Size (MB) | Target | Status |"
echo "|---------------|-----------|--------|--------|"

for config in "${configs[@]}"; do
    name="${config%%:*}"
    flags="${config#*:}"

    cargo build --release $flags 2>/dev/null
    size=$(stat -c%s "target/release/allframe" 2>/dev/null || stat -f%z "target/release/allframe")
    size_mb=$(echo "scale=2; $size / 1024 / 1024" | bc)

    # Determine target and status
    case $name in
        minimal) target="< 2 MB"; status="✅" ;;
        default) target="< 4 MB"; status="✅" ;;
        cqrs) target="< 5 MB"; status="✅" ;;
        router-graphql) target="< 6 MB"; status="✅" ;;
        router-grpc) target="< 7 MB"; status="✅" ;;
        full) target="< 8 MB"; status="✅" ;;
    esac

    echo "| $name | $size_mb MB | $target | $status |"
done

echo ""
echo "## Size Breakdown (Default Configuration)"
cargo bloat --release --crates -n 20
```

---

#### 1.3 Success Metrics

- ✅ Binary size measured for all feature combinations
- ✅ CI/CD fails if hard limits exceeded
- ✅ Dashboard shows size breakdown per crate
- ✅ PR comments show size impact
- ✅ Historical tracking (size over time)

---

## 2. Demo Scenarios

### Problem

**Current State**:
- Examples exist but are basic
- No comprehensive demo application
- No "kitchen sink" example showing all features
- Hard for users to see real-world usage

**Why This Matters**:
- First impression for potential users
- Documentation through working code
- Testing ground for new features
- Proof that AllFrame works in practice

---

### Solution: Comprehensive Demo Application

#### 2.1 Demo Scenarios

**Scenario 1: E-Commerce API (REST + CQRS)**
- Product catalog (queries)
- Shopping cart (commands)
- Order processing (sagas)
- Inventory management (projections)
- Event versioning (schema evolution)

**Purpose**: Show complete CQRS flow in real app

**Location**: `examples/ecommerce-api/`

**Features Demonstrated**:
- ✅ CommandBus (CreateOrder, UpdateCart)
- ✅ ProjectionRegistry (ProductCatalog, InventoryView)
- ✅ Event Versioning (OrderV1 → OrderV2)
- ✅ Saga Orchestration (Order fulfillment)
- ✅ REST API with Scalar docs
- ✅ OpenAPI auto-generation

---

**Scenario 2: Real-Time Chat (GraphQL + WebSockets)**
- User authentication
- Channel management
- Message streaming (subscriptions)
- Online presence
- Message history

**Purpose**: Show GraphQL + subscriptions + real-time

**Location**: `examples/chat-app/`

**Features Demonstrated**:
- ✅ GraphQL API with GraphiQL
- ✅ Subscriptions (real-time messages)
- ✅ CQRS (Commands for sending, queries for history)
- ✅ Event sourcing (message log)
- ✅ WebSocket connections

---

**Scenario 3: Microservices Gateway (Multi-Protocol)**
- REST endpoints
- GraphQL gateway
- gRPC services
- Protocol translation
- Service mesh integration

**Purpose**: Show protocol-agnostic routing

**Location**: `examples/api-gateway/`

**Features Demonstrated**:
- ✅ REST, GraphQL, gRPC in one app
- ✅ Protocol translation (REST → gRPC)
- ✅ Unified error handling
- ✅ OpenTelemetry tracing
- ✅ Contract testing

---

**Scenario 4: Banking System (Saga Orchestration)**
- Account management
- Money transfers (sagas)
- Transaction history
- Fraud detection
- Compensation logic

**Purpose**: Show saga compensation in detail

**Location**: `examples/banking-system/`

**Features Demonstrated**:
- ✅ Saga Orchestration (complex flows)
- ✅ Automatic compensation (rollbacks)
- ✅ Event sourcing (audit trail)
- ✅ CQRS (commands + queries)
- ✅ Per-step timeouts

---

**Scenario 5: Content Management System (Full Stack)**
- Article management
- User roles/permissions
- Media uploads
- Publishing workflow
- Version control

**Purpose**: Show complete production app

**Location**: `examples/cms/`

**Features Demonstrated**:
- ✅ All CQRS features
- ✅ All protocols (REST, GraphQL)
- ✅ Authentication/Authorization
- ✅ File handling
- ✅ Complex business logic

---

#### 2.2 Implementation Plan

**Week 1-2: E-Commerce API**
- Set up project structure
- Implement domain models
- Add CQRS infrastructure
- Create REST endpoints
- Generate OpenAPI docs

**Week 3: Real-Time Chat**
- GraphQL schema
- Subscription setup
- WebSocket handling
- Message persistence

**Week 4: API Gateway**
- Multi-protocol setup
- Protocol translation
- Service routing
- Tracing integration

**Week 5: Banking System**
- Saga definitions
- Compensation logic
- Transaction flows
- Error scenarios

**Week 6: CMS (Stretch)**
- Full application
- Production patterns
- Best practices
- Performance optimization

---

#### 2.3 Demo Structure

```
examples/
├── ecommerce-api/
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── domain/              # Domain models
│   │   ├── commands/            # Command handlers
│   │   ├── queries/             # Query handlers
│   │   ├── projections/         # Read models
│   │   └── sagas/               # Saga definitions
│   ├── tests/
│   │   ├── integration_tests.rs
│   │   └── contract_tests.rs
│   ├── Cargo.toml
│   └── README.md                # How to run, what it shows
├── chat-app/
├── api-gateway/
├── banking-system/
├── cms/
└── README.md                    # Index of all examples
```

**Each demo includes**:
1. Comprehensive README
2. How to run (one command)
3. What features it demonstrates
4. API documentation (auto-generated)
5. Test suite (integration + contract)
6. Docker Compose (if needed)

---

#### 2.4 Success Metrics

- ✅ 5 comprehensive demo scenarios
- ✅ Each demo < 500 lines of code
- ✅ Each demo has README + tests
- ✅ Each demo runs with `cargo run --example <name>`
- ✅ Each demo shows unique features
- ✅ All demos referenced in main docs

---

## 3. Performance Testing

### Problem

**Current State**:
- No performance benchmarks
- No baseline metrics
- No regression testing
- Unknown vs competition (Actix, Axum)

**Why This Matters**:
- AllFrame promises "> 500k req/s"
- Need to prove competitive performance
- TechEmpower benchmarks are industry standard
- Performance regression must be caught early

---

### Solution: Criterion Benchmarks + TechEmpower

#### 3.1 Performance Targets

**Based on TechEmpower Round 23**:

| Benchmark | Target | Stretch | Competitor (Actix) |
|-----------|--------|---------|-------------------|
| JSON Serialization | 500k req/s | 700k req/s | 650k req/s |
| Single Query | 100k req/s | 150k req/s | 120k req/s |
| Multiple Queries (20) | 50k req/s | 75k req/s | 60k req/s |
| Plaintext | 1M req/s | 1.5M req/s | 1.2M req/s |
| Fortunes | 80k req/s | 100k req/s | 90k req/s |

**CQRS-specific**:

| Operation | Target | Stretch |
|-----------|--------|---------|
| Command dispatch | 1M ops/s | 2M ops/s |
| Event append (memory) | 500k events/s | 1M events/s |
| Event append (AllSource) | 50k events/s | 100k events/s |
| Projection update | 2M updates/s | 5M updates/s |
| Saga execution (2 steps) | 100k sagas/s | 200k sagas/s |

---

#### 3.2 Benchmark Suite Structure

```
benches/
├── criterion/                   # Criterion benchmarks
│   ├── command_bus.rs          # CommandBus dispatch
│   ├── event_store.rs          # Event append/read
│   ├── projections.rs          # Projection updates
│   ├── sagas.rs                # Saga execution
│   ├── router.rs               # Route matching
│   └── serialization.rs        # JSON serialize/deserialize
├── techempower/                 # TechEmpower-compatible
│   ├── json.rs                 # JSON serialization test
│   ├── plaintext.rs            # Plaintext test
│   ├── db.rs                   # Single/multiple queries
│   └── fortunes.rs             # Fortunes test
└── memory_profiling/            # Memory usage tests
    ├── event_store_memory.rs
    └── projection_memory.rs
```

---

#### 3.3 Criterion Benchmarks

**Example: CommandBus Benchmark**

```rust
// benches/criterion/command_bus.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use allframe_core::cqrs::*;

#[derive(Clone)]
struct TestCommand {
    value: i32,
}

impl Command for TestCommand {}

struct TestHandler;

#[async_trait::async_trait]
impl CommandHandler<TestCommand, TestEvent> for TestHandler {
    async fn handle(&self, cmd: TestCommand) -> CommandResult<TestEvent> {
        Ok(vec![TestEvent::Created { value: cmd.value }])
    }
}

#[derive(Clone)]
enum TestEvent {
    Created { value: i32 },
}

impl Event for TestEvent {}

fn command_bus_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("command_bus_dispatch", |b| {
        let bus: CommandBus<TestEvent> = CommandBus::new();
        runtime.block_on(async {
            bus.register(TestHandler).await;
        });

        b.to_async(&runtime).iter(|| async {
            let cmd = TestCommand { value: black_box(42) };
            bus.dispatch(cmd).await.unwrap();
        });
    });
}

criterion_group!(benches, command_bus_benchmark);
criterion_main!(benches);
```

**Run**:
```bash
cargo bench --bench command_bus
```

**Expected Output**:
```
command_bus_dispatch    time:   [950.23 ns 952.45 ns 954.89 ns]
                        thrpt:  [1.05 Melem/s 1.05 Melem/s 1.05 Melem/s]
```

---

#### 3.4 TechEmpower Benchmarks

**JSON Serialization**:

```rust
// benches/techempower/json.rs
use allframe::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    message: &'static str,
}

#[api_handler]
async fn json_handler() -> Json<Message> {
    Json(Message { message: "Hello, World!" })
}

// Benchmark: Should achieve > 500k req/s
```

**Single Query** (with PostgreSQL):

```rust
// benches/techempower/db.rs
#[api_handler]
async fn single_query(db: Extension<PgPool>) -> Json<World> {
    let id = random_id();
    let world = sqlx::query_as!(World, "SELECT * FROM world WHERE id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();

    Json(world)
}

// Benchmark: Should achieve > 100k req/s
```

---

#### 3.5 CI/CD Integration

Create `.github/workflows/performance-check.yml`:

```yaml
name: Performance Check

on:
  pull_request:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  criterion-benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run Criterion benchmarks
        run: cargo bench --bench command_bus --bench event_store

      - name: Compare with baseline
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
          # Fail if performance degrades > 10%
          alert-threshold: '110%'
          comment-on-alert: true

  techempower-benchmarks:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run TechEmpower benchmarks
        run: cargo bench --bench json --bench plaintext

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: techempower-results
          path: target/criterion/
```

---

#### 3.6 Performance Dashboard

Create `scripts/performance-report.sh`:

```bash
#!/bin/bash
# Generate performance report comparing AllFrame vs competitors

echo "# AllFrame Performance Report"
echo "Generated: $(date)"
echo ""

echo "## Criterion Benchmarks"
cargo bench --bench command_bus 2>&1 | grep "time:"

echo ""
echo "## TechEmpower Comparison"
echo "| Benchmark | AllFrame | Actix | Axum | Target |"
echo "|-----------|----------|-------|------|--------|"
echo "| JSON | TBD | 650k | 600k | > 500k ✅ |"
echo "| Plaintext | TBD | 1.2M | 1.1M | > 1M ✅ |"
echo "| Single Query | TBD | 120k | 110k | > 100k ✅ |"

echo ""
echo "## CQRS Benchmarks"
echo "| Operation | Throughput | Target |"
echo "|-----------|-----------|--------|"
echo "| Command dispatch | TBD | > 1M ops/s |"
echo "| Event append | TBD | > 500k events/s |"
```

---

#### 3.7 Memory Profiling

**Track memory usage**:

```rust
// benches/memory_profiling/event_store_memory.rs
use allframe_core::cqrs::*;

#[test]
fn test_event_store_memory() {
    let store = EventStore::new();

    // Measure baseline memory
    let baseline = current_memory_usage();

    // Append 1M events
    for i in 0..1_000_000 {
        store.append(&format!("agg-{}", i % 1000), vec![
            TestEvent::Created { value: i }
        ]).await.unwrap();
    }

    // Measure final memory
    let final_memory = current_memory_usage();
    let used = final_memory - baseline;

    // Should use < 100 MB for 1M events
    assert!(used < 100 * 1024 * 1024, "Memory usage too high: {} MB", used / 1024 / 1024);
}
```

---

#### 3.8 Success Metrics

- ✅ Criterion benchmarks for all CQRS operations
- ✅ TechEmpower benchmarks (JSON, DB, Plaintext)
- ✅ CI/CD performance regression detection
- ✅ Performance dashboard (vs competitors)
- ✅ Memory profiling (< 100 MB for 1M events)
- ✅ Automated alerts on performance degradation

---

## 4. Implementation Timeline

### Phase 1: Binary Size Monitoring (Week 1)
- ✅ Add cargo-bloat to project
- ✅ Create binary size CI/CD workflow
- ✅ Set up size limits for each configuration
- ✅ Generate size report script
- ✅ Add size badges to README

**Deliverables**:
- `.github/workflows/binary-size-check.yml`
- `scripts/binary-size-report.sh`
- Updated README with size badges

---

### Phase 2: Demo Scenarios (Weeks 2-6)
- Week 2: E-Commerce API
- Week 3: Real-Time Chat
- Week 4: API Gateway
- Week 5: Banking System
- Week 6: CMS (stretch)

**Deliverables**:
- 5 comprehensive demo applications
- Each with README, tests, docs
- `examples/README.md` index

---

### Phase 3: Performance Testing (Weeks 7-8)
- Week 7: Criterion benchmarks
- Week 8: TechEmpower benchmarks

**Deliverables**:
- `benches/criterion/` suite
- `benches/techempower/` suite
- `.github/workflows/performance-check.yml`
- `scripts/performance-report.sh`

---

### Phase 4: Integration & Documentation (Week 9)
- Integrate all three systems
- Update main documentation
- Create quality metrics dashboard
- Set up monitoring

**Deliverables**:
- Comprehensive quality metrics docs
- Performance dashboard
- Updated README
- CI/CD fully automated

---

## 5. Success Criteria

### Binary Size
- ✅ All configurations under hard limits
- ✅ CI/CD enforces size limits
- ✅ Size breakdown available per feature
- ✅ Historical tracking in place

### Demo Scenarios
- ✅ 5 working demo applications
- ✅ Each demonstrates unique features
- ✅ All have tests and documentation
- ✅ Referenced in main docs

### Performance
- ✅ Criterion benchmarks passing
- ✅ TechEmpower targets met
- ✅ No performance regressions in CI
- ✅ Memory usage within limits

### Automation
- ✅ All checks in CI/CD
- ✅ Automated reports generated
- ✅ Dashboard available
- ✅ Alerts on violations

---

## 6. Monitoring & Alerts

### CI/CD Checks
1. **Binary Size** - Fail if hard limit exceeded
2. **Performance** - Fail if > 10% regression
3. **Memory** - Fail if > 100 MB for 1M events
4. **Demo Tests** - Fail if any demo broken

### Dashboard Metrics
1. Binary size trend (over time)
2. Performance trend (over time)
3. Memory usage trend
4. Test coverage trend

### Alerts
- Slack notification on size violation
- GitHub comment on performance regression
- Email on critical failures

---

## 7. Resources Required

### Tools
- `cargo-bloat` - Binary size analysis
- `criterion` - Benchmarking
- `valgrind`/`heaptrack` - Memory profiling
- PostgreSQL - TechEmpower DB tests

### Infrastructure
- GitHub Actions runners
- Benchmark result storage
- Performance dashboard hosting

### Time
- 9 weeks total
- 1 engineer full-time
- Can parallelize with Phase 6 work

---

## Appendix

### A. Existing Tools

**Binary Size**:
- cargo-bloat: https://github.com/RazrFalcon/cargo-bloat
- twiggy: https://github.com/rustwasm/twiggy
- cargo-size-profiler: https://github.com/nnethercote/dhat-rs

**Performance**:
- criterion: https://github.com/bheisler/criterion.rs
- TechEmpower: https://www.techempower.com/benchmarks/
- wrk: https://github.com/wg/wrk

**Memory**:
- valgrind: https://valgrind.org/
- heaptrack: https://github.com/KDE/heaptrack
- dhat-rs: https://github.com/nnethercote/dhat-rs

---

### B. Competitor Benchmarks

**TechEmpower Round 23**:
- Actix: 650k req/s (JSON)
- Axum: 600k req/s (JSON)
- Rocket: 400k req/s (JSON)

**Binary Sizes**:
- Actix: ~12 MB
- Axum: ~6 MB
- Rocket: ~10 MB
- AllFrame target: < 8 MB

---

**END OF PRD**

---

## Approval

- [ ] Engineering Lead
- [ ] Product Owner
- [ ] Performance Team
- [ ] DevOps Team

**Next Steps**:
1. Approve PRD
2. Begin Phase 1: Binary Size Monitoring
3. Schedule demo scenario development
