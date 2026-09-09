# UX Friendliness Audit — Plan

Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`. Current: `PLANNING`.
Branch: `feature/ux-friendliness-audit`. Baseline: `main` + RC1 audit `qa/rc1-static-audit` (P1: 14, P2: 25).

## Goal
Audit every user-facing feature for UX friendliness (clarity, forgiveness, feedback, discoverability, consistency) and produce a prioritized, implementable remediation plan. No behavior change in this planning step — source evidence only.

## Scope (feature areas)
1. Welcome / onboarding 2. Connections (list, dialog, test, SSH, SQLite) 3. Explorer / sidebar search 4. Workspace tabs (preview, pinned, close guard, orphan recovery) 5. Query editor (editing, history, saved, snippets, run configs) 6. Results grid + EXPLAIN 7. Data grid editing (staging, commit, delete) 8. Schema inspector (columns, FK, indexes, triggers, DDL, CRUD gen) 9. ER diagram (search-first, LOD, minimap, overview) 10. Export/Import/Backup 11. User management 12. Command palette / Quick Open / Agent panel 13. Shell chrome (topbar, statusbar, docks, shortcuts, i18n, tokens)

## Non-goals
- No code fixes in this planning change; no provider runtime claims; no new Agent/MCP features (RC1 freeze respected).

## Method (per feature)
Heuristics: visibility of status, match with mental model, error prevention/recovery, feedback within 1s, empty/loading/error states, destructive-action guard, keyboard + a11y, consistency (tokens, shortcuts, dialogs), provider-appropriate language (PG vs SQLite).

## Acceptance
- `PLAN/CHECKLIST/FINDINGS/VERIFICATION.md` exist under `docs/plans/active/ux-friendliness-audit/`
- Every feature has ≥1 friendliness verdict + evidence file path
- Findings ranked UX-P1 (blocks/confuses/misles primary flow) / UX-P2 (friction/polish)
- No COMPLETED claim; runtime verification explicitly pending.
