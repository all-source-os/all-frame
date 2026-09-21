# CLAUDE.md

## Rules

This repo follows the portable rule set published at
[decebal-claude-skills](https://github.com/decebal/decebal-claude-skills) → `rules/`.
Each rule carries the incident that produced it; that is the load-bearing part.

**Loaded by reference, not by copy.** Install once per machine:

```bash
git clone https://github.com/decebal/decebal-claude-skills
mkdir -p ~/.claude/rules
cp decebal-claude-skills/rules/*.md ~/.claude/rules/
```

then `@`-import the subset you want from `~/.claude/CLAUDE.md`. Copying the rule
text into this repo would fork it, and a rule written twice drifts in one copy
without anyone noticing which — that is what the single source prevents.

### The subset that applies here

| Rule | Covers |
|---|---|
| `git-discipline` | One PR = one commit, merged by rebase. Dead-branch liveness check before the first commit. Never push to the trunk |
| `evidence-discipline` | Check the destination before trusting an absence; read runtime state, never guess it |
| `definition-of-done` | End-to-end or not done; size is never a reason to split |
| `estimation` | Never estimate in time; rock / sand / water is confidence, not size |
| `comments` | What a comment must earn; never narrate the fix |
| `anti-slop` | Report findings, don't perform them |
| `pr-evidence-report` | The HTML report a PR ships: claim → evidence → limits, screenshots with provenance, SHA-anchored before/after, copy blocks with a Pass line |
| `documents-not-artifacts` | Deliverables are versioned documents under `docs/`, never hosted pages |

Add `agent-parallelism`, `timeouts` and `process-ownership` when running several
agents against this repo at once.

### Merge settings, already enforced on this repo

Rebase-only — `allow_squash_merge=false`, `allow_merge_commit=false`,
`allow_rebase_merge=true` — with `delete_branch_on_merge=true`.

A PR therefore cannot merge until its branch is a **single commit**. Collapse
before review:

```bash
git reset --soft "$(git merge-base origin/main HEAD)"
git commit -F <message-file>
git diff <old-head> HEAD          # MUST be empty — that is the proof
git push --force-with-lease=refs/heads/<branch>:<old-head> origin HEAD:<branch>
```

Pin the lease to the old head you actually reviewed. The bare form does not
refuse when someone else pushed in between; the pinned form does.
