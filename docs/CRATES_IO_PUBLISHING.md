# Publishing to crates.io - Complete Guide

**Date**: 2025-12-04
**Status**: ✅ Ready for v0.1.0 release
**Crates**: allframe-macros, allframe-core, allframe-forge, allframe-mcp

---

## Quick Start

```bash
# 1. Ensure logged in
cargo login <your-api-token>

# 2. Publish in order (wait 120s between each)
cd crates/allframe-macros && cargo publish && sleep 120
cd ../allframe-core && cargo publish && sleep 120
cd ../allframe-forge && cargo publish && sleep 120

# 3. Update allframe-mcp Cargo.toml: change path to version "0.1.0"
cd ../allframe-mcp && cargo publish

# 4. Tag release
cd ../.. && git tag -a v0.1.0 -m "Release v0.1.0" && git push origin v0.1.0
```

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Pre-Publication Checklist](#pre-publication-checklist)
3. [Publishing Order](#publishing-order)
4. [Step-by-Step Instructions](#step-by-step-instructions)
5. [Post-Publication](#post-publication)
6. [Troubleshooting](#troubleshooting)
7. [Automated Publishing (CI/CD)](#automated-publishing-cicd)

---

## Prerequisites

### 1. crates.io Account

Create a free account at https://crates.io:

1. Visit https://crates.io
2. Click "Log in with GitHub"
3. Authorize crates.io to access your GitHub account
4. Verify your email address

### 2. API Token

Get your personal API token:

1. Log in to https://crates.io
2. Visit https://crates.io/me
3. Click "New Token"
4. Name: `allframe-publishing`
5. Click "Generate Token"
6. **IMPORTANT**: Copy the token immediately (you won't see it again!)

### 3. Configure cargo

```bash
# Store your API token
cargo login <your-api-token>

# This saves the token to ~/.cargo/credentials.toml
```

**Security Note**: Keep your API token secret! Never commit it to git.

---

## Pre-Publication Checklist

### Workspace-Level (Cargo.toml)

- [x] `rust-version = "1.86"` - MSRV specified
- [x] `authors` - Correct authors listed
- [x] `license = "MIT OR Apache-2.0"` - Dual license
- [x] `repository` - GitHub URL correct
- [x] `homepage` - Project homepage correct
- [x] All workspace dependencies use exact versions (e.g., `"1.48"` not `"1"`)

### allframe-core (crates/allframe-core/Cargo.toml)

```toml
[package]
name = "allframe-core"
version = "0.1.0"  # Semantic versioning
description = "Protocol-agnostic Rust web framework with TDD, CQRS, and zero-bloat design"
keywords = ["web", "framework", "api", "graphql", "grpc"]  # Max 5 keywords
categories = ["web-programming", "api-bindings", "asynchronous"]  # Max 5 categories
readme = "README.md"  # Package-level README
```

**Checklist**:
- [ ] `version = "0.1.0"` - Correct version number
- [ ] `description` - Clear, concise (< 100 chars)
- [ ] `keywords` - Relevant keywords (max 5)
- [ ] `categories` - Correct categories (see https://crates.io/categories)
- [ ] `readme = "README.md"` - Package README exists
- [ ] All dependencies are published or workspace-local
- [ ] No `path` dependencies to external crates
- [ ] Documentation compiles: `cargo doc --no-deps`
- [ ] All tests pass: `cargo test -p allframe-core --lib`
- [ ] Examples work: `cargo run --example rest_api`

### allframe-mcp (crates/allframe-mcp/Cargo.toml)

```toml
[package]
name = "allframe-mcp"
version = "0.1.0"
description = "MCP (Model Context Protocol) server for AllFrame - Expose APIs as LLM-callable tools"
keywords = ["mcp", "llm", "ai", "tools", "claude"]
categories = ["development-tools", "api-bindings"]
readme = "README.md"
```

**Checklist**:
- [x] `version = "0.1.0"` - Matches allframe-core
- [x] `description` - Clear MCP focus
- [x] `keywords` - MCP-relevant keywords
- [x] `categories` - Correct categories
- [x] `readme = "README.md"` - Package README exists
- [x] `allframe-core` dependency - Use crates.io version after publishing allframe-core
- [x] All tests pass: `cargo test -p allframe-mcp --lib`
- [x] Examples work: `cargo run --example mcp_stdio_server`

### Documentation

- [x] Root README.md - Up-to-date installation instructions
- [x] crates/allframe-core/README.md - Exists (if different from root)
- [x] crates/allframe-mcp/README.md - Complete usage guide
- [x] CHANGELOG.md - Version 0.1.0 documented (create if missing)
- [x] LICENSE-MIT - Exists
- [x] LICENSE-APACHE - Exists

### Code Quality

```bash
# Clean build
cargo clean

# Format
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Test all features
cargo test -p allframe-core --lib --features="di,openapi,router,router-graphql,router-grpc,router-full,cqrs,otel"
cargo test -p allframe-mcp --lib

# Build release
cargo build -p allframe-core --release
cargo build -p allframe-mcp --release

# Check documentation
cargo doc --no-deps -p allframe-core
cargo doc --no-deps -p allframe-mcp
```

All should pass without warnings!

---

## Publishing Order

**IMPORTANT**: Publish in dependency order!

```
1. allframe-core (no dependencies on allframe crates)
2. allframe-mcp (depends on allframe-core)
```

---

## Step-by-Step Instructions

### Step 1: Prepare Release

```bash
# Ensure you're on main branch
git checkout main
git pull origin main

# Ensure working directory is clean
git status  # Should show no uncommitted changes

# Tag the release
git tag -a v0.1.0 -m "Release v0.1.0: Initial crates.io publication"
```

### Step 2: Dry Run (Recommended!)

```bash
# Test publish without actually publishing
cd crates/allframe-core
cargo publish --dry-run

# Review output carefully
# Check:
# - Package size (should be reasonable, < 10 MB)
# - Files included (README, LICENSE, src/*, etc.)
# - Files excluded (target/, .git/, etc.)

# Test allframe-mcp (will fail because allframe-core not published yet)
cd ../allframe-mcp
cargo publish --dry-run --allow-dirty
```

### Step 3: Publish allframe-core

```bash
cd crates/allframe-core

# Final check
cargo test --lib --features="di,openapi,router,router-graphql,router-grpc,router-full,cqrs,otel"

# Publish!
cargo publish

# Wait for success message
# "Uploading allframe-core v0.1.0"
# "   Uploaded allframe-core v0.1.0 to registry"
```

**Wait 1-2 minutes for crates.io to index the package!**

Verify at: https://crates.io/crates/allframe-core

### Step 4: Update allframe-mcp Dependency

After allframe-core is published, update `allframe-mcp/Cargo.toml`:

```toml
[dependencies]
allframe-core = "0.1.0"  # Change from { path = "../allframe-core" }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tokio = { workspace = true }
```

Test that it works:

```bash
cd crates/allframe-mcp

# Remove Cargo.lock if it exists
rm -f Cargo.lock

# Build with published allframe-core
cargo build

# Test
cargo test --lib
```

### Step 5: Publish allframe-mcp

```bash
cd crates/allframe-mcp

# Final check
cargo test --lib

# Publish!
cargo publish

# Wait for success message
```

Verify at: https://crates.io/crates/allframe-mcp

### Step 6: Push Tags

```bash
# Go back to root
cd ../..

# Push the version tag
git push origin v0.1.0

# Or push all tags
git push --tags
```

---

## Post-Publication

### 1. Verify Published Packages

**allframe-core**:
- Visit https://crates.io/crates/allframe-core
- Check version shows "0.1.0"
- Click "Documentation" → should redirect to docs.rs
- Wait 5-10 minutes for docs.rs to build documentation

**allframe-mcp**:
- Visit https://crates.io/crates/allframe-mcp
- Check version shows "0.1.0"
- Click "Documentation" → should redirect to docs.rs

### 2. Test Installation

Create a new test project:

```bash
mkdir /tmp/test-allframe
cd /tmp/test-allframe

cargo new --bin test-app
cd test-app

# Add to Cargo.toml:
# [dependencies]
# allframe-core = "0.1"
# allframe-mcp = "0.1"
# tokio = { version = "1.48", features = ["full"] }

cargo build
```

Should download and build successfully!

### 3. Update Documentation

**Root README.md**:
```markdown
## Installation

```toml
[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"  # Optional: MCP server support
```
```

**Update badges** (if needed):
```markdown
[![Crates.io](https://img.shields.io/crates/v/allframe-core.svg)](https://crates.io/crates/allframe-core)
[![Documentation](https://docs.rs/allframe-core/badge.svg)](https://docs.rs/allframe-core)
[![Downloads](https://img.shields.io/crates/d/allframe-core.svg)](https://crates.io/crates/allframe-core)
```

### 4. Announcement

Create announcement in `docs/announcements/CRATES_IO_PUBLICATION.md`:

```markdown
# AllFrame v0.1.0 Published to crates.io!

**Date**: 2025-12-04

We're excited to announce that AllFrame v0.1.0 is now available on crates.io!

## Packages Published

- **allframe-core** - Protocol-agnostic web framework
  - https://crates.io/crates/allframe-core
  - https://docs.rs/allframe-core

- **allframe-mcp** - MCP server for LLM integration
  - https://crates.io/crates/allframe-mcp
  - https://docs.rs/allframe-mcp

## Installation

```toml
[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"
```

## What's Included

- ✅ Protocol-agnostic routing (REST, GraphQL, gRPC)
- ✅ CQRS + Event Sourcing infrastructure
- ✅ OpenAPI, GraphQL, gRPC documentation
- ✅ Native MCP server (separate crate, zero bloat!)
- ✅ 291+ tests passing

## Get Started

Visit https://docs.rs/allframe-core for complete documentation!
```

### 5. Social Media

Post announcement to:
- Twitter/X
- LinkedIn
- Reddit r/rust
- Hacker News (Show HN)
- Dev.to

---

## Troubleshooting

### Error: "the package 'allframe-core' cannot be published"

**Cause**: Package name already taken or reserved.

**Solution**:
- Choose a different name (e.g., `allframe-web`)
- Or request name transfer if you own the project

### Error: "failed to select a version for 'allframe-core'"

**Cause**: Circular dependency or unpublished local dependency.

**Solution**:
- Publish dependencies in order (allframe-core first)
- Ensure no circular dependencies in Cargo.toml

### Error: "crate size limit exceeded"

**Cause**: Package > 10 MB (crates.io limit).

**Solution**:
```toml
# Add to Cargo.toml
[package]
exclude = [
    "target/",
    "examples/large_files/",
    "*.mp4",
    "*.png"
]
```

### Error: "documentation failed to build on docs.rs"

**Cause**: Missing features or dependencies for docs.

**Solution**:
```toml
# Add to Cargo.toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### Yanking a Published Version

If you published a broken version:

```bash
# Yank (makes version unavailable for new users)
cargo yank --vers 0.1.0

# Unyank (if you change your mind)
cargo yank --vers 0.1.0 --undo
```

**Note**: Yanking doesn't delete the version, just marks it as "do not use".

---

## Automated Publishing (CI/CD)

### GitHub Actions Workflow

Create `.github/workflows/publish.yml`:

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'  # Trigger on version tags like v0.1.0

permissions:
  contents: read

jobs:
  publish:
    name: Publish to crates.io
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Publish allframe-core
        run: |
          cd crates/allframe-core
          cargo publish --token ${{ secrets.CARGO_TOKEN }}

      - name: Wait for allframe-core to be indexed
        run: sleep 120  # Wait 2 minutes

      - name: Update allframe-mcp dependency
        run: |
          cd crates/allframe-mcp
          # Update Cargo.toml to use crates.io version
          sed -i 's|allframe-core = { path = "../allframe-core" }|allframe-core = "0.1"|g' Cargo.toml

      - name: Publish allframe-mcp
        run: |
          cd crates/allframe-mcp
          cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

### Setup GitHub Secret

1. Go to GitHub repository → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `CARGO_TOKEN`
4. Value: Your crates.io API token
5. Click "Add secret"

### Test Workflow

```bash
# Create and push a test tag
git tag v0.1.0-test
git push origin v0.1.0-test

# Monitor GitHub Actions
# Workflow should trigger and run (but will fail on publish unless you want to actually publish)
```

---

## Version Management

### Semantic Versioning

AllFrame follows [Semver 2.0.0](https://semver.org):

- **MAJOR** (0.x.0): Breaking changes
- **MINOR** (0.1.x): New features, backwards compatible
- **PATCH** (0.1.1): Bug fixes, backwards compatible

### Version Bump Process

```bash
# Update version in Cargo.toml files
# crates/allframe-core/Cargo.toml
# crates/allframe-mcp/Cargo.toml

# Update CHANGELOG.md with version changes

# Commit changes
git add .
git commit -m "Bump version to 0.1.1"

# Tag release
git tag -a v0.1.1 -m "Release v0.1.1"

# Push
git push origin main
git push origin v0.1.1
```

---

## Checklist Summary

Before publishing:
- [ ] All tests pass
- [ ] Documentation builds
- [ ] Cargo.toml metadata complete
- [ ] README files up-to-date
- [ ] LICENSE files present
- [ ] Working directory clean
- [ ] On main branch
- [ ] Version tagged

During publishing:
- [ ] Run `cargo publish --dry-run` first
- [ ] Publish in dependency order (allframe-core → allframe-mcp)
- [ ] Wait for crates.io to index between publishes
- [ ] Verify each package on crates.io

After publishing:
- [ ] Test installation in fresh project
- [ ] Verify docs.rs builds documentation
- [ ] Update README badges
- [ ] Create announcement
- [ ] Social media posts
- [ ] Push version tag to GitHub

---

## Support

If you encounter issues:
1. Check https://doc.rust-lang.org/cargo/reference/publishing.html
2. Ask on crates.io Discord: https://discord.gg/rust-lang
3. File an issue: https://github.com/all-source-os/all-frame/issues

---

**Status**: Ready for v0.1.0 publication
**Owner**: @all-source-os
**Last Updated**: 2025-12-04
