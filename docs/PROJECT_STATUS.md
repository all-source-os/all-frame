# AllFrame Project Status

**Last Updated**: 2025-11-26
**Version**: 0.1.0
**Status**: Active Development

---

## Quick Status

| Category | Status | Progress |
|----------|--------|----------|
| **CQRS Infrastructure** | ✅ Complete | 5/5 phases (100%) |
| **Router + Documentation** | 📝 Planning | PRD complete, ready for implementation |
| **Dependency Injection** | ✅ Complete | Production-ready |
| **OpenTelemetry** | ✅ Complete | Tracing support |
| **CI/CD** | ⚠️ In Progress | Fixed compatibility matrix issues |

---

## Completed Work (Phases 1-5)

### Phase 1: AllSource Integration ✅
**Status**: Complete (2025-11-23)
**Achievement**: Pluggable event store backend architecture

**Deliverables**:
- `EventStoreBackend` trait for custom backends
- `InMemoryBackend` for testing/MVP
- `AllSourceBackend` for production (embedded DB)
- Backend statistics and monitoring
- Snapshot support for optimization

**Impact**: Production-ready storage with zero code changes to switch backends

**Documentation**: [docs/phases/PHASE1_COMPLETE.md](./phases/PHASE1_COMPLETE.md)

---

### Phase 2: CommandBus ✅
**Status**: Complete (2025-11-24)
**Achievement**: 90% boilerplate reduction for command handling

**Deliverables**:
- `CommandBus<E>` for type-safe command dispatch
- `CommandHandler` trait for command processing
- Automatic validation and error handling
- Handler registry with type-safe lookup
- 90% less code vs manual implementation

**Before**: 30-40 lines per command
**After**: 3 lines per command
**Reduction**: 90%

**Documentation**: [docs/phases/PHASE2_COMPLETE.md](./phases/PHASE2_COMPLETE.md)

---

### Phase 3: ProjectionRegistry ✅
**Status**: Complete (2025-11-25)
**Achievement**: 90% boilerplate reduction for projections

**Deliverables**:
- `ProjectionRegistry<E, B>` for automatic projection lifecycle
- Automatic event subscription and updates
- Projection rebuild functionality
- Consistency tracking via `ProjectionPosition`
- Metadata and monitoring support

**Before**: 50+ lines per projection
**After**: 5 lines per projection
**Reduction**: 90%

**Documentation**: [docs/phases/PHASE3_COMPLETE.md](./phases/PHASE3_COMPLETE.md)

---

### Phase 4: Event Versioning ✅
**Status**: Complete (2025-11-25)
**Achievement**: 95% boilerplate reduction for event schema evolution

**Deliverables**:
- `VersionRegistry<E>` for centralized version management
- `AutoUpcaster` using Rust's `From` trait
- Migration path tracking
- Type-erased upcaster storage
- Zero manual version checking

**Before**: 30-40 lines per event type for version management
**After**: 5 lines per event type
**Reduction**: 95%

**Documentation**: [docs/phases/PHASE4_COMPLETE.md](./phases/PHASE4_COMPLETE.md)

---

### Phase 5: Saga Orchestration ✅
**Status**: Complete (2025-11-26)
**Achievement**: 75% boilerplate reduction for distributed transactions

**Deliverables**:
- `SagaOrchestrator<E>` for saga execution
- `SagaStep` trait for defining saga steps
- `SagaDefinition` builder for composing sagas
- Automatic reverse-order compensation
- Per-step timeout management
- Saga execution history and tracking

**Before**: 100+ lines per saga
**After**: 20 lines per saga
**Reduction**: 75%

**Documentation**: [docs/phases/PHASE5_COMPLETE.md](./phases/PHASE5_COMPLETE.md)

---

## CQRS Infrastructure Summary

**Total Achievement**: 85% average boilerplate reduction

| Phase | Feature | Reduction | Lines Before | Lines After |
|-------|---------|-----------|--------------|-------------|
| 2 | CommandBus | 90% | 30-40 | 3-5 |
| 3 | ProjectionRegistry | 90% | 50+ | 5 |
| 4 | Event Versioning | 95% | 30-40 | 5 |
| 5 | Saga Orchestration | 75% | 100+ | 20 |
| **Average** | **All Features** | **85%** | **~220** | **~33** |

**Test Coverage**:
- 72 tests across all CQRS infrastructure
- 100% passing
- Property-based tests for key components
- Integration tests for full CQRS flow

