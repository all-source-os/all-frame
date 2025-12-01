# Binary Size Monitoring - Implementation Plan

**Status**: 📋 READY FOR IMPLEMENTATION
**Created**: 2025-11-27
**Target**: 1 week (Dual Track with Phase 6.2)
**Priority**: P0 (Critical for 1.0)

---

## Executive Summary

Establish automated binary size monitoring to ensure AllFrame stays lightweight (<8 MB target). Track size per feature flag, enforce limits in CI/CD, and provide visibility into size impact of code changes.

**Goal**: Never ship binaries larger than our promises.

---

## Binary Size Targets

| Configuration | Target | Hard Limit | Status |
|---------------|--------|------------|--------|
| **Minimal** (no features) | <2 MB | 3 MB | 🚧 |
| **Default** (di, openapi, router) | <4 MB | 5 MB | 🚧 |
| **CQRS** (di, openapi, cqrs) | <5 MB | 6 MB | 🚧 |
| **Router Full** (router-full) | <6 MB | 7 MB | 🚧 |
| **All Features** | <8 MB | 10 MB | 🚧 |

**Hard Limits**: CI/CD fails if exceeded
**Targets**: Warning if exceeded, informational

---

## Implementation Tasks

### Task 1: Baseline Measurement (Day 1)

**Goal**: Establish current binary sizes

**Deliverables**:
1. Install `cargo-bloat`
2. Measure all feature combinations
3. Document current sizes
4. Identify largest dependencies

**Commands**:
```bash
# Install cargo-bloat
cargo install cargo-bloat

# Measure minimal build
cargo bloat --release --bin allframe-core --crates

# Measure with features
cargo bloat --release --features=di,openapi,router --crates
cargo bloat --release --features=cqrs --crates
cargo bloat --release --features=router-full --crates
cargo bloat --release --all-features --crates
```

**Output Documentation**:
Create `docs/metrics/BINARY_SIZE_BASELINE.md` with:
- Current sizes for each configuration
- Top 10 largest dependencies
- Size breakdown by crate
- Comparison to targets

**Acceptance**:
- All configurations measured
- Baseline documented
- Know if we're under/over targets

---

### Task 2: CI/CD Integration (Days 2-3)

**Goal**: Automated size checking in CI/CD

**Deliverables**:
1. Create `.github/workflows/binary-size.yml`
2. Size check on every PR
3. Fail if hard limits exceeded
4. Comment on PR with size changes

**Workflow File**:
```yaml
name: Binary Size Check

on:
  pull_request:
  push:
    branches: [main]

jobs:
  check-size:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal

      - name: Install cargo-bloat
        run: cargo install cargo-bloat

      - name: Check minimal size
        run: |
          cd crates/allframe-core
          SIZE=$(cargo bloat --release --bin allframe-core -n 0 | grep '.text' | awk '{print $2}')
          echo "Minimal size: $SIZE"
          # Check against 3 MB limit
          ./scripts/check_size.sh "$SIZE" 3145728

      - name: Check default features size
        run: |
          cd crates/allframe-core
          SIZE=$(cargo bloat --release --features=di,openapi,router --bin allframe-core -n 0 | grep '.text' | awk '{print $2}')
          echo "Default features size: $SIZE"
          ./scripts/check_size.sh "$SIZE" 5242880

      - name: Check all features size
        run: |
          cd crates/allframe-core
          SIZE=$(cargo bloat --release --all-features --bin allframe-core -n 0 | grep '.text' | awk '{print $2}')
          echo "All features size: $SIZE"
          ./scripts/check_size.sh "$SIZE" 10485760

      - name: Generate size report
        run: |
          ./scripts/generate_size_report.sh > size_report.md

      - name: Comment PR
        uses: actions/github-script@v6
        if: github.event_name == 'pull_request'
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('size_report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

**Helper Script** (`scripts/check_size.sh`):
```bash
#!/usr/bin/env bash
# Check if size is under limit
# Usage: ./check_size.sh <size_bytes> <limit_bytes>

