# MCP Server Zero-Bloat Strategy

**Goal**: Ensure MCP server adds ZERO bytes to binaries when not enabled
**Status**: Implementation Guide

---

## Problem

MCP server is a powerful feature, but:
- Not all AllFrame users need LLM integration
- Should not increase binary size for non-MCP users
- Must remain completely optional

## Solution: Multi-Layered Isolation

### Layer 1: Feature Flag (Current ✅)

**Status**: Already implemented

```toml
# Cargo.toml
[features]
mcp = []  # Optional, NOT in default features

# User opts in explicitly
allframe-core = { version = "0.5", features = ["mcp"] }
```

**Result**: Code only compiles when `--features mcp` is used

### Layer 2: Separate Crate (Recommended 🎯)

**Problem with current approach**:
Even with feature flags, the code exists in `allframe-core/src/mcp/`

**Better approach**: Move to separate crate

```
crates/
├── allframe-core/        # Core framework (NO MCP code)
├── allframe-mcp/         # MCP server (separate crate)
└── allframe-cli/         # CLI tool (uses allframe-mcp)
```

**Benefits**:
1. **Zero Bloat**: MCP code never compiled unless explicitly added
2. **Clear Separation**: Framework vs. tooling
3. **Independent Versioning**: MCP can evolve separately
4. **Smaller Core**: allframe-core stays lean

### Layer 3: Binary Size Verification

**Add CI Check**:

```yaml
# .github/workflows/binary-size.yml
name: Binary Size Check

on: [pull_request]

jobs:
  check-size:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Build WITHOUT mcp feature
      - name: Build minimal binary
        run: |
          cargo build --release --no-default-features --example minimal
          SIZE=$(stat -c%s "target/release/examples/minimal")
          echo "Binary size: $SIZE bytes"

          # Fail if > 2MB (2,097,152 bytes)
          if [ $SIZE -gt 2097152 ]; then
            echo "❌ Binary too large: $SIZE bytes (max: 2MB)"
            exit 1
          fi

      # Build WITH mcp feature
      - name: Build with MCP
        run: |
          cargo build --release --features mcp --example mcp_server
          SIZE=$(stat -c%s "target/release/examples/mcp_server")
          echo "Binary size with MCP: $SIZE bytes"
```

---

## Implementation Plan

### Option A: Keep Current Structure (Quick Fix)

**Pros**: No refactoring needed
**Cons**: MCP code still in allframe-core repo

**Actions**:
1. ✅ Keep `mcp` feature flag optional
2. ✅ Exclude from default features
3. Add binary size CI check
4. Document that users must opt-in

**Result**: ~90% bloat prevention

### Option B: Separate Crate (Recommended)

**Pros**: 100% zero bloat, cleaner architecture
**Cons**: Requires refactoring

**Step 1**: Create `allframe-mcp` crate

```bash
mkdir -p crates/allframe-mcp
```

```toml
# crates/allframe-mcp/Cargo.toml
[package]
name = "allframe-mcp"
version = "0.5.0"
description = "MCP server for AllFrame - Expose APIs as LLM tools"

[dependencies]
allframe-core = { path = "../allframe-core" }
serde = "1.0"
serde_json = "1.0"
tokio = "1.0"
```

**Step 2**: Move MCP code

```bash
mv crates/allframe-core/src/mcp crates/allframe-mcp/src/
```

**Step 3**: Update exports

```rust
// crates/allframe-mcp/src/lib.rs
pub mod server;
pub mod tools;
pub mod schema;

pub use server::McpServer;
pub use tools::McpTool;
pub use schema::*;
```

**Step 4**: Users opt-in via separate dependency

```toml
# User's Cargo.toml (Framework only)
[dependencies]
allframe-core = "0.5"

# User's Cargo.toml (Framework + MCP)
[dependencies]
allframe-core = "0.5"
allframe-mcp = "0.5"  # Explicit opt-in
```

**Result**: 100% zero bloat for non-MCP users

---

## Verification Methods

### Method 1: Size Comparison

```bash
# Build without MCP
cargo build --release --no-default-features
ls -lh target/release/liballframe_core.rlib

# Build with MCP (current approach)
cargo build --release --features mcp
ls -lh target/release/liballframe_core.rlib

# Compare sizes - should be identical if properly isolated
```

### Method 2: Symbol Analysis

```bash
# Check for MCP symbols in minimal build
nm target/release/liballframe_core.rlib | grep -i mcp

# Should return NOTHING if properly isolated
```

### Method 3: Dependency Tree

```bash
# Verify MCP deps only loaded when feature enabled
cargo tree --features mcp | grep mcp
cargo tree                  | grep mcp  # Should be empty
```

### Method 4: Bloaty McBloatface (Advanced)

```bash
# Install bloaty
cargo install cargo-bloat

# Analyze binary sections
cargo bloat --release --features mcp
cargo bloat --release  # Compare
```

---

## Recommended Architecture

### Final Structure

