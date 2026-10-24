---
schema_version: 1
id: "MEM-SRC-738633a592981805"
title: "Source mirror: docs/plans/active/agents-memory-framework/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/agents-memory-framework/CHECKLIST.md"
source_sha256: "738633a59298180588df5e35aafd20525c191c4e8c40a3e81c1addabc38b6927"
mirror_body_sha256: "738633a59298180588df5e35aafd20525c191c4e8c40a3e81c1addabc38b6927"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Agents Memory Framework — Checklist

## Planning

- [x] Read `docs/plans/STATUS.md` and `docs/plans/FEATURE_LIFECYCLE.md` before claiming done
- [x] Single plan directory `docs/plans/active/agents-memory-framework/`
- [x] Scope / non-goals explicit in PLAN.md
- [x] Provider matrix N/A documented

## Implementation

- [x] Blueprint rsync into `.agents/` (skills preserved)
- [x] `project.md` filled for db-pro
- [x] Curated memory entries (3) + INDEX
- [x] `migrate-plans-to-memory.py` — v1 wave **197** excerpt mirrors / **605** scoped sources (reconcile PASS)
- [x] `build-knowledge-inventory.py` + `migrate-pending-knowledge.py` + `reconcile-knowledge-sources-v2.py` — **1581** repo sources (pipeline JSON không inventory), **0** truncated, `scope_complete`; pipeline ×2 idempotent (counts/status)
- [x] Blueprint 3 lifecycle skills (**13**) + `security-review.md` template + `validate-skill-references.py`
- [x] Root `AGENTS.md` framework section (after plan backlog — see FINDINGS P1 process)
- [x] `FEATURE_LIFECYCLE.md` durable memory section
- [x] `MEMORY_MIGRATION.md` pointer to this plan

## Tests / verification

- [x] Discovery script: required paths, manifest count, search probes (VERIFICATION.md)
- [x] `search-memory.sh` executable
- [x] `python3 .agents/scripts/test-knowledge-mirror-utils.py`
- [x] `.agents/scripts/validate-skills.sh` (69 skills, 14 template refs)
- [ ] Workspace `cargo test` — **not required** for docs-only; recorded as N/A
- [ ] Re-run migrate idempotency after checklist freeze — optional

## Inventory migration (repo)

- [x] `KNOWLEDGE_SOURCE_INVENTORY.json` built
- [x] `repo_pending_count: 0`
- [x] `KNOWLEDGE_MIGRATION_STATUS.md` + skills crosswalk
- [x] User home catalogued (831 rows)

## Review

- [x] FINDINGS.md updated for full inventory scope
- [x] P0 = 0
- [x] Process P1 mitigated via plan pack
- [ ] Curation `needs-review` → `active` (follow-up)

## Runtime

- [n/a] UI → DB path
- [x] VERIFICATION.md contains actual commands and outputs
- [x] STATUS.md updated to match folder/state
