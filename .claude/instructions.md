# Claude Instructions - AllFrame

---

## 🎓 Claude Skills

This project uses **Claude Skills** for language-specific best practices:

- **`rust-clean-architecture`** - `.claude/skills/rust-clean-architecture.md`

Skills auto-activate based on file type or can be invoked manually via the Skill tool.

---

## ⚠️ MANDATORY: TDD Workflow

**BEFORE starting ANY feature, bug fix, or refactoring:**

1. **READ** `.claude/TDD_CHECKLIST.md` in its entirety
2. **VERIFY** Definition of Ready (DoR) - if not met, STOP and ask for clarification
3. **WRITE** failing tests FIRST (RED phase)
4. **IMPLEMENT** minimal code to pass tests (GREEN phase)
5. **REFACTOR** while keeping tests passing
6. **VERIFY** Definition of Done (DoD) before marking complete

---

## ⚠️ MANDATORY: Quality Gates

**BEFORE marking ANY task as complete, ALL of these MUST pass:**

```bash
cargo test              # All tests pass
cargo clippy            # No warnings
cargo fmt -- --check    # Formatting correct
cargo llvm-cov          # 100% coverage required
```

**If ANY quality gate fails:**
1. **STOP** - Do not proceed or mark complete
2. **FIX** - Address all errors immediately
3. **VERIFY** - Re-run all quality gates
4. **ONLY THEN** - Mark feature as complete

---

## ⚠️ MANDATORY: Verify Claims with Concrete Evidence

**NEVER claim something works without concrete evidence. ALWAYS test and verify.**

1. **RUN THE ACTUAL CODE** - Don't just review code changes
2. **CAPTURE REAL OUTPUT** - Show actual terminal output, test results, or logs
3. **DEMONSTRATE SUCCESS** - Prove the issue is fixed or feature works
4. **TEST FAILURE CASES** - Verify error handling and edge cases

**Examples of concrete evidence:**
- ✅ Terminal output showing tests passing
- ✅ `cargo test` output with all tests green
- ✅ `cargo clippy` showing no warnings
- ✅ Coverage report showing 100%

**NOT acceptable:**
- ❌ "This should work because..."
- ❌ "The code looks correct..."
- ❌ "Based on the implementation..."

---

## 📦 Project Overview

**AllFrame** - The Composable Rust API Framework

### Vision

AllFrame is the first Rust web framework **designed, built, and evolved exclusively through Test-Driven Development**. Every feature, macro, and public API must have a failing test before it is written.

### Core Promise

We ship **one crate** (`allframe-core`) that gives you, out of the box and with zero external dependencies:

- Compile-time DI
- Auto OpenAPI 3.1 + Swagger UI
- OpenTelemetry auto-instrumentation
- Protocol-agnostic routing (REST ↔ GraphQL ↔ gRPC ↔ WebSockets via config)
- Enforced Clean Architecture + CQRS/ES
- Native Model Context Protocol (MCP) server
- LLM-powered code generation CLI (`allframe forge`)

All of this in binaries < 8 MB, > 500 k req/s (TechEmpower parity with Actix), and **100% test coverage enforced by CI**.

---

## 🚀 Development Workflow

### Package Manager: CARGO ONLY

**CRITICAL**: This is a pure Rust project.

```bash
# ✅ CORRECT
cargo build
cargo test
cargo run --example hello_world
cargo add <crate>

# ❌ NEVER USE
npm / yarn / pnpm / bun
```

### Quick Start

```bash
# Run all tests (TDD-first!)
cargo test

# Run tests with coverage
cargo llvm-cov

# Run specific test
cargo test test_name

# Run tests in watch mode
cargo watch -x test

# Check code quality
cargo clippy
cargo fmt

# Run examples
cargo run --example <name>

# Build release
cargo build --release
```

### Development Commands

```bash
# Testing
cargo test                    # Run all tests
cargo test --lib              # Unit tests only
cargo test --test '*'         # Integration tests only
cargo llvm-cov                # Coverage report
cargo llvm-cov --html         # HTML coverage report

# Code Quality
cargo clippy -- -D warnings   # Fail on warnings
cargo fmt                     # Format code
cargo fmt -- --check          # Check formatting

# Examples & Benchmarks
cargo run --example <name>    # Run example
cargo bench                   # Run benchmarks
```

---

## 📁 Repository Structure

```
allframe/
├── crates/
│   ├── allframe-core/        # Single public crate
│   ├── allframe-macros/      # Proc macros (internal)
│   └── allframe-forge/       # LLM code-gen CLI
├── tests/
│   ├── integration/          # Integration tests (one per feature)
│   ├── property/             # Property-based tests
│   └── forge_regression/     # LLM generation golden files
├── examples/
│   └── transformer_modes/    # Protocol examples
├── benches/
│   └── techempower/          # Performance benchmarks
├── docs/
│   ├── current/              # Active docs (PRD, etc.)
│   ├── guides/               # How-to guides
│   └── architecture/         # ADRs
└── .claude/
    ├── skills/               # Language-specific patterns
    ├── instructions.md       # This file
    └── TDD_CHECKLIST.md      # TDD workflow
```

