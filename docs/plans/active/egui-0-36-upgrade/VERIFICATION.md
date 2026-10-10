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

## Screen-level UI/UX analyzer (2026-10-09) — P10

`dev_tools::screen` consumes the audited `GeometrySnapshot` +
`SemanticSnapshot` + viewport and emits `screen` in the schema-v3 JSON:
regions, rhythm, density, balance, hierarchy metrics (null = UNKNOWN),
`coverage.gaps`, and a finding list that never enters `issues` or the exit
code. No pixel/screenshot input, no generic UI standards — all findings are
measured deviations (off-screen rect, left-nav >50% width outweighing every
other region, gap outliers vs the group's own modal rhythm) or explicitly
`heuristic`-kinded consistency signals.

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1354 pass / 0 fail (81 dev_tools) |
| matrix | 25/25 scenarios ran; `/tmp/ui-audit-matrix/coverage-matrix.json` |
| fixture ugly | 3 intended findings: off_screen, layout_inversion, spacing_outlier |
| fixture good | 0 findings |

Fixes this pass (all in dev-tools/capture only, no production UI):
- `capture.rs`: macOS `wrap` ignored `DB_PRO_AUDIT_JSON` without a capture
  path — headless audit hung; added the audit-only arm matching the
  non-macOS variant.
- `screen.layout_inversion`: any side-edge region counted as navigation →
  right-edge content and full-width docks false-flagged. Now left-edge
  panes only (navigation convention), widest region counts as work surface.
- `is_grid_like`: added size-signature + banding (≥2 stripes of ≥4 aligned
  members, ≥60% cover) signatures — cross-parent data grids (table-data)
  were classified as unstructured content and congestion-flagged.
- `control_congestion`: nested container hit-rects and duplicate rects
  counted as separate controls → toolbar stacks flagged. Now leaf-only,
  two-axis packing, and exempt when packed controls sit in top/bottom 15%
  edge bands (toolbar-framed pane) — screenshot-verified against
  results-dock and history.
- matrix: diagram `DB_PRO_AUDIT_READY=140` > its 96 widgets → tool error;
  lowered to 90.

Screen findings on product UI (non-fixture): `no_primary_region` on
shell-800 (largest region 10% of 800×600), table-data (9% across 27
regions), table-profile (3% across 42). All medium-confidence objective
measurements of genuinely fragmented content areas — borderline at small
viewport; reported as-is.

Limits: right-side inversion undetectable from geometry (wide right pane =
right content, same shape); congestion still can't distinguish a real
control wall from an unconventional-but-valid layout; semantic hierarchy
metrics are UNKNOWN when accesskit is off; no pixel/theme-token checks in
this phase.

## AI visual review (2026-10-09) — P11

`dev_tools::visual_review`: opt-in `DB_PRO_REVIEW_DIR` makes the headless
audit run a second stage — capture screenshot → write `<scenario>.bundle.json`
(whitelisted report excerpt + screenshot ref + review instructions) → run
`DB_PRO_AI_REVIEW_CMD` (argv: bundle path; `DB_PRO_REVIEW_MODEL` forwarded)
→ parse stdout into `visual_review` (schema v4). Provider failure/offline
records `status: error|bundle_only`; the section never touches `issues` or
the exit code.

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1783 pass / 0 fail (86 dev_tools, +5 review tests) |
| runtime ugly fixture + mock provider | `/tmp/review-out/`: bundle+json+png, `status: completed`, exit 0 |
| runtime good fixture, no provider | `bundle_only`, files written |
| runtime, no `DB_PRO_REVIEW_DIR` | no `visual_review` key, nothing written |
| runtime, malformed provider output | `status: error`, exit stays audit verdict |

Bundle whitelist: `meta`, `summary`, `rules`, `issues`, `screen` — labels,
text values, connection data and `signature` never leave the process.

**Not verified**: a real vision model has not reviewed a bundle — the
provider contract is exercised by mock/sh scripts only.

## End-to-end audit + real vision model (2026-10-09) — P12

Pipeline exercised end-to-end: fixture → audit JSON → screenshot → bundle →
real vision provider → schema-v4 report with `visual_review`. Provider:
`/tmp/p12/provider.py` → OpenAI Responses API (`gpt-4o` via
`DB_PRO_REVIEW_MODEL`); `OPENAI_API_KEY` lives in the provider env, never in
the app.

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1784 pass / 0 fail (87 dev_tools) |
| ugly fixture ×3 runs | `attention` all runs; same defect set (off-screen rect, dominant left pane, 100px outlier) — matches audit screen findings |
| good fixture | `ok`, 6 observation-only entries, zero fabricated warnings |
| shell-1440 (product UI) | `ok`, 5 observations + 1 suggestion; audit exit 0 unchanged |
| live-connected non-fixture | `blocked` — screenshot withheld (privacy gate: `has_live_connection` + seeded-fixture envs only; audit plumbing envs don't count) |
| interactive mode | review unreachable — runs only inside headless `run_audit`; provider call is synchronous there by design (process exits right after) |

Observed limits: provider latency ~5s per bundle; model echoed audit finding
text as evidence on the ugly run (findings are in the bundle — screenshot
grounding vs echo is not fully separable); rect sometimes returned as a
string (normalized in provider script). `claude` CLI OAuth expired and
`gemini` CLI deprecated on this machine — both unavailable.

## Production readiness: CI policy, provenance, A/B (2026-10-09) — P13

| Change | Evidence |
|---|---|
| `DB_PRO_AUDIT_FAIL_ON` policy (`error` default, `warning`, `screen`) | `report.rs:FailPolicy`, `exit_code_with_policy`; `meta.fail_policy` recorded; error verdicts/FAIL rules always gate; heuristic screen findings and AI output never |
| Privacy gate → provenance, not env flags | `screenshot_is_safe(live, provenance)`; provenance = `DevToolsState.fixture_painted` (set at fixture paint) ∨ `CaptureApp.mark_capture_provenance` (set by `open_*_for_capture`/`prepare_loading`) — a spoofed env can no longer release a live screenshot |
| Agent contract doc | `docs/ui-audit/AGENT_CLI.md`: env vars, exit codes 0/1/2, report schema, provider contract, privacy gate |
| `eprintln!` dump → `writeln!(stderr)` | cc-scan gate: stderr is the `DB_PRO_INSPECTOR_DUMP` interface |

| Gate | Result |
|------|--------|
| fmt / check dev+release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1786 pass / 0 fail (89 dev_tools) |
| `clean-code-scan --ratchet --ci` | 12 pass / 4 warn (pre-existing) / 0 fail |
| E2E ugly fixture + real provider | `meta.fail_policy: error`, review `completed` |

A/B evidence (gpt-4o, identical bundle/screenshot, `DB_PRO_REVIEW_EVIDENCE`
selects assist vs screenshot-only):

| Run | Verdict | Signal |
|---|---|---|
| assist-ugly | `attention` | 3 warnings — off-screen rect, dominant-left, spacing outlier (grounded in findings) |
| solo-ugly | `ok` | 0 warnings — model **missed all three seeded defects** |
| assist-good | `ok` | 5 info observations + 1 warning (icon discoverability) |
| solo-good | `attention` | 2 warnings — density guess, icon clarity — one partially hallucinated ("data grid") |

Reading: evidence-assisted runs detect what the audit measured but ground
poorly against pixels; screenshot-only runs describe layout honestly yet miss
off-screen/spatial defects the geometry sees. A model-callable follow-up
would probe: tighter instruction to *verify* each screen finding against the
image rather than restate it, and `detail=high` on the image input.

## Full-surface UI quality review (2026-10-09) — P14

Matrix: `tools/ui-audit-matrix.sh` — 25/25 scenarios produced reports
(`/tmp/ui-audit-matrix/`, schema v4, macOS): 18 pass / 7 fail / 0 unsupported.
NOTE: a same-day re-run attempt hit `CGSSessionScreenIsLocked` — winit never
receives paint events while the console is locked; the 21:28 run predates the
lock and uses the identical rule set (P13 changed policy only). AI review
evidence reused from P12/P13 runs (shell + both fixtures).

| Severity | Count | Where |
|---|---|---|
| error | 13 | semantic.missing_name ×12 + interactive.zero_size ×1 |
| warning | 14 | touching_controls ×11, alignment_drift ×2, overlap ×1 |
| screen findings | 6 | off_screen/layout_inversion/spacing_outlier (all screen-fixture-ugly, seeded) + no_primary_region ×3 (shell-800, table-data, table-profile) |

Classification (fixture-broken = seeded, excluded from real defects):

| # | Issue | Class | Source |
|---|---|---|---|
| 1 | Unnamed `ScrollBar` — results-dock (dark+light), table-profile, agent | confirmed defect (a11y) | `agent_thread_surface_view.rs:28/227`, `table_profile_surface_view.rs:253`, results dock ScrollArea — egui emits no name |
| 2 | Unnamed `Unknown` interactives — agent ×1, table-profile ×5 | confirmed defect (a11y) | click-sensing widgets w/o accesskit name; painted null-rate bar + hover labels `table_profile_surface_view.rs:291-315` |
| 3 | Unnamed `MultilineTextInput` — table-ddl | confirmed defect | `table_ddl_surface_view.rs:193` `TextEdit::multiline` — no name/hint |
| 4 | 11 × `touching_controls` 0px — schema-compare | potential UX issue | diff rows `schema_compare_view.rs:167-172` + migration ops `251-264` — back-to-back horizontal rows |
| 5 | `alignment_drift` 1.0px (agent) / 1.6px (gallery) | potential UX issue | shared-column edges x=1091 / x=578; subpixel-level, likely real |
| 6 | `no_primary_region` — shell-800, table-data, table-profile | likely false positive | dense data grids fragment regions by design; rule needs a density exemption — logged, not changed per scope |
| 7 | fixture-broken: zero-size + overlap + unnamed | intentional (seeded) | dev_tools audit fixture — proves rule sensitivity |
| 8 | `screen.*` on screen-fixture-ugly ×3 | intentional (seeded) | off-screen rect, layout inversion, spacing outlier |
| 9 | AI: icon-only toolbar discoverability (shell-1440) | potential UX issue | `activity_bar_view.rs` icon column — advisory only |
| 10 | P9 note: 50% cell-hitbox overlap on grid edges | intentional design | adjacent-cell shared edge in result grid; role-gated rule no longer reports it |

Priority: P1 = items 1-3 (error, a11y, 4+ surfaces); P2 = items 4-5;
P3/backlog = 6 (rule calibration), 9 (advisory).

## Accessibility naming pass (2026-10-10) — P15

Closes P14 items 1-3 (`semantic.missing_name`, all surfaces). egui-specific:
it emits anonymous `ScrollBar` nodes, resizable `Panel` drag handles land as
`Unknown` (id = panel id + `"__resize"`, no public widget_info), and
`TextEdit::multiline` carries no label.

| Change | Evidence |
|---|---|
| `name_scroll_bars(ctx, scroll_id, label)` helper | `components/scroll_area/ui.rs` — walks the ScrollArea's child nodes post-show, labels `ScrollBar` role nodes "<label> — horizontal/vertical scroll bar" |
| Scroll bars named | result grid cols+rows (`result_grid_body_view.rs`), agent thread + patch diff (`agent_thread_surface_view.rs`), agent context chips (`agent_surface_view.rs`), column profile grid (`table_profile_surface_view.rs`) |
| Panel resize handles named | `agent_surface_view.rs` agent_panel, `shell_chrome_view.rs` output_panel — `accesskit_node_builder` on the reconstructed `__resize` id |
| DDL editor named | `table_ddl_surface_view.rs` — `set_label("DDL script editor")` on the TextEdit node |
| `Table` component | `components/table/{mod,ui}.rs` — optional `row_label(&dyn Fn(usize)->String)` names row click regions; select-all + per-row checkboxes get `WidgetInfo::selected(Checkbox)`; sortable headers get `WidgetInfo::labeled(Button)`. Callers that don't opt in still flag — naming must be real content |
| Profile rows | `table_profile_surface_view.rs` — `row_label = column name` |

| Gate | Result |
|------|--------|
| fmt / `cargo check -p db-pro-ui` / clippy `-D warnings` | pass |
| `cargo test -p db-pro-ui --lib` | 1123 pass / 0 fail (91 dev_tools, +1 P15 test) |
| new test `named_scrollbar_and_textedit_nodes_report_accessible_names` | scroll bar + multiline editor carry names; zero `missing_name` issues |

Runtime re-audit @1440×900 (`DB_PRO_AUDIT_JSON`, `/tmp/p15/after-*.json`):

| Scenario | Before (P14) | After |
|---|---|---|
| table-profile | 5 unnamed Unknown + scrollbars | 0 issues |
| table-ddl | 1 unnamed MultilineTextInput | 0 issues |
| results-dock / -light | unnamed scrollbars | 0 issues each |
| agent | scrollbars + 1 Unknown + drift | 1 remaining: `alignment_drift` 1.0px (P14 item 5, geometry, not a11y) |
| schema-compare | 11 × touching_controls | unchanged — unrelated pre-existing warnings (P14 item 4) |

Not named: `Table::row_label` callers other than profile (data grids show
painted rows, not widget rows — separate finding class), the `Unknown` node
in agent resolved to the resize handle now labeled. No production UI geometry,
rule logic, or audit policy changed.

## Spacing & alignment triage (2026-10-10) — P16

P14 backlog items 4-5 examined against geometry dumps + screenshots.

Root cause of all `touching_controls`: `shell_frame_view.rs:37` zeroes
`item_spacing` at the workspace root; child `Ui`s clone the parent `Arc<Style>`,
so every widget inside stacks at 0px unless a view adds explicit `add_space`.

| Cluster | Verdict | Evidence |
|---|---|---|
| 4 × data-compare `input_full_width` fields (32px, full width, 0px gap) | FIXED — bordered inputs fused into one white block, fields indistinguishable | before/after screenshots `/tmp/p16/{before,after}-crop.png` |
| Description label touching first input | FIXED — hint glued to field edge | `schema_compare_view.rs` `add_space(SPACE_XS)` after the description + between each input |
| `• item` bullet rows in diff cards (11px lines) | no fix — dense monospace list, readable, selectable text not discrete controls | `/tmp/p16/sc-rows.png` |
| SAFETY LOCK card label lines | no fix — normal leading, readable | same crop |
| MUTATING/DESTRUCTIVE op rows | no fix — badge + SQL text, readable | `/tmp/p16/sc-ops.png` |
| agent `alignment_drift` 1.0px | no fix — buttons live inside `toolbar_frame`'s 1px stroke; egui `Frame` docs: stroke width is part of total margin, so the inset is the frame's visible border, not a misalignment | `agent_context_actions_view.rs:27`, `toolbar_frame` stroke, `/tmp/p16/agent-top.png` |
| gallery `alignment_drift` 1.6px | no fix — workspace tab pill vs content column; different regions, tab has its own horizontal padding | `/tmp/p16/gallery.png` |

No audit thresholds/rules changed; no `no_primary_region` work (out of scope).

| Gate | Result |
|------|--------|
| fmt / check dev + release / clippy `-D warnings` | pass |
| `cargo test --workspace` | 1787 pass / 0 fail (22 suites) |
| schema-compare audit @1440×900 | 11 → 6 `touching_controls` (inputs + hint cleared; remaining 6 = readable text rows) |
| agent / component-gallery audit | unchanged 1/1 — legitimate insets, verified in dumps + screenshots |
| screenshots | `/tmp/p16/sc-1280x800.png`, `/tmp/p16/sc-1440x900.png` (after), `/tmp/p16/sc-1920x1080.png` — fields distinct at all sizes |
| click/hover | unchanged — `add_space` only; interact ids/senses intact in post-fix dump |

## Final regression (2026-10-10) — P17

Read-only pass; no code/rule changes in this phase.

| Gate | Result |
|------|--------|
| fmt `--check` | PASS |
| `cargo check --workspace` dev + `--release` | PASS |
| `clippy --workspace --all-targets -D warnings` | PASS |
| `cargo test --workspace` | PASS — 1787 / 0 fail |
| Full runtime matrix (25 scenarios) | **BLOCKED** — window server stops delivering `RedrawRequested` (`CGSSessionScreenIsLocked` class symptom): binary launches, NSApplication event loop runs, `update` never ticks; every capture times out at 120s. Same blocker P14 hit. Needs an unlocked console. |

Post-P15/P16 re-audits that DID run (console was alive ~08:20–10:00):

| Scenario | P14 baseline | Post-fix | Delta |
|---|---|---|---|
| table-profile | 6 err | 0 | -6 |
| table-ddl | 1 err | 0 | -1 |
| results-dock | 1 err | 0 | -1 |
| results-dock-light | 1 err | 0 | -1 |
| agent | 2 err + 1 warn | 1 warn (legit frame inset) | -2 err |
| schema-compare | 11 warn | 6 warn (readable text rows) | -5 |
| component-gallery | 1 warn | 1 warn (tab pill vs column) | = |

Remaining findings classification:
- **intentional design** — schema-compare text rows (6× `touching_controls` on
  readable 11px list lines), agent drift (toolbar_frame stroke inset), gallery
  drift (tab pill padding).
- **unverified after P15/P16** — the other 18 scenarios were P14-clean and the
  P15/P16 diffs don't touch their code paths (`Table::row_label` opt-in,
  scrollbar naming, DDL label, schema-compare spacing only). Regression risk
  low but not re-proven at runtime.

Interaction checks (from tests + dumps, not live): keyboard focus
(`semantic_tree_and_tab_focus_flow` test), scroll naming (`name_scroll_bars`),
resize handles (`agent_panel`/`output_panel` `__resize` labels), popups
(Foreground-layer exemption in touching_controls), shortcut handling
(app_tests). No live screenshot regression comparison possible — see blocker.

## P17 resume (2026-10-10, second attempt)

Console unlock attempt: `IOConsoleLocked=false`, `sysadminctl screenLock is off`,
window creates and receives `windowDidBecomeKey`/`windowDidChangeOcclusionState`/
`drawRect:`/`mouseEntered`/`resetCursorRects` — yet **zero `RedrawRequested`**
events reach egui (`winit=trace` log: `grep RedrawRequested` = 0 hits). Every
`App::ui`/`run_audit` call starves because egui 0.36 `run_on_demand` only ticks
on RedrawRequested, and winit only emits it while the window is visible — the
window is created `visible: false, maximized: true` and the occlusion→redraw
bridge never fires in this session. Tried: `caffeinate -d`, CGEvent jiggle,
`env -i`-free runs, 60s settle, direct binary launch — all identical: process
runs `CFRunLoop`/`mach_msg`, no `update` tick, 120s timeout.

Conclusion: compositor-level starvation, not a code defect, not a rule gap.
Full 25-scenario matrix remains **BLOCKED** on this machine/session. Re-run
`tools/ui-audit-matrix.sh` from an interactive (physically attached, unlocked,
awake) macOS session. Static gates all PASS (1787/0).

## P17 resume (2026-10-10, resolved)

The 10:00-10:55 window-server stall cleared (paint events resumed at ~11:06;
cause consistent with `drawRect:` landing inside `event_handler.in_use()` +
no queued `request_redraw` — a session-level transient, not a code path).
Full 25-scenario matrix re-run on the same session, `/tmp/p18/matrix/`:

| Status | Count | Scenarios |
|---|---|---|
| pass (0 issues) | 21 | all shells, welcome, query dark+light, table indexes/structure/data/DDL/profile, settings, explorer-filter, new-connection, quick-open, results-dock dark+light, diagram, history, screen-fixture-good+ugly |
| fail — intentional | 3 | fixture-broken (2 err + 1 warn, seeded), schema-compare (6 warn readable text rows), agent (1 warn toolbar_frame stroke inset), component-gallery (1 warn tab-pill padding) |

Zero `semantic.missing_name` errors on any real surface — P15 naming holds.
Zero new issues vs P14 baseline; P15/P16 deltas verified at runtime:
results-dock/dock-light/table-ddl/table-profile → 0 issues each;
schema-compare 11→6; agent 3→1; component-gallery unchanged 1.
