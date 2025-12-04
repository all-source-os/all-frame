# Dependency Upgrade Plans

This directory contains upgrade plans for all dependency updates. Each plan documents the rationale, testing, and results of dependency upgrades.

---

## Purpose

Dependency upgrade plans ensure:
- **Deliberate decisions** - Updates are conscious, not automatic
- **Risk assessment** - Potential issues identified before upgrade
- **Testing coverage** - Thorough validation of changes
- **Rollback capability** - Clear path to revert if needed
- **Knowledge transfer** - Future maintainers understand past decisions

---

## Creating an Upgrade Plan

### 1. Copy the Template

```bash
cp TEMPLATE.md $(date +%Y-%m-%d)-your-update-name.md
```

### 2. Fill Out the Plan

Complete all sections:
- **Motivation**: Why are we upgrading?
- **Dependencies**: What's being updated?
- **Risk Assessment**: What could go wrong?
- **Testing Plan**: How will we verify?
- **Rollback Plan**: How to revert?

### 3. Get Approval (if needed)

For high-risk updates:
- Share plan with team
- Get reviewer approval
- Address concerns

### 4. Execute the Plan

Follow the testing plan step by step:
- Update dependencies
- Build and test
- Document results
- Commit changes

### 5. Document Results

Update the plan with:
- Actual test results
- Issues encountered
- Lessons learned
- Sign-off status

---

## Plan Structure

### Filename Format

```
YYYY-MM-DD-brief-description.md
```

Examples:
- `2025-12-04-december-updates.md`
- `2025-12-15-security-patch-tokio.md`
- `2026-01-01-quarterly-review.md`

### Required Sections

1. **Header**: Date, type, risk level
2. **Motivation**: Why upgrade?
3. **Dependencies**: What's changing?
4. **Risk Assessment**: What could go wrong?
5. **Testing Plan**: How to verify?
6. **Rollback Plan**: How to revert?
7. **Implementation Log**: What happened?
8. **Validation Results**: Test outcomes
9. **Lessons Learned**: Takeaways
10. **Sign-Off**: Completion status

---

## Upgrade Types

### Security Patch
- **Priority**: Immediate
- **Risk**: Usually low (patch version)
- **Testing**: Basic verification
- **Example**: `tokio 1.48.0 → 1.48.1` (CVE fix)

### Minor Update
- **Priority**: Regular maintenance
- **Risk**: Low to medium
- **Testing**: Comprehensive
- **Example**: `tokio 1.35 → 1.48`

### Major Update
- **Priority**: Planned initiative
- **Risk**: Medium to high
- **Testing**: Extensive
- **Example**: `tokio 1.x → 2.x`

---

## Testing Requirements

### Minimum (Security Patches)
```bash
cargo update -p <package>
cargo test --all
cargo clippy --all
```

### Standard (Minor Updates)
```bash
cargo update
cargo test --all
cargo test -p allframe-core --all-features
cargo +<msrv> test --all
```

### Comprehensive (Major Updates)
```bash
# All standard tests plus:
cargo bench  # Performance regression
cargo tree   # Dependency conflicts
# Platform testing (Linux, macOS, Windows)
# Manual feature testing
```

---

## Review Schedule

### Monthly
- Security advisories check
- Quick patch updates

### Quarterly
- Minor version updates
- Dependency health review

### Annual
- Major version consideration
- Dependency audit
- Policy review

---

## Historical Plans

### 2025

- [`2025-12-04-december-updates.md`](2025-12-04-december-updates.md)
  - Type: Minor updates
  - Packages: tokio, hyper, testing utilities
  - Status: ✅ Complete
  - Result: 291+ tests passing

---

## Quick Reference

### Common Commands

```bash
# Check for outdated dependencies
cargo outdated

# Check for security issues
cargo audit

# Update specific package
cargo update -p <package>

# Update to specific version
cargo update -p <package> --precise <version>

# Show dependency tree
cargo tree

# Find why a dependency exists
cargo tree -i <package>
```

### Useful Links

- [Main Dependency Policy](../DEPENDENCY_MANAGEMENT.md)
- [Contributing Guide](../../CONTRIBUTING.md)
- [Cargo Book - Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [RustSec Advisory DB](https://rustsec.org/)

---

## Template Usage

The `TEMPLATE.md` file provides a comprehensive structure for upgrade plans. Key features:

- **Checklists**: Track progress
- **Risk levels**: Categorize updates
- **Testing phases**: Structured validation
- **Implementation log**: Timeline tracking
- **Lessons learned**: Knowledge capture

Copy and customize for each upgrade.

---

## Best Practices

### DO ✅
- Create plan BEFORE upgrading
- Test thoroughly
- Document issues encountered
- Update plan with results
- Learn from each upgrade

### DON'T ❌
- Skip planning for "quick updates"
- Update during feature development
- Ignore test failures
- Forget to document lessons
- Rush major updates

---

## Emergency Rollbacks

If an upgrade causes critical issues:

1. **Immediate Action**:
   ```bash
   git revert <commit-hash>
   cargo clean && cargo build --all
   ```

2. **Create Incident Report**:
   - Document in `docs/incidents/`
   - Reference upgrade plan
   - Note specific issue

3. **Update Plan**:
   - Mark as rolled back
   - Document reason
   - Plan remediation

---

## Questions?

- Check the [main dependency policy](../DEPENDENCY_MANAGEMENT.md)
- Review existing plans for examples
- Ask in project discussions
- Create an issue for policy clarification

---

**Remember**: Good planning prevents production problems!
