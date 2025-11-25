# Milestone 0.3 Status - Protocol-Agnostic Routing

**Date**: 2025-11-25
**Status**: RED Phase - Tests Written, Ready for Implementation
**Previous Milestone**: v0.2 Complete (22/22 tests passing)

---

## Current Phase: RED (Test-Driven Development)

✅ **Phase Complete**: All failing tests written
⏳ **Next Phase**: GREEN (Make tests pass)

---

## Test Suite Summary

### Total Tests Written: 26 Tests

| Test File | Tests | Status | Focus Area |
|-----------|-------|--------|------------|
| `04_router_core.rs` | 5 | 🔴 All Failing | Core router abstractions |
| `04_router_rest.rs` | 5 | 🔴 All Failing | REST/HTTP adapter |
| `04_router_graphql.rs` | 5 | 🔴 All Failing | GraphQL adapter |
| `04_router_grpc.rs` | 5 | 🔴 All Failing | gRPC adapter |
| `04_router_config.rs` | 6 | 🔴 All Failing | Config-driven switching |
| **TOTAL** | **26** | **🔴 RED** | **All protocols** |

---

## Test Breakdown

### Core Router Tests (5 tests)

`tests/04_router_core.rs`:
1. `test_register_handler` - Handler registration
2. `test_execute_handler` - Handler execution
3. `test_handler_with_result` - Result type handling
4. `test_multiple_handler_signatures` - Multiple signatures
5. `test_register_protocol_adapter` - Protocol adapter registration

**Purpose**: Foundation for protocol-agnostic routing

### REST Adapter Tests (5 tests)

`tests/04_router_rest.rs`:
1. `test_rest_get_request` - Basic GET requests
2. `test_rest_post_with_body` - POST with JSON body
3. `test_rest_query_parameters` - Query parameter extraction
4. `test_rest_error_handling` - HTTP status codes
5. `test_rest_route_matching` - Route pattern matching

**Purpose**: REST/HTTP protocol support

### GraphQL Adapter Tests (5 tests)

`tests/04_router_graphql.rs`:
1. `test_graphql_query` - Basic GraphQL queries
2. `test_graphql_mutation` - GraphQL mutations
3. `test_graphql_schema_generation` - SDL schema generation
4. `test_graphql_nested_types` - Nested type handling
5. `test_graphql_error_handling` - GraphQL error format

**Purpose**: GraphQL protocol support

### gRPC Adapter Tests (5 tests)

`tests/04_router_grpc.rs`:
1. `test_grpc_unary_call` - Unary RPC calls
2. `test_grpc_proto_generation` - .proto file generation
3. `test_grpc_message_types` - Protobuf message types
4. `test_grpc_error_status` - gRPC status codes
5. `test_grpc_service_registration` - Service registration

**Purpose**: gRPC protocol support

### Config-Driven Tests (6 tests)

`tests/04_router_config.rs`:
1. `test_load_router_config` - TOML config loading
2. `test_single_handler_multiple_protocols` - Multi-protocol handler
3. `test_protocol_specific_config` - Protocol-specific settings
4. `test_protocol_enablement` - Protocol toggling
5. `test_e2e_multi_protocol` - End-to-end execution
6. `test_config_change_no_code_change` - Config flexibility

**Purpose**: Config-driven protocol selection (key differentiator)

---

## Acceptance Criteria Coverage

From PRD_01.md line 55:
> **Acceptance Criteria**: Same handler works as REST, GraphQL, gRPC via config

### ✅ Test Coverage for Each Criterion:

| Criterion | Tests Covering It |
|-----------|-------------------|
| Handler works as REST | `test_rest_*` (5 tests) |
| Handler works as GraphQL | `test_graphql_*` (5 tests) |
| Handler works as gRPC | `test_grpc_*` (5 tests) |
| Protocol selection via config | `test_load_router_config`, `test_protocol_enablement` |
| Same handler, multiple protocols | `test_single_handler_multiple_protocols`, `test_e2e_multi_protocol` |
| No code changes needed | `test_config_change_no_code_change` |

---

## Implementation Plan

### Phase 1: Core Abstractions (Days 1-2)

**Goal**: Make core router tests pass

**Files to create**:
- `crates/allframe-core/src/router/mod.rs`
- `crates/allframe-core/src/router/handler.rs`
- `crates/allframe-core/src/router/adapter.rs`

**Tests to pass**: 5 tests in `04_router_core.rs`

### Phase 2: REST Adapter (Days 3-4)

**Goal**: Make REST tests pass

**Files to create**:
- `crates/allframe-core/src/router/rest.rs`

**Tests to pass**: 5 tests in `04_router_rest.rs`