SIZE=$1
LIMIT=$2

if [ "$SIZE" -gt "$LIMIT" ]; then
    echo "❌ Size $SIZE exceeds limit $LIMIT"
    exit 1
else
    echo "✅ Size $SIZE is under limit $LIMIT"
fi
```

**Acceptance**:
- CI/CD runs on every PR
- Size checked for all configurations
- PR commented with size report
- Fails if hard limits exceeded

---

### Task 3: Size Report Generation (Day 4)

**Goal**: Human-readable size reports

**Deliverables**:
1. `scripts/generate_size_report.sh` script
2. Markdown table format
3. Size change detection
4. Trend visualization (ASCII)

**Report Script**:
```bash
#!/usr/bin/env bash
# Generate binary size report

cat <<EOF
# Binary Size Report

## Summary

| Configuration | Current | Target | Hard Limit | Status |
|---------------|---------|--------|------------|--------|
| Minimal | $(get_size_mb minimal) | 2 MB | 3 MB | $(get_status minimal 2 3) |
| Default | $(get_size_mb default) | 4 MB | 5 MB | $(get_status default 4 5) |
| CQRS | $(get_size_mb cqrs) | 5 MB | 6 MB | $(get_status cqrs 5 6) |
| Router Full | $(get_size_mb router-full) | 6 MB | 7 MB | $(get_status router-full 6 7) |
| All Features | $(get_size_mb all) | 8 MB | 10 MB | $(get_status all 8 10) |

## Top 10 Dependencies

$(cargo bloat --release --all-features --crates -n 10)

## Size Trend

\`\`\`
Minimal:     ████░░░░░░ ($(get_percentage minimal 3))
Default:     ███████░░░ ($(get_percentage default 5))
CQRS:        ████████░░ ($(get_percentage cqrs 6))
Router Full: █████████░ ($(get_percentage router-full 7))
All:         ████████░░ ($(get_percentage all 10))
\`\`\`

EOF
```

**Acceptance**:
- Report is readable
- Shows all configurations
- Includes top dependencies
- Visual trend bar

---

### Task 4: Size Optimization Detection (Day 5)

**Goal**: Identify optimization opportunities

**Deliverables**:
1. Dependency analysis script
2. Feature flag impact analysis
3. Optimization recommendations
4. Documentation

**Analysis Script** (`scripts/analyze_size.sh`):
```bash
#!/usr/bin/env bash
# Analyze binary size and suggest optimizations

echo "## Size Analysis"
echo ""

echo "### Dependency Impact"
cargo bloat --release --all-features --crates | head -20

echo ""
echo "### Feature Flag Impact"

# Measure each feature individually
for feature in di openapi router cqrs otel mcp arch; do
    SIZE=$(cargo bloat --release --features=$feature --bin allframe-core -n 0 | grep '.text' | awk '{print $2}')
    echo "$feature: $SIZE bytes"
done

echo ""
echo "### Optimization Opportunities"

# Check for duplicate dependencies
echo "Duplicate dependencies:"
cargo tree --duplicates

echo ""
echo "### Unused Dependencies"
# Would require cargo-udeps
echo "(Run 'cargo install cargo-udeps && cargo +nightly udeps' to detect)"
```

**Acceptance**:
- Can identify largest dependencies
- Can measure feature impact
- Suggests optimizations
- Documented for developers

---

### Task 5: Documentation (Day 5)

**Goal**: Document binary size monitoring

**Deliverables**:
1. `docs/metrics/BINARY_SIZE_MONITORING.md`
2. Usage guide for developers
3. Optimization best practices
4. CI/CD integration docs

**Documentation Sections**:

1. **Overview**
   - Why size matters
   - Targets and limits
   - How monitoring works

2. **For Developers**
   - How to check size locally
   - How to reduce size
   - Common pitfalls

3. **For CI/CD**
   - How workflow works
   - What triggers checks
   - How to fix failures

4. **Optimization Guide**
   - Dependency selection
   - Feature flag design
   - Compilation flags

**Acceptance**:
- Docs complete
- Examples tested
- Links verified