**Code Quality**:
- Zero breaking changes across all phases
- 100% TDD from day one
- ~1,500 lines of framework code
- Full documentation for all features

**Announcement**: [docs/announcements/CQRS_INFRASTRUCTURE_COMPLETE.md](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

---

## Active Work

### CI/CD Fixes ✅
**Status**: Complete (2025-11-26)

**Issues Fixed**:
1. ✅ `-Z minimal-versions` now runs on nightly (was failing on stable)
2. ✅ MSRV updated to 1.80 (required for edition2024 dependencies)
3. ✅ Documented prost version conflict (allsource-core uses 0.13, we use 0.14)

**Changes Made**:
- Updated `.github/workflows/compatibility-matrix.yml`
- Updated workspace `rust-version` to 1.80
- Added toolchain selection for minimal-versions test
- Improved dependency version testing

**Remaining**:
- prost version conflict needs allsource-core update (external dependency)
- Monitoring for upstream dependency changes

---

## Next Phase

### Phase 6: Router + API Documentation 📝
**Status**: PRD Complete, Ready for Implementation
**Priority**: P0 (Next Major Phase)

**Goal**: Build best-in-class, protocol-agnostic routing and documentation system

**Key Features**:
1. **Unified Router Core** - Protocol-agnostic routing abstraction
2. **REST Documentation** - Scalar.com integration (<50KB bundle)
3. **GraphQL Documentation** - GraphiQL playground + auto-docs
4. **gRPC Documentation** - Reflection API + service explorer
5. **Contract Testing** - Built-in contract test generators

**Timeline**: 11 weeks (5 sub-phases)
- Phase 6.1: Router Core (3 weeks)
- Phase 6.2: REST + Scalar (2 weeks)
- Phase 6.3: GraphQL Docs (2 weeks)
- Phase 6.4: gRPC Docs (2 weeks)
- Phase 6.5: Contract Testing (2 weeks)

**PRD**: [docs/current/PRD_ROUTER_DOCS.md](./current/PRD_ROUTER_DOCS.md)

**Why This Matters**:
- Swagger UI is outdated (2015), heavy (100KB+), slow
- Scalar is modern (2023), light (<50KB), fast
- GraphQL needs better auto-documentation
- gRPC has NO standard web documentation solution
- Contract testing is manual and error-prone

**Competitive Advantage**:
- **Zero configuration** - Docs generated automatically from code
- **Best-in-class UIs** - Scalar for REST, GraphiQL for GraphQL
- **First-class gRPC** - No other framework has web-based gRPC docs
- **Built-in contract testing** - No other framework has this

---

## Technical Debt

### Known Issues

1. **prost Version Conflict**
   - allsource-core (external) uses prost 0.13
   - allframe-core uses prost 0.14
   - **Status**: Documented, not blocking
   - **Action**: Monitor upstream, coordinate update

2. **Feature Flag Tests**
   - Some feature flag combinations not fully tested
   - **Status**: Low priority
   - **Action**: Add feature matrix tests (Phase 6)

3. **MSRV Testing**
   - MSRV updated from 1.75 to 1.80
   - **Status**: Documented
   - **Action**: Monitor ecosystem for MSRV changes

### Technical Improvements Needed

1. **Performance Benchmarks**
   - Need comprehensive benchmarks for all CQRS operations
   - **Priority**: P1
   - **Target**: Phase 6.1

2. **Documentation Examples**
   - Need more real-world examples in phase docs
   - **Priority**: P2
   - **Target**: Ongoing

3. **Error Messages**
   - Improve error messages for common mistakes
   - **Priority**: P2
   - **Target**: Phase 6

---

## Repository Statistics

### Code

| Component | Files | Lines | Tests |
|-----------|-------|-------|-------|
| allframe-core | ~40 | ~5,000 | 43 |
| allframe-macros | ~10 | ~1,500 | 8 |
| allframe-forge | ~5 | ~500 | 5 |
| Integration tests | ~15 | ~3,000 | 25 |
| **Total** | **~70** | **~10,000** | **81** |

### Documentation

| Type | Count | Location |
|------|-------|----------|
| Phase docs | 5 | docs/phases/ |
| Announcements | 3 | docs/announcements/ |
| Guides | 3 | docs/guides/ |
| PRDs | 2 | docs/current/ |
| Milestones | 6 | docs/milestones/ |
| Archive | 6 | docs/archive/ |
| **Total** | **25** | **docs/** |

---

## Development Workflow

### Current Standards

**TDD Mandatory**:
- RED: Write failing test first
- GREEN: Minimal implementation to pass
- REFACTOR: Clean up while maintaining tests
- 100% TDD, no exceptions

**Code Quality**:
- `cargo clippy` must pass (zero warnings)
- `cargo fmt` enforced
- All tests must pass
- Documentation required for public APIs

**Git Workflow**:
- Feature branches from `main`
- PR required for merge
- CI/CD must pass
- Code review required

---

## Roadmap

### Q1 2025: Router + Documentation (Phase 6)
- ✅ PRD Complete
- ⏳ Phase 6.1: Router Core (3 weeks)
- ⏳ Phase 6.2: REST + Scalar (2 weeks)
- ⏳ Phase 6.3: GraphQL Docs (2 weeks)
- ⏳ Phase 6.4: gRPC Docs (2 weeks)
- ⏳ Phase 6.5: Contract Testing (2 weeks)

### Q2 2025: Performance + Ecosystem
- Performance benchmarks and optimization
- TechEmpower benchmarks participation
- Ecosystem integration (Axum, Actix compatibility)
- VS Code extension

### Q3 2025: Advanced Features
- API versioning support
- Multi-language code examples
- Request/response recording
- Analytics and monitoring

### Q4 2025: Production Hardening
- Security audit
- Performance optimization
- Documentation polish
- 1.0 release preparation

---

## Key Decisions

### Architectural Decisions

1. **Backend Abstraction** (Phase 1)
   - **Decision**: Pluggable backend via trait
   - **Why**: Support in-memory (testing) and AllSource (production)
   - **Trade-off**: Slight API complexity for flexibility

2. **Type Erasure** (Phases 3, 4, 5)
   - **Decision**: Use `Box<dyn Trait>` for heterogeneous collections
   - **Why**: Store different types in single registry
   - **Trade-off**: Heap allocation for flexibility

3. **Zero Config** (All phases)
   - **Decision**: Automatic everything, zero configuration required
   - **Why**: Best developer experience
   - **Trade-off**: Magic behavior, less explicit control

4. **Scalar over Swagger** (Phase 6)
   - **Decision**: Use Scalar for REST documentation
   - **Why**: Modern, lighter, better UX
   - **Trade-off**: Less mature than Swagger UI

5. **GraphiQL over Playground** (Phase 6)
   - **Decision**: Use GraphiQL, not GraphQL Playground
   - **Why**: Playground is deprecated, GraphiQL is industry standard
   - **Trade-off**: Less features than Playground

### Technology Choices

| Technology | Purpose | Why Chosen |
|------------|---------|------------|
| Tokio | Async runtime | Industry standard, mature |
| Hyper | HTTP server | Low-level, performant |
| async-graphql | GraphQL | Best Rust GraphQL library |
| tonic | gRPC | Official gRPC for Rust |
| Scalar | REST docs | Modern, lightweight, beautiful |
| GraphiQL | GraphQL docs | Industry standard |
| AllSource | Event store | Embedded DB, no external deps |

---

## Team & Contributors

**Core Team**:
- Engineering Lead: TBD
- Product Owner: TBD
- Contributors: Open to community

**Community**:
- GitHub: https://github.com/all-source-os/all-frame
- Discussions: https://github.com/all-source-os/all-frame/discussions
- Issues: https://github.com/all-source-os/all-frame/issues

---

## Resources

### Documentation
- [Main README](../README.md)
- [Documentation Index](./INDEX.md)
- [PRD 01](./current/PRD_01.md) - Original product requirements
- [PRD Router/Docs](./current/PRD_ROUTER_DOCS.md) - Router + documentation system

### CQRS Infrastructure
- [Phase 1: AllSource](./phases/PHASE1_COMPLETE.md)
- [Phase 2: CommandBus](./phases/PHASE2_COMPLETE.md)
- [Phase 3: ProjectionRegistry](./phases/PHASE3_COMPLETE.md)
- [Phase 4: Event Versioning](./phases/PHASE4_COMPLETE.md)
- [Phase 5: Saga Orchestration](./phases/PHASE5_COMPLETE.md)

### Announcements
- [CQRS Infrastructure Complete](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

---

## Contact

For questions, feedback, or contributions:
- **Issues**: https://github.com/all-source-os/all-frame/issues
- **Discussions**: https://github.com/all-source-os/all-frame/discussions
- **Email**: contact@allframe.rs (TBD)

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
