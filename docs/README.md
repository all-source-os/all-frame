# AllFrame Documentation Index

**Last Updated**: 2025-11-26
**Repository**: all-frame
**Project**: AllFrame - The Composable Rust API Framework

---

## 📖 Overview

AllFrame is the first Rust web framework **designed, built, and evolved exclusively through Test-Driven Development (TDD)**. This documentation supports the development and use of AllFrame as a framework, not an application.

**Core Promise:**
One crate (`allframe-core`) that provides compile-time DI, auto OpenAPI 3.1, OpenTelemetry, protocol-agnostic routing, CQRS/ES enforcement, and MCP server capabilities — all with zero external runtime dependencies.

---

## 📂 Documentation Structure

### Root Documentation (`/docs`)
```
docs/
├── current/          # Active, current documentation (PRDs, architecture)
├── phases/           # Development phase documentation (CQRS infrastructure)
├── milestones/       # Milestone tracking and status
├── announcements/    # Public announcements and social posts
├── guides/           # How-to guides and tutorials
├── architecture/     # Architecture Decision Records (ADRs)
├── operations/       # Operational guides (deployment, CI/CD)
└── archive/          # Historical/deprecated docs (timestamped)
```

### Key Documents

#### **Product Requirements**
- **[PRD_01.md](./current/PRD_01.md)** - Original Product Requirements (PRIMARY SOURCE OF TRUTH)
- **[PRD_ROUTER_DOCS.md](./current/PRD_ROUTER_DOCS.md)** - Router + API Documentation System (Phase 6)

#### **Project Status**
- **[PROJECT_STATUS.md](./PROJECT_STATUS.md)** - Complete project status, roadmap, and metrics

#### **CQRS Infrastructure (Phases 1-5)** ✅ **COMPLETE**
- **[Phase 1: AllSource Integration](./phases/PHASE1_COMPLETE.md)** - Event store backend abstraction
- **[Phase 2: CommandBus](./phases/PHASE2_COMPLETE.md)** - Command dispatch & validation (90% reduction)
- **[Phase 3: ProjectionRegistry](./phases/PHASE3_COMPLETE.md)** - Automatic projections (90% reduction)
- **[Phase 4: Event Versioning](./phases/PHASE4_COMPLETE.md)** - Automatic upcasting (95% reduction)
- **[Phase 5: Saga Orchestration](./phases/PHASE5_COMPLETE.md)** - Distributed transactions (75% reduction)

#### **Development Guides**
- **[.claude/skills/rust-clean-architecture.md](../.claude/skills/rust-clean-architecture.md)** - Clean Architecture patterns for AllFrame
- **[.claude/TDD_CHECKLIST.md](../.claude/TDD_CHECKLIST.md)** - Mandatory TDD workflow

#### **Announcements**
- **[CQRS Infrastructure Complete](./announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)** - 5 phases, 85% avg boilerplate reduction

---

## 🗄️ Documentation Conventions

### Timestamps
All timestamped documentation uses format: `YYYY-MM-DD_FILENAME.md`

Example: `2025-11-23_ARCHITECTURE_DECISIONS.md`

### Status Markers
- ✅ **CURRENT** - Active, up-to-date documentation
- ⚠️ **DEPRECATED** - No longer accurate, kept for history
- 🔄 **SUPERSEDED** - Replaced by newer document (link provided)
- 📝 **DRAFT** - Work in progress

### Linking
Always use relative paths:
```markdown
[PRD](./current/PRD_01.md)
[Clean Architecture Skill](../.claude/skills/rust-clean-architecture.md)
```

---

## 🔍 Finding Documentation

### By Topic
- **Product Requirements**: `/docs/current/PRD_01.md`
- **Architecture**: `/docs/architecture/`
- **How-To Guides**: `/docs/guides/`
- **Historical**: `/docs/archive/`

### By Development Phase
- **Planning**: Start with `PRD_01.md`
- **Implementation**: Follow TDD workflow in `.claude/TDD_CHECKLIST.md`
- **Architecture**: Apply patterns from `.claude/skills/rust-clean-architecture.md`

---

## 📝 Contributing Documentation