```
allframe/
├── crates/
│   ├── allframe-core/       # Core framework (lean!)
│   │   ├── src/
│   │   │   ├── router/      # Protocol routing
│   │   │   ├── cqrs/        # Event sourcing
│   │   │   └── otel/        # Telemetry
│   │   └── Cargo.toml       # NO mcp in dependencies
│   │
│   ├── allframe-mcp/        # MCP server (separate!)
│   │   ├── src/
│   │   │   ├── server.rs    # MCP server
│   │   │   ├── tools.rs     # Tool definitions
│   │   │   └── schema.rs    # Schema generation
│   │   └── Cargo.toml       # Depends on allframe-core
│   │
│   └── allframe-cli/        # CLI tool
│       ├── src/
│       │   └── main.rs      # MCP serve command
│       └── Cargo.toml       # Depends on allframe-mcp
│
└── examples/
    ├── rest_api.rs          # Uses allframe-core only
    └── mcp_server.rs        # Uses allframe-mcp
```

### Usage Patterns

**Pattern 1: Framework Only** (Most users)
```toml
[dependencies]
allframe-core = "0.5"
```

**Pattern 2: Framework + MCP** (LLM integration)
```toml
[dependencies]
allframe-core = "0.5"
allframe-mcp = "0.5"
```

**Pattern 3: CLI Tool** (Developer tooling)
```bash
cargo install allframe-cli  # Includes MCP server binary
```

---

## Migration Plan

### Phase 1: Immediate (Keep Current)
- [x] MCP behind feature flag
- [ ] Add binary size CI check
- [ ] Document zero-bloat guarantee
- [ ] Measure baseline sizes

### Phase 2: Refactor (Next Release)
- [ ] Create `allframe-mcp` crate
- [ ] Move MCP code
- [ ] Update examples
- [ ] Update documentation
- [ ] Test both patterns

### Phase 3: Verification
- [ ] Binary size comparison
- [ ] Symbol analysis
- [ ] Dependency tree check
- [ ] CI enforcement

---

## Binary Size Targets

| Configuration | Target | Status |
|---------------|--------|--------|
| **Core only** | < 2 MB | 🎯 Target |
| **Core + Router** | < 3 MB | 🎯 Target |
| **Core + Router + CQRS** | < 4 MB | 🎯 Target |
| **Core + MCP** | < 5 MB | 🎯 Target |
| **Full features** | < 8 MB | 🎯 Target |

---

## Testing Strategy

### Test 1: Minimal Build
```bash
# Should produce smallest binary
cargo build --release --no-default-features --example minimal
stat -c%s target/release/examples/minimal
# Expected: < 2MB
```

### Test 2: Core + Router
```bash
# Add router but no MCP
cargo build --release --features router --example rest_api
stat -c%s target/release/examples/rest_api
# Expected: < 3MB
```

### Test 3: With MCP
```bash
# Add MCP server
cargo build --release --features mcp --example mcp_server
stat -c%s target/release/examples/mcp_server
# Expected: < 5MB (only 2MB added for MCP)
```

---

## Documentation

### User Guide

**For Framework Users** (in README):
```markdown
## Installation

### Core Framework (Minimal)
```toml
[dependencies]
allframe-core = "0.5"
```

Binary size: ~2 MB

### With MCP Server (LLM Integration)
```toml
[dependencies]
allframe-core = "0.5"
allframe-mcp = "0.5"  # Optional: adds ~2 MB
```

Binary size: ~5 MB
```

### Feature Flag Table

| Feature | Binary Impact | Default | Use Case |
|---------|---------------|---------|----------|
| `di` | +0 KB | ✅ | Dependency injection |
| `router` | +500 KB | ✅ | Multi-protocol routing |
| `cqrs` | +800 KB | ❌ | Event sourcing |
| `mcp` | +2 MB | ❌ | LLM integration |

---

## Monitoring

### CI Pipeline

```yaml
name: Zero Bloat Check

on: [pull_request, push]

jobs:
  size-check:
    runs-on: ubuntu-latest
    steps:
      - name: Build minimal
        run: cargo build --release --no-default-features

      - name: Check size
        run: |
          SIZE=$(stat -c%s target/release/liballframe_core.rlib)
          echo "::set-output name=size::$SIZE"

          # Fail if core exceeds 2MB
          [ $SIZE -lt 2097152 ] || exit 1

      - name: Report
        run: |
          echo "✅ Core library: ${{ steps.check.outputs.size }} bytes"
          echo "📊 Target: < 2MB (2,097,152 bytes)"
```

---

## Best Practices

### For Core Contributors

1. **Never add MCP deps to allframe-core**
2. **Always use feature flags for optional code**
3. **Prefer separate crates for tooling**
4. **Test minimal build regularly**
5. **Document binary impact**

### For MCP Contributors

1. **Keep MCP code in allframe-mcp crate**
2. **Minimize dependencies**
3. **Use dynamic linking where possible**
4. **Profile binary size impact**
5. **Document size tradeoffs**

---

## Decision Matrix

**When to use Feature Flag**:
- ✅ Small optional features (< 100 KB)
- ✅ Compile-time toggles
- ✅ Cross-cutting concerns

**When to use Separate Crate**:
- ✅ Large optional features (> 1 MB)
- ✅ Distinct functionality
- ✅ **MCP Server** (2 MB+)
- ✅ Developer tools
- ✅ Optional integrations

---

## Recommendation: Separate Crate ✅

**Reasons**:
1. MCP adds ~2 MB (significant)
2. Not core framework functionality
3. Clear use case separation
4. Better for maintenance
5. 100% zero bloat guarantee

**Timeline**:
- **v0.5.0**: Ship with feature flag (current)
- **v0.6.0**: Move to `allframe-mcp` crate

---

**Status**: Ready to implement
**Owner**: @all-source-os
**Next**: Create allframe-mcp crate structure
