# AllFrame Documentation Index

**Last Updated**: 2026-02-28

Welcome to the AllFrame documentation! This index provides a comprehensive overview of all available documentation.

---

## Quick Links

- **[Main README](./README.md)** - Documentation overview and conventions
- **[Project Status](./PROJECT_STATUS.md)** - Complete project status and roadmap
- **[Roadmap](./current/ROADMAP.md)** - Complete roadmap to v1.0
- **[Ignite Vision](./current/IGNITE_VISION.md)** - Cloud-native microservice generator vision
- **[Product Requirements](./current/PRD_01.md)** - Original PRD (PRIMARY SOURCE OF TRUTH)
- **[Router + Docs PRD](./current/PRD_ROUTER_DOCS.md)** - Phase 6 PRD (Next major phase)
- **[CQRS Complete Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)** - Latest achievement

---

## Documentation Structure

```
docs/
├── README.md                    # Documentation overview
├── INDEX.md                     # This file - complete index
│
├── current/                     # Active documentation
│   ├── PRD_01.md               # Product Requirements (PRIMARY SOURCE)
│   ├── PRD_ROUTER_DOCS.md      # Router + API Documentation PRD (Phase 6)
│   ├── PRD_QUALITY_METRICS.md  # Binary Size, Demos, Performance (P0)
│   ├── PRD_SERVERLESS.md       # Serverless + IaC PRD (Phase 7)
│   ├── PRD_ALLSOURCE_CLOUD.md  # AllSource Cloud Evolution PRD
│   ├── ROADMAP.md              # Complete roadmap to v1.0 (Ignite Vision)
│   ├── ALLSOURCE_CORE_ISSUES.md # External dependency issues tracker
│   └── IGNITE_VISION.md        # Detailed microservice generator vision
│
├── phases/                      # Implementation Phases
│   ├── PHASE1_COMPLETE.md      # AllSource Integration ✅
│   ├── PHASE2_COMPLETE.md      # CommandBus (90% reduction) ✅
│   ├── PHASE3_COMPLETE.md      # ProjectionRegistry (90% reduction) ✅
│   ├── PHASE4_COMPLETE.md      # Event Versioning (95% reduction) ✅
│   ├── PHASE5_COMPLETE.md      # Saga Orchestration (75% reduction) ✅
│   ├── PHASE6_1_COMPLETE.md           # Router Core Enhancement ✅
│
├── announcements/               # Public announcements
│   ├── CQRS_INFRASTRUCTURE_COMPLETE.md  # Main announcement (2025-11-26)
│   ├── ANNOUNCEMENT_DI.md               # DI announcement
│   └── SOCIAL_POSTS.md                  # Social media posts
│
├── milestones/                  # Milestone tracking
│   ├── WARNING_CLEANUP_COMPLETE.md  # Code quality milestone (2025-11-27)
│   ├── milestone-0.4-plan.md
│   ├── MILESTONE_0.2_COMPLETE.md
│   ├── MILESTONE_0.2_STATUS.md
│   ├── MILESTONE_0.3_PLAN.md
│   ├── MILESTONE_0.3_STATUS.md
│   └── MILESTONE_0.4_COMPLETE.md
│
├── guides/                      # How-to guides
│   ├── ALLSOURCE_INTEGRATION.md
│   ├── FEATURE_FLAGS.md
│   └── feature-flags.md
│
└── archive/                     # Historical documentation
    ├── CQRS_ALLSOURCE_ASSESSMENT.md
    ├── MIGRATION_SUMMARY.md
    ├── NEXT_STEPS.md
    ├── SESSION_COMPLETE.md
    ├── SESSION_SUMMARY.md
    └── SUMMARY.md
```

---

## CQRS Infrastructure (Phases 1-5) ✅ Complete

**Status**: Production-ready
**Achievement**: 85% average boilerplate reduction

### Phase Documentation