### Creating New Documentation
1. Determine type (guide, architecture, operations)
2. Place in appropriate directory
3. Add timestamp if appropriate
4. Update this README.md
5. Add status marker (CURRENT, DRAFT, etc.)

### Deprecating Documentation
1. Move to `/docs/archive/` with timestamp prefix
2. Add deprecation marker to title
3. Update this INDEX
4. Add link to replacement document if applicable

### Updating Documentation
1. Update the document
2. Update "Last Updated" timestamp in document
3. If major changes, consider creating new timestamped version

---

## 🎯 AllFrame Core Concepts

### Tech Stack
- **Language**: Rust (edition 2021+)
- **Async Runtime**: Tokio
- **HTTP Server**: Hyper
- **Zero External Runtime Dependencies**: Only Tokio + Hyper + std

### Key Features
- **Compile-time DI**: Dependency injection resolved at compile time
- **Auto OpenAPI 3.1**: Swagger UI generated automatically
- **OpenTelemetry**: Auto-instrumentation for observability
- **Protocol-agnostic**: REST ↔ GraphQL ↔ gRPC ↔ WebSockets via config
- **CQRS + Event Sourcing**: Enforced architectural patterns
- **MCP Server**: LLMs can call your API as tools
- **LLM Code Generation**: `allframe forge` CLI

### Development Workflow
```bash
# Run tests (TDD-first!)
cargo test

# Run tests with coverage
cargo llvm-cov

# Check code quality
cargo clippy
cargo fmt

# Run examples
cargo run --example hello_world

# Build release
cargo build --release
```

---

## 🧪 Testing Philosophy

**100% TDD - NO EXCEPTIONS**

Every feature, macro, and public API must have:
1. A failing test written FIRST (RED phase)
2. Minimal implementation to pass (GREEN phase)
3. Refactoring while maintaining tests (REFACTOR phase)

See [TDD_CHECKLIST.md](../.claude/TDD_CHECKLIST.md) for detailed workflow.

---

## 📚 Quick Reference Links

### Core Documentation
- [PRD_01.md](./current/PRD_01.md) - Product Requirements
- [Clean Architecture Skill](../.claude/skills/rust-clean-architecture.md) - Development patterns
- [TDD Checklist](../.claude/TDD_CHECKLIST.md) - Testing workflow
- [Claude Instructions](../.claude/instructions.md) - AI assistant guidelines

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Hyper Documentation](https://hyper.rs/)
- [Clean Architecture (Uncle Bob)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)

---

## 🚀 Getting Started

### For Framework Users
1. Read [PRD_01.md](./current/PRD_01.md) to understand AllFrame's vision
2. Follow getting started guide (TBD in `/docs/guides/`)
3. Run `allframe ignite my-api` to scaffold a new project

### For Framework Contributors
1. Read [PRD_01.md](./current/PRD_01.md) thoroughly
2. Review [.claude/skills/rust-clean-architecture.md](../.claude/skills/rust-clean-architecture.md)
3. Follow [.claude/TDD_CHECKLIST.md](../.claude/TDD_CHECKLIST.md) for ALL changes
4. Ensure 100% test coverage before submitting PR

---

## 🔗 Repository Structure

```
allframe/
├── crates/
│   ├── allframe-core         # Single public crate
│   ├── allframe-macros       # Proc macros (internal)
│   └── allframe-forge        # LLM code-gen CLI
├── tests/
│   ├── integration/          # One file per feature flag
│   ├── property/             # Property-based tests
│   └── forge_regression/     # LLM generation tests
├── examples/
│   └── transformer_modes/    # Protocol examples
├── benches/
│   └── techempower/          # Performance benchmarks
└── docs/
    ├── current/              # Active documentation
    ├── guides/               # Tutorials
    └── architecture/         # ADRs
```

---

## 📧 Documentation Maintainers

For questions or suggestions about documentation:
- Create an issue with `[docs]` prefix
- Follow contribution guidelines
- Ensure all documentation follows TDD principles (code examples must have tests)

---

**Navigation**: [Home](../README.md) | [PRD](./current/PRD_01.md) | [Guides](./guides/) | [Architecture](./architecture/) | [Archive](./archive/)

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.*
