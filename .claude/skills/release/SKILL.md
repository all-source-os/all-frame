---
name: release
description: "AllFrame comprehensive release workflow: version bumping, quality gates, CHANGELOG, git tag, GitHub release, crates.io publishing, and issue closure. Use when the user says 'release', 'publish', 'cut a release', 'bump version', 'publish crates', or wants to ship a new version of AllFrame to crates.io."
---

# AllFrame Release

Execute all steps in order. Stop and report if any quality gate fails.

## Step 1: Pre-flight Checks

Verify clean state before anything else:

```
git status            # must be clean (no uncommitted changes)
git pull              # must be up to date with remote
```

If there are uncommitted changes, ask the user whether to commit or stash them first.

## Step 2: Quality Gates

Run these checks. All must pass before proceeding.

### 2a. Workspace build

```
cargo check --workspace
```

### 2b. Tests — changed crates

Run tests for all workspace crates. Focus on crates that changed since last tag:

```
git diff $(git describe --tags --abbrev=0)..HEAD --name-only | grep '^crates/' | cut -d/ -f2 | sort -u
```

Then `cargo test -p <crate>` for each. At minimum always run:

```
cargo test -p allframe-core -p allframe-macros -p allframe-tauri
```

### 2c. Version sync

Run `bash .claude/skills/release/scripts/check_version_sync.sh` to verify all version strings in root `Cargo.toml` are in sync.

### 2d. Clippy

```
cargo clippy --workspace -- -D warnings
```

If clippy or tests fail, fix the issues before continuing. Do not skip.

## Step 3: Determine New Version

Read the current version from `[workspace.package] version` in root `Cargo.toml`.

Ask the user what kind of release this is:
- **patch** (0.1.X → 0.1.X+1) — bug fixes, no API changes
- **minor** (0.X.0 → 0.X+1.0) — new features, backward compatible
- **major** (X.0.0 → X+1.0.0) — breaking changes

Or the user can specify an exact version.

## Step 4: Bump Version

Update **all 6 version strings** in root `Cargo.toml`. See [references/crate-graph.md](references/crate-graph.md) for exact locations.

Use the Edit tool to change each occurrence of the old version to the new version:
- `[package]` → `version`
- `[workspace.package]` → `version`
- `[workspace.dependencies]` → `allframe-core`, `allframe-forge`, `allframe-macros`, `allframe-tauri`

Re-run `bash .claude/skills/release/scripts/check_version_sync.sh <new_version>` to confirm.

## Step 5: Update CHANGELOG

Prepend a new entry to `CHANGELOG.md` following Keep a Changelog format. See [references/crate-graph.md](references/crate-graph.md) for the entry structure.

Populate the entry by analyzing `git log $(git describe --tags --abbrev=0)..HEAD --oneline`. Categorize changes into Fixed/Added/Changed/Removed/Migration/Documentation sections.

Include a `### Migration` section if there are breaking changes to APIs, event names, frontend invocation patterns, or capability identifiers.

## Step 6: Commit, Tag, Push

```
git add Cargo.toml CHANGELOG.md
# also stage any other files changed during quality gate fixes
git commit -m "release: vX.Y.Z"
git tag -a vX.Y.Z -m "vX.Y.Z: <one-line summary>"
git push origin main --tags
```

## Step 7: Create GitHub Release

```
gh release create vX.Y.Z --title "vX.Y.Z" --notes "<release notes>"
```

Use the CHANGELOG entry as the release notes body. Include a link to the full CHANGELOG for migration details.

## Step 8: Publish to crates.io

Publish in dependency order (leaves first). Wait for each to be available before publishing dependents:

1. `cargo publish -p allframe-macros`
2. `cargo publish -p allframe-forge`
3. `cargo publish -p allframe-core`
4. `cargo publish -p allframe-tauri`
5. `cargo publish -p allframe`

If any publish fails, diagnose and fix before continuing. Common issues:
- Version already exists → the version was not bumped
- Dependency not found → previous crate not yet indexed, wait a moment and retry

## Step 9: Close Issues & Notify

For each GitHub issue referenced in the CHANGELOG:

1. Post a comment with:
   - Version number and link to release
   - Migration instructions (if applicable)
   - Code examples showing the fix or new API
2. Close the issue (if it was fully resolved)

```
gh issue comment <number> --body "<comment>"
gh issue close <number>
```

## Step 10: Post-release Verification

Verify the release is live:

```
gh release view vX.Y.Z
```

Check crates.io (wait ~60s for indexing):

```
cargo search allframe --limit 1
```

Report the final status to the user with links to the release and published crates.

## Resources

### scripts/

- **check_version_sync.sh** — Verify all version strings in root Cargo.toml match. Run before and after version bump.

### references/

- **[crate-graph.md](references/crate-graph.md)** — Crate dependency graph, publish order, version locations, and CHANGELOG format.