| Phase | Feature | Reduction | Documentation |
|-------|---------|-----------|---------------|
| 1 | AllSource Integration | - | [PHASE1_COMPLETE.md](./phases/PHASE1_COMPLETE.md) |
| 2 | CommandBus | 90% | [PHASE2_COMPLETE.md](./phases/PHASE2_COMPLETE.md) |
| 3 | ProjectionRegistry | 90% | [PHASE3_COMPLETE.md](./phases/PHASE3_COMPLETE.md) |
| 4 | Event Versioning | 95% | [PHASE4_COMPLETE.md](./phases/PHASE4_COMPLETE.md) |
| 5 | Saga Orchestration | 75% | [PHASE5_COMPLETE.md](./phases/PHASE5_COMPLETE.md) |
| 6.1 | Router Core | - | [PHASE6_1_COMPLETE.md](./phases/PHASE6_1_COMPLETE.md) |
| 6.2 | Scalar API Docs | - | [SCALAR_INTEGRATION_COMPLETE.md](./phases/SCALAR_INTEGRATION_COMPLETE.md) |
| 6.3 | GraphQL Docs | - | [PHASE6_3_COMPLETE.md](./phases/PHASE6_3_COMPLETE.md) |
| 6.4 | Protocol Routing | - | [PROTOCOL_AGNOSTIC_ROUTING_COMPLETE.md](./phases/PROTOCOL_AGNOSTIC_ROUTING_COMPLETE.md) |
| MCP | MCP Server | - | [MCP_ZERO_BLOAT_COMPLETE.md](./phases/MCP_ZERO_BLOAT_COMPLETE.md) |

