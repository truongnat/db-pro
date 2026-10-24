---
schema_version: 1
id: "MEM-PLAN-ui-stitch-parity-pass-98224004746e"
title: "Plan archive: Ui Stitch Parity Pass"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-stitch-parity-pass]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "98224004746e5a1db062df0b27360828d541b85101bccf412a7890a63ec321f2"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-stitch-parity-pass/PLAN.md"
  - "docs/plans/active/ui-stitch-parity-pass/FINDINGS.md"
  - "docs/plans/active/ui-stitch-parity-pass/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-stitch-parity-pass/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native UI Stitch Parity Pass — Plan

State: PLANNING
Branch: `main` (owner override — no worktree)

## Goal

Bring the remaining native egui surfaces to parity with the canonical Stitch
references under `docs/design/stitch_screens/`, continuing the polish work already done
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


## Source: `FINDINGS.md`

# Native UI Stitch Parity Pass — Findings

Findings are evidence-backed. Severity: P0 catastrophic · P1 wrong/unsafe/broken
flow · P2 polish/maintainability.

## Pre-audit baseline (source inspection @ 52d3600c)

| # | Surface | Spec | Suspected delta | Sev | Task |
|---|---|---|---|---|---|
| F1 | Explorer object filter | 11 | name-contains only; no evaluation mode, no object-kind toggles, no persisted scope | P2 | T4 |
| F2 | Execution history | 12 | `query_history_entries` state exists; unified history surface (filters, day groups, record details) not confirmed in UI | P1* | T3 |
| F3 | Table workspace footer | 04, 16 | no Ready/Active Session/Auto-fallback/UTF-8/Ping telemetry strip | P2 | T1 |
| F4 | Queries sidebar | 14 | grouping/actions vs spec unaudited | P2 | T2 |
| F5 | Schema compare | 09 | diff summary + target-mismatch lock unaudited | P2 | T5 |

*P1 only if spec'd surface is wholly absent; downgrades to P2 if a basic
history view exists and only details are missing.

## T0 audit results

Captures: `/tmp/audit-table-dark.png`, `/tmp/audit-table-light.png`,
`/tmp/audit-diagram-dark.png` (1280×800). Source audit @ `52d3600c`.

| Surface | Evidence | Delta | Sev | Disposition |
|---|---|---|---|---|
| Table data (dark+light) | capture | Toolbar/tabs/pagination/type chips/TRUE·FALSE·NULL badges all present and close to spec; footer has conn+schema+ping+object counts but lacks spec's `Ready · Active Session · Auto-fallback · UTF-8 · Ping:` fields | P2 | T1 (footer fields only — toolbar already good) |
| ER diagram (dark) | capture | Grid canvas, PK/FK badges, edge labels, zoom controls, Design Mode, schema-map sidebar all present; no obvious spec 10 delta at this zoom | — | T6 optional, likely skip |
| Saved queries | source | Context menu already has Open-as-new/Insert/Copy/Rename/Delete (`sidebar_query_library_view.rs:172-198`); missing per-item "Source: conn · db · schema" line + Recent SQL + snippet shortcut codes from spec 14 | P2 | T2 |
| Execution history | source | `query_history_entries` exists, persisted via `app_lifecycle.rs:61`, surfaced only as palette rows + a count in `query_output_actions_view.rs:149` — **no unified history surface** (spec 12: filters, day groups, record details, open-as-query) | P1 | T3 — primary feature of this pass |
| Schema compare | source | Diff groups + Apply button + data-row states exist; spec 09 summary counts (`1 Added · 1 Changed · 0 Removed`) and target-mismatch safety-lock banner not found | P2 | T5 |
| Files workspace | source | Trusted/Untrusted badge + toggle exists (`files_surface_view.rs:181-195`) | — | skip |
| Explorer filter | source | name-contains only (`explorer_schema_object_folders_view.rs:41`); spec 11 workbench (mode/kinds/persisted) absent | P2 | T4 |

## Post-implementation notes

- **T5 target-lock semantics**: the lock compares the `connection_name · active_schema`
  string recorded at plan time against the live session at apply time. This is the
  honest identity available in the UI model — connection *ids* are not threaded
  through the compare view. Re-diffing clears the recorded target so a stale lock
  cannot outlive its plan.
