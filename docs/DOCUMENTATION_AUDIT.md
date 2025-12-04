# Documentation Audit & Classification

**Date**: 2025-12-04
**Purpose**: Organize and classify all documentation files
**Status**: Audit complete, cleanup recommendations below

---

## Classification

### ✅ KEEP - Active & Essential

#### Root Level (docs/)
- **DEPENDENCY_MANAGEMENT.md** - Core policy, referenced in CONTRIBUTING.md ✅
- **DEPENDENCY_UPDATES_DEC_2025.md** - Recent update summary ✅
- **GITHUB_ACTIONS_PERMISSIONS.md** - CI permissions guide ✅
- **CI_PIPELINE_FIXES_COMPLETE.md** - Important CI fixes documentation ✅
- **PROJECT_STATUS.md** - Current project state ✅
- **README.md** - Documentation index ✅
- **INDEX.md** - Alternative index (consider merging with README.md)

#### Subdirectories
- **upgrade-plans/** - All files ✅ (new system, all relevant)
  - `2025-12-04-december-updates.md`
  - `README.md`
  - `TEMPLATE.md`

- **current/** - Active PRDs ✅
  - `PRD_01.md` - Core PRD
  - `PRD_ALLSOURCE_CLOUD.md`
  - `PRD_QUALITY_METRICS.md`
  - `PRD_ROUTER_DOCS.md`
  - `PRD_SERVERLESS.md`
  - `ALLSOURCE_CORE_ISSUES.md`

- **guides/** - User guides ✅
  - `ALLSOURCE_INTEGRATION.md`
  - `FEATURE_FLAGS.md` (note: duplicate with feature-flags.md)
  - `GRAPHQL_DOCUMENTATION.md`
  - `SCALAR_DOCUMENTATION.md`
  - `MCP_ZERO_BLOAT_STRATEGY.md`

- **phases/** - Completion records ✅
  - Keep all PHASE*_COMPLETE.md files
  - Keep PROTOCOL_AGNOSTIC_ROUTING_COMPLETE.md
  - Keep MCP_ZERO_BLOAT_COMPLETE.md
  - Keep BINARY_SIZE_MONITORING_COMPLETE.md

---

### 🗄️ ARCHIVE - Historical Value

#### Should Move to archive/
- **DOCUMENTATION_REFACTORING_PLAN.md** - Historical planning doc
- **DUAL_TRACK_KICKOFF.md** - Historical planning doc
- **LAUNCH_CHECKLIST.md** - Outdated checklist
- **REFACTORING_COMPLETE.md** - Historical completion doc
- **ROADMAP_WRAPUP_PLAN.md** - Historical planning doc
- **WARNINGS_CLEANUP.md** - Completed cleanup doc
- **CI_FIXES_SUMMARY.md** - Superseded by CI_PIPELINE_FIXES_COMPLETE.md

#### Plans (docs/plans/)
- **ALLFRAME_FORGE_PLAN.md** - Move to archive (forge is implemented)
- **MCP_SERVER_PLAN.md** - Move to archive (MCP is implemented)

#### Phases (docs/phases/)
- **BINARY_SIZE_MONITORING_PLAN.md** - Move to archive (completed)
- **PHASE6_1_ROUTER_CORE_PLAN.md** - Move to archive (completed)
- **PHASE6_2_SCALAR_PLAN.md** - Move to archive (completed)
- **PROTOCOL_AGNOSTIC_ROUTING_PLAN.md** - Move to archive (completed)
- **REMAINING_WORK_PLAN.md** - Move to archive (completed)

#### Milestones (docs/milestones/)
- **milestone-0.4-plan.md** - Move to archive (completed)
- **MILESTONE_0.2_STATUS.md** - Move to archive (completed)
- **MILESTONE_0.3_STATUS.md** - Move to archive (completed)

---

### ❌ DELETE - Redundant or Obsolete

#### Duplicates
- **docs/guides/feature-flags.md** - Duplicate of FEATURE_FLAGS.md (lowercase)
  - Action: Delete lowercase, keep FEATURE_FLAGS.md

---

### 📝 CLEANUP ACTIONS

#### 1. Remove Duplicates
```bash
rm docs/guides/feature-flags.md  # Keep FEATURE_FLAGS.md
```

#### 2. Move Historical Docs to Archive
```bash
# Root level
mv docs/DOCUMENTATION_REFACTORING_PLAN.md docs/archive/
mv docs/DUAL_TRACK_KICKOFF.md docs/archive/
mv docs/LAUNCH_CHECKLIST.md docs/archive/
mv docs/REFACTORING_COMPLETE.md docs/archive/
mv docs/ROADMAP_WRAPUP_PLAN.md docs/archive/
mv docs/WARNINGS_CLEANUP.md docs/archive/
mv docs/CI_FIXES_SUMMARY.md docs/archive/

# Plans
mv docs/plans/ALLFRAME_FORGE_PLAN.md docs/archive/plans/
mv docs/plans/MCP_SERVER_PLAN.md docs/archive/plans/

# Phases (planning docs only)
mv docs/phases/BINARY_SIZE_MONITORING_PLAN.md docs/archive/phases/
mv docs/phases/PHASE6_1_ROUTER_CORE_PLAN.md docs/archive/phases/
mv docs/phases/PHASE6_2_SCALAR_PLAN.md docs/archive/phases/
mv docs/phases/PROTOCOL_AGNOSTIC_ROUTING_PLAN.md docs/archive/phases/
mv docs/phases/REMAINING_WORK_PLAN.md docs/archive/phases/

# Milestones
mv docs/milestones/milestone-0.4-plan.md docs/archive/milestones/
mv docs/milestones/MILESTONE_0.2_STATUS.md docs/archive/milestones/
mv docs/milestones/MILESTONE_0.3_STATUS.md docs/archive/milestones/
```

#### 3. Consider Merging
- **INDEX.md** + **README.md** - Merge into single README.md
  - Currently both serve as documentation index
  - README.md is more complete
  - Move useful content from INDEX.md to README.md, then delete INDEX.md

---

## Proposed Structure

After cleanup:

```
docs/
├── README.md                              # Main documentation index
├── DEPENDENCY_MANAGEMENT.md               # Core policy ✅
├── DEPENDENCY_UPDATES_DEC_2025.md         # Recent updates ✅
├── GITHUB_ACTIONS_PERMISSIONS.md          # CI permissions ✅
├── CI_PIPELINE_FIXES_COMPLETE.md          # CI fixes ✅
├── PROJECT_STATUS.md                      # Current status ✅
│
├── upgrade-plans/                         # Dependency upgrade tracking ✅
│   ├── README.md
│   ├── TEMPLATE.md
│   └── 2025-12-04-december-updates.md
│
├── current/                               # Active PRDs ✅
│   ├── PRD_01.md
│   ├── PRD_ALLSOURCE_CLOUD.md
│   ├── PRD_QUALITY_METRICS.md
│   ├── PRD_ROUTER_DOCS.md
│   ├── PRD_SERVERLESS.md
│   └── ALLSOURCE_CORE_ISSUES.md
│
├── guides/                                # User guides ✅
│   ├── ALLSOURCE_INTEGRATION.md
│   ├── FEATURE_FLAGS.md
│   ├── GRAPHQL_DOCUMENTATION.md
│   ├── SCALAR_DOCUMENTATION.md
│   └── MCP_ZERO_BLOAT_STRATEGY.md
│
├── phases/                                # Completion records ✅
│   ├── PHASE1_COMPLETE.md
│   ├── PHASE2_COMPLETE.md
│   ├── PHASE3_COMPLETE.md
│   ├── PHASE4_COMPLETE.md
│   ├── PHASE5_COMPLETE.md
│   ├── PHASE6_1_COMPLETE.md
│   ├── PHASE6_3_COMPLETE.md
│   ├── PROTOCOL_AGNOSTIC_ROUTING_COMPLETE.md
│   ├── PROTOCOL_AGNOSTIC_ROUTING_PHASE1_COMPLETE.md
│   ├── PROTOCOL_AGNOSTIC_ROUTING_PHASE3_COMPLETE.md
│   ├── MCP_SERVER_PHASE1_COMPLETE.md
│   ├── MCP_ZERO_BLOAT_COMPLETE.md
│   ├── BINARY_SIZE_MONITORING_COMPLETE.md
│   ├── SCALAR_INTEGRATION_COMPLETE.md
│   └── EXAMPLES_UPDATED.md
│
├── milestones/                            # Milestone tracking ✅
│   ├── MILESTONE_0.2_COMPLETE.md
│   ├── MILESTONE_0.3_PLAN.md
│   ├── MILESTONE_0.4_COMPLETE.md
│   └── WARNING_CLEANUP_COMPLETE.md
│
├── metrics/                               # Metrics and monitoring ✅
│   ├── BINARY_SIZE_BASELINE.md
│   └── BINARY_SIZE_MONITORING.md
│
├── announcements/                         # Public announcements ✅
│   ├── ANNOUNCEMENT_DI.md
│   ├── API_DOCUMENTATION_COMPLETE.md
│   ├── CQRS_INFRASTRUCTURE_COMPLETE.md
│   ├── PHASE_6_COMPLETE.md
│   ├── PHASE6_1_ROUTER_COMPLETE.md
│   ├── SOCIAL_POSTS.md
│   ├── TWITTER_THREAD_2025_12_01.md
│   └── TWITTER_THREAD_PHASE_6.md
│
├── archive/                               # Historical documents 🗄️
│   ├── DOCUMENTATION_REFACTORING_PLAN.md
│   ├── DUAL_TRACK_KICKOFF.md
│   ├── LAUNCH_CHECKLIST.md
│   ├── REFACTORING_COMPLETE.md
│   ├── ROADMAP_WRAPUP_PLAN.md
│   ├── WARNINGS_CLEANUP.md
│   ├── CI_FIXES_SUMMARY.md
│   ├── CQRS_CHRONOS_ASSESSMENT.md
│   ├── MIGRATION_SUMMARY.md
│   ├── NEXT_STEPS.md
│   ├── SESSION_COMPLETE.md
│   ├── SESSION_SUMMARY.md
│   ├── SUMMARY.md
│   │
│   ├── plans/
│   │   ├── ALLFRAME_FORGE_PLAN.md
│   │   └── MCP_SERVER_PLAN.md
│   │
│   ├── phases/
│   │   ├── BINARY_SIZE_MONITORING_PLAN.md
│   │   ├── PHASE6_1_ROUTER_CORE_PLAN.md
│   │   ├── PHASE6_2_SCALAR_PLAN.md
│   │   ├── PROTOCOL_AGNOSTIC_ROUTING_PLAN.md
│   │   └── REMAINING_WORK_PLAN.md
│   │
│   └── milestones/
│       ├── milestone-0.4-plan.md
│       ├── MILESTONE_0.2_STATUS.md
│       └── MILESTONE_0.3_STATUS.md
│
└── _templates/                            # Documentation templates ✅
    └── FOOTER.md
```

---

## Summary

### Current State
- **Total Files**: 77 markdown files
- **Redundant**: 1 (duplicate feature-flags.md)
- **Historical**: ~20 (completed plans/status docs)
- **Active**: ~50 (policies, guides, completion records)

### Recommended Actions
1. ✅ Delete 1 duplicate file
2. 🗄️ Move 20 files to archive/
3. 📝 Merge INDEX.md into README.md
4. 🎯 Result: Clean, organized documentation structure

### Benefits
- **Easier navigation** - Less clutter in root
- **Clear history** - Archive preserves context
- **Better organization** - Logical grouping
- **Maintained knowledge** - Nothing lost, just organized

---

## Implementation

Run cleanup script:

```bash
#!/bin/bash
cd /Users/decebaldobrica/Projects/open-source/all-frame/docs

# 1. Remove duplicate
rm guides/feature-flags.md

# 2. Create archive subdirs
mkdir -p archive/plans archive/phases archive/milestones

# 3. Move historical docs
mv DOCUMENTATION_REFACTORING_PLAN.md archive/
mv DUAL_TRACK_KICKOFF.md archive/
mv LAUNCH_CHECKLIST.md archive/
mv REFACTORING_COMPLETE.md archive/
mv ROADMAP_WRAPUP_PLAN.md archive/
mv WARNINGS_CLEANUP.md archive/
mv CI_FIXES_SUMMARY.md archive/

# 4. Move plans
mv plans/ALLFRAME_FORGE_PLAN.md archive/plans/
mv plans/MCP_SERVER_PLAN.md archive/plans/
rmdir plans 2>/dev/null  # Remove if empty

# 5. Move phase planning docs
mv phases/BINARY_SIZE_MONITORING_PLAN.md archive/phases/
mv phases/PHASE6_1_ROUTER_CORE_PLAN.md archive/phases/
mv phases/PHASE6_2_SCALAR_PLAN.md archive/phases/
mv phases/PROTOCOL_AGNOSTIC_ROUTING_PLAN.md archive/phases/
mv phases/REMAINING_WORK_PLAN.md archive/phases/

# 6. Move milestone status docs
mv milestones/milestone-0.4-plan.md archive/milestones/
mv milestones/MILESTONE_0.2_STATUS.md archive/milestones/
mv milestones/MILESTONE_0.3_STATUS.md archive/milestones/

echo "✅ Documentation cleanup complete!"
```

---

**Next**: Execute cleanup script to organize documentation structure.