**Summary**: [CQRS Infrastructure Complete Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

---

## Core Documentation

### Project Status
- **[PROJECT_STATUS.md](./PROJECT_STATUS.md)** - Complete Project Status
  - Current phase status (CQRS: 100% complete)
  - Next phase planning (Router + Docs: PRD complete)
  - Technical debt tracking
  - Repository statistics

### Roadmap & Vision
- **[ROADMAP.md](./current/ROADMAP.md)** - Complete Roadmap to v1.0
  - Phase 7-14 planning (v0.2.0 - v1.0.0)
  - Architecture configuration, service archetypes, multi-cloud IaC
  - Version milestones and timelines

- **[IGNITE_VISION.md](./current/IGNITE_VISION.md)** - Cloud-Native Microservice Generator Vision
  - Detailed configuration schema (TOML)
  - 12 service archetypes with examples
  - Infrastructure as Code generation (Terraform, Pulumi)
  - Demo architecture templates

### Product Requirements
- **[PRD_01.md](./current/PRD_01.md)** - Original Product Requirements
  - PRIMARY SOURCE OF TRUTH for AllFrame vision and scope
  - Defines all core features and capabilities
  - Reference for all development decisions

- **[PRD_ROUTER_DOCS.md](./current/PRD_ROUTER_DOCS.md)** - Router + API Documentation PRD (Phase 6)
  - Best-in-class REST documentation (Scalar)
  - GraphQL documentation (GraphiQL)
  - gRPC documentation (custom UI)
  - Contract testing system
  - 11-week implementation plan

- **[PRD_QUALITY_METRICS.md](./current/PRD_QUALITY_METRICS.md)** - Quality Metrics & Performance (P0)
  - Binary size monitoring (< 8 MB target)
  - Demo scenarios (5 comprehensive examples)
  - Performance testing (TechEmpower benchmarks)
  - Automated CI/CD monitoring
  - 9-week implementation plan

- **[PRD_SERVERLESS.md](./current/PRD_SERVERLESS.md)** - AllFrame Serverless (Phase 7)
  - AWS Lambda runtime adapter
  - GCP Cloud Run support
  - DynamoDB event store backend
  - Infrastructure-from-Code (Terraform generation)
  - 12-week implementation plan

- **[PRD_ALLSOURCE_CLOUD.md](./current/PRD_ALLSOURCE_CLOUD.md)** - AllSource Cloud-Ready Evolution
  - Fix existing compilation issues
  - DynamoDB and Firestore backends
  - Multi-tenant support with isolation
  - S3/GCS archival for cold storage
  - Serverless optimization (connection pooling, cold start)
  - 12-week implementation plan

### Development Guides
- **[Rust Clean Architecture](../.claude/skills/rust-clean-architecture.md)** - Architecture patterns
- **[TDD Checklist](../.claude/TDD_CHECKLIST.md)** - Mandatory TDD workflow
- **[Claude Instructions](../.claude/instructions.md)** - AI assistant guidelines

---

## Announcements

### 2025-11-26: CQRS Infrastructure Complete
**[Full Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)**

All 5 CQRS phases complete with 85% average boilerplate reduction:
- ✅ AllSource Integration - Pluggable backends
- ✅ CommandBus - Type-safe dispatch (90% reduction)
- ✅ ProjectionRegistry - Automatic projections (90% reduction)
- ✅ Event Versioning - Automatic upcasting (95% reduction)
- ✅ Saga Orchestration - Distributed transactions (75% reduction)

**Stats**:
- 72 tests (100% passing)
- ~1,500 lines of framework code
- Zero breaking changes
- 100% TDD from day one

---

## Guides

### Integration Guides
- **[AllSource Integration](./guides/ALLSOURCE_INTEGRATION.md)** - Using AllSource as event store backend
- **[Feature Flags](./guides/FEATURE_FLAGS.md)** - Feature flag configuration

---

## Milestones

### Completed
- **[Milestone 0.2](./milestones/MILESTONE_0.2_COMPLETE.md)** - Complete
- **[Milestone 0.4](./milestones/MILESTONE_0.4_COMPLETE.md)** - Complete

### In Progress
- **[Milestone 0.3 Status](./milestones/MILESTONE_0.3_STATUS.md)** - Current status
- **[Milestone 0.3 Plan](./milestones/MILESTONE_0.3_PLAN.md)** - Planning

---

## Archive

Historical documentation moved to `/docs/archive/`:
- CQRS AllSource Assessment
- Migration Summaries
- Session Summaries
- Previous Status Documents

---

## Finding What You Need

### By Topic
- **CQRS/Event Sourcing**: Start with [phases/](./phases/) directory
- **Product Vision**: Read [PRD_01.md](./current/PRD_01.md)
- **How-To**: Check [guides/](./guides/) directory
- **History**: See [archive/](./archive/) directory

### By Development Phase
1. **Planning**: Start with PRD_01.md
2. **Implementation**: Follow TDD workflow in .claude/TDD_CHECKLIST.md
3. **Architecture**: Apply patterns from .claude/skills/rust-clean-architecture.md
4. **CQRS Features**: Reference phase documentation

### By Feature
- **Commands**: [PHASE2_COMPLETE.md](./phases/PHASE2_COMPLETE.md)
- **Projections**: [PHASE3_COMPLETE.md](./phases/PHASE3_COMPLETE.md)
- **Event Versioning**: [PHASE4_COMPLETE.md](./phases/PHASE4_COMPLETE.md)
- **Sagas**: [PHASE5_COMPLETE.md](./phases/PHASE5_COMPLETE.md)
- **Storage Backends**: [PHASE1_COMPLETE.md](./phases/PHASE1_COMPLETE.md)
- **Router & OpenAPI**: [PHASE6_1_COMPLETE.md](./phases/PHASE6_1_COMPLETE.md)
- **Authentication**: Layered auth system (auth, auth-jwt, auth-axum, auth-tonic) ✅ **NEW**
- **Resilience**: KeyedCircuitBreaker, Redis rate limiting ✅ **NEW**

---

## Statistics

**Current Metrics**: See **[PROJECT_STATUS.md](./PROJECT_STATUS.md#repository-statistics)** for detailed, up-to-date statistics.

### Quick Stats
- **Total Tests**: 455+ (100% passing)
- **CQRS Phases**: 5 (all complete, 85% avg boilerplate reduction)
- **Router Phase**: 6 (complete - REST, GraphQL, gRPC)
- **MCP Server**: Complete (zero-bloat separate crate)
- **Authentication**: Complete (layered: core, JWT, Axum, gRPC)
- **Enhanced Resilience**: KeyedCircuitBreaker, Redis rate limiting
- **Documentation**: 30+ files across phases, guides, and PRDs

---

## Contributing to Documentation

### Creating New Documentation
1. Determine type (guide, phase, announcement, milestone)
2. Place in appropriate directory
3. Add timestamp if appropriate
4. Update this INDEX.md
5. Add status marker (CURRENT, DRAFT, etc.)

### Deprecating Documentation
1. Move to `/docs/archive/` with timestamp prefix
2. Add deprecation marker to title
3. Update this INDEX
4. Add link to replacement document if applicable

---

## External Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Hyper Documentation](https://hyper.rs/)
- [Clean Architecture (Uncle Bob)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [CQRS Pattern](https://martinfowler.com/bliki/CQRS.html)
- [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)

---

## Quick Start

See **[README.md#getting-started](./README.md#getting-started)** for detailed getting started guide.

---

**Navigation**: [Home](../README.md) | [README](./README.md) | [PRD](./current/PRD_01.md) | [Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
