# Documentation Refactoring Plan

**Created**: 2025-11-27
**Status**: 📋 READY FOR IMPLEMENTATION
**Priority**: P1 (Quality & Maintenance)

---

## Executive Summary

After completing Phase 6.1 Router Core Enhancement (Tasks 1-6), the AllFrame documentation contains:
- **Duplicated information** across INDEX.md, README.md, and PROJECT_STATUS.md
- **Outdated router implementation references** (pre-Phase 6.1 string-based routing)
- **Missing Phase 6.1 completion documentation**
- **Inconsistent statistics** (test counts, file counts)

This plan identifies all duplications and proposes consolidation to make documentation more maintainable and efficient.

---

## Current State Analysis

### Documentation Files Analyzed

1. **docs/README.md** (239 lines)
2. **docs/INDEX.md** (274 lines)
3. **docs/PROJECT_STATUS.md** (468 lines)
4. **docs/phases/PHASE6_1_ROUTER_CORE_PLAN.md** (587 lines)
5. **docs/current/PRD_01.md** (140 lines)
6. **docs/current/PRD_ROUTER_DOCS.md** (885 lines)

### Phase 6.1 Completion Status

**Completed Work** (2025-11-27):
- ✅ Task 1: Route Metadata Extraction (`src/router/metadata.rs`)
- ✅ Task 2: Type-Safe Route Registration (`src/router/method.rs`)
- ✅ Task 3: JSON Schema Generation (`src/router/schema.rs`)
- ✅ Task 4: OpenAPI 3.1 Generation (`src/router/openapi.rs`)
- ✅ Task 5: Route Builder API (`src/router/builder.rs`)
- ✅ Task 6: Documentation Serving (`src/router/docs.rs`)

**Metrics**:
- Tests added: 60 (from 39 to 99)
- Files created: 6 new router modules
- Lines of code: ~1,210 router enhancement code
- Breaking changes: 0

---

## Identified Duplications

### 1. **Documentation Structure** (3 locations)

**Duplication**:
- docs/README.md:18-31 - Full directory structure
- docs/INDEX.md:19-31 - Same directory structure
- Both describe identical `/docs` folder layout

**Impact**:
- Maintenance burden (update in 2 places)
- Inconsistency risk

**Recommendation**:
- **Keep**: INDEX.md as the canonical structure reference
- **Remove**: README.md structure section (lines 18-31)
- **Replace with**: Link to INDEX.md from README.md

---

### 2. **CQRS Infrastructure Summary** (3 locations)

**Duplication**:
- docs/README.md:43-48 - Lists 5 phases with links
- docs/INDEX.md:71-87 - Full table with reductions
- docs/PROJECT_STATUS.md:21-142 - Detailed phase documentation

**Impact**:
- Same information repeated 3 times
- Test counts, reduction percentages duplicated
- High maintenance burden

**Recommendation**:
- **Keep**: PROJECT_STATUS.md as canonical source (most detailed)
- **Keep**: INDEX.md table (navigation aid)
- **Remove**: README.md phase listing (lines 43-48)
- **Replace with**: Link to PROJECT_STATUS.md#completed-work

---

### 3. **Product Requirements Links** (3 locations)

**Duplication**:
- docs/README.md:35-39 - PRD links
- docs/INDEX.md:100-126 - Same PRD links with descriptions
- docs/PROJECT_STATUS.md:213-215 - Same links

**Impact**:
- Redundant navigation
- Inconsistent descriptions

**Recommendation**:
- **Keep**: INDEX.md as canonical link directory (most comprehensive)
- **Simplify**: README.md to high-level overview only
- **Simplify**: PROJECT_STATUS.md to current phase links only

---

### 4. **Repository Statistics** (3 locations)

**Duplication**:
- docs/INDEX.md:207-222 - CQRS stats + doc counts
- docs/PROJECT_STATUS.md:117-141 - CQRS detailed stats
- docs/PROJECT_STATUS.md:300-323 - Code and doc statistics

**Impact**:
- Numbers become outdated quickly
- Inconsistency across files
- **CRITICAL**: Stats are now wrong (99 tests vs 81 shown)

**Current Stats** (as of Phase 6.1 completion):
- Total tests: **99** (was 81, added 60 in Phase 6.1)
- Router tests: 99 (all router tests)
- Router files: 6 new modules + mod.rs
- Router lines: ~1,210 new code

