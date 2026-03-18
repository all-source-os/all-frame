# AGENTS.md

## Essential Commands

```bash
# Testing
 cargo test              # All tests
 cargo test --lib        # Unit tests
 cargo test --test *     # Integration tests
 cargo llvm-cov           # Coverage report
 cargo llvm-cov --html   # HTML coverage

# Code Quality
 cargo clippy -- -D warnings  # Linter
 cargo fmt                   # Format code
 cargo fmt -- --check       # Check formatting

# Examples & Benchmarks
 cargo run --example <name>  # Run example
 cargo bench                 # Run benchmarks

# Feature-Specific
 cargo test --features di     # Dependency Injection
 cargo test --features openapi # OpenAPI
 cargo test --features otel   # OpenTelemetry
```

## Project Structure

```
allframe/
├── crates/                # Core crates
│   ├── allframe-core/       # Main public crate
│   ├── allframe-macros/     # Proc macros
│   └── allframe-forge/      # Codegen CLI
├── tests/                  # Integration/property tests
├── examples/               # Protocol examples
├── benches/                # Performance benchmarks
├── docs/                   # Documentation
├──.claude/               # Agent instructions
└── Cargo.toml              # Build configuration
```

## Code Organization

- **`crates/allframe-core/`**: Primary crate containing framework code
  - `src/arch/` - Architectural tests
  - `src/router/` - Protocol-agnostic routing
  - `src/resilience/` - Circuit breakers, rate limiting
  - `src/otel/` - OpenTelemetry integration
  - `src/cqrs/` - Command Query Responsibility Segregation
- **`crates/allframe-macros/`**: Proc macros for DI, logging, etc.
- **`crates/allframe-forge/`**: LLM-powered code generation CLI

## Testing Strategy

### TDD Workflow (RED-GREEN-REFACTOR)
1. **RED**: Write failing tests FIRST
2. **GREEN**: Implement minimal code to pass
3. **REFACTOR**: Clean up while maintaining tests

### Test Types
- **Unit Tests**: `src/` files (business logic, entities)
- **Integration Tests**: `tests/` directory (feature flags, APIs)
- **Property Tests**: Invariants and edge cases
- **Macro Tests**: Compile-time checks for proc macros
- **Doc Tests**: Examples in documentation

### Coverage Requirements
- **100% line + branch coverage** enforced by CI
- `cargo llvm-cov` for coverage reports

## Gotchas & Non-Obvious Patterns

1. **TDD First**: Never implement before writing tests
2. **Zero Runtime Dependencies**: Only Tokio + Hyper + std
3. **No `unwrap()`**: Always handle errors properly
4. **Feature Flags**: Test all combinations
5. **Clean Architecture**: Domain → Application → Infrastructure → Presentation
6. **100% Coverage**: CI fails if coverage drops
7. **Quality Gates**: All must pass before marking complete

## Agent-Specific Notes

- **Activate Skills**: `rust-clean-architecture` for domain-specific patterns
- **Mandatory Files**:
  - `.claude/instructions.md` - Workflow rules
  - `.claude/TDD_CHECKLIST.md` - Testing requirements
- **Quality Gates Command**:
  ```bash
  cargo test && cargo clippy -- -D warnings && cargo fmt -- --check && cargo llvm-cov
  ```

## Common Pitfalls

- Implementing before tests → ❌
- Using `panic!`/`unwrap()` in production code → ❌
- Skipping coverage checks → ❌
- Ignoring clippy warnings → ❌
- Breaking Clean Architecture layers → ❌

## Key Concepts

- **Compile-Time DI**: `di_container!` macro
- **Protocol Agnostic**: Switch between REST/GraphQL/gRPC via config
- **MCP Server**: Model Context Protocol implementation
- **OpenTelemetry**: Auto-instrumentation built-in

## Documentation

- **Primary**: `docs/current/PRD_01.md`
- **Guides**: `docs/guides/` for implementation details
- **Architecture**: `docs/architecture/` for design decisions

## Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_injection() {
        // Test DI container
    }

    #[test]
    fn test_circuit_breaker() {
        // Test resilience patterns
    }
}
```

## Feature Flag Testing

cargo test --features di
 cargo test --features openapi
 cargo test --features otel

## Macro Expansion Testing

```rust
// Should compile
#[test]
fn test_valid_macro() {
    di_container! {
        UserRepository => PostgresUserRepository
    };
}

// Should fail to compile
#[test]
fn test_invalid_macro() {
    di_container! {
        // Missing implementation
        UserRepository =>
    };
}
```

## Property-Based Testing Example

```rust
proptest! {
    #[test]
    fn test_uuid_roundtrip(id in any::<Uuid>()) {
        let id = UserId::from(id);
        let serialized = id.to_string();
        let deserialized = UserId::parse(&serialized).unwrap();
        prop_assert_eq!(id, deserialized);
    }
}
```

## Commit Checklist

- [ ] ✅ Tests written FIRST
- [ ] ✅ Tests passing
- [ ] ✅ 100% line + branch coverage
- [ ] ✅ No clippy warnings
- [ ] ✅ Formatting correct
- [ ] ✅ Integration tests passing
- [ ] ✅ Documentation updated

## Accountability

**Claude Commits To:**
- Following TDD cycle
- Writing tests before implementation
- Running quality gates
- Enforcing 100% coverage

**User Commits To:**
- Clear acceptance criteria
- Allowing TDD time
- Reviewing test coverage
- Holding Claude accountable