### Phase 3: GraphQL Adapter (Days 5-7)

**Goal**: Make GraphQL tests pass

**Files to create**:
- `crates/allframe-core/src/router/graphql.rs`
- `crates/allframe-macros/src/graphql.rs`

**Tests to pass**: 5 tests in `04_router_graphql.rs`

### Phase 4: gRPC Adapter (Days 8-10)

**Goal**: Make gRPC tests pass

**Files to create**:
- `crates/allframe-core/src/router/grpc.rs`
- `crates/allframe-macros/src/grpc.rs`

**Tests to pass**: 5 tests in `04_router_grpc.rs`

### Phase 5: Config System (Days 11-12)

**Goal**: Make config tests pass

**Files to create**:
- `crates/allframe-core/src/config/mod.rs`
- `crates/allframe-core/src/config/router.rs`

**Tests to pass**: 6 tests in `04_router_config.rs`

---

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Tests Written | 26+ | ✅ 26 |
| Tests Passing | 26/26 (100%) | 🔴 0/26 (RED phase) |
| Protocols Supported | 3 (REST, GraphQL, gRPC) | ⏳ Pending |
| Config-Driven | Yes | ⏳ Pending |
| Code Coverage | 100% | ⏳ Pending |

---

## Current State

### What We Have:
- ✅ Complete test specifications (26 tests)
- ✅ Clear acceptance criteria
- ✅ Detailed implementation plan
- ✅ File structure defined
- ✅ All tests in RED phase (as expected)

### What We Need:
- ⏳ Core router implementation
- ⏳ REST adapter implementation
- ⏳ GraphQL adapter implementation
- ⏳ gRPC adapter implementation
- ⏳ Config system implementation

---

## Dependencies

### Already Have:
- HTTP/Hyper (from v0.2)
- JSON/serde (from v0.2)
- OpenAPI infrastructure (from v0.2)

### Need to Add:
- GraphQL parser/executor (minimal implementation)
- Protobuf codec (minimal implementation)
- TOML config parser (lightweight)

### Strategy:
- Build minimal implementations first
- Avoid heavy external dependencies
- Can add full-featured libraries later as optional features

---

## Test Execution

### Run All Router Tests:
```bash
cargo test 04_router
```

**Expected Output** (RED phase):
```
running 26 tests
test result: FAILED. 0 passed; 26 failed; 0 ignored
```

### Run Specific Test Suite:
```bash
cargo test --test 04_router_core
cargo test --test 04_router_rest
cargo test --test 04_router_graphql
cargo test --test 04_router_grpc
cargo test --test 04_router_config
```

---

## Next Steps

1. ✅ Write failing tests (RED phase) - **COMPLETE**
2. ⏳ Implement core router (GREEN phase) - **NEXT**
3. ⏳ Implement REST adapter (GREEN phase)
4. ⏳ Implement GraphQL adapter (GREEN phase)
5. ⏳ Implement gRPC adapter (GREEN phase)
6. ⏳ Implement config system (GREEN phase)
7. ⏳ Refactor (REFACTOR phase)
8. ⏳ Document and create examples

---

## Progress Tracking

### Milestones Completed:
- ✅ **0.1**: `allframe ignite` + hello world (2/2 tests passing)
- ✅ **0.2**: Compile-time DI + OpenAPI (22/22 tests passing)
- 🚧 **0.3**: Protocol-agnostic routing (0/26 tests passing)

### Overall Progress:
- **Tests Written**: 48 total (22 passing + 26 failing)
- **Tests Passing**: 22/48 (45.8%)
- **Completion**: ~45% toward v1.0 MVP

---

## Key Design Decisions

### 1. Unified Handler Interface
- Single handler registration works for all protocols
- Protocol adapters transform requests/responses
- Type safety maintained throughout

### 2. Config-Driven Architecture
- TOML configuration determines enabled protocols
- Zero code changes to switch protocols
- Protocol-specific settings per adapter

### 3. Minimal Dependencies
- Build lightweight protocol implementations first
- Avoid heavy external libraries where possible
- Can add optional features later

### 4. Test-First Approach
- All 26 tests written before any implementation
- Each test documents expected behavior
- Tests serve as living specification

---

## References

- **Plan**: `docs/MILESTONE_0.3_PLAN.md`
- **PRD**: `docs/current/PRD_01.md` (lines 17, 55, 68)
- **Previous Milestone**: `docs/MILESTONE_0.2_COMPLETE.md`
- **TDD Checklist**: `.claude/TDD_CHECKLIST.md`

---

**Status**: ✅ RED phase complete. Ready to begin GREEN phase (implementation).

🚀 **Milestone 0.3 - Protocol-Agnostic Routing - Tests Ready!**