**Recommendation**:
- **Keep**: PROJECT_STATUS.md:300-323 as single source of truth
- **Remove**: INDEX.md:207-222 statistics
- **Update**: PROJECT_STATUS.md with Phase 6.1 completion
- **Add**: Automation script to generate stats from `cargo test` and `tokei`

---

### 5. **Development Workflow / TDD** (3 locations)

**Duplication**:
- docs/README.md:156-165 - TDD philosophy
- docs/INDEX.md:194-197 - Development workflow
- docs/PROJECT_STATUS.md:326-347 - Current standards

**Impact**:
- Same TDD principles repeated
- Different levels of detail

**Recommendation**:
- **Keep**: .claude/TDD_CHECKLIST.md as canonical TDD reference
- **Keep**: PROJECT_STATUS.md:326-347 as current standards
- **Remove**: README.md:156-165 (redundant)
- **Simplify**: INDEX.md:194-197 to link only

---

### 6. **External Resources** (2 locations)

**Duplication**:
- docs/README.md:177-182 - External links
- docs/INDEX.md:242-249 - Same external links

**Impact**:
- Duplicate maintenance
- Link rot in 2 places

**Recommendation**:
- **Keep**: INDEX.md:242-249 as canonical external resources
- **Remove**: README.md:177-182
- **Add**: Link rot checker in CI/CD

---

### 7. **Quick Start / Getting Started** (2 locations)

**Duplication**:
- docs/README.md:185-197 - Getting started sections
- docs/INDEX.md:253-265 - Quick start sections
- Identical content, different formatting

**Impact**:
- Same instructions repeated
- Update in 2 places

**Recommendation**:
- **Keep**: README.md:185-197 as primary getting started
- **Remove**: INDEX.md:253-265
- **Replace**: INDEX.md section with link to README.md#getting-started

---

### 8. **Navigation Footer** (3 locations)

**Duplication**:
- docs/README.md:233 - Navigation links
- docs/INDEX.md:268 - Navigation links
- Similar navigation, different links

**Impact**:
- Maintenance burden
- Inconsistent navigation

**Recommendation**:
- **Standardize**: All docs use same footer format
- **Template**: Create `_FOOTER.md` template
- **Automate**: Script to inject consistent footer

---

## Outdated References

### 1. **Old Router Implementation**

**Location**: docs/phases/PHASE6_1_ROUTER_CORE_PLAN.md

**Outdated Content**:
- Lines 169-585: All tasks marked as "Week 1, Days 1-2" terminology
- Lines 332-342: Deliverables checklist still unchecked

**Current State**:
- All 6 tasks completed (2025-11-27)
- 60 tests added (99 total)
- All success criteria met

**Recommendation**:
- **Create**: `docs/phases/PHASE6_1_COMPLETE.md` (completion document)
- **Archive**: PHASE6_1_ROUTER_CORE_PLAN.md with "✅ COMPLETE" marker
- **Update**: PROJECT_STATUS.md with Phase 6.1 completion

---

### 2. **Phase 6 Status**

**Location**: docs/PROJECT_STATUS.md:193-223

**Outdated Content**:
- Line 194: "Phase 6.1 Plan Complete, Ready for Implementation"
- Line 219: "🎯 Ready to begin implementation"
- Line 222: "Ready to start: Yes ✅" (but now complete)

**Current State**:
- Phase 6.1 implementation **COMPLETE**
- Phase 6.2 (Scalar) is next

**Recommendation**:
- **Update**: PROJECT_STATUS.md lines 193-223
- **Add**: Phase 6.1 completion summary
- **Update**: Roadmap to mark 6.1 complete

---

### 3. **Test Statistics**

**Location**: Multiple files

**Outdated Content**:
- docs/INDEX.md:210: "Total Tests: 72 (100% passing)"
- docs/PROJECT_STATUS.md:130: "72 tests across all CQRS infrastructure"
- docs/PROJECT_STATUS.md:306: "Total tests: 81"

**Current State**:
- **99 tests total** (confirmed in Phase 6.1 completion)
- 39 original tests + 60 Phase 6.1 tests

**Recommendation**:
- **Update**: All test counts to 99
- **Breakdown**:
  - CQRS tests: 39 (Phases 1-5)
  - Router tests: 60 (Phase 6.1)
  - Total: 99

---

### 4. **Router API Examples**

**Location**: docs/current/PRD_ROUTER_DOCS.md

**Outdated Content**:
- Lines 486-510: Shows old string-based registration
- Lines 526-547: GraphQL examples (not implemented yet)
- Lines 561-591: gRPC examples (not implemented yet)

