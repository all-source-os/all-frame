# Milestone 0.2 Status: Compile-time DI + OpenAPI

**Status**: RED Phase Complete, GREEN Phase In Progress
**Date**: 2025-01-23
**PRD Reference**: PRD_01.md Lines 67-68

## Overview

Milestone 0.2 aims to deliver:
- ✅ Compile-time Dependency Injection with zero runtime reflection
- ✅ Automatic OpenAPI 3.1 schema generation
- ✅ 100% test coverage requirement

## Progress Summary

### ✅ Completed: RED Phase

#### 1. DI Container Tests (`tests/02_di_container.rs`)

Created 5 comprehensive tests following TDD principles:

1. **`test_di_basic_injection`** - Basic 3-level dependency chain
   - DatabaseService → UserRepository → UserService
   - Tests automatic dependency resolution
   - Validates service functionality through entire chain

2. **`test_di_trait_injection`** - Trait-based dependencies
   - Tests `#[provide(expression)]` attribute
   - Validates Box<dyn Trait> injection
   - Ensures trait objects work correctly

3. **`test_di_thread_safety`** - Concurrent access
   - Tests Arc-based shared services
   - Spawns 10 threads accessing same service
   - Validates thread-safety of container

4. **`test_di_nested_dependencies`** - Deep nesting (4 levels)
   - Database → Repository → Service → Controller
   - Tests complex dependency graphs
   - Validates correct initialization order

5. **`test_di_multiple_instances`** - Multiple services of same type
   - Tests `#[provide]` with custom initialization
   - Validates multiple Counter instances with different names

**Test Status**: All tests fail as expected (RED phase complete)

#### 2. API Handler Tests (`tests/03_api_handler.rs`)

Created 8 comprehensive tests for OpenAPI generation:

1. **`test_api_handler_basic`** - Basic handler with schema generation
   - POST /users endpoint
   - Tests OpenAPI schema contains path, method, description
   - Validates request/response types in schema

2. **`test_api_handler_with_query`** - Query parameter handling
   - GET /users with pagination
   - Tests query parameters in OpenAPI schema
   - Validates optional parameter handling

3. **`test_api_handler_with_path_params`** - Path parameter extraction
   - GET /users/{id}
   - Tests path parameter in schema
   - Validates parameter type information

4. **`test_api_handler_with_error_responses`** - Multiple response codes
   - POST /validate with 200/400 responses
   - Tests Result<Success, Error> handling
   - Validates multiple response schemas

5. **`test_api_handler_valid_json_schema`** - Schema validation
   - GET /health endpoint
   - Tests schema is valid JSON
   - Validates OpenAPI structure

6. **`test_api_handler_with_validation`** - Request validation
   - POST /register with validation
   - Tests validation rules in schema

7. **`test_openapi_schema_aggregation`** - Multiple handlers
   - Tests combining schemas from multiple endpoints
   - Validates complete API documentation

**Test Status**: All tests fail as expected (RED phase complete)

### ⏳ In Progress: GREEN Phase

#### 1. DI Container Implementation (`crates/allframe-macros/src/di.rs`)

**Created**: Initial implementation structure
**Status**: Partial implementation with known issues

**What Works**:
- ✅ Struct parsing with syn
- ✅ Field extraction (names, types, attributes)
- ✅ Accessor method generation
- ✅ Basic code structure

**What Needs Work**:
- ❌ Dependency graph analysis
- ❌ Correct field initialization order
- ❌ Constructor signature parsing
- ❌ Dependency resolution algorithm
- ❌ #[provide(...)] attribute parsing (basic version exists but incomplete)

**Current Error**:
```
error[E0425]: cannot find value `database` in this scope
   --> tests/02_di_container.rs:72:9
```

The macro is generating field names as variables instead of struct field initializers.

#### 2. API Handler Implementation

**Status**: Not started (placeholder implementation only)

## Technical Challenges

### DI Container Macro Complexity

The #[di_container] macro requires sophisticated compile-time analysis:

1. **Dependency Graph Construction**
   - Parse each service's `new()` method signature
   - Extract parameter types and names
   - Build directed graph of dependencies
   - Detect cycles (compile-time error)

2. **Topological Sorting**
   - Order services based on dependencies
   - Ensure dependencies are created before dependents
   - Handle multiple valid orderings

3. **Code Generation**
   - Generate correct initialization code
   - Pass dependencies to constructors
   - Handle both `Type::new()` and `Type::new(dep1, dep2)`
   - Support #[provide(...)] for custom initialization

4. **Attribute Processing**
   - Parse `#[provide(expression)]` attributes
   - Extract and validate expressions
   - Generate code that evaluates expressions in context

**Example of Expected Output**:

```rust
// Input:
#[di_container]
struct AppContainer {
    database: DatabaseService,
    repository: UserRepository,
    service: UserService,
}

// Expected Generated Code:
impl AppContainer {
    fn new() -> Self {
        let database = DatabaseService::new();
        let repository = UserRepository::new(database);
        let service = UserService::new(repository);

        Self {
            database,
            repository,
            service,
        }
    }

    fn database(&self) -> &DatabaseService {
        &self.database
    }

    fn repository(&self) -> &UserRepository {
        &self.repository
    }

    fn service(&self) -> &UserService {
        &self.service
    }
}
```

