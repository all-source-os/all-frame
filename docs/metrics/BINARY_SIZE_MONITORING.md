# Binary Size Monitoring

**Status**: ✅ ACTIVE
**Date**: 2025-12-01
**Owner**: AllFrame Core Team

---

## Overview

AllFrame uses automated binary size monitoring to ensure the framework stays lightweight and performant. This document describes the monitoring infrastructure and how to use it.

## Size Targets

### Hard Limits

| Configuration | Target | Hard Limit | Status |
|--------------|--------|------------|---------|
| Minimal (no features) | < 1 MB | 2 MB | ✅ |
| Default features | < 3 MB | 5 MB | ✅ |
| All features | < 6 MB | 8 MB | ✅ |

**Philosophy**: Stay well under limits to leave headroom for growth.

---

## Monitoring Infrastructure

### CI/CD Integration

Binary size checks run automatically on:
- ✅ Every pull request
- ✅ Every push to `main`
- ✅ Release builds

**Workflow**: `.github/workflows/binary-size.yml`

### Local Scripts

**Check sizes quickly**:
```bash
./scripts/check_size.sh
```

**Detailed analysis**:
```bash
# Analyze specific configuration
./scripts/analyze_size.sh minimal
./scripts/analyze_size.sh default
./scripts/analyze_size.sh all

# Analyze all configurations
./scripts/analyze_size.sh full
```

---

## How It Works

### 1. Build Configurations

We measure three configurations:

**Minimal** (no features):
```bash
cargo build --release --no-default-features
```

**Default** (main features):
```bash
cargo build --release
```

**All Features**:
```bash
cargo build --release --all-features
```

### 2. Size Measurement

Using `cargo-bloat` to analyze:
- Total binary size
- Per-crate contributions
- Top functions by size
- Dependency impact

### 3. Enforcement

**On PR**:
- Automatic size check
- Report in PR comments
- Fail if limits exceeded

**On Main**:
- Track size trends
- Alert on significant growth
- Document baselines

---

## Using the Scripts

### Quick Check

```bash
$ ./scripts/check_size.sh

🔍 AllFrame Binary Size Check
===============================

📦 Building configurations...

Building minimal (no features)...
Building default features...
Building all features...

📊 Size Analysis
================

✅ Minimal: 1.23MB (under 2.0MB limit)
✅ Default: 2.89MB (under 5.0MB limit)
✅ All Features: 5.67MB (under 8.0MB limit)

✅ All binary sizes within limits!
```

### Detailed Analysis

```bash
$ ./scripts/analyze_size.sh all

🔬 AllFrame Binary Size Analysis
==================================

📊 Analyzing: All features
-------------------

Top 15 crate dependencies by size:

File  .text    Size Crate
0.8%  16.2%  256.5KiB std
0.5%  10.1%  160.0KiB tokio
0.3%   6.5%  103.2KiB hyper
...

Top 15 functions by size:

File  .text   Size Name
0.1%   2.3% 36.5KiB allframe_core::router::Router::new
0.1%   1.9% 30.2KiB allframe_core::cqrs::EventStore::append
...

✅ Analysis complete
```

---

## Integration with cargo-make

Size checks are integrated into the quality gate workflow:

```bash
# Include size check in CI
cargo make ci-with-size

# Standalone size check
cargo make check-size

# Detailed analysis
cargo make analyze-size
```

---

## What to Do When Sizes Grow

### Investigation Steps

1. **Run detailed analysis**:
   ```bash
   ./scripts/analyze_size.sh full
   ```

2. **Identify the culprit**:
   - Check which crate grew
   - Find which features contribute most
   - Review recent changes

3. **Optimization strategies**:
   - Remove unused dependencies
   - Use feature flags to make dependencies optional
   - Enable LTO (Link Time Optimization)
   - Use `strip` to remove debug symbols
   - Consider smaller alternatives

### Example Optimization

**Before** (3.5 MB):
```toml
[dependencies]
serde_json = "1.0"  # Full JSON support
```

**After** (2.8 MB):
```toml
[dependencies]
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
```

---

## Cargo.toml Optimizations

Our release profile is optimized for size:

```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link Time Optimization
codegen-units = 1  # Single codegen unit for better optimization
strip = true       # Strip debug symbols
```

---

## Feature Impact Analysis

Use `cargo-bloat` to measure feature impact:

```bash
# Without feature
cargo bloat --release --no-default-features

# With feature
cargo bloat --release --features "di"

# Compare the difference
```

---

## Continuous Monitoring

### Weekly Reviews

Every week:
1. Check size trends
2. Review growth > 5%
3. Document changes
4. Plan optimizations

### Release Checklist

Before each release:
- ✅ Run full size analysis
- ✅ Verify all sizes under limits
- ✅ Document any growth
- ✅ Update baselines

---

## Tools

### cargo-bloat

Install:
```bash
cargo install cargo-bloat
```

Basic usage:
```bash
# Analyze by crate
cargo bloat --release --crates

# Analyze by function
cargo bloat --release -n 20

# Compare two builds
cargo bloat --release --filter "allframe" > before.txt
# ... make changes ...
cargo bloat --release --filter "allframe" > after.txt
diff before.txt after.txt
```

---

## Troubleshooting

### "cargo-bloat not found"

```bash
cargo install cargo-bloat
```

### Script permissions

```bash
chmod +x scripts/*.sh
```

### Build failures

```bash
# Clean and rebuild
cargo clean
./scripts/check_size.sh
```

---

## Size History

### v0.1.0 Baseline (2025-12-01)

| Configuration | Size | Status |
|--------------|------|---------|
| Minimal | 1.23 MB | ✅ Under limit |
| Default | 2.89 MB | ✅ Under limit |
| All Features | 5.67 MB | ✅ Under limit |

---

## References

- [cargo-bloat documentation](https://github.com/RazrFalcon/cargo-bloat)
- [Rust Binary Size Optimization](https://github.com/johnthagen/min-sized-rust)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

**AllFrame. One frame. Infinite transformations.**
*Built with quality, from day zero.* 🦀
