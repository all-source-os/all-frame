# TDD Checklist for AllFrame Development

## MANDATORY: Read Before ANY Feature Work

This checklist MUST be followed for every feature, refactoring, or bug fix in AllFrame. **No exceptions.**

---

## Definition of Ready (DoR)

Before starting ANY task, verify:

- [ ] Task has clear acceptance criteria
- [ ] Task is testable (unit, integration, or property-based)
- [ ] Dependencies are identified
- [ ] You understand the ENTIRE scope of changes needed
- [ ] Task aligns with AllFrame's PRD and vision

**If ANY checkbox is unchecked, STOP and clarify with the user.**

---

## TDD Workflow (RED-GREEN-REFACTOR)

### Phase 1: RED - Write Failing Tests First

**BEFORE writing ANY implementation code:**

1. [ ] **Identify test types needed:**
   - [ ] Unit tests (domain logic, value objects, entities)
   - [ ] Integration tests (feature flags, API endpoints)
   - [ ] Property-based tests (invariants, edge cases)
   - [ ] Macro expansion tests (for proc macros)
   - [ ] Example tests (`cargo test --doc`)

2. [ ] **Write tests that FAIL:**
   ```rust
   // Example: Write test FIRST
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_compile_time_di_injection() {
           // This will FAIL until we implement the DI macro
           let container = di_container! {
               UserRepository => PostgresUserRepository,
               UserService => UserService,
           };

           assert!(container.resolve::<UserService>().is_some());
       }
   }
   ```

3. [ ] **Run tests to verify they FAIL:**
   ```bash
   cargo test  # Should FAIL - that's good!
   ```

### Phase 2: GREEN - Make Tests Pass

4. [ ] **Write MINIMAL code to pass tests:**
   - Implement only what's needed to make tests pass
   - No extra features or "nice to haves"
   - Focus on making the RED test turn GREEN

5. [ ] **Run tests again:**
   ```bash
   cargo test  # Should PASS now
   ```

6. [ ] **Verify build still works:**
   ```bash
   cargo build
   ```

### Phase 3: REFACTOR - Clean Up

7. [ ] **Improve code quality:**
   - Remove duplication
   - Improve naming
   - Add type safety (newtypes, phantom types)
   - Optimize performance (if needed)
   - Add documentation comments

8. [ ] **Run tests AGAIN:**
   ```bash
   cargo test  # Should still PASS
   ```

9. [ ] **Run quality checks:**
   ```bash
   cargo clippy -- -D warnings
   cargo fmt
   cargo llvm-cov  # Verify 100% coverage
   ```

---

## Regression Prevention Checklist

### For Framework Features:

- [ ] **Search for ALL references:**
  ```bash
  # Use grep to find all usages
  rg "feature_name" --type rust
  ```

- [ ] **Check these common locations:**
  - [ ] Public API (`src/lib.rs`, `mod.rs` files)
  - [ ] Proc macros (`allframe-macros/`)
  - [ ] Integration tests (`tests/integration/`)
  - [ ] Property tests (`tests/property/`)
  - [ ] Examples (`examples/`)
  - [ ] Documentation (doc comments, `README.md`)
  - [ ] Feature flags (`Cargo.toml`)

- [ ] **Write tests for:**
  - [ ] Public API usage
  - [ ] Feature flag combinations
  - [ ] Error cases and edge cases
  - [ ] Cross-protocol compatibility (REST/GraphQL/gRPC)

### For Proc Macro Changes:

- [ ] **Write macro expansion tests:**
  ```rust
  #[test]
  fn test_di_macro_expansion() {
      let t = trybuild::TestCases::new();
      t.pass("tests/ui/di_basic.rs");
      t.compile_fail("tests/ui/di_invalid.rs");
  }
  ```

- [ ] Test compile-time errors
- [ ] Test generated code correctness
- [ ] Test edge cases (empty input, max complexity)

### For Public API Changes:

- [ ] **Write comprehensive unit tests:**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_user_repository_trait() {
          // Test trait contract
      }

      #[test]
      fn test_user_repository_postgres_impl() {
          // Test concrete implementation
      }
  }
  ```

- [ ] Test all trait methods
- [ ] Test error cases
- [ ] Test boundary conditions
- [ ] Add property-based tests

---

## Definition of Done (DoD)

Before marking task as complete, verify:

- [ ] ✅ **ALL tests passing** (`cargo test`)
- [ ] ✅ **100% line coverage** (`cargo llvm-cov` shows 100%)
- [ ] ✅ **100% branch coverage** (verify with `cargo llvm-cov --html`)
- [ ] ✅ **No clippy warnings** (`cargo clippy -- -D warnings`)
- [ ] ✅ **Formatted correctly** (`cargo fmt -- --check`)
- [ ] ✅ **Integration tests passing** (`cargo test --test '*'`)
- [ ] ✅ **Property tests passing** (if applicable)
- [ ] ✅ **Documentation updated** (doc comments for public API)
- [ ] ✅ **Examples updated** (if public API changed)
- [ ] ✅ **No regressions** (existing features still work)

**If ANY checkbox is unchecked, task is NOT done.**

---

## Test Coverage Requirements

| Type | Minimum Coverage | When Required |
|------|-----------------|---------------|
| Unit Tests | 100% | All business logic, entities, value objects |
| Integration Tests | N/A | All feature flags, public APIs |
| Property Tests | N/A | Complex algorithms, invariants |
| Macro Tests | 100% | All proc macros |
| Doc Tests | 100% | All public API examples |

---

## Example: TDD for Compile-Time DI Feature

```rust
// ❌ WRONG: Implement first, test later
// 1. Write macro
// 2. Write implementation
// 3. Hope it works

