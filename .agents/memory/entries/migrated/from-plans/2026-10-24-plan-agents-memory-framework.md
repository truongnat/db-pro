---
schema_version: 1
id: "MEM-PLAN-agents-memory-framework-936411033529"
title: "Plan archive: Agents Memory Framework"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, agents-memory-framework]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "936411033529c3c2280a1e0e8d7695d2f146df9dade9058bc9290f1d628e5fc5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/agents-memory-framework/PLAN.md"
  - "docs/plans/active/agents-memory-framework/FINDINGS.md"
  - "docs/plans/active/agents-memory-framework/VERIFICATION.md"
  - "docs/plans/active/agents-memory-framework/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/agents-memory-framework/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Agents Memory Framework — Plan

State: **REVIEW** (infrastructure landed; curation + owner sign-off pending)

Branch: `main` (working tree; no dedicated feature branch yet)

## Goal

Adopt the **agents-memory blueprint** for db-pro: versioned `.agents/` workflow (rules, template, scripts), searchable project memory, and a one-time mirror of `docs/plans/` evidence into `.agents/memory/` — without replacing the existing feature lifecycle in `docs/plans/`.

## Evidence / baseline

- Prior state: `.agents/skills/` + `skills-lock.json` only; no project memory corpus, no `.agents/AGENTS.md`.
- User request: read `~/Downloads/agents-memory-blueprint 2` and apply to this repo; then complete migration with plan gates (`STATUS.md`, `FEATURE_LIFECYCLE.md`, single plan directory).

## Scope

- Copy blueprint skeleton into `.agents/` (excluding overwriting vendored `skills/`).
- Fill `.agents/rules/project.md` for db-pro.
- Seed curated memory entries (release pin, theme tokens, search workflow).
- Script `migrate-plans-to-memory.py` + manifest for `docs/plans/active`, `completed`, `_audit-2026-10-13`.
- Link root `AGENTS.md` and `FEATURE_LIFECYCLE.md` to the framework (after this plan exists).
- Bootstrap `~/.agents/memory` skeleton + global `search-memory.sh` (user machine).
- **This plan directory** as the single owner for the feature (`agents-memory-framework`).

## Non-goals

- Deleting or moving existing `docs/plans/**` trees.
- Promoting all 196 mirrored entries to `status: active` without human review.
- Replacing `skills-lock.json` or re-vendoring skills.
- Orca/Cursor harness auto-registration (document discovery only).

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | N/A | Infrastructure/docs only |
| SQLite | N/A | Infrastructure/docs only |

## Architecture

```text
AGENTS.md (root) ──► .agents/AGENTS.md (workflow)
                 ──► .agents/rules/* (policy + project context)
                 ──► .agents/memory/* (searchable corpus)
                 ──► .agents/scripts/search-memory.sh
docs/plans/<feature>/ ──► lifecycle truth (PLAN/CHECKLIST/FINDINGS/VERIFICATION)
                 ──► mirror ──► .agents/memory/entries/migrated/from-plans/
```

## Acceptance criteria

- [x] Exactly one plan dir: `docs/plans/active/agents-memory-framework/` with PLAN/CHECKLIST/FINDINGS/VERIFICATION/AGENT_EVIDENCE.
- [x] `docs/plans/STATUS.md` row matches folder and state.
- [x] Blueprint required paths present (see VERIFICATION.md discovery table).
- [x] `MIGRATION_MANIFEST.json` entry count matches files on disk.
- [x] `search-memory.sh` returns hits for curated + plan-slug queries.
- [ ] Owner accepts REVIEW → COMPLETED after curation policy agreed (or defers curation to follow-up task).
- [ ] Optional: CI/doc check that plan dir exists before merging large `.agents/memory` changes (future).


## Source: `FINDINGS.md`

# Agents Memory Framework — Findings

Date: 2026-10-24. Baseline: see AGENT_EVIDENCE.md §1.

## P1 — Process: repo gates skipped on first landing

**Observation:** Root `AGENTS.md` and `FEATURE_LIFECYCLE.md` were amended and `.agents/**` landed before `docs/plans/active/agents-memory-framework/` existed and before `STATUS.md` was updated.

**Impact:** Violates FEATURE_LIFECYCLE “one plan directory per feature” and agent evidence contract timing.

**Remediation (this plan):** Retroactive plan pack + STATUS row + VERIFICATION discovery; do not claim COMPLETED until REVIEW accepted.

**Status:** Mitigated by this directory; owner review pending.

## P2 — Mirror quality

- **196** auto entries default `status: needs-review`; duplicate/overlap with curated entries possible (e.g. release pin).
- **Truncation:** bodies >12k chars truncated in mirror; full text remains in `docs/plans/`.

## P2 — Harness discovery

- Cursor has `.cursor/skills/` only; no `.cursor/rules` pointing at `.agents/AGENTS.md` (discovery relies on root `AGENTS.md`).
- `dbpro-orca-ui-drive` skill still absent from Orca list (pre-existing RQM gap).

## P0

None.

## Decisions

| ID | Decision |
|---|---|
| D1 | Keep `docs/plans/` as lifecycle source; memory is searchable mirror + curated lessons. |
| D2 | Auto-migrated files live under `entries/migrated/from-plans/` until curated. |
| D3 | Global `~/.agents/memory` skeleton only; db-pro corpus stays in repo. |