- **T5 capture gap**: `DB_PRO_CAPTURE_COMPARE` helper is committed, but during the
  session the host's windowing stopped emitting `ViewportCommand::Screenshot`
  replies — even the previously-working `DB_PRO_CAPTURE_HISTORY` env produced no
  PNG (process stays alive, no `capture: wrote framebuffer` in the log). The lock
  is covered by unit tests; visual evidence remains pending.
- **T2 scope decision**: saved-query "Source:" line skipped — no source binding in
  `UiSavedQuerySummary` (id/name/sql/folder only).
- **Concurrent-session hazard observed**: a file watcher or other session rewrote
  `result_grid_cell_editor_surface_view.rs` mid-task during earlier phases; final
  diffs were always re-verified before commit.


## Source: `VERIFICATION.md`

# Native UI Stitch Parity Pass — Verification

Only commands actually executed are recorded, with their real output summary.

## Baseline

| Command | Commit | Result |
|---|---|---|
| `cargo test -p db-pro-ui` | `52d3600c` | 972 passed / 0 failed |

## Per-task evidence

| Task | Commit | Gates run | Runtime evidence |
|---|---|---|---|
| T1 status telemetry | `7ede34a5` | fmt · clippy `-D warnings` · ui tests | `evidence/t1-statusbar.png` — `Ping: 1 ms · UTF-8` visible |
| T2 sidebar count + snippet codes | `73340682` | fmt · clippy · ui tests | source-verified; capture folded into query-surface shots |
| T3 execution history | `153756e2` | fmt · clippy · **974 ui tests** | `evidence/t3-history-dark.png` — Today group, outcome icons, detail card with Open-as-new-query/Copy |
| T4 explorer filter workbench | `bb137c33` | fmt · clippy · ui tests (mode-aware + nav-cache) | `evidence/t4-filter.png` — accent dot on filter button, Views folder hidden when disabled |
| T5 compare summary + target lock | `34601996` | fmt · clippy · **976 ui tests** incl. 2 new lock tests · clean-code 0 fail | ~~visual capture pending~~ **[CAPTURED 2026-10-13]** `docs/plans/_audit-2026-10-13/captures/compare_{1280x800,1440x900,1920x1080}.png` — SAFETY LOCK banner visible on divergent target, destructive ops marked, confirm-gate checkbox present |

Query-editor polish landed earlier in the same session and is covered by this
plan's parity intent: gutter/active-line (`90759bea`), selected-lines telemetry
(`1b1c2c00`), row-limit selector (`52d3600c`) — captures in
`evidence/query-dark-v3.png`, `query-light-v2.png`, `query-limit-dark.png`,
`query-limit-light.png`.

## Session-end gates (final run @ `34601996`+docs)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| `cargo test -p db-pro-ui` | **976 passed / 0 failed** |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | 11 pass / 5 warn / 0 fail |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (partial) — 4 executed checks passed; 4 not executed (need live DB / window server). Release binary 32.2 MB |
| `cargo build --release --locked -p db-pro-native` | succeeded |

## Known gaps

- ~~T5 safety-lock visual capture pending (environmental outage; unit tests cover
  the lock semantics).~~ **[RESOLVED 2026-10-13]** visual capture landed via
  `DB_PRO_CAPTURE_COMPARE` in this audit's 102-PNG pass (`captures/compare_*.png`);
  SAFETY LOCK banner + disabled-confirm flow verified on the images.
- T6 ER-diagram spec-10 diff not performed — T0 audit found no visible delta and
  the capture outage removed verification ability.
- Evidence PNGs are dark-theme only where noted; light-theme captures exist for
  table + query editor, not for history/filter/compare surfaces.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/` (slice 06, SUPPORTS verdict): all five T-commits
confirmed in history; history surface, filter workbench, status telemetry and
compare safety lock verified in code at HEAD. New runtime artifacts:
`captures/history_*.png`, `explorer-filter_*.png`, `compare_*.png` (3 sizes each,
dark theme; host still caps 1440/1920 at 838 logical height). Remaining: T6
decision record (deferred), light-theme captures for history/filter/compare,
and independent review. State unchanged: REVIEW.

