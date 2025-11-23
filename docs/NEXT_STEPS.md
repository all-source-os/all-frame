# Next Steps for AllFrame Development

**Last Updated**: 2025-01-23
**Current Milestone**: 0.2 (Compile-time DI + OpenAPI)
**Status**: RED phase complete, GREEN phase 30%

## Quick Reference

### Current State
- ✅ v0.1 Complete: Project scaffolding with `allframe ignite`
- ✅ RED Phase Complete: 13 failing tests written for v0.2
- ⏳ GREEN Phase 30%: Basic macro structure created
- ❌ Implementation blocked on DI dependency resolution

### Files to Work On

**Priority 1: Fix DI Container Macro**
- `crates/allframe-macros/src/di.rs` - Main implementation
- `tests/02_di_container.rs` - Tests to make pass

**Priority 2: Implement API Handler Macro**
- `crates/allframe-macros/src/lib.rs` - Add api_handler module
- Create `crates/allframe-macros/src/api.rs` - OpenAPI implementation
- `tests/03_api_handler.rs` - Tests to make pass

**Reference Documentation**
- `docs/MILESTONE_0.2_STATUS.md` - Detailed status and plan
- `docs/current/PRD_01.md` - Requirements and acceptance criteria

## Immediate Next Session Tasks

### Task 1: Debug DI Macro with cargo expand

```bash
# Install cargo-expand if not already installed
cargo install cargo-expand

# View generated code for a specific test
cargo expand --test 02_di_container test_di_basic_injection

# This will show you exactly what the macro is generating
# Compare it to what you expect to see
```

**Expected Output Structure**:
```rust
// What we want to generate:
impl AppContainer {
    fn new() -> Self {
        // Create dependencies in order
        let database = DatabaseService::new();
        let user_repository = UserRepository::new(database);
        let user_service = UserService::new(user_repository);

        // Return container
        Self {
            database,
            user_repository,
            user_service,
        }
    }
}
```

**Current Issue**: The macro is trying to initialize fields in-place instead of creating intermediate variables.

### Task 2: Simplify DI Implementation (MVP Approach)

**Option A: Require All #[provide] Attributes**
```rust
#[di_container]
struct AppContainer {
    #[provide(DatabaseService::new())]
    database: DatabaseService,

    #[provide({
        let db = DatabaseService::new();
        UserRepository::new(db)
    })]
    user_repository: UserRepository,
}
```

Pros:
- Simpler to implement
- User has full control
- Can get tests passing quickly

Cons:
- More boilerplate
- Less "magical"
- Not true auto-wiring

**Option B: Simple Heuristic (Current Approach)**
- Assume dependencies are declared in order
- Each service's `new()` takes the previous field as argument
- Only works for linear dependency chains

**Option C: Full Dependency Analysis (Complex)**
- Parse `new()` signatures
- Build dependency graph
- Topological sort
- Most powerful but most complex

**Recommendation**: Start with Option A to get tests passing, then iterate.

### Task 3: Fix #[provide] Attribute Parsing

Current code has this section (lines 70-84 in di.rs):
```rust
if attr.path().is_ident("provide") {
    let tokens = &attr.meta;
    let expr_str = quote!(#tokens).to_string();
    if let Some(expr_content) = expr_str
        .strip_prefix("provide (")
        .and_then(|s| s.strip_suffix(')'))
    {
        let expr: syn::Expr = syn::parse_str(expr_content)?;
        field_inits.push(quote! {
            #name: #expr
        });
    }
}
```

**Problem**: String manipulation is fragile. Use syn's proper attribute parsing.

**Better Approach**:
```rust
use syn::{Attribute, Meta};

fn extract_provide_expr(attr: &Attribute) -> Result<Option<syn::Expr>> {
    if !attr.path().is_ident("provide") {
        return Ok(None);
    }

    match &attr.meta {
        Meta::List(meta_list) => {
            // Parse the tokens inside provide(...)
            Ok(Some(syn::parse2(meta_list.tokens.clone())?))
        }
        _ => Err(Error::new_spanned(attr, "provide attribute must be a list")),
    }
}
```

### Task 4: Test-Driven Development Loop

1. **Pick ONE test to focus on**
   - Start with `test_di_basic_injection`
   - This has the simplest dependency chain

2. **Run the test**
   ```bash
   cargo test --test 02_di_container test_di_basic_injection -- --nocapture
   ```

3. **Use cargo expand to see generated code**
   ```bash
   cargo expand --test 02_di_container test_di_basic_injection
   ```

4. **Compare generated code to expected**
   - Write down what you see
   - Write down what you want
   - Identify the gap

