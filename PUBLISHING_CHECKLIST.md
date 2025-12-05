# Publishing Checklist - v0.1.0

**Date**: 2025-12-04
**Crates to Publish**: 4 crates (allframe-macros, allframe-core, allframe-forge, allframe-mcp)
**Status**: Ready for publication (after fixes below)

---

## Pre-Publishing Fixes Required

### 1. Fix allframe-core Cargo.toml

**Issue**: `allframe-macros` dependency needs version specified

**File**: `crates/allframe-core/Cargo.toml:49`

**Change**:
```toml
# Before:
allframe-macros = { path = "../allframe-macros", optional = true }

# After:
allframe-macros = { version = "0.1.0", path = "../allframe-macros", optional = true }
```

### 2. Add Keywords/Categories to allframe-macros

**File**: `crates/allframe-macros/Cargo.toml`

**Add**:
```toml
keywords = ["proc-macro", "macros", "framework", "web", "allframe"]
categories = ["development-tools::procedural-macro-helpers"]
```

### 3. Add Keywords/Categories to allframe-forge

**File**: `crates/allframe-forge/Cargo.toml`

**Add**:
```toml
keywords = ["cli", "scaffold", "generator", "allframe", "web"]
categories = ["command-line-utilities", "development-tools"]
```

---

## Publishing Order (Critical!)

**MUST publish in this exact order due to dependencies:**

```
1. allframe-macros      (no allframe dependencies)
2. allframe-core        (depends on allframe-macros)
3. allframe-forge       (independent)
4. allframe-mcp         (depends on allframe-core)
```

---

## Step-by-Step Publishing Commands

### Prerequisites

```bash
# 1. Ensure you're logged in to crates.io
cargo login <your-api-token>

# 2. Ensure working directory is clean
git status  # Should be clean

# 3. Ensure on main branch
git checkout main
git pull origin main
```

### Step 1: Publish allframe-macros

```bash
cd crates/allframe-macros

# Dry run
cargo publish --dry-run

# Review output - should see:
# ✓ Packaging allframe-macros v0.1.0
# ✓ Packaged 10 files, ~55KiB
# ✓ Verifying allframe-macros v0.1.0
# ✓ Compiling...
# ✓ warning: aborting upload due to dry run

# If dry run succeeds, publish for real
cargo publish

# Expected output:
# Uploading allframe-macros v0.1.0
# Uploaded allframe-macros v0.1.0 to registry

# IMPORTANT: Wait 2 minutes for crates.io indexing!
echo "Waiting 120 seconds for crates.io to index allframe-macros..."
sleep 120

# Verify on crates.io
open https://crates.io/crates/allframe-macros
```

### Step 2: Publish allframe-core

```bash
cd ../allframe-core

# Dry run
cargo publish --dry-run

# Expected warnings (safe to ignore):
# - "dependency `allsource-core` does not have a version requirement"
#   → This is a git dependency, intentional for unreleased package

# If dry run succeeds, publish
cargo publish

# Wait for indexing
echo "Waiting 120 seconds for crates.io to index allframe-core..."
sleep 120

# Verify
open https://crates.io/crates/allframe-core
```

### Step 3: Publish allframe-forge

```bash
cd ../allframe-forge

# Dry run
cargo publish --dry-run

# Publish
cargo publish

# Wait for indexing
echo "Waiting 120 seconds for crates.io to index allframe-forge..."
sleep 120

# Verify
open https://crates.io/crates/allframe-forge
```

### Step 4: Update allframe-mcp Dependencies

**IMPORTANT**: After allframe-core is published, update allframe-mcp to use the published version.

**Edit**: `crates/allframe-mcp/Cargo.toml`

```toml
# Change this line:
allframe-core = { path = "../allframe-core" }

# To this:
allframe-core = "0.1.0"
```

**Test it works**:
```bash
cd ../allframe-mcp
rm -f Cargo.lock
cargo build
cargo test --lib
```

### Step 5: Publish allframe-mcp

```bash
cd crates/allframe-mcp

# Dry run
cargo publish --dry-run

# Publish
cargo publish

# Wait for indexing
echo "Waiting 120 seconds for crates.io to index allframe-mcp..."
sleep 120

# Verify
open https://crates.io/crates/allframe-mcp
```

---

## Post-Publishing

### 1. Tag the Release

```bash
# Go back to root
cd ../..

# Create annotated tag
git tag -a v0.1.0 -m "Release v0.1.0

Published to crates.io:
- allframe-macros v0.1.0
- allframe-core v0.1.0
- allframe-forge v0.1.0
- allframe-mcp v0.1.0

Initial public release with:
- Protocol-agnostic routing (REST, GraphQL, gRPC)
- CQRS + Event Sourcing infrastructure
- Native MCP server (separate crate, zero bloat)
- 291+ tests passing
"

# Push tag
git push origin v0.1.0
```

### 2. Verify All Packages

Visit each page and verify:
- ✅ Version shows "0.1.0"
- ✅ Documentation link works (may take 5-10 min for docs.rs to build)
- ✅ README displays correctly
- ✅ License shows "MIT OR Apache-2.0"

URLs:
- https://crates.io/crates/allframe-macros
- https://crates.io/crates/allframe-core
- https://crates.io/crates/allframe-forge
- https://crates.io/crates/allframe-mcp

