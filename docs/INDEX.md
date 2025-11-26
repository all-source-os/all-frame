# AllFrame Documentation Index

**Last Updated**: 2025-11-26

Welcome to the AllFrame documentation! This index provides a comprehensive overview of all available documentation.

---

## Quick Links

- **[Main README](./README.md)** - Documentation overview and conventions
- **[Product Requirements](./current/PRD_01.md)** - PRIMARY SOURCE OF TRUTH
- **[CQRS Complete Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)** - Latest achievement

---

## Documentation Structure

```
docs/
├── README.md                    # Documentation overview
├── INDEX.md                     # This file - complete index
│
├── current/                     # Active documentation
│   └── PRD_01.md               # Product Requirements (PRIMARY SOURCE)
│
├── phases/                      # CQRS Infrastructure (COMPLETE ✅)
│   ├── PHASE1_COMPLETE.md      # AllSource Integration
│   ├── PHASE2_COMPLETE.md      # CommandBus (90% reduction)
│   ├── PHASE3_COMPLETE.md      # ProjectionRegistry (90% reduction)
│   ├── PHASE4_COMPLETE.md      # Event Versioning (95% reduction)
│   └── PHASE5_COMPLETE.md      # Saga Orchestration (75% reduction)
│
├── announcements/               # Public announcements
│   ├── CQRS_INFRASTRUCTURE_COMPLETE.md  # Main announcement (2025-11-26)
│   ├── ANNOUNCEMENT_DI.md               # DI announcement
│   └── SOCIAL_POSTS.md                  # Social media posts
│
├── milestones/                  # Milestone tracking
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
    ├── CQRS_CHRONOS_ASSESSMENT.md
    ├── MIGRATION_SUMMARY.md
    ├── NEXT_STEPS.md
    ├── SESSION_COMPLETE.md
    ├── SESSION_SUMMARY.md
    └── SUMMARY.md
```

---

## CQRS Infrastructure (Phases 1-5) ✅ COMPLETE

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

**Summary**: [CQRS Infrastructure Complete Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

---

## Core Documentation

### Product Requirements
- **[PRD_01.md](./current/PRD_01.md)** - Final Product Requirements Document
  - PRIMARY SOURCE OF TRUTH for AllFrame vision and scope
  - Defines all core features and capabilities
  - Reference for all development decisions

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
- CQRS Chronos Assessment
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

---

## Statistics

### CQRS Infrastructure
- **Total Phases**: 5 (all complete)
- **Total Tests**: 72 (100% passing)
- **Framework Code**: ~1,500 lines
- **Average Boilerplate Reduction**: 85%
- **Breaking Changes**: 0

### Documentation
- **Total Documents**: 24+
- **Guides**: 3
- **Phase Docs**: 5
- **Announcements**: 3
- **Milestones**: 6

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

### For Framework Users
1. Read [CQRS Infrastructure Complete](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)
2. Review phase documentation for features you need
3. Check [guides/](./guides/) for integration instructions

### For Framework Contributors
1. Read [PRD_01.md](./current/PRD_01.md) thoroughly
2. Review [.claude/skills/rust-clean-architecture.md](../.claude/skills/rust-clean-architecture.md)
3. Follow [.claude/TDD_CHECKLIST.md](../.claude/TDD_CHECKLIST.md) for ALL changes
4. Ensure 100% test coverage before submitting PR

---

**Navigation**: [Home](../README.md) | [README](./README.md) | [PRD](./current/PRD_01.md) | [Announcement](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
