# Changesets for Rust - Implementation Plan

**Date**: 2025-12-04
**Inspired by**: [@changesets/cli](https://github.com/changesets/changesets)
**Goal**: Automated changelog generation and version management for Rust workspaces

---

## Overview

A changeset system for Rust that handles:
1. **Changeset creation** - Developers describe changes with semantic version bump
2. **Changelog generation** - Automated CHANGELOG.md per crate
3. **Version bumping** - Automatic Cargo.toml version updates
4. **Pre-publishing validation** - Ensure all crates ready for publish
5. **Dependency resolution** - Handle workspace dependency updates

---

## Architecture

```
.changeset/
├── config.json                 # Configuration
├── README.md                   # Usage instructions
└── changes/                    # Pending changesets
    ├── happy-pandas-jump.md    # Changeset 1
    └── brave-lions-roar.md     # Changeset 2

scripts/
├── changeset                   # CLI tool (Rust binary)
├── changeset-add               # Create new changeset
├── changeset-version           # Bump versions + update CHANGELOGs
└── changeset-publish           # Publish to crates.io
```

---

## Configuration

**.changeset/config.json**:
```json
{
  "changelog": {
    "repo": "all-source-os/all-frame",
    "cwd": ".",
    "prerelease": false
  },
  "commit": false,
  "linked": [],
  "access": "public",
  "baseBranch": "main",
  "updateInternalDependencies": "patch",
  "ignore": []
}
```

---

## Changeset File Format

**.changeset/happy-pandas-jump.md**:
```markdown
---
"allframe-core": minor
"allframe-mcp": minor
---

Add native MCP server support

- Implement McpServer with auto-discovery
- Add JSON Schema generation and validation
- Add type coercion utilities
- Support for Claude Desktop integration
```

**Format**:
- YAML frontmatter with crate names and bump types (major, minor, patch)
- Markdown description of changes
- Auto-generated filename (random adjective-noun-verb)

---

## CLI Commands

### 1. `changeset add` - Create Changeset

```bash
$ changeset add

🦋  Which packages would you like to include?
  [x] allframe-core
  [x] allframe-mcp
  [ ] allframe-forge
  [ ] allframe-macros

🦋  Which packages should have a major bump?
  [ ] allframe-core
  [ ] allframe-mcp

🦋  Which packages should have a minor bump?
  [x] allframe-core
  [x] allframe-mcp

🦋  Please enter a summary for this change:
Add native MCP server support

...

🦋  Changeset added! - .changeset/happy-pandas-jump.md
```

**Implementation**:
```rust
// scripts/changeset-add/src/main.rs
use clap::Parser;
use dialoguer::{MultiSelect, Select, Input};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    empty: bool,  // Skip prompts
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Find all workspace crates
    let crates = discover_workspace_crates()?;

    if !cli.empty {
        // Interactive prompts
        let selected_crates = prompt_crates(&crates)?;
        let bump_types = prompt_bump_types(&selected_crates)?;
        let summary = prompt_summary()?;

        // Create changeset file
        let filename = generate_random_filename();
        write_changeset(&filename, &selected_crates, &bump_types, &summary)?;

        println!("🦋  Changeset added! - .changeset/{}.md", filename);
    } else {
        // Empty changeset (for CI)
        println!("🦋  Empty changeset created");
    }

    Ok(())
}
```

### 2. `changeset version` - Bump Versions

```bash
$ changeset version

🦋  Applying changesets...
  ✓ allframe-core@0.1.0 → 0.2.0
  ✓ allframe-mcp@0.1.0 → 0.2.0
  ✓ Updated workspace dependencies

🦋  Changesets applied!
  2 changesets processed
  2 crates versioned
  CHANGELOGs updated
```

**What it does**:
1. Read all changesets from `.changeset/changes/`
2. Calculate version bumps (major > minor > patch)
3. Update `Cargo.toml` versions
4. Update workspace dependency versions
5. Generate/update CHANGELOG.md per crate
6. Delete processed changesets
7. Git commit (optional)

**Implementation**:
```rust
// scripts/changeset-version/src/main.rs
use semver::Version;
use toml_edit::Document;

fn main() -> Result<()> {
    // Read all changesets
    let changesets = read_changesets(".changeset/changes")?;

    // Calculate version bumps
    let bumps = calculate_bumps(&changesets)?;

    // Update Cargo.toml files
    for (crate_name, new_version) in &bumps {
        update_cargo_toml(crate_name, new_version)?;
    }

    // Update workspace dependencies
    update_workspace_dependencies(&bumps)?;

    // Generate CHANGELOGs
    for (crate_name, _) in &bumps {
        update_changelog(crate_name, &changesets)?;
    }

    // Delete processed changesets
    for changeset in &changesets {
        fs::remove_file(&changeset.path)?;
    }

    println!("🦋  Changesets applied!");
    Ok(())
}
```

### 3. `changeset publish` - Publish to crates.io

```bash
$ changeset publish

🦋  Publishing packages...
  ✓ allframe-macros@0.2.0
    Published to crates.io
  ⏳ Waiting 120s for indexing...
  ✓ allframe-core@0.2.0
    Published to crates.io
  ⏳ Waiting 120s for indexing...
  ✓ allframe-mcp@0.2.0
    Published to crates.io

🦋  Published 3 packages!
```

**What it does**:
1. Determine publishing order (dependency graph)
2. Run `cargo publish` for each crate
3. Wait for crates.io indexing between publishes
4. Create git tags
5. Push to GitHub

**Implementation**:
```rust
// scripts/changeset-publish/src/main.rs
fn main() -> Result<()> {
    // Get publishing order
    let order = topological_sort_crates()?;

    for crate_name in order {
        let crate_path = get_crate_path(&crate_name)?;
        let version = get_crate_version(&crate_path)?;

        // Publish
        println!("🦋  Publishing {crate_name}@{version}...");
        Command::new("cargo")
            .args(["publish"])
            .current_dir(&crate_path)
            .status()?;

        // Wait for indexing
        println!("⏳  Waiting 120s for indexing...");
        thread::sleep(Duration::from_secs(120));
    }

    // Create git tags
    for (crate_name, version) in &published {
        let tag = format!("{crate_name}-v{version}");
        Command::new("git")
            .args(["tag", "-a", &tag, "-m", &format!("Release {crate_name} v{version}")])
            .status()?;
    }

    println!("🦋  Published {} packages!", published.len());
    Ok(())
}
```

---

## CHANGELOG Format

**crates/allframe-core/CHANGELOG.md**:
```markdown
# Changelog

All notable changes to allframe-core will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-05

### Added

- Add native MCP server support ([#123](https://github.com/all-source-os/all-frame/pull/123))
  - Implement McpServer with auto-discovery
  - Add JSON Schema generation and validation
  - Add type coercion utilities
  - Support for Claude Desktop integration

### Changed

- Update router to support protocol-agnostic handlers ([#120](https://github.com/all-source-os/all-frame/pull/120))

### Fixed

- Fix GraphQL schema generation for nested types ([#118](https://github.com/all-source-os/all-frame/pull/118))

## [0.1.0] - 2025-12-04

Initial release.
```

---

## Workflow

### Developer Flow

```bash
# 1. Make changes to code
$ git checkout -b add-mcp-support

# 2. Add changeset describing changes
$ changeset add
# Interactive prompts for packages, bump type, summary

# 3. Commit changeset
$ git add .changeset/happy-pandas-jump.md
$ git commit -m "Add changeset for MCP support"

# 4. Push PR
$ git push origin add-mcp-support
```

### Maintainer Flow (Release)

```bash
# 1. Bump versions and update CHANGELOGs
$ changeset version
# Updates Cargo.toml, CHANGELOG.md, deletes changesets

# 2. Review changes
$ git diff

# 3. Commit version updates
$ git add .
$ git commit -m "Version packages"

# 4. Publish to crates.io
$ changeset publish
# Publishes in dependency order, creates tags

# 5. Push tags
$ git push --tags
```

---

## Implementation Roadmap

### Phase 1: Basic Changeset System

**Goal**: Create and read changesets

```bash
cargo new --bin scripts/changeset-add
cargo new --bin scripts/changeset-version
cargo new --lib scripts/changeset-core
```

**Dependencies**:
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
dialoguer = "0.11"
toml_edit = "0.22"
semver = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
walkdir = "2.4"
petgraph = "0.6"  # For dependency graph
```

**Files**:
- `scripts/changeset-core/src/lib.rs` - Shared utilities
- `scripts/changeset-add/src/main.rs` - Create changesets
- `scripts/changeset-version/src/main.rs` - Bump versions

### Phase 2: CHANGELOG Generation

**Goal**: Auto-generate CHANGELOG.md

**Features**:
- Parse changesets
- Group by category (Added, Changed, Fixed, etc.)
- Link to PRs/commits
- Follow Keep a Changelog format

### Phase 3: Publishing

**Goal**: Automated publishing to crates.io

**Features**:
- Dependency-order publishing
- Wait for crates.io indexing
- Git tag creation
- Rollback on failure

### Phase 4: CI Integration

**Goal**: GitHub Actions workflows

**Workflows**:
1. **PR Changeset Check** - Ensure changeset added
2. **Release** - Trigger on changeset version commit
3. **Snapshot Releases** - Pre-release versions

---

## File Structure After Implementation

```
.changeset/
├── config.json
├── README.md
└── changes/              # Pending changesets (gitignored after version)

scripts/
├── changeset-core/       # Shared library
│   └── src/
│       ├── lib.rs
│       ├── workspace.rs  # Workspace discovery
│       ├── version.rs    # Version bumping
│       ├── changelog.rs  # CHANGELOG generation
│       └── publish.rs    # Publishing logic
├── changeset-add/        # CLI: Add changeset
│   └── src/main.rs
├── changeset-version/    # CLI: Bump versions
│   └── src/main.rs
└── changeset-publish/    # CLI: Publish crates
    └── src/main.rs

crates/
├── allframe-core/
│   ├── CHANGELOG.md      # Auto-generated
│   └── Cargo.toml
├── allframe-mcp/
│   ├── CHANGELOG.md      # Auto-generated
│   └── Cargo.toml
└── ...

.github/
└── workflows/
    ├── changeset-check.yml    # PR validation
    └── release.yml            # Auto-publish
```

---

## Example GitHub Actions

**.github/workflows/changeset-check.yml**:
```yaml
name: Changeset Check

on:
  pull_request:
    types: [opened, synchronize, reopened]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Check for changesets
        run: |
          # Check if .changeset/changes/ has files
          if [ -z "$(ls -A .changeset/changes 2>/dev/null)" ]; then
            echo "❌ No changeset found!"
            echo "Please run 'changeset add' to document your changes"
            exit 1
          fi
          echo "✓ Changeset found"
```

**.github/workflows/release.yml**:
```yaml
name: Release

on:
  push:
    branches:
      - main
    paths:
      - 'crates/*/Cargo.toml'
      - 'CHANGELOG.md'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish to crates.io
        run: cargo run --bin changeset-publish
        env:
          CARGO_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

---

## Benefits

1. **Consistency** - All changes documented uniformly
2. **Automation** - Version bumping and CHANGELOG generation automated
3. **Safety** - Enforces semantic versioning
4. **Transparency** - Clear changelog for users
5. **Workspace-aware** - Handles complex dependency graphs

---

## Next Steps

1. ✅ Create `.changeset/config.json`
2. 📋 Implement `changeset-core` library
3. 📋 Implement `changeset-add` CLI
4. 📋 Implement `changeset-version` CLI
5. 📋 Implement `changeset-publish` CLI
6. 📋 Add GitHub Actions workflows
7. 📋 Document in CONTRIBUTING.md

---

## Comparison to @changesets/cli

| Feature | @changesets/cli (JS) | Rust Implementation |
|---------|---------------------|---------------------|
| Interactive CLI | ✅ | ✅ |
| Version bumping | ✅ | ✅ |
| CHANGELOG generation | ✅ | ✅ |
| Publishing | ✅ npm | ✅ crates.io |
| Workspace support | ✅ | ✅ |
| Prerelease | ✅ | 📋 Planned |
| Snapshot releases | ✅ | 📋 Planned |

---

**Status**: Design complete, ready for implementation
**Owner**: @all-source-os
**Last Updated**: 2025-12-04