---

## 🏗️ Architecture Guidelines

### Rust Framework Development

**Activate skill:** `rust-clean-architecture`

**Key Principles:**
- **TDD-First**: Every public API has a failing test before implementation
- **Zero Runtime Dependencies**: Only Tokio + Hyper + std
- **100% Test Coverage**: CI fails if coverage drops below 100%
- **Clean Architecture**: Domain → Application → Infrastructure → Presentation
- **SOLID Principles**: Enforced across all code
- **No `unwrap()`**: Always use proper error handling

**Example:**
```rust
// ✅ CORRECT - Test-first development
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("test@example.com").unwrap();
        assert_eq!(user.email(), "test@example.com");
    }
}

// Then implement:
pub struct User {
    email: String,
}

impl User {
    pub fn new(email: impl Into<String>) -> Result<Self, DomainError> {
        let email = email.into();
        if !email.contains('@') {
            return Err(DomainError::InvalidEmail);
        }
        Ok(Self { email })
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}
```

---

## 🧪 Testing Strategy

### Test-First Development (RED-GREEN-REFACTOR)

1. **RED**: Write a failing test
2. **GREEN**: Write minimal code to pass
3. **REFACTOR**: Clean up while maintaining tests

### Test Types

```bash
# Unit tests (in src/ files)
cargo test --lib

# Integration tests (in tests/ directory)
cargo test --test '*'

# Property-based tests (using proptest)
cargo test --test property

# Benchmark tests
cargo bench
```

### Coverage Requirements

- **Minimum**: 100% line coverage
- **Minimum**: 100% branch coverage
- **CI Enforcement**: Build fails if coverage < 100%

```bash
# Generate coverage report
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html --open
```

---

## 📚 Additional Documentation

### Core Documents
- **PRD**: `docs/current/PRD_01.md` - PRIMARY SOURCE OF TRUTH
- **TDD Workflow**: `.claude/TDD_CHECKLIST.md`
- **Rust Architecture**: `.claude/skills/rust-clean-architecture.md`
- **Documentation Index**: `docs/README.md`

---

## 🚫 Common Mistakes to Avoid

### Framework Development
- ❌ Implementing before writing tests
- ❌ Using `unwrap()` or `panic!` in production code
- ❌ Adding runtime dependencies beyond Tokio + Hyper
- ❌ Ignoring compiler warnings
- ❌ Skipping coverage checks
- ❌ Business logic in presentation layer
- ❌ Breaking Clean Architecture boundaries

---

## 📋 Definition of Done Checklist

Before marking ANY task as complete:

- [ ] ✅ Tests written FIRST (failing)
- [ ] ✅ Tests now passing
- [ ] ✅ 100% line coverage
- [ ] ✅ 100% branch coverage
- [ ] ✅ `cargo clippy` - no warnings
- [ ] ✅ `cargo fmt -- --check` - passes
- [ ] ✅ All integration tests passing
- [ ] ✅ Documentation updated (if public API)
- [ ] ✅ No regressions

---

## 🎯 Quick Reference

### Quality Gates (Run Before Commit)
```bash
cargo test && cargo clippy -- -D warnings && cargo fmt -- --check && cargo llvm-cov
```

### Common Tasks
```bash
# TDD Cycle
cargo watch -x test          # Watch mode for tests

# Feature Development
cargo test --test <feature>  # Run feature tests
cargo test --lib             # Run unit tests

# Code Quality
cargo clippy                 # Linter
cargo fmt                    # Formatter

# Coverage
cargo llvm-cov               # Coverage report
cargo llvm-cov --html        # HTML report
```

### Feature Flags

Each Cargo feature has its own test suite:

```bash
# Test specific feature
cargo test --features di
cargo test --features openapi
cargo test --features otel
cargo test --features router
cargo test --features cqrs
cargo test --features mcp
```

---

## 🔗 Links

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Hyper Documentation](https://hyper.rs/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [TechEmpower Benchmarks](https://www.techempower.com/benchmarks/)

---

## 🎓 Non-Negotiable Principles

| Principle                     | Enforcement                                                                 |
|-------------------------------|-----------------------------------------------------------------------------|
| Red → Green → Refactor        | Every commit must contain at least one new failing test                     |
| 100% line + branch coverage   | CI fails if coverage drops below 100%                                       |
| Feature flags = test features | Each Cargo feature has its own integration test suite                       |
| Example-driven documentation  | `cargo test --doc` runs all code examples                                   |
| No implementation without spec| Every public type/trait/macro has a test file with property tests           |

---

**Remember:**

1. **TDD-First** - Tests before implementation, ALWAYS
2. **100% Coverage** - No exceptions
3. **Quality gates** must pass before marking complete
4. **Verify with evidence** - Run the code, show the output
5. **Clean Architecture** - Respect layer boundaries
6. **Zero runtime deps** - Only Tokio + Hyper + std

---

_Last Updated: 2025-11-23_
_Project: AllFrame - The Composable Rust API Framework_