5. **Make ONE small change to the macro**
   - Fix ONE thing at a time
   - Don't try to fix everything at once

6. **Repeat until test passes**

7. **Move to next test**

## Code Snippets to Reference

### Parsing Function Signatures (for future dependency analysis)

```rust
use syn::{ItemFn, FnArg, Pat, Type};

fn parse_new_signature(impl_block: &syn::ItemImpl) -> Result<Vec<(String, Type)>> {
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident == "new" {
                let mut params = Vec::new();
                for input in &method.sig.inputs {
                    if let FnArg::Typed(pat_type) = input {
                        if let Pat::Ident(pat_ident) = &*pat_type.pat {
                            let name = pat_ident.ident.to_string();
                            let ty = (*pat_type.ty).clone();
                            params.push((name, ty));
                        }
                    }
                }
                return Ok(params);
            }
        }
    }
    Err(Error::new(impl_block.span(), "No new() method found"))
}
```

### Generating Intermediate Variables

```rust
// Instead of:
quote! {
    Self {
        database: DatabaseService::new(),
        repository: UserRepository::new(database), // ERROR: database not in scope
    }
}

// Generate:
quote! {
    let database = DatabaseService::new();
    let repository = UserRepository::new(database);

    Self {
        database,
        repository,
    }
}
```

### Testing Macro Output

Create a test file: `crates/allframe-macros/tests/expand.rs`

```rust
#[test]
fn test_di_expansion() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/di_basic.rs");
}
```

Then create `tests/ui/di_basic.rs`:
```rust
use allframe_macros::di_container;

struct DatabaseService;
impl DatabaseService {
    fn new() -> Self { Self }
}

struct UserRepository {
    db: DatabaseService,
}

impl UserRepository {
    fn new(db: DatabaseService) -> Self {
        Self { db }
    }
}

#[di_container]
struct AppContainer {
    database: DatabaseService,
    user_repository: UserRepository,
}

fn main() {
    let container = AppContainer::new();
    let _ = container.database();
    let _ = container.user_repository();
}
```

## Resources

### Proc Macro Development
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [syn documentation](https://docs.rs/syn/latest/syn/)
- [quote documentation](https://docs.rs/quote/latest/quote/)
- [proc-macro2 documentation](https://docs.rs/proc-macro2/latest/proc_macro2/)

### Example DI Implementations
- [shaku](https://github.com/Mcat12/shaku) - Compile-time DI
- [dependency-injection](https://github.com/Wopple/dependency-injection) - Runtime DI

### Debugging Tools
- `cargo expand` - View macro output
- `RUST_LOG=trace cargo test` - Verbose test output
- `cargo +nightly rustc -- -Zmacro-backtrace` - Macro backtrace

## Quality Gates Before Merge

Before considering v0.2 complete:

```bash
# All tests must pass
cargo test

# No clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# Code must be formatted
cargo fmt -- --check

# 100% test coverage (requires llvm-cov)
cargo llvm-cov --all-features

# All examples must compile
cargo build --examples

# Documentation must build
cargo doc --no-deps --all-features
```

## When You're Stuck

### Strategy 1: Simplify the Problem
- Can you make the test simpler?
- Can you remove features temporarily?
- Can you hard-code something to get it working first?

### Strategy 2: Study Working Examples
- Find a similar proc macro in the wild
- Use `cargo expand` on their examples
- See how they solve the same problem

### Strategy 3: Ask for Help
- Rust forums: https://users.rust-lang.org/
- Rust Discord: https://discord.gg/rust-lang
- Stack Overflow with `[rust] [procedural-macros]` tags

### Strategy 4: Take a Break
- Sometimes the solution comes when you're not actively thinking about it
- Go for a walk
- Work on documentation
- Come back fresh

## Success Metrics

You'll know you're making progress when:

1. ✅ `cargo expand` shows code that looks reasonable
2. ✅ At least one test compiles (even if it fails at runtime)
3. ✅ `test_di_basic_injection` passes
4. ✅ All 5 DI tests pass
5. ✅ Basic API handler generates a schema string
6. ✅ All 8 API handler tests pass
7. ✅ Quality gates pass
8. ✅ Can create a real project and use the macros

## Final Notes

- **Don't aim for perfection** - Get it working first, then refactor
- **One test at a time** - Don't try to pass all tests at once
- **Use TDD discipline** - If you write code without a test, delete it
- **Document as you go** - Future you will thank present you
- **Commit often** - Small commits are easier to review and revert

Good luck! You've got this. 🚀

**Next session goal**: Get `test_di_basic_injection` passing.
