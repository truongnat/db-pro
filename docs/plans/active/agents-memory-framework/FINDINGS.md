# Agents Memory Framework — Findings

Date: 2026-10-24. Baseline: see AGENT_EVIDENCE.md §1.

## P1 — Process: repo gates skipped on first landing

**Observation:** Root `AGENTS.md` and `FEATURE_LIFECYCLE.md` were amended and `.agents/**` landed before `docs/plans/active/agents-memory-framework/` existed and before `STATUS.md` was updated.

**Impact:** Violates FEATURE_LIFECYCLE “one plan directory per feature” and agent evidence contract timing.

**Remediation (this plan):** Retroactive plan pack + STATUS row + VERIFICATION discovery; do not claim COMPLETED until REVIEW accepted.

**Status:** Mitigated by this directory; owner review pending.

## P2 — Mirror quality (đã đo)

- **197** excerpt mirrors; **605** source files; **38** truncated; **~327** `docs/plans/*.md` **không** trong v1.
- Mirror **không** bảo toàn full bytes — chỉ excerpt ≤12 KB/file; `sha256_full` vs `mirror_excerpt_sha256` trong manifest.
- Entry `needs-review` — không phải knowledge đã xác minh.

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
