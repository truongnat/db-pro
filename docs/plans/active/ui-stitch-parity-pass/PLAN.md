# Native UI Stitch Parity Pass — Plan

State: PLANNING
Branch: `main` (owner override — no worktree)

## Goal

Bring the remaining native egui surfaces to parity with the canonical Stitch
references under `docs/stitch_screens/`, continuing the polish work already done
on the result grid, cell editors, and query editor. The session is time-boxed to
one continuous ~5-hour run; tasks are ordered by user-facing impact and each
lands as an independently revertable commit.

## Evidence / baseline

- Current implementation: native `eframe`/`egui` in `crates/ui`; all major
  surfaces exist (query editor, table workspace, explorer, queries sidebar,
  files workspace, schema compare, ER diagram, settings).
- Done in prior commits this week: result-grid Stitch styling (`7fe066ff`,
  `396458d2`), typed cell editors (`b7a5b0d6`, `8740f09c`, `5cccf17b`,
  `63466a24`), query-editor gutter/active-line (`90759bea`), status-bar
  selected-lines telemetry (`1b1c2c00`), toolbar row-limit selector
  (`52d3600c`).
- Known gaps vs spec, pre-audit:
  - Explorer object filter is name-contains only; spec 11 defines a filter
    workbench (evaluation mode, object kinds, persisted scope).
  - `query_history_entries` exists in state but spec 12's unified execution
    history surface (filter chips, day grouping, record details, "open as new
    query") is not confirmed present in the UI.
  - Table workspace footer lacks spec 04/16 telemetry (Active Session,
    Auto-fallback, UTF-8, Ping).
  - Queries sidebar vs spec 14 grouping/actions not yet audited.
  - Schema compare safety messaging vs spec 09 not yet audited.
- Existing tests: `cargo test -p db-pro-ui` = 972 at `52d3600c`.

## Scope

Time-boxed task list (ordered); each task = capture baseline → implement →
regression test → capture → gates → commit.

| # | Task | Spec | Budget |
|---|---|---|---|
| T0 | Baseline audit: capture every major surface dark+light @1280×800, log deltas into FINDINGS.md | all | ~25m |
| T1 | Table workspace chrome: status telemetry strip (Ready / Active Session / Auto-fallback / UTF-8 / Ping), WHERE filter bar, Export CSV placement | 04, 16 | ~45m |
| T2 | Queries activity sidebar: Open (dirty badge) / Saved (source context + item actions) / Recent / Snippets / Scratch grouping per spec | 14 | ~45m |
| T3 | Execution history surface: filter chips (conn/schema/outcome/time), day-grouped rows, record details w/ SQL preview + Open-as-new-query; reuse `query_history_entries` | 12 | ~60m |
| T4 | Explorer object-filter workbench: evaluation mode + object-kind toggles + persisted-per-scope, graceful empty states ("No views match …") | 11 | ~45m |
| T5 | Schema compare: diff summary counts + target-mismatch safety lock banner per spec | 09 | ~40m |
| T6 | ER diagram polish if budget remains (schema-scoped refresh affordance) | 03, 10 | ~30m |
| T7 | Final gate sweep + evidence pack (captures → `evidence/`, VERIFICATION.md, STATUS.md) | — | ~20m |

## Non-goals

- No new backend capabilities beyond UI-level wiring of existing state/commands.
- No changes to `_archive/frontend/` or `_archive/bench/`.
- No changes to row-limit semantics (`52d3600c`) or the destructive-confirm flow.
- No ER architecture rework; T6 is cosmetic only.
- Tasks that blow their budget by >2× are cut mid-scope and recorded as P2
  findings rather than force-landed.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | automated tests + capture fixture (live PG optional; record NOT VERIFIED if no fixture) |
| SQLite | yes | `/tmp/db_pro_sample.db` runtime capture + tests |

## Architecture

UI-only parity work. No `UiCommand`/`UiEvent` contract changes expected except
T3 if history needs an open-as-query action — reuse the existing
new-query-from-text path if present; otherwise scope T3 to read-only display +
copy. All colors/roles must come from `DbProTheme` tokens; no hard-coded hex in
views. Capture hooks reuse the existing `DB_PRO_CAPTURE_*` env pattern
(`OnceLock`, zero-cost when unset).

## Acceptance criteria

- [ ] Every implemented task has dark+light 1280×800 captures in `evidence/`
- [ ] `cargo fmt --all -- --check`, `clippy --workspace --all-targets -- -D warnings`, `cargo test -p db-pro-ui` green after each commit
- [ ] `clean-code-scan.sh rust --diff --ratchet --ci` → 0 fail
- [ ] `perf-scan.sh` PASS at session end
- [ ] No per-frame allocations / no new per-frame logging in render paths
- [ ] Each spec delta either fixed or filed as a P1/P2 finding with reason
