# Dependency Management - Best Practices

**Last Updated**: 2025-12-04
**Policy Version**: 1.0

---

## Philosophy

AllFrame follows a **deliberate dependency management** approach:

1. **Pin exact versions** - Use exact version specifications, not ranges
2. **Document before upgrading** - Create upgrade plans before making changes
3. **Test thoroughly** - Verify all features and platforms
4. **Track changes** - Maintain upgrade history

This ensures stability, reproducibility, and conscious decisions about our dependency tree.

---

## Table of Contents

- [Version Specification Policy](#version-specification-policy)
- [Upgrade Process](#upgrade-process)
- [Pre-Upgrade Checklist](#pre-upgrade-checklist)
- [Testing Requirements](#testing-requirements)
- [Documentation Requirements](#documentation-requirements)
- [Cargo.toml Hygiene](#cargotoml-hygiene)
- [Examples](#examples)

---

## Version Specification Policy

### ✅ DO: Use Exact Minor Versions

```toml
# GOOD - Exact minor version, allows patch updates only
tokio = "1.48"
hyper = "1.8"
serde = "1.0"
```

**Why**: Patch updates (1.48.0 → 1.48.1) are typically bug fixes and safe. Minor updates (1.48 → 1.49) may introduce new APIs or subtle changes.

### ❌ DON'T: Use Version Ranges or Wildcards

```toml
# BAD - Too permissive
tokio = "1"      # Allows 1.0 → 1.999
hyper = "*"      # Allows any version
serde = "^1.0"   # Caret ranges are implicit but not obvious
```

### ⚠️ EXCEPTION: Major Version Only for Stable APIs

```toml
# ACCEPTABLE - For very stable, mature crates
thiserror = "1.0"
async-trait = "0.1"
```

**Why**: Some crates like `thiserror` have very stable APIs and rarely introduce breaking changes within a major version.

### 🔒 STRICT: Use Exact Versions for Critical Dependencies

```toml
# BEST - For security-critical or frequently-breaking crates
anyhow = "1.0.75"  # Exact version with rationale in comment
```

---

## Upgrade Process

### Step 1: Plan the Upgrade

Create an upgrade plan document before making any changes.

**Template**: `docs/upgrade-plans/YYYY-MM-DD-dependency-upgrade.md`

```markdown
# Dependency Upgrade Plan - [Date]

## Motivation
Why are we upgrading? (security, new features, compatibility)

## Dependencies to Upgrade

| Package | Current | Target | Reason |
|---------|---------|--------|--------|
| tokio   | 1.35    | 1.48   | Performance improvements |
| hyper   | 1.1     | 1.8    | HTTP/2 fixes |

## Risk Assessment

- **Low Risk**: Patch updates
- **Medium Risk**: Minor updates with changelog review
- **High Risk**: Major updates or ecosystem changes

## Testing Plan

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] CI matrix passes (all Rust versions, platforms)
- [ ] Performance benchmarks (if applicable)
- [ ] Manual testing of key features

## Rollback Plan

How to revert if issues are discovered:
1. Revert commit: `git revert <commit-hash>`
2. Restore Cargo.lock: `git checkout HEAD~1 Cargo.lock`
```

### Step 2: Research the Changes

Before upgrading, review:

1. **Changelogs** - Read CHANGELOG.md or GitHub releases
2. **Breaking Changes** - Look for migration guides
3. **Security Advisories** - Check RustSec database
4. **Community Feedback** - Search GitHub issues for reported problems

```bash
# Check for security advisories
cargo audit

# View outdated dependencies
cargo outdated
```

### Step 3: Create Upgrade Branch

```bash
# Create feature branch
git checkout -b deps/upgrade-YYYY-MM-DD

# Document the plan
vim docs/upgrade-plans/YYYY-MM-DD-dependency-upgrade.md
git add docs/upgrade-plans/
git commit -m "docs: Add dependency upgrade plan for YYYY-MM-DD"
```

### Step 4: Update Cargo.toml

Update version specifications in `Cargo.toml` (workspace root):

```toml
[workspace.dependencies]
# Updated: 2025-12-04 - Performance and security fixes
tokio = "1.48"  # Was: 1.35
hyper = "1.8"   # Was: 1.1
```

**Always add comments** explaining why a specific version is pinned:

```toml
anyhow = "1.0.75"  # Minimum version for Rust 1.80 compatibility
async-graphql = "7.0"  # Requires Rust 1.86+ for edition2024
```

### Step 5: Update Cargo.lock

```bash
# Update all dependencies
cargo update

# Or update specific packages
cargo update -p tokio
cargo update -p hyper

# Review the changes
git diff Cargo.lock
```

### Step 6: Build and Test

```bash
# Clean build
cargo clean

# Build all workspace members
cargo build --all

# Build with all features
cargo build -p allframe-core --all-features

# Run all tests
cargo test --all

# Run tests with specific features
cargo test -p allframe-core --features="di,openapi,router,cqrs,otel"

# Check for compilation warnings
cargo clippy --all -- -D warnings

# Format check
cargo fmt --all -- --check
```

### Step 7: Document the Upgrade

Create a summary document: `docs/DEPENDENCY_UPDATES_[MONTH]_[YEAR].md`

**Template**:

```markdown
# Dependency Updates - [Month] [Year]

**Date**: YYYY-MM-DD
**Status**: ✅ Complete
**Tests**: [X] passing

## Summary

Brief description of what was updated and why.

## Updated Dependencies

| Package | Old | New | Notes |
|---------|-----|-----|-------|
| tokio   | 1.35 | 1.48 | Performance improvements |

## Breaking Changes

List any breaking changes and migration steps.

## Test Results

```
allframe-core: X tests passing
allframe-mcp: Y tests passing
Total: Z tests passing
```

## Compatibility Matrix

- ✅ Rust stable
- ✅ Rust 1.86.0 (MSRV)
- ✅ All platforms
```

### Step 8: Commit Changes

```bash
# Stage changes
git add Cargo.toml Cargo.lock docs/

# Commit with detailed message
git commit -m "deps: Update dependencies - December 2025

Updated workspace dependencies to latest compatible versions:
- tokio: 1.35 → 1.48
- hyper: 1.1 → 1.8
- http: 1.3.1 → 1.4.0
- predicates: 3.0 → 3.1
- tempfile: 3.8 → 3.15
- proptest: 1.4 → 1.6
- mockall: 0.12 → 0.13
- clap: 4.4 → 4.5

All 291+ tests passing.

See docs/DEPENDENCY_UPDATES_DEC_2025.md for details."
```

---

## Pre-Upgrade Checklist

Before updating dependencies, verify:

- [ ] **Motivation is clear** - Why are we upgrading?
- [ ] **Upgrade plan documented** - Created plan document
- [ ] **Changelogs reviewed** - Read release notes for all updated packages
- [ ] **Breaking changes identified** - Migration steps documented
- [ ] **Security advisories checked** - No known vulnerabilities
- [ ] **CI is green** - All tests passing before upgrade
- [ ] **Rollback plan ready** - Know how to revert if needed

---

## Testing Requirements

### Minimum Testing (Required for All Upgrades)

```bash
# 1. Clean build
cargo clean
cargo build --all

# 2. Run all tests
cargo test --all

# 3. Check clippy
cargo clippy --all -- -D warnings

# 4. Format check
cargo fmt --all -- --check
```

### Comprehensive Testing (Required for Major/Minor Upgrades)

```bash
# 1. Test all feature combinations
cargo test -p allframe-core --no-default-features
cargo test -p allframe-core --features="di"
cargo test -p allframe-core --features="openapi"
cargo test -p allframe-core --features="router"
cargo test -p allframe-core --features="router-graphql"
cargo test -p allframe-core --features="router-grpc"
cargo test -p allframe-core --all-features

# 2. Test on MSRV
rustup install 1.86.0
cargo +1.86.0 build --all
cargo +1.86.0 test --all

# 3. Platform-specific testing (if relevant)
# Run CI pipeline or test on Linux, macOS, Windows

# 4. Performance benchmarks (if available)
cargo bench
```

---

## Documentation Requirements

### Required Documentation

Every dependency upgrade must include:

1. **Upgrade Plan** (`docs/upgrade-plans/`)
   - What, why, risk assessment, testing plan

2. **Upgrade Summary** (`docs/DEPENDENCY_UPDATES_[MONTH]_[YEAR].md`)
   - Version changes, breaking changes, test results

3. **Updated Comments in Cargo.toml**
   - Rationale for specific version pins

4. **Commit Message**
   - Clear summary of changes
   - Link to upgrade summary document

### Optional Documentation

For major upgrades, consider:

- **Migration Guide** - If breaking changes affect users
- **Performance Analysis** - If performance-critical deps updated
- **Security Analysis** - If security-related updates

---

## Cargo.toml Hygiene

### Structure and Organization

```toml
[workspace.dependencies]
# === Async Runtime ===
# Updated: 2025-12-04 - Latest stable for performance
tokio = "1.48"

# === HTTP Server ===
# Updated: 2025-12-04 - HTTP/2 improvements
hyper = "1.8"

# === Core Utilities ===
async-trait = "0.1"  # Stable, minor updates safe
thiserror = "1.0"     # Stable, minor updates safe
serde = "1.0"         # Stable, minor updates safe

# === Testing ===
# Updated: 2025-12-04 - Latest test utilities
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.15"
mockall = "0.13"

# === Critical Dependencies ===
# Pinned to exact version - DO NOT UPDATE without thorough testing
anyhow = "1.0.75"  # Minimum for Rust 1.80+ compatibility
```

### Comments Guidelines

**DO add comments for:**
- Exact version pins (with rationale)
- MSRV-related constraints
- Security-related pins
- Unusual version choices
- Last update date for groups of deps

**DON'T add comments for:**
- Self-explanatory standard versions
- Every single dependency (too verbose)

### Workspace vs Package Dependencies

**Workspace dependencies** (`Cargo.toml` root):
```toml
[workspace.dependencies]
tokio = "1.48"
serde = "1.0"
```

**Package dependencies** (crate `Cargo.toml`):
```toml
[dependencies]
tokio = { workspace = true }
serde = { workspace = true, features = ["derive"] }
```

**Benefits:**
- Single source of truth for versions
- Easier to audit and update
- Prevents version conflicts

---

## Examples

### Example 1: Security Patch Update

```bash
# 1. Check advisory
cargo audit
# Output: vulnerability in `somelib` 1.2.3, fixed in 1.2.4

# 2. Document
echo "Security update for somelib 1.2.3 → 1.2.4" > docs/upgrade-plans/2025-12-05-security-patch.md

# 3. Update
cargo update -p somelib

# 4. Test
cargo test --all

# 5. Commit
git commit -m "deps: Security patch for somelib 1.2.3 → 1.2.4

Fixes CVE-YYYY-XXXXX

See: https://rustsec.org/advisories/..."
```

### Example 2: Planned Minor Update

```bash
# 1. Create plan
cat > docs/upgrade-plans/2025-12-15-quarterly-update.md <<EOF
# Q4 2025 Dependency Updates

## Scope
Update all dependencies to latest minor versions

## Testing
- All unit tests
- CI matrix
- Manual feature testing
EOF

# 2. Update Cargo.toml versions
vim Cargo.toml
# Change tokio = "1.35" to tokio = "1.48"

# 3. Update lockfile
cargo update

# 4. Test thoroughly
cargo test --all
cargo +1.86.0 test --all

# 5. Document results
cat > docs/DEPENDENCY_UPDATES_DEC_2025.md <<EOF
# Dependency Updates - December 2025
...
EOF

# 6. Commit
git commit -m "deps: Q4 2025 dependency updates

See docs/DEPENDENCY_UPDATES_DEC_2025.md"
```

### Example 3: Major Version Upgrade

```bash
# 1. Create detailed plan
cat > docs/upgrade-plans/2026-01-01-tokio-2.0.md <<EOF
# Tokio 2.0 Upgrade Plan

## Breaking Changes
- API change X
- API change Y

## Migration Steps
1. Update trait bounds
2. Replace deprecated APIs

## Testing Plan
- Full CI matrix
- Performance benchmarks
- Manual testing
EOF

# 2. Create feature branch
git checkout -b deps/tokio-2.0

# 3. Update and migrate code
# ... make necessary code changes ...

# 4. Comprehensive testing
cargo test --all
cargo bench

# 5. Document thoroughly
# Create migration guide if needed

# 6. PR for review
```

---

## Cargo Commands Reference

### Check for Updates

```bash
# Check for outdated dependencies
cargo outdated

# Check for security advisories
cargo audit

# Show dependency tree
cargo tree

# Show duplicate dependencies
cargo tree --duplicates
```

### Update Dependencies

```bash
# Update all dependencies
cargo update

# Update specific package
cargo update -p tokio

# Update to specific version
cargo update -p tokio --precise 1.48.0

# Update with minimal versions (testing)
cargo +nightly update -Z minimal-versions
```

### Verify Dependencies

```bash
# Generate lockfile without updating
cargo generate-lockfile

# Verify lockfile is up-to-date
cargo verify-project

# Show why a dependency is included
cargo tree -i tokio

# Check license compatibility
cargo license
```

---

## Emergency Rollback Procedure

If an upgrade causes critical issues:

### Step 1: Immediate Revert

```bash
# Revert the commit
git revert <upgrade-commit-hash>

# Or reset to previous state
git reset --hard HEAD~1

# Restore Cargo.lock
git checkout HEAD~1 Cargo.lock
```

### Step 2: Verify Rollback

```bash
# Clean build
cargo clean

# Verify old versions restored
cargo tree | grep problematic-package

# Run tests
cargo test --all
```

### Step 3: Document Issue

Create incident report: `docs/incidents/YYYY-MM-DD-rollback.md`

```markdown
# Rollback Incident - [Date]

## What Happened
Upgraded package X from A to B caused issue Y

## Impact
- Severity: [Low/Medium/High/Critical]
- Affected systems: ...
- Duration: ...

## Resolution
Reverted to previous version

## Prevention
- Better testing needed for Z
- Add test case for Y scenario
```

---

## Automation (Future)

Consider these automation tools:

- **Dependabot**: Automated PR creation for updates
- **Renovate**: More configurable than Dependabot
- **cargo-deny**: Policy enforcement for licenses and advisories
- **cargo-audit**: Security scanning in CI

**Note**: Even with automation, maintain human oversight and follow this policy!

---

## Policy Violations

### What NOT to Do

❌ **Don't** update dependencies without documentation
❌ **Don't** use wildcard versions (`*`)
❌ **Don't** commit Cargo.lock changes without Cargo.toml changes
❌ **Don't** skip testing after updates
❌ **Don't** update during feature development (separate PRs)
❌ **Don't** update before a release without buffer time

### Enforcement

- PRs with dependency changes require upgrade summary doc
- CI checks for version range violations (future)
- Monthly dependency audit review

---

## Review Schedule

### Monthly Review
- Check `cargo audit` for security issues
- Review `cargo outdated` for major updates
- Quick patch updates if needed

### Quarterly Review
- Plan minor version updates
- Review dependency tree for bloat
- Update this policy document if needed

### Annual Review
- Consider major version upgrades
- Audit all dependencies for necessity
- Update MSRV if needed

---

## Questions?

**For dependency upgrade questions:**
- Check this document first
- Review previous upgrade docs in `docs/`
- Create an issue for policy clarification

**For security issues:**
- Run `cargo audit` immediately
- Create security upgrade plan
- Prioritize over feature work

---

## Changelog

### v1.0 - 2025-12-04
- Initial policy document
- Established pinned version requirement
- Created upgrade process workflow
- Added documentation templates

---

**Remember**: Dependencies are long-term commitments. Choose deliberately, upgrade consciously, test thoroughly.