**Current State**:
- Type-safe registration implemented (`router.get()`, `router.post()`, etc.)
- OpenAPI generation working
- GraphQL/gRPC examples are future work (Phase 6.3, 6.4)

**Recommendation**:
- **Update**: Lines 486-510 to show new type-safe API
- **Mark**: GraphQL/gRPC examples as "Future: Phase 6.3/6.4"
- **Add**: Actual working examples from `src/router/mod.rs` tests

---

## Proposed File Structure Changes

### Current Structure
```
docs/
├── README.md                    (239 lines, duplicates INDEX.md)
├── INDEX.md                     (274 lines, duplicates README.md + PROJECT_STATUS.md)
├── PROJECT_STATUS.md            (468 lines)
├── phases/
│   ├── PHASE6_1_ROUTER_CORE_PLAN.md  (587 lines, outdated)
```

### Proposed Structure
```
docs/
├── README.md                    (~100 lines, high-level overview only)
├── INDEX.md                     (~200 lines, navigation hub)
├── PROJECT_STATUS.md            (~500 lines, single source of truth for status)
├── phases/
│   ├── PHASE6_1_ROUTER_CORE_PLAN.md     (archived with ✅ marker)
│   └── PHASE6_1_COMPLETE.md             (NEW - completion document)
├── _templates/
│   └── FOOTER.md                (NEW - standard footer template)
```

---

## Refactoring Tasks

### Task 1: Create Phase 6.1 Completion Document ⭐

**Priority**: P0 (Critical)

**Action**: Create `docs/phases/PHASE6_1_COMPLETE.md`

**Content**:
- Summary of all 6 tasks completed
- Code snippets showing new APIs
- Test statistics (60 tests added)
- Migration guide (old → new router API)
- Examples from actual code
- Success metrics achieved

**Timeline**: 1 hour

---

### Task 2: Update PROJECT_STATUS.md ⭐

**Priority**: P0 (Critical)

**Changes**:
1. Lines 193-223: Update Phase 6 status to "Phase 6.1 Complete"
2. Lines 300-323: Update test counts (99 total, breakdown by phase)
3. Lines 352-359: Update Q1 2025 roadmap (6.1 complete)
4. Add Phase 6.1 completion summary (similar to CQRS summary)

**Timeline**: 30 minutes

---

### Task 3: Consolidate README.md

**Priority**: P1 (High)

**Changes**:
1. Remove lines 18-31 (structure) → link to INDEX.md
2. Remove lines 43-48 (CQRS phases) → link to PROJECT_STATUS.md
3. Remove lines 156-165 (TDD) → link to .claude/TDD_CHECKLIST.md
4. Remove lines 177-182 (external resources) → link to INDEX.md
5. Keep only: Overview, Quick Links, Getting Started
6. Target: ~100 lines (from 239)

**Timeline**: 30 minutes

---

### Task 4: Consolidate INDEX.md

**Priority**: P1 (High)

**Changes**:
1. Remove lines 207-222 (statistics) → link to PROJECT_STATUS.md
2. Remove lines 253-265 (quick start) → link to README.md
3. Update navigation links to reflect Phase 6.1 completion
4. Add Phase 6.1 to phase documentation list
5. Target: ~200 lines (from 274)

**Timeline**: 30 minutes

---

### Task 5: Archive Phase 6.1 Plan

**Priority**: P1 (High)

**Changes**:
1. Add "✅ COMPLETE (2025-11-27)" to title
2. Check all deliverables in checklist (lines 332-342)
3. Add completion notes at top
4. Update timeline section to show actual completion
5. Link to PHASE6_1_COMPLETE.md for details

**Timeline**: 15 minutes

---

### Task 6: Update PRD_ROUTER_DOCS.md

**Priority**: P2 (Medium)

**Changes**:
1. Lines 193-223: Mark Phase 6.1 as complete
2. Lines 486-510: Update REST examples to show type-safe API
3. Add actual code examples from `src/router/mod.rs`
4. Mark GraphQL/gRPC examples as "Future" phases
5. Update success criteria to reflect Phase 6.1 completion

**Timeline**: 45 minutes

---

### Task 7: Create Documentation Templates

**Priority**: P2 (Medium)

**Action**: Create `docs/_templates/` directory

**Templates**:
1. `FOOTER.md` - Standard footer for all docs
2. `PHASE_COMPLETE.md.template` - Template for phase completion docs
3. `PRD.md.template` - Template for new PRDs

**Timeline**: 30 minutes