**Current Implementation Gap**:
The macro currently tries to initialize fields directly in the struct literal, but doesn't create intermediate variables or analyze constructor signatures.

### OpenAPI Handler Macro Complexity

The #[api_handler] macro needs to:

1. **Type Extraction**
   - Extract function signature
   - Identify request types (body, query, path params)
   - Identify response types
   - Handle Result<T, E> for error responses

2. **Schema Generation**
   - Generate OpenAPI 3.1 compliant JSON
   - Create schema for each type (recursive for nested types)
   - Handle serde attributes (rename, skip, etc.)
   - Generate parameter definitions

3. **Code Generation**
   - Keep original function intact
   - Generate `{function_name}_openapi_schema()` helper
   - Return valid JSON string

## Implementation Plan

### Phase 1: DI Container (Priority: High)

**Step 1: Simplify MVP Scope**
- Support only explicit dependencies via #[provide]
- Require all fields to have #[provide] attribute
- Remove automatic dependency resolution for v0.2
- Get basic tests passing

**Step 2: Improve #[provide] Parsing**
- Fix attribute expression extraction
- Validate expressions at compile time
- Generate correct field initialization

**Step 3: Add Dependency Analysis (v0.3)**
- Parse `new()` signatures using syn
- Build dependency graph
- Implement topological sort
- Generate initialization in correct order

### Phase 2: OpenAPI Handler (Priority: High)

**Step 1: Basic Schema Generation**
- Extract function signature
- Generate minimal OpenAPI JSON
- Return static schema string

**Step 2: Type Schema Generation**
- Use serde to infer types
- Generate JSON schema for structs
- Handle primitive types

**Step 3: Advanced Features**
- Path parameter extraction
- Query parameter handling
- Multiple response codes
- Schema aggregation

### Phase 3: Integration & Testing

**Step 1: Get Tests Passing**
- Fix DI container tests
- Fix API handler tests
- Verify 100% test coverage

**Step 2: Quality Gates**
- cargo test (all pass)
- cargo clippy (no warnings)
- cargo fmt (formatted)
- cargo llvm-cov (100% coverage)

**Step 3: Documentation**
- Add usage examples
- Document limitations
- Update README

## Next Steps

### Immediate Actions (Next Session)

1. **Fix DI Container Macro**
   - Use `cargo expand` to debug generated code
   - Fix field initialization to use intermediate variables
   - Get `test_di_basic_injection` passing

2. **Implement Simple #[provide] Support**
   - Fix attribute parsing
   - Get `test_di_multiple_instances` passing

3. **Start API Handler Implementation**
   - Create basic structure
   - Generate minimal schema
   - Get `test_api_handler_basic` passing

### Tools & Resources Needed

- **cargo expand**: View generated macro code
- **syn documentation**: Better understanding of AST parsing
- **quote documentation**: Code generation patterns
- **Reference implementations**: Study existing DI crates

### Alternative Approaches

If macro complexity becomes blocking:

**Option A**: Runtime DI (Compromise)
- Use type registry pattern
- Zero-cost abstractions still possible
- Defer full compile-time DI to v0.3

**Option B**: Code Generation (Not Macros)
- Use build.rs for code generation
- Easier to debug
- Less "magical" but more explicit

**Option C**: Trait-Based DI
- Define Provider trait
- Manual implementations
- Macro generates boilerplate only

## Current File Structure

```
crates/
├── allframe-core/
│   └── src/lib.rs (minimal)
├── allframe-macros/
│   ├── src/
│   │   ├── lib.rs (macro exports)
│   │   └── di.rs (DI implementation - partial)
│   └── Cargo.toml
└── allframe-forge/
    └── src/
        ├── lib.rs
        ├── main.rs
        ├── templates.rs
        ├── scaffolding.rs
        └── validation.rs

tests/
├── 01_ignite_project.rs (✅ 5/5 passing)
├── 02_di_container.rs (❌ 0/5 passing - RED phase)
└── 03_api_handler.rs (❌ 0/8 passing - RED phase)
```

## Acceptance Criteria (from PRD)

| Criterion | Status |
|-----------|--------|
| Inject 50+ nested services, zero runtime reflection | ❌ Not implemented |
| `curl /openapi.json` returns valid OpenAPI 3.1 | ❌ Not implemented |
| Swagger UI loads and displays endpoints | ❌ Not implemented |
| MCP schema present | ❌ Not implemented |
| 100% test coverage | ✅ Tests written, need implementation |

## Conclusion

**Milestone 0.2 Progress**: ~30% complete

- ✅ **RED Phase**: Complete (comprehensive tests written)
- ⏳ **GREEN Phase**: 10% complete (basic structure, needs implementation)
- ❌ **REFACTOR Phase**: Not started

**Estimated Effort Remaining**: 6-8 hours of focused development

**Blockers**:
- DI macro dependency resolution complexity
- Need to study existing proc macro patterns more

**Recommendation**:
Continue with simplified MVP approach - get basic functionality working, then iterate to add advanced features in subsequent versions.
