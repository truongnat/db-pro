# VERIFICATION — egui 0.36.2

Date: 2026-10-09. All commands executed on this workspace.

| Gate | Command | Result |
|------|---------|--------|
| fmt | `cargo fmt --all -- --check` | exit 0 |
| check | `cargo check --workspace` | exit 0 |
| check release | `cargo check --workspace --release` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| tests | `cargo test --workspace` | 1712 passed / 0 failed / 0 ignored (22 suites) |
| dev_tools | `cargo test -p db-pro-ui --lib dev_tools` | 18/18 pass |
| clean-code | `clean-code-scan.sh rust --diff --ratchet --ci` | 11 pass / 5 warn (pre-existing) / 0 fail |
| runtime | `db-pro-native` capture @1440x900 | `/tmp/egui36-1440x900.png`, `/tmp/inspector-pick.png` — app renders, inspector opens + pins a widget with live highlight |

## Runtime evidence

- `/tmp/egui36-1280x800.png`, `/tmp/egui36-1440x900.png`, `/tmp/egui36-query.png` — app at capture sizes, welcome + query surfaces render.
- `/tmp/inspector-off.png`, `/tmp/inspector-on.png`, `/tmp/inspector-pick.png` — dev-tools inspector closed / open / with pinned widget (`DB_PRO_INSPECTOR_PICK=200,40` → selection shows id, local/screen rect, interact rect; highlight strokes the pinned widget).

## Remaining

- No P0/P1. The 5 clean-code warnings are inherited baseline items (long functions, >3 args), not regressions.
- No commit made, per instruction.

## Geometry audit rule engine (2026-10-09)

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | all pass |
| `cargo test --workspace` | 1728 passed / 0 failed |
| `cargo test -p db-pro-ui --lib dev_tools` | 35/35 (incl. 15 new audit tests + 2 integration) |
| runtime broken fixture | `DB_PRO_INSPECTOR_AUDIT_FIXTURE=1` @1440×900 → `Audit — 2 issue(s) in 130 widgets`; `interactive.zero_size` error + `overlap.suspicious` warning (66% coverage, medium) — `/tmp/audit-fixture.png` |
| runtime clean | same app, no fixture → `Audit — clean (123 widgets)`, all rules PASS; overlap evaluated 1128 pairs, skipped 515 clipped — `/tmp/audit-clean.png` |

Rules: `geometry.invalid` (error, never skips) · `interactive.zero_size` (error, skips clipped) · `overlap.suspicious` (warning; skips cross-layer/Area = intentional overlay + clipped; containment = no issue). Click an issue → pins the widget, highlight follows frames.

## Semantic & UX audit (2026-10-09)

Semantic collection: `AccessKitCapture` (egui `Plugin::output_hook`) clones
`PlatformOutput::accesskit_update` each pass while the inspector is open —
`enable_accesskit` is sticky and the clone leaves real AT delivery intact.
`SemanticSnapshot` flattens role / name (label, `labelled_by`, or Label-role
`value`) / disabled / hidden / Click+Focus actions / parent (from children
lists) / `update.focus`, joined to widgets via `Id::accesskit_id`.

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1744 pass / 0 fail |
| dev_tools tests | 50 (incl. semantic node mapping, all rules, Tab-focus integration) |
| runtime | `/tmp/audit-semantic2.png` @1440×900 + fixture: 8 rules all reporting — `semantic.missing_name` 43 (real unnamed controls), `disabled_mismatch`/`orphan_node`/`focus.*` PASS, `disabled_mismatch` skipped 1 widget w/o node |

New rules: `semantic.missing_name` (Error; role requires name, or Click-action node whose widget also senses clicks; skips hidden + unnamed-role containers) · `semantic.disabled_mismatch` (Warning; AT vs widget enabled disagree; skips node-less widgets) · `semantic.orphan_node` (Error; non-root node with no parent) · `focus.orphaned` (Error; focus id not in tree) · `focus.unfocusable` (Error; focus on non-Focus node; root exempt = "nothing focused").

Limits not yet verified: real screen-reader tree diffs (only `TreeUpdate` shape is asserted), Tab-order depth beyond "focus resolves to a live node", and focus-trap detection (needs modal-session state, not geometry).

## Visual quality analyzer (2026-10-09)

Measured inputs (never inferred): `Style::text_styles` FontId sizes ·
`Visuals.widgets.*` fg/bg colors (WCAG ratio computed) · same-`parent_id`
sibling rects (gap/drift deltas) · semantic roles.

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1754 pass / 0 fail |
| dev_tools | 60 tests (incl. 9 new visual-rule tests + WCAG reference point 21:1) |
| runtime clean | `/tmp/audit-visual.png` @1440×900: all 4 visual rules PASS on real UI |
| runtime fixture | `/tmp/audit-visual3.png`: `touching_controls` correctly PASS after dedup with `overlap.suspicious` |