---

### Task 8: Add Automation

**Priority**: P2 (Medium)

**Scripts**:
1. `scripts/update_stats.sh` - Auto-update test counts from `cargo test`
2. `scripts/check_links.sh` - Check for broken links
3. `scripts/inject_footer.sh` - Inject consistent footers

**Timeline**: 1 hour

---

## Success Metrics

### Quantitative Metrics

1. **Duplication Reduction**:
   - Before: ~300 lines of duplicated content
   - After: 0 lines of duplicated content
   - Target: 100% duplication removed

2. **File Size Reduction**:
   - README.md: 239 → ~100 lines (58% reduction)
   - INDEX.md: 274 → ~200 lines (27% reduction)
   - Total reduction: ~200 lines

3. **Accuracy**:
   - Test counts: 99 (currently wrong in multiple places)
   - Phase 6.1 status: Complete (currently says "ready to start")
   - Router examples: Type-safe (currently shows old API)

### Qualitative Metrics

1. **Maintainability**:
   - Single source of truth for each topic
   - Clear ownership of documentation sections
   - Automated stat updates

2. **Discoverability**:
   - Clear navigation hierarchy
   - Consistent linking structure
   - Updated phase completion markers

3. **Accuracy**:
   - All statistics current
   - All examples use latest APIs
   - All phase statuses correct

---

## Timeline

| Task | Duration | Dependencies |
|------|----------|--------------|
| Task 1: PHASE6_1_COMPLETE.md | 1 hour | None |
| Task 2: Update PROJECT_STATUS.md | 30 min | Task 1 |
| Task 3: Consolidate README.md | 30 min | Task 1, 2 |
| Task 4: Consolidate INDEX.md | 30 min | Task 1, 2 |
| Task 5: Archive Phase 6.1 Plan | 15 min | Task 1 |
| Task 6: Update PRD_ROUTER_DOCS.md | 45 min | Task 1 |
| Task 7: Create Templates | 30 min | None |
| Task 8: Add Automation | 1 hour | None |

**Total Estimated Time**: 5 hours

**Recommended Order**:
1. Task 1 (critical - completion doc)
2. Task 2 (critical - status update)
3. Task 5 (archive old plan)
4. Task 3 (consolidate README)
5. Task 4 (consolidate INDEX)
6. Task 6 (update PRD)
7. Task 7 (templates - can be done in parallel)
8. Task 8 (automation - can be done in parallel)

---

## Risk Mitigation

### Risk 1: Breaking Documentation Links

**Risk**: Removing content breaks existing links

**Mitigation**:
- Search all `.md` files for internal links before removing
- Create redirects for critical removed sections
- Test all links after refactoring

### Risk 2: Information Loss

**Risk**: Removing duplicates loses important context

**Mitigation**:
- Review all removed content before deletion
- Ensure replacement links are correct
- Keep git history for recovery

### Risk 3: Inconsistency During Transition

**Risk**: Partial refactoring creates inconsistent state

**Mitigation**:
- Complete all tasks in single PR
- Review all changes together
- Run link checker before merge

---

## Open Questions

1. **Q**: Should we keep both README.md and INDEX.md?
   **A**: Yes. README.md = overview, INDEX.md = detailed navigation

2. **Q**: Where should test statistics live?
   **A**: PROJECT_STATUS.md is single source of truth

3. **Q**: Should we automate stat updates in CI/CD?
   **A**: Yes. Task 8 creates automation scripts

4. **Q**: How do we handle outdated examples in PRDs?
   **A**: Update examples, mark future features clearly

---

## Next Steps

### Immediate Actions (Today)

1. ✅ Create this refactoring plan
2. Create Phase 6.1 completion document (Task 1)
3. Update PROJECT_STATUS.md (Task 2)
4. Archive Phase 6.1 plan (Task 5)

### Short Term (This Week)

1. Consolidate README.md (Task 3)
2. Consolidate INDEX.md (Task 4)
3. Update PRD_ROUTER_DOCS.md (Task 6)

### Medium Term (Next Week)

1. Create documentation templates (Task 7)
2. Add automation scripts (Task 8)
3. Run full documentation audit
4. Create PR for all changes

---

## Approval Checklist

- [ ] Engineering Lead review
- [ ] Documentation accuracy verified
- [ ] All links tested
- [ ] Stats updated from actual code
- [ ] Examples tested and working
- [ ] Templates created
- [ ] Automation working

**Ready to Start**: Yes ✅

**Blockers**: None

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
