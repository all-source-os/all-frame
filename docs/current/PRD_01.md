# AllFrame — Final Product Requirements Document
**The Composable Rust API Framework**  
**One frame to rule them all. Transform, compose, ignite.**

**Version:** 1.0-final  
**Date:** November 23, 2025  
**Status:** Ready for implementation (TDD-first from day zero)

## 1. Vision & Core Promise
AllFrame is the first Rust web framework that is **designed, built, and evolved exclusively through Test-Driven Development** — every feature, macro, and public API must have a failing test before it is written.

We ship **one crate** (`allframe-core`) that gives you, out of the box and with zero external dependencies:

- Compile-time DI
- Auto OpenAPI 3.1 + Swagger UI
- OpenTelemetry auto-instrumentation
- Protocol-agnostic routing (REST ↔ GraphQL ↔ gRPC ↔ WebSockets via config)
- Enforced Clean Architecture + CQRS/ES
- Native Model Context Protocol (MCP) server so LLMs can call your API as tools
- LLM-powered code generation CLI (`allframe forge`)
- Single Dockerfile + K8s manifests generated from config

All of this in binaries < 8 MB, > 500 k req/s (TechEmpower parity with Actix), and 100 % test coverage enforced by CI.

**Tagline:** *“One frame. Infinite transformations.”*

## 2. Non-Negotiable Principles (TDD is law)

| Principle                     | Enforcement                                                                 |
|-------------------------------|-----------------------------------------------------------------------------|
| Red → Green → Refactor        | Every commit must contain at least one new failing test                     |
| 100 % line + branch coverage  | CI fails if coverage drops below 100 %                                      |
| Feature flags = test features | Each Cargo feature has its own integration test suite                       |
| Example-driven documentation  | `cargo test --doc` runs all code examples                                   |
| No implementation without spec| Every public type/trait/macro has a `.spec.rs` file with property tests     |

## 3. Success Metrics (measurable by tests & benchmarks)

| Metric                              | Target                              | Measured by                     |
|-------------------------------------|-------------------------------------|---------------------------------|
| MVP “Hello World” in one command    | < 15 seconds                        | `allframe ignite demo && cargo run` |
| Zero external runtime dependencies  | Only Tokio + Hyper + std            | `cargo tree --workspace`        |
| Binary size (release, stripped)     | ≤ 8 MB                              | CI artifact check               |
| JSON throughput                     | ≥ 500 k req/s (TechEmpower Round 23)| Official benchmark suite        |
| Test suite execution time           | ≤ 45 seconds (full)                 | `cargo llvm-cov`                |
| Code generation accuracy            | ≥ 92 % compile-first-try (100 runs)| `allframe forge` regression suite |

## 4. Feature Matrix & TDD Acceptance Criteria

| Feature                        | Cargo Feature Flag | Primary Test Suite                          | Acceptance Criteria (must pass before merge) |
|--------------------------------|--------------------|---------------------------------------------|---------------------------------------------|
| Compile-time DI                | `di`               | `tests/di_*`                                | Inject 50+ nested services, zero runtime reflection |
| Auto OpenAPI 3.1 + Swagger UI  | `openapi`          | `tests/openapi_*`                           | `curl /openapi.json` valid, UI loads, MCP schema present |
| OpenTelemetry auto-tracing     | `otel`             | `tests/tracing_*`                           | Spans exported to Jaeger with DI context propagation |
| Protocol-agnostic router       | `router`           | `tests/router_*`                            | Same handler works as REST, GraphQL, gRPC via config |
| CQRS + Event Sourcing          | `cqrs`             | `tests/cqrs_*`                              | Command → Event → Projection tested with property-based testing |
| Clean Architecture enforcement | `arch`             | `tests/arch_enforce_*`                      | Compile fail if handler calls repo directly |
| MCP server (LLM tool calling)  | `mcp`              | `tests/mcp_*`                               | Claude 3.5 can call `/users/{id}` via MCP and get JSON |
| LLM code generation CLI        | `forge`            | `tests/forge_regression_*`                  | 50 golden prompts → compile + tests pass |
| Container & K8s manifests      | `deploy`           | `tests/deploy_*`                            | `allframe deploy` produces valid Dockerfile + helm chart |

## 5. MVP Scope (Q1 2026) — 100 % TDD

| Milestone | Deliverable                              | Test Coverage Required |
|----------|------------------------------------------|------------------------|
| 0.1      | `allframe ignite my-api` + hello world   | 100 %                  |
| 0.2      | Compile-time DI + OpenAPI                | 100 %                  |
| 0.3      | Protocol router + config-driven switching| 100 %                  |
| 0.4      | OTEL + CQRS + Clean Arch enforcement    | 100 %                  |
| 0.5      | MCP server (LLMs can call handlers)      | 100 %                  |
| 0.6      | `allframe forge` CLI (LLM code gen)      | 100 %                  |
| 1.0      | Production release, benchmarks, docs     | 100 %                  |

## 6. Repository Structure (enforces TDD)

```
allframe/
├── crates/
│   ├── allframe-core         # single public crate
│   ├── allframe-macros       # proc macros (internal)
│   └── allframe-forge        # LLM code-gen CLI
├── tests/
│   ├── integration/          # one file per feature flag
│   ├── property/             # proptest for CQRS, routing, DI graphs
│   └── forge_regression/     # 100+ golden LLM prompts
├── examples/
│   └── transformer_modes/    # REST, GraphQL, gRPC versions of same code
├── benches/
│   └── techempower/
└── .github/
    └── workflows/ci.yml      # fails if coverage < 100%
```

## 7. Branding & Cybertron Lore (locked)

- **Name:** AllFrame
- **Logo:** Hexagonal protoform that unfolds into different modes
- **CLI commands:** `ignite`, `forge`, `transform`, `deploy`
- **Error type:** `CybertronError`
- **Config section:** `[allframe]` → the single source of truth

## 8. Risks & Mitigations (all tracked by tests)

| Risk                              | Mitigation                                    |
|-----------------------------------|-----------------------------------------------|
| LLM generation flaky              | 200+ regression golden files, nightly canary  |
| Feature creep                     | Every new idea = new failing test first       |
| Trademark issues                  | “AllFrame” is original creation, no conflict  |
| Single-crate bloat                | Feature flags + `cargo hack` matrix testing   |

## 9. Final Call to Action

We do not write a single line of implementation until a test fails for it.

**First failing test to write today (red phase):**

```rust
// tests/01_ignite_project.rs
#[test]
fn ignite_creates_compilable_project_with_all_features() {
    let temp = assert_cmd::Command::cargo_bin("allframe")
        .arg("ignite")
        .arg("testproject")
        .assert()
        .success();

    let output = std::process::Command::new("cargo")
        .current_dir(temp.get_ref().path())
        .arg("test")
        .output()
        .unwrap();

    assert!(output.status.success());
}
```

Once this fails → we start building the greatest Rust API framework the world has ever seen.

**AllFrame. One frame. Infinite transformations.**  
Let’s ignite it — TDD first, always.