### 3. Test Installation

Create a fresh test project:

```bash
mkdir /tmp/test-allframe
cd /tmp/test-allframe
cargo new test-app
cd test-app

# Add to Cargo.toml:
cat >> Cargo.toml <<'EOF'

[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"
tokio = { version = "1.48", features = ["full"] }
EOF

# Build
cargo build

# Should download and compile successfully!
```

### 4. Update README Badges (Optional)

Add crates.io badges to root README.md:

```markdown
[![allframe-core](https://img.shields.io/crates/v/allframe-core.svg)](https://crates.io/crates/allframe-core)
[![allframe-mcp](https://img.shields.io/crates/v/allframe-mcp.svg)](https://crates.io/crates/allframe-mcp)
[![Downloads](https://img.shields.io/crates/d/allframe-core.svg)](https://crates.io/crates/allframe-core)
```

### 5. Create Announcement

File: `docs/announcements/CRATES_IO_v0.1.0.md`

```markdown
# AllFrame v0.1.0 Published to crates.io! 🎉

**Date**: 2025-12-04

We're excited to announce that AllFrame v0.1.0 is now available on crates.io!

## Published Packages

### allframe-core
**Protocol-agnostic Rust web framework**
- 📦 https://crates.io/crates/allframe-core
- 📚 https://docs.rs/allframe-core
- ✅ 258 tests passing

### allframe-mcp
**MCP server for LLM integration**
- 📦 https://crates.io/crates/allframe-mcp
- 📚 https://docs.rs/allframe-mcp
- ✅ 33 tests passing

### allframe-macros
**Procedural macros for AllFrame**
- 📦 https://crates.io/crates/allframe-macros
- 📚 https://docs.rs/allframe-macros

### allframe-forge
**AllFrame CLI - Project scaffolding**
- 📦 https://crates.io/crates/allframe-forge
- 📚 https://docs.rs/allframe-forge

## Installation

```toml
[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"  # Optional: MCP server
```

## What's Included

- ✅ Protocol-agnostic routing (REST, GraphQL, gRPC)
- ✅ CQRS + Event Sourcing infrastructure
- ✅ OpenAPI, GraphQL, gRPC documentation
- ✅ Native MCP server (separate crate, zero bloat!)
- ✅ Compile-time dependency injection
- ✅ 291+ tests passing

## Get Started

Visit https://docs.rs/allframe-core for complete documentation!

## Social Media

Tweet/share:
```

---

## Troubleshooting

### Error: "all dependencies must have a version requirement"

**Cause**: Path dependencies need version specified for publishing.

**Solution**: Add `version = "0.1.0"` to all path dependencies.

### Error: "crate name is already taken"

**Cause**: Package name already exists on crates.io.

**Solution**: Choose a different name or request transfer if you own the project.

### Error: "failed to verify package"

**Cause**: Build failed with published dependencies.

**Solution**: Ensure previous crates are published and indexed (wait 2 minutes).

### Docs.rs Build Failed

**Check**: https://docs.rs/crate/allframe-core/latest/builds

**Common fixes**:
- Add `[package.metadata.docs.rs]` with `all-features = true`
- Ensure all feature dependencies are available

---

## Rollback Plan

If something goes wrong:

### Yank a Version

```bash
# Yank (makes version unavailable for new installs)
cargo yank --vers 0.1.0 allframe-core

# Unyank (if you change your mind)
cargo yank --vers 0.1.0 --undo allframe-core
```

**Note**: Yanking doesn't delete, just marks as "do not use". Users who already depend on it can still build.

---

## Complete One-Liner Script

Once fixes are applied, run this (at your own risk!):

```bash
#!/bin/bash
set -e

echo "🚀 Publishing AllFrame v0.1.0 to crates.io..."

# Publish allframe-macros
cd crates/allframe-macros
cargo publish
sleep 120
cd ../..

# Publish allframe-core
cd crates/allframe-core
cargo publish
sleep 120
cd ../..

# Publish allframe-forge
cd crates/allframe-forge
cargo publish
sleep 120
cd ../..

# Update allframe-mcp dependency
# (Manual step - edit Cargo.toml)

# Publish allframe-mcp
cd crates/allframe-mcp
cargo publish
sleep 120
cd ../..

# Tag release
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

echo "✅ All packages published!"
echo "📦 https://crates.io/crates/allframe-core"
echo "📦 https://crates.io/crates/allframe-mcp"
```

---

## Checklist Summary

Before publishing:
- [ ] Apply Cargo.toml fixes (version, keywords)
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo test -p allframe-core --lib`
- [ ] Run `cargo test -p allframe-mcp --lib`
- [ ] Commit all changes
- [ ] `git status` is clean
- [ ] On main branch
- [ ] Logged in to crates.io

During publishing:
- [ ] Publish in order: macros → core → forge → mcp
- [ ] Wait 120s between publishes
- [ ] Update allframe-mcp Cargo.toml after allframe-core published
- [ ] Verify each package on crates.io

After publishing:
- [ ] Tag release v0.1.0
- [ ] Push tag to GitHub
- [ ] Test installation in fresh project
- [ ] Verify docs.rs builds
- [ ] Create announcement
- [ ] Social media posts

---

**Status**: Ready to publish (after Cargo.toml fixes)
**Owner**: @all-source-os
**Last Updated**: 2025-12-04