Rules: `visual.typography_hierarchy` (Heading>Body≥Small ordering, skips absent styles) · `visual.text_contrast` (4.5:1 AA, skips non-opaque + exempts disabled/active) · `visual.touching_controls` (<1px vertical gap in a column; skips Foreground/Tooltip layers, negative gaps, non-interactive) · `visual.alignment_drift` (0.5–4px off a ≥3-member column's modal left edge). Baseline: per-frame issue-count Δ shown in the Audit header.

Limits: no font metrics/padding inference (egui exposes none per-widget); drift band is local tuning, not a component standard; contrast checks configured tokens, not rendered pixels.

## Audit reliability (2026-10-09) — P5

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1757 pass / 0 fail (63 dev_tools) |
| runtime | `/tmp/audit-p5.png` — 4-state verdicts live: FAIL/SKIP/N/A distinct, `(+N new, −M resolved)` identity diff |

Reliability changes:
- `Verdict {Pass, Fail, Skipped, NotApplicable}` — `RuleReport::verdict()`: 0 evaluated never shows PASS; all-skip → SKIP; empty scope → N/A (`stats.not_applicable` counted for out-of-scope groups, e.g. drift columns < 3).
- Baseline: `prev_audit: Option<AuditReport>` + `AuditReport::diff` — compares issue *identity* (rule + widget + context), returns `None` unless `widget_ids` sets are equal → diffs only run over equivalent snapshots. Header shows `(+new, −resolved)` instead of a count delta.
- Contrast: reads `ctx.style_of(ctx.theme())` — active-theme token pairs only; evidence names the token pair, never per-widget colors that don't exist in the API.
- Coverage: `evaluated / skipped(reason) / n/a` reported per rule; dump via `DB_PRO_INSPECTOR_DUMP=1` prints rule tallies + per-issue kind/rect/parent (waits for warm semantic tree, 60-frame fallback).

43 missing-name classification (runtime, 123 widgets / 101 semantic evals):
- 33 real defects — 28 clickable `Unknown`-role nodes (activity-bar icons x:6–42, tab row, search field hit rects, window-resize grip strips) + 4 unnamed `ComboBox` + 1 `SpinButton`: announced as unlabeled controls by a screen reader.
- 9 instrumentation gaps — 8 `Splitter` + 1 `ScrollBar`: egui 0.36 emits the role but never a name; fix belongs in egui or per-handle `ui.labelled_by` calls.
- 0 false positives — name-required predicate (role OR both-sides-clickable) held in every sampled case.

Not fixed per scope: the 33 real defects need per-widget labels across shell/activity/tab/combobox code — a separate UI pass.

## Automated audit CLI (2026-10-09) — P6

Headless mode: `DB_PRO_AUDIT_JSON=<path>` on `db-pro-native` (debug + `capture`
feature). The inspector's 12-rule pipeline runs with the window closed; the
process writes schema-v1 JSON and exits. Envs shared with capture:
`DB_PRO_WINDOW_SIZE`, `DB_PRO_INSPECTOR_AUDIT_FIXTURE`, all `DB_PRO_CAPTURE_*`
surface openers. `DB_PRO_AUDIT_BASELINE=<path>` diffs issue keys, but only when
the stored signature (schema + fixture + viewport + theme + widget set) equals
the current run's.

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1762 pass / 0 fail |
| clean UI | `/tmp/audit-run-clean.json` exit 1 — 28 Error issues (known missing names) |
| broken fixture | `/tmp/audit-run-fixture.json` exit 1 — FAIL: `interactive.zero_size` + `overlap.suspicious` 66% + `missing_name` |
| baseline, same conditions | `comparable: true`, empty new/resolved |
| baseline, fixture+viewport differ | `comparable: false`, `reason: "fixture true ≠ false"` |
| exit codes | 0 clean · 1 findings (FAIL/Error) · 2 tool error (write failure, timeout) |

Sizing-pass trap found & fixed: an Area's first paint registers children
`enabled=false` (egui sizing pass) — emitting at frame 2 would audit a fake
snapshot; warm now requires ≥5 audited frames plus the accesskit tree.

## Accessibility remediation (2026-10-09) — P7

P6 audit JSON drove the fix list; every flagged widget was mapped to source by
rect/layer/parent + role before touching code.

| Before | After |
|--------|-------|
| 28 product-UI missing-name issues (exit 1) | 0 issues, exit 0 (`/tmp/p7-after.json`) |
| 42 with inspector open | 14 — all inside egui's own `inspection_ui`/scroll internals (Splitter×8, ComboBox, SpinButton, ScrollBar, Unknowns): upstream instrumentation gap, not our widgets |

Fixed (all real defects, no fake labels — names match the hover text users see):
- `activity_bar_view.rs` — 11 rail icon buttons: `WidgetInfo::labeled(Button, hint)`
- `shell_topbar_view.rs` — 8 frameless resize handles: labeled "Resize window (<dir>)"; topbar quick-open field: "Search commands, tables, schemas (<mod>P)"
- `sidebar_chrome_view.rs` — connection selector (name + shortcut) and New Query button
- `workspace_tab_primitives.rs` — tab hit target: `WidgetInfo::selected(SelectableLabel, selected, title)`

Verify: fixture run `/tmp/p7-fixture.json` still flags its injected defects
(zero_size Error, overlap 66%, 1 name) → audit pipeline unaffected; screenshot
`/tmp/p7-visual.png` shows identical layout; `cargo test --workspace` 1762/0;
fmt + clippy clean; dev + release check pass.

Limits: light theme not audited (no global env; light-only surface hooks don't
apply to the audited shell — names are theme-independent text, so risk is low).
Inspector-internal egui chrome left unnamed — requires egui changes, not ours.

## Audit stability & coverage (2026-10-09) — P8

Stable-snapshot warm: `layout_fingerprint` (sorted hash of id+rect@0.5px) must
match on 3 consecutive audited frames before emit (replaces the fixed frame
count; an Area sizing-pass snapshot can no longer be audited). Fallbacks:
>30 frames for geometry, >120 for accesskit. Schema v2 adds `scenario`
(`DB_PRO_AUDIT_SCENARIO`) + `theme_label` (DbProTheme, not egui::Theme) to the
signature — baseline diffs refuse cross-scenario comparisons.

Driver: `tools/ui-audit-matrix.sh` — 13 scenarios over the existing
`DB_PRO_CAPTURE_*` envs, exits 0/1/2 aggregated into `coverage-matrix.json`.

| Scenario | Theme | Verdict |
|---|---|---|
| shell @800×600 / 1440×900 / 1920×1080 | Dark | clean |
| welcome-light, query-dark, query-light, explorer-filter | Dark*/Light | clean |
| settings | Dark | FAIL — 1 unlabeled TextEdit (keybindings row), 5 touching nav rows (dense list, flagged as-is) |
| table-indexes / table-structure-light | Dark/Light | FAIL — unlabeled rows: DDL surface row + header rows not yet mapped |
| new-connection-dialog | Light | FAIL — egui-internal zero-size widget, unlabeled ScrollBar, 4px left-edge drift |
| quick-open-light | Light | FAIL — egui-internal zero-size, ScrollBar gap, palette backdrop region |
| fixture-broken | Light | FAIL (expected) — zero_size + overlap 66% + 1 name |

Fixed this round: shared `tab_button` (all users), `SearchInput` (all users),
explorer `CodexTreeRow`, palette query input + items, metadata filter TextEdit,
keybindings edit, query context chip.

False-positive audit: touching flags are real 0.0px stacks in dense nav/lists —
visible finding, not hidden; egui-internal ScrollBar/zero-size are upstream
gaps (same class as P7 Splitter finding).

## Deterministic fixtures (2026-10-09) — P9

`DB_PRO_AUDIT_READY=<min-widgets>` adds a per-scenario readiness gate on top of
the 3-frame fingerprint streak — deterministic, no DB connection (all surfaces
use existing `*_for_capture` fixtures, seeded at app level).

Coverage grew 13 → 23 scenarios: +results-dock(dark/light), table-data,
table-ddl, table-profile, diagram, history, schema-compare, agent,
component-gallery. `/tmp/ui-audit-matrix/coverage-matrix.json`.

| Before (P8) | After (P9) |
|---|---|
| 8 clean / 5 fail / 0 unrun | 10 clean / 13 fail / 0 unsupported |
| DB-dependent surfaces untested | all exercised with seeded fixtures |

Repeatability: table-data ×3 runs — identical layout fingerprint AND identical
issue-key sets. Light/dark pairs (results-dock) produce same findings —
theme-independent defects.

New real findings on data surfaces (mapped): 41 unnamed interactives in the
result grid (cells/rows), 25 in table-data grid, per-scenario names on
diagram/compare/agent/profile; systematic 50% cell-hitbox overlaps (adjacent
columns share a 4px edge — layout-intended overlap inside one row, reported
as-is per "no rule changes").