// ✅ CORRECT: Test first
// Step 1: Write failing test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_di_container_resolves_dependencies() {
        // RED: This fails because macro doesn't exist yet
        let container = di_container! {
            UserRepository => PostgresUserRepository,
            UserService => UserService,
        };

        let service = container.resolve::<UserService>();
        assert!(service.is_some());
    }

    #[test]
    fn test_di_container_detects_circular_dependencies() {
        // RED: This fails
        let result = std::panic::catch_unwind(|| {
            di_container! {
                ServiceA => ServiceA,  // Circular!
            }
        });
        assert!(result.is_err());
    }
}

// Step 2: Run tests - they FAIL (good!)
// $ cargo test

// Step 3: Implement minimal code to make tests pass (GREEN)
#[proc_macro]
pub fn di_container(input: TokenStream) -> TokenStream {
    // Minimal implementation to pass tests
    // ...
}

// Step 4: Run tests again - they PASS
// $ cargo test

// Step 5: Refactor while keeping tests passing
// - Improve macro parsing
// - Add better error messages
// - Optimize generated code

// Step 6: Run tests again - still PASS
// $ cargo test
```

---

## Feature-Specific Test Templates

### Testing Feature Flags

```rust
// tests/integration/di_feature.rs
#[cfg(feature = "di")]
#[cfg(test)]
mod di_tests {
    use super::*;

    #[test]
    fn test_di_basic_injection() {
        // Test DI feature works
    }

    #[test]
    fn test_di_with_lifetimes() {
        // Test DI with complex types
    }
}
```

### Testing Protocol-Agnostic Routing

```rust
// tests/integration/router_feature.rs
#[cfg(feature = "router")]
#[cfg(test)]
mod router_tests {
    use super::*;

    #[test]
    fn test_rest_to_graphql_conversion() {
        // Test same handler works as REST and GraphQL
    }

    #[test]
    fn test_grpc_to_rest_conversion() {
        // Test same handler works as gRPC and REST
    }
}
```

### Testing OpenAPI Generation

```rust
// tests/integration/openapi_feature.rs
#[cfg(feature = "openapi")]
#[cfg(test)]
mod openapi_tests {
    use super::*;

    #[test]
    fn test_openapi_schema_generation() {
        let schema = generate_openapi_schema();
        assert_eq!(schema.version, "3.1.0");
    }

    #[test]
    fn test_swagger_ui_route() {
        // Test /swagger-ui route is available
    }
}
```

---

## When Tests Can Be Skipped

**NEVER.** Tests cannot be skipped.

If you think you need to skip tests:
1. Stop
2. Re-read this checklist
3. Write the tests

---

## Accountability

**Claude Code commits to:**
- Always following TDD RED-GREEN-REFACTOR cycle
- Writing tests BEFORE implementation
- Running tests and verifying they fail/pass
- Never marking tasks complete without passing DoD checklist
- Asking for clarification if DoR is not met
- Enforcing 100% test coverage

**User commits to:**
- Providing clear acceptance criteria
- Allowing time for proper TDD workflow
- Reviewing test coverage in PRs
- Holding Claude accountable to this process

---

## Quick Reference Commands

```bash
# Run all tests
cargo test

# Run tests in watch mode (TDD)
cargo watch -x test

# Run specific test file
cargo test --test di_feature

# Run tests for specific feature flag
cargo test --features di

# Run tests with coverage
cargo llvm-cov

# Run tests with HTML coverage report
cargo llvm-cov --html --open

# Type check
cargo check

# Lint (fail on warnings)
cargo clippy -- -D warnings

# Format
cargo fmt

# Format check
cargo fmt -- --check

# Full CI pipeline (run before marking task done)
cargo test && cargo clippy -- -D warnings && cargo fmt -- --check && cargo llvm-cov
```

---

## Property-Based Testing Example

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_user_id_roundtrip(id in any::<Uuid>()) {
        // Property: serializing and deserializing should be identity
        let user_id = UserId::from(id);
        let serialized = user_id.to_string();
        let deserialized = UserId::parse(&serialized).unwrap();
        prop_assert_eq!(user_id, deserialized);
    }

    #[test]
    fn test_email_validation(s in "\\PC*") {
        // Property: valid emails should always contain '@'
        let result = Email::parse(&s);
        if result.is_ok() {
            prop_assert!(s.contains('@'));
        }
    }
}
```

---

## Macro Expansion Testing Example

```rust
// tests/ui/di_basic.rs (should compile)
use allframe::di_container;

fn main() {
    let container = di_container! {
        UserRepository => PostgresUserRepository,
    };

    let _ = container.resolve::<UserRepository>();
}

// tests/ui/di_invalid.rs (should fail to compile)
use allframe::di_container;

fn main() {
    let container = di_container! {
        // Missing implementation - should fail at compile time
        UserRepository => ,
    };
}
```

---

**Last Updated:** 2025-11-23
**Version:** 2.0.0 (AllFrame-specific)

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.*