---

## Scripts to Create

### 1. `scripts/check_size.sh`
- Check if size is under limit
- Exit 1 if over
- Exit 0 if under

### 2. `scripts/generate_size_report.sh`
- Generate markdown report
- Include all configurations
- Show trends

### 3. `scripts/analyze_size.sh`
- Analyze dependencies
- Measure feature impact
- Suggest optimizations

### 4. `scripts/update_baseline.sh`
- Update baseline measurements
- Store in docs/metrics/
- Track history

---

## Success Metrics

### Monitoring

| Metric | Target |
|--------|--------|
| Configurations tracked | 5 |
| CI/CD integration | ✅ |
| PR comments | ✅ |
| Hard limit enforcement | ✅ |

### Documentation

| Metric | Target |
|--------|--------|
| Monitoring guide | Complete |
| Optimization guide | Complete |
| CI/CD docs | Complete |
| Examples | 3+ |

### Quality

| Metric | Target |
|--------|--------|
| All sizes < hard limits | ✅ |
| CI/CD reliable | ✅ |
| Reports accurate | ✅ |

---

## Timeline

| Day | Tasks | Deliverable |
|-----|-------|-------------|
| 1 | Baseline measurement | Sizes documented |
| 2-3 | CI/CD integration | Workflow working |
| 4 | Report generation | Reports readable |
| 5 | Optimization + Docs | Complete |

**Total**: 5 working days (1 week)

---

## Example Output

### PR Comment
```markdown
## 📊 Binary Size Report

| Configuration | Current | Target | Status |
|---------------|---------|--------|--------|
| Minimal | 1.8 MB | 2 MB | ✅ -200 KB |
| Default | 3.5 MB | 4 MB | ✅ -500 KB |
| All Features | 7.2 MB | 8 MB | ✅ -800 KB |

### Top 5 Dependencies
1. tokio: 850 KB
2. hyper: 420 KB
3. serde_json: 180 KB
4. allframe-core: 150 KB
5. async-trait: 120 KB

### Changes in this PR
- ✅ All sizes under targets
- ↓ Reduced default size by 50 KB
- ↓ Optimized JSON parsing
```

---

## Deliverables Checklist

- [ ] `cargo-bloat` installed in CI/CD
- [ ] `.github/workflows/binary-size.yml` workflow
- [ ] `scripts/check_size.sh` script
- [ ] `scripts/generate_size_report.sh` script
- [ ] `scripts/analyze_size.sh` script
- [ ] `scripts/update_baseline.sh` script
- [ ] `docs/metrics/BINARY_SIZE_BASELINE.md` documentation
- [ ] `docs/metrics/BINARY_SIZE_MONITORING.md` guide
- [ ] All configurations < hard limits
- [ ] PR comments working

---

## Risk Mitigation

### Risk 1: Sizes Exceed Hard Limits

**Current Status**: Unknown until measured
**Mitigation**: If over, create optimization tasks
**Priority**: P0 if over

### Risk 2: CI/CD Too Slow

**Mitigation**: Cache cargo-bloat installation
**Priority**: P1

### Risk 3: False Positives

**Mitigation**: Test workflow thoroughly before enabling
**Priority**: P0

---

## Integration with Phase 6.2

**Parallel Work**:
- Binary size (Track B) - Week 1 only
- Scalar integration (Track A) - Weeks 1-2

**No Conflicts**:
- Different codebases (CI/CD vs router code)
- Different skillsets (DevOps vs feature development)
- Can work independently

**Week 1 Focus**:
- Mon-Wed: Binary size setup
- Thu-Fri: Start Scalar prototype while monitoring runs

---

## Next Steps

### Immediate (Today)

1. Install `cargo-bloat` locally
2. Measure all configurations
3. Document baseline
4. Check if under targets

### This Week (Days 1-5)

1. Complete all 5 tasks
2. Get CI/CD working
3. Document everything
4. Verify under limits

### Next Week

1. Monitor in production
2. Optimize if needed
3. Iterate based on feedback

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