## Source: `VERIFICATION.md`

# Agents Memory Framework — Verification

Recorded: 2026-10-24. Baseline SHA: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`.

## 1. Lifecycle reads (gate)

```bash
# Confirmed read before closing this verification pack:
# - docs/plans/STATUS.md (feature table)
# - docs/plans/FEATURE_LIFECYCLE.md (required plan files + memory section)
```

## 2. Blueprint discovery (required paths)

| Path | Present |
|---|---|
| `.agents/AGENTS.md` | yes |
| `.agents/README.md` | yes |
| `.agents/rules/project.md` | yes |
| `.agents/rules/memory-policy.md` | yes |
| `.agents/scripts/search-memory.sh` | yes (executable) |
| `.agents/template/memory/memory-entry.md` | yes |
| `.agents/memory/INDEX.md` | yes |
| `.agents/memory/MIGRATION_MANIFEST.json` | yes |

## 3. Migration counts

```bash
python3 .agents/scripts/migrate-plans-to-memory.py
# {"written": 196, "manifest": ".agents/memory/MIGRATION_MANIFEST.json"}

find .agents/memory/entries/migrated/from-plans -name '*.md' | wc -l
# 196

python3 -c "import json; print(json.load(open('.agents/memory/MIGRATION_MANIFEST.json'))['entry_count'])"
# 196
```

Curated entries (outside `migrated/`): **3** (`environment`, `patterns`, `workflows`).

## 4. Search probes

```bash
.agents/scripts/search-memory.sh "rustc pin"
# → hits INDEX + environment entry (exit 0)

.agents/scripts/search-memory.sh "release-qualification-matrix"
# → hits migrated plan mirror (exit 0)

.agents/scripts/search-memory.sh "chart palette"
# → hits audit slice mirrors (exit 0)
```

## 5. Skills inventory (unchanged lock)

```bash
python3 -c "import json; print(len(json.load(open('skills-lock.json'))['skills']))"
# 41
```

`.agents/skills/README.md` documents lock + Superpowers + `.skills/`.

## 6. Global home (user machine)

```bash
test -d ~/.agents/memory && test -f ~/.agents/scripts/search-memory.sh && echo OK
# OK (bootstrap 2026-10-24)
```

Not reproducible in CI; documented for local agent harness.

## 7. Product gates (explicit N/A)

| Gate | Result |
|---|---|
| `cargo test --workspace` | **not run** — docs/infrastructure only |
| `cargo clippy` | **not run** |
| Native UI runtime | **N/A** |

## 8. Git diff scope (working tree at verification time)

```bash
git status --short .agents AGENTS.md docs/plans/FEATURE_LIFECYCLE.md docs/plans/MEMORY_MIGRATION.md docs/plans/active/agents-memory-framework/
```

Untracked: `.agents/` tree (except pre-tracked skills), new plan dir, `MEMORY_MIGRATION.md`.  
Modified: `AGENTS.md`, `FEATURE_LIFECYCLE.md`.

Machine-readable discovery snapshot: `evidence/framework-discovery-2026-10-24.json`.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — agents-memory-framework

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Composer (Cursor) — infrastructure lane |
| Issue(s) | n/a (user-directed blueprint adoption) |
| Task state | `Review` |
| Baseline SHA | `dd2bf5e4090e6d509d34c0c8c58ca95015d91640` |
| Branch / PR | `main` · PR `not opened` |
| Scope interpretation | Land agents-memory blueprint, mirror plan evidence to `.agents/memory`, document discovery; satisfy FEATURE_LIFECYCLE plan pack retroactively. |
| Out of scope | Curating 196 entries to `active`; product Rust changes; Orca skill install |

## 2. Progress checkpoint

- Current HEAD: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640` (unchanged; working tree adds `.agents/` + plans)
- Completed: blueprint tree, migrate script, 196 mirrors, curated×3, global bootstrap, plan pack, VERIFICATION
- Remaining: owner REVIEW → COMPLETED; optional curation wave
- Findings: **P1** process gap (FINDINGS.md); **P2** mirror/harness notes
- Tests run: **none** (docs-only); see VERIFICATION §7

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `dd2bf5e4090e6d509d34c0c8c58ca95015d91640` + uncommitted tree |
| File / surface inventory | `.agents/{AGENTS,README,rules,memory,scripts,template,tools,mcp,skills/README}.md`; `AGENTS.md`; `FEATURE_LIFECYCLE.md`; `docs/plans/MEMORY_MIGRATION.md`; `docs/plans/active/agents-memory-framework/*` |
| Acceptance mapping | PLAN.md acceptance rows ↔ VERIFICATION.md |
| Commands and counts | migrate `196` written; search probes exit `0`; skills lock `41` |
| CI run IDs / status | not run |
| Known limitations | 196× `needs-review`; first edit wave preceded plan dir |
| Migrations / config implications | `~/.agents/memory` skeleton on developer machine |
| Out-of-scope changes | `crates/*`, CI workflows |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | pending owner |
| Verdict | pending |
| P0 / P1 / P2 | P0=0; P1=1 process (mitigated); P2=2 |
| CI disposition | not executed |

