---
schema_version: 1
id: "MEM-PLAN-quick-open-action-visuals-3fcfd4fb0736"
title: "Plan archive: Quick Open Action Visuals"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, quick-open-action-visuals]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3fcfd4fb0736ae5ca3c1568fe9ca12c425c18bf9f007ab6db5615fccf9a6bff4"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/quick-open-action-visuals/PLAN.md"
  - "docs/plans/active/quick-open-action-visuals/FINDINGS.md"
  - "docs/plans/active/quick-open-action-visuals/VERIFICATION.md"
  - "docs/plans/active/quick-open-action-visuals/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/quick-open-action-visuals/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Quick Open and borderless action states

State: RUNTIME_VERIFY. Baseline: `57cea2bf746e132564cde952cc06d0d04d796aa0`.

## Goal

Make selected Quick Open scope text use the selected foreground, and remove decorative strokes from action controls and selected Quick Open rows. Keep hover fill, press feedback, and the keyboard focus ring.

## Scope

- Fix selected scope-filter text in both light and dark themes.
- Remove selected-row outlines in Quick Open and hover/active strokes from its scope filters.
- Keep Button action variants borderless in rest, hover, loading, and disabled states.
- Add regression coverage and native capture hooks for the affected surfaces.

## Non-goals

- No new user-facing feature, database behavior, or renderer architecture change.
- No change to contrast-driven `text_on_solid`; black text on bright dark-theme accent fills remains intentional.
- No removal of the keyboard focus ring or input-field borders.

## Acceptance

- Selected filter text matches `visuals.selection.stroke.color` in light and dark themes.
- Action and selected-row decoration has no stroke; selection/hover fill remains legible.
- Button stroke stays `Stroke::NONE` during hover animation, loading, and disabled states.
- Native Quick Open and Button Gallery captures confirm the result at supported viewport sizes.
- Full requested 1440×900 and 1920×1080 logical heights remain pending because this display clamps height to 838 points.

## Owner-reported regression pass — 2026-10-19

Baseline: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; changes remain uncommitted on `main`.
Keep this existing plan as the single owner of the follow-up fixes:

- Suppress duplicate egui shell separator strokes; retain semantic subtle dividers and resize affordances.
- Restore visible Light/Dark switching, consistent settings/persistence, and theme propagation into the root UI.
- Keep shared single-line SearchInput compact inside tall containers, so Quick Open results remain visible.
- Verify with interaction/geometry regressions, native light/dark captures at the three requested sizes, and Rust gates.
- Preserve existing layout, typography, database behavior, and keyboard focus rings.

## Border follow-up after owner rejection

The owner reported the borders reappearing. The earlier separator-only change retained
explicit frame strokes; that was insufficient. Remove decorative outer strokes at the
topbar, statusbar, output panel, Agent panel, and activity rail callers. Preserve
input/focus borders and interactive resize feedback.



Provider impact: PostgreSQL and SQLite n/a; presentation-only change.


## Source: `FINDINGS.md`

# Findings

At baseline `57cea2bf746e132564cde952cc06d0d04d796aa0`, the selected Quick Open scope was painted by `ui.selectable_label`. `DbProTheme::apply` sets `Visuals::override_text_color`, which egui bakes into the label galley before selected widget visuals are applied. The baseline regression test observed `#1A1C1F` in the selected glyph while the selection foreground was `#FFFFFF`.

P2, fixed in `78d914887642dd400c7e2048db9a24d0403619c8`: selected filter labels now pass the current egui selection foreground explicitly. The separate selected-row outline and scope-filter hover/active strokes were removed.

P2, fixed in `78d914887642dd400c7e2048db9a24d0403619c8`: Button Outline, disabled, and loading palettes emitted decorative strokes. All action Button variants now stay stroke-free through hover interpolation, loading, and disabled states. The dedicated keyboard focus ring remains.

Dark-theme solid buttons can still have black text by design. `DbProTheme::text_on_solid` chooses black when the accent fill is bright enough to provide stronger normal-text contrast; this is separate from selected-label color.

No P0/P1 findings. No database or provider impact.

## 2026-10-19 regression evidence

Baseline SHA: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`, followed by an uncommitted regression test.

- P2: `SearchInput::show` used `with_layout` with a centered horizontal layout but no row-height constraint. In a tall vertical container the frame consumed the remaining height. The regression command `cargo test -p db-pro-ui search_keeps_results_visible_in_a_tall_container -- --nocapture` failed with the first result at `[[8.0 796.0] - [130.2 812.0]]`. The shared component now allocates an explicitly row-sized left-to-right layout.
- P2: shell TopBottomPanel/SidePanel defaults independently painted separator strokes over existing semantic subtle frame borders. Disable idle separators on the topbar, statusbar, Agent panel, and shell output dock; preserve resize behavior.
- PostgreSQL: n/a, no SQL/provider path changes. SQLite: n/a, no SQL/provider path changes.

### Theme and modal follow-up

Baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; candidate remains an uncommitted patch, bound by `evidence/2026-10-19/source-manifest.json`.

- P2: egui 0.36 `set_visuals` did not select the context's Light/Dark style. Eframe also creates the root `Ui` before calling application logic, so that UI retained its earlier style snapshot. The fix selects the egui theme explicitly, mutates its global style consistently, and refreshes the root style before drawing the shell.
- Toolbar toggles now synchronize the Settings value and request repaint; Settings and Gallery follow the same behavior. The pointer-driven regression covers toolbar, Settings, three subsequent frames, rendered fills, and save/restore. Settings apply-on-restore was not a per-frame overwrite; that initial hypothesis was rejected.
- P2: after fixing SearchInput, runtime inspection still showed excess empty list space and keyboard help clipped by the scrolling modal body. The list now shrinks to content and the help lives in DialogFrame's fixed footer. `empty_palette_keeps_keyboard_help_inside_the_dialog` failed before and passes after.
- Observed verification limitations, not UI regressions fixed here: native sample SQLite initialization cannot open its fixture path; full-workspace tests stop on that existing-path test; Clippy reports errors in untouched core/UI paths. No baseline checkout was executed, so those gates are not presented as a measured before/after comparison.
- Initial captures encountered AppKit `NSRangeException` while shutting down after writing PNGs; six larger captures timed out. Final native matrix is 36/36 exit 0 with valid images. The successful environment attempted a separate HOME and backend data directory, but the rendered app still restored the saved PostgreSQL connection; do not infer full storage isolation.
- Existing saved PostgreSQL connection reports database-not-found in the background. Quick Open's Schema scope therefore has no live schema entries; captures prove compact search, empty content, and visible keyboard help, not populated live-database search.


## Source: `VERIFICATION.md`

# Verification

Historical record — 2026-10-02 implementation source SHA: `78d914887642dd400c7e2048db9a24d0403619c8`.
The automated results, captures, and lifecycle notes below belong only to that earlier implementation.
Current candidate results (including failed workspace/Clippy gates) are in the 2026-10-19 section.

## Automated

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 1606 passed, 0 failed, 41 ignored |
| `cargo test -p db-pro-ui selected_scope_label_uses_the_theme_selection_foreground -- --nocapture` | PASS — 1 passed for light and dark themes; before the fix it failed with `#1A1C1F` vs `#FFFFFF` |
| `cargo test -p db-pro-ui action_variants_stay_borderless_through_hover_loading_and_disabled_states -- --nocapture` | PASS — 1 passed |
| `cargo clippy -p db-pro-native --features capture --all-targets -- -D warnings` | PASS |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `git diff --check` | PASS |

## Native captures

The capture-enabled native app used the seeded Sample E-Commerce SQLite connection. Captures are in `evidence/`:

- `quick-open-dark-1280x800.png`: dark Quick Open, selected Schema scope, populated results; 1280×800 logical at 2× pixels.
- `quick-open-empty-light-1280x800.png`: light Quick Open empty state; 1280×800 logical at 2× pixels.
- `action-buttons-dark-1280x800.png`: dark Button Gallery with variants, loading, and disabled states; 1280×800 logical at 2× pixels.
- `quick-open-light-1440x838-clamped.png`: light Quick Open with populated results; the requested 1440×900 viewport was clamped to 1440×838 logical.
- `quick-open-dark-1920x838-clamped.png`: dark Quick Open with populated results; the requested 1920×1080 viewport was clamped to 1920×838 logical.

The host display cannot provide the full requested heights at 1440 and 1920 logical widths. Quick Open has no loading or error state; its empty state is captured. Capture runs emitted pre-existing egui warnings about replacement glyphs (`◻`/`?`).

## Lifecycle and limitations

At historical SHA `78d914887642dd400c7e2048db9a24d0403619c8`, source and automated gates were recorded as passing. Its native visual evidence confirms selected text, action strokes, loading/disabled Buttons, and empty Quick Open at 1280×800. The full-height 1440×900 and 1920×1080 visual gate remained pending, so the plan stayed `RUNTIME_VERIFY`. No cache or rasterization performance metrics were collected; that change did not alter font or renderer paths.

## 2026-10-19 owner-reported regressions

Baseline HEAD: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`.
Candidate: uncommitted changes on main; exact file SHA-256 values are recorded in
`evidence/2026-10-19/source-manifest.json`. No commit or PR was created.

### Commands actually executed

| Command | Observed result |
|---|---|
| `cargo test -p db-pro-ui search_keeps_results_visible_in_a_tall_container -- --nocapture` | Before fix: 0 passed / 1 failed; first result y=796. After: 1 passed / 0 failed / 0 ignored |
| `cargo test -p db-pro-app toolbar_and_settings_switch_rendered_theme_and_persist_the_same_choice -- --nocapture` | 1 passed / 0 failed / 0 ignored; actual pointer events target accessible toolbar/radio bounds, paint selected theme, then save/reload |
| `cargo test -p db-pro-ui applying_theme_selects_matching_egui_style_across_frames -- --nocapture` | 1 passed / 0 failed / 0 ignored |
| `cargo test -p db-pro-app empty_palette_keeps_keyboard_help_inside_the_dialog -- --nocapture` | Before footer fix: 0 passed / 1 failed; after: 1 passed / 0 failed / 0 ignored |
| `cargo test -p db-pro-app -p db-pro-ui` | Final source: 1299 passed (599 app + 700 UI), 0 failed, 0 ignored; exit 0 |
| `cargo test -p db-pro-ui -p db-pro-runtime` | 742 passed (700 UI + 42 runtime), 0 failed, 0 ignored; exit 0; before palette-only follow-up |
| `cargo test --workspace` | Failed, exit 101: 1366 passed / 1 failed / 55 ignored before abort; native `sample_seed_creates_and_verifies_only_marked_fixture` cannot open its SQLite fixture; UI/runtime suites were subsequently exercised separately |
| `cargo +nightly fmt --all -- --check` | Final source PASS, exit 0 |
| `cargo check --workspace` | Final source PASS, exit 0; warning: unused DbConnector import in untouched SQL Server introspection |
| `bash scripts/check-ui-architecture.sh` | Final source PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Failed, exit 101, 65 errors in core (large transaction error + async-trait double-must-use); not modified here |
| `cargo clippy -p db-pro-ui -p db-pro-app --all-targets --no-deps -- -D warnings` | Failed, exit 101, untouched UI diagnostics: formatting borrow, manual div_ceil, chunks_exact and test borrows; not suppressed |
| `cargo build --locked -p db-pro-native --features capture` | Final source PASS, exit 0 |
| `cargo build --release --locked -p db-pro-native` | Final source PASS, exit 0 |
| `python3 tools/cc-scan.py <14 changed Rust paths> --fail-on error` | Exit 0, 0 errors / 6 warnings; before final palette/footer follow-up |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | WARN, exit 2: 3 checks passed, 1 warning (Clippy), 4 measurements not executed; release binary 32.1MB at that run; no performance improvement claimed |
| `orca skills get computer-use --json` | Unavailable: `command not found: orca`, exit 127; no native OS-pointer interaction claim |

Workspace/Clippy failures are outside this patch's changed behavior. A baseline checkout
was not executed; these results are not a fully measured baseline-versus-candidate comparison.

### Native visual evidence

Final captures: `evidence/2026-10-19/final/`, with per-run sizes/exits in `captures.json`.
36 runs, 36 PNGs, all exit 0. Both light/dark were inspected as original key images and
six contact sheets covering every final capture:

- shell query/editor, including empty saved-query state;
- Agent panel empty state and subtle shell boundary;
- Quick Open focused empty query, selected Schema scope;
- Quick Open typed no-match state;
- connection test pending (fixture, not a live-provider success claim);
- connection validation error (fixture).

| Requested logical size | Actual framebuffer pixels | Effective logical size |
|---|---|---|
| 1280×800 | 2560×1600 | 1280×800 |
| 1440×900 | 2880×1676 | 1440×838 |
| 1920×1080 | 3840×1676 | 1920×838 |

The 900/1080-height filenames encode **requests**, not achieved heights. No image was
stretched/padded to claim a full-height viewport. Quick Open search is now one compact row,
the no-match message is visible, and keyboard help stays in the footer. Light shell lines
are subtle rather than black; dark shell boundaries use the corresponding semantic tones.
Focused search and selected scope states are visible. Hover/press is exercised in the
headless pointer regression; native OS hover and splitter dragging remain unverified.

Example invocation (change the matching mode/theme variables for other rows):

```bash
HOME=/tmp/db-pro-capture-home-20261019 \
DB_PRO_DATA_DIR=/tmp/db-pro-quick-open-theme-20261019 \
DB_PRO_DISABLE_KEYRING=1 DB_PRO_CAPTURE_SETTLE_FRAMES=20 \
DB_PRO_CAPTURE_QUICK_OPEN=1 DB_PRO_CAPTURE_QUICK_OPEN_LIGHT=1 \
DB_PRO_WINDOW_SIZE=1280x800 \
DB_PRO_CAPTURE_TO="$PWD/docs/plans/active/quick-open-action-visuals/evidence/2026-10-19/final/quick-open-light-1280x800.png" \
./target/debug/db-pro-native
```

Agent light evidence uses `DB_PRO_CAPTURE_AGENT=1 DB_PRO_CAPTURE_AGENT_LIGHT=1`.
Normal launches are unchanged by capture-only environment variables.

Initial non-final runs: two PNG-producing processes exited with AppKit NSTouchBar
NSRangeException; six 1440×900 runs timed out, followed by a bounded retry strategy and
the final successful matrix. Their earlier logs/metadata remain one directory above `final/`.
The app still restored a saved PostgreSQL connection despite attempted HOME separation;
its database-not-found status is background context, not a claim about provider correctness.
No secrets were exposed by the masked password fields.

### Disposition

Plan remains `RUNTIME_VERIFY`: requested full heights, independent review and native pointer
resize/hover evidence are not complete. PostgreSQL and SQLite are independently n/a for
the changed presentation logic. No SQL generation, execution, or connection behavior was changed.

## Border follow-up after owner rejection — 2026-10-19

Same baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus uncommitted follow-up.
The earlier fix deliberately retained subtle frame strokes; the owner rejected their
remaining visible boundaries. Topbar/statusbar/output frame flags are now borderless;
Agent and activity-rail callers explicitly suppress their outer frame strokes.
Global input and card styles are unchanged.

- `cargo test -p db-pro-app shell_panel_boundaries_stay_borderless_in_both_themes -- --nocapture`: 1 passed / 0 failed / 0 ignored, exit 0. Checks actual painted large boundary rectangles over repeated frames with Agent and output open/closed.
- `cargo build --locked -p db-pro-native --features capture`: exit 0.
- `evidence/2026-10-19/border-followup/`: 12 PNGs, all exit 0, light/dark shell and Agent at the three requested widths. Actual logical sizes remain 1280×800, 1440×838, 1920×838. Original 1280 Agent images and a contact sheet containing all twelve were inspected.
- Exact source snapshot: that directory's `source-manifest.json`.
- Full package/release verification follows in the separate native feature-cleanup pass; no claim that previous run counts cover this later patch.

This corrects the earlier insufficient border fix; it is not described as a theme regression.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — shell borders, Light/Dark, Quick Open regression pass

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex parent; ThemeToggle and ShellBorders implementation slices |
| Issue(s) | Owner's two screenshots, 2026-10-19 |
| Task state | Done for reachable implementation/verification; feature lifecycle RUNTIME_VERIFY |
| Baseline SHA | `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` |
| Branch / PR | main / no PR; no commit created |
| Scope interpretation | Soften shell boundaries, restore consistent rendered Light/Dark selection, keep Quick Open search/results/footer correctly sized |
| Out of scope | Provider logic, SQL, unrelated sample-seed and Clippy failures, broader Welcome layout |

Historical 2026-10-02 changes at `78d914887642dd400c7e2048db9a24d0403619c8` remain documented in PLAN/FINDINGS/VERIFICATION and the original evidence images.

## 2. Progress checkpoint

- Source base: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` + uncommitted patch. Exact candidate file bytes: `evidence/2026-10-19/source-manifest.json`.
- Completed: [x] subtle shell boundaries; [x] toolbar/Settings/Gallery synchronization; [x] explicit egui theme and fresh root style; [x] compact SearchInput; [x] shrinking empty list and sticky keyboard-help footer; [x] native light/dark screenshots.
- Final app/UI tests: 1299 passed / 0 failed / 0 ignored, exit 0. Runtime suite separately: 42 passed / 0 failed / 0 ignored.
- Remaining: [ ] exact 900/1080 logical heights; [ ] independent review; [ ] actual OS-pointer resize/hover checks.
- Risks: no open defect found in the requested changed paths. Wider application failures and inherited visual limitations are not hidden; see VERIFICATION.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; no candidate commit SHA exists; SHA-256 source manifest binds uncommitted files |
| Commit list | none |
| Acceptance mapping | Borders → panel idle separators disabled, semantic frames retained; theme → pointer-driven lifecycle regression + context-style regression; Quick Open → SearchInput geometry regression + footer clipping regression; runtime → 36 final PNGs |
| Commands and counts | Final app/UI 1299/0/0 PASS; workspace fmt/check/UI architecture PASS; native dev-capture/release builds PASS; full workspace tests FAIL 1366/1/55 before abort; both strict Clippy runs FAIL in untouched paths; exact commands in VERIFICATION.md |
| CI run IDs / status | not run |
| Known limitations | Larger images cap at 838 logical px; actual native hover/resize not exercised; restored PostgreSQL connection missing its database; Schema scope therefore empty, not a populated-provider search demonstration |
| Migrations / config implications | No key/schema migration. Capture-only `DB_PRO_CAPTURE_AGENT_LIGHT=1` now supports light Agent evidence. |
| Out-of-scope changes | none |

File/surface inventory (full paths in source manifest):

- `crates/ui/src/components/input/search.rs`: allocate a compact left-to-right row; register `tests/search_tests.rs`.
- `crates/ui/src/components/input/tests/search_tests.rs`: first result remains visible in tall containers, light/dark.
- `crates/ui/src/components/input/API.md`: document compact row behavior.
- `crates/ui/src/theme/mod.rs`: explicitly select context theme and update visuals/style consistently.
- `crates/ui/src/theme/tests/mod_tests.rs`: light→dark→light rendering-context regression.
- `crates/app/src/shell/chrome/shell_topbar_view.rs`, `shell_statusbar_view.rs`: suppress duplicate idle separators.
- `crates/app/src/shell/chrome/shell_chrome_view.rs`: synchronize toolbar preference/repaint; suppress output-dock idle separator.
- `crates/app/src/agent/agent_surface_view.rs`: suppress Agent idle separator.
- `crates/app/src/settings/settings_appearance_view.rs`: repaint after selecting a theme, retain reduced-motion preference.
- `crates/app/src/component_gallery/component_gallery_view.rs`: synchronize theme toggle preference/repaint.
- `crates/app/src/app/app_lifecycle.rs`: refresh root Ui style before shell rendering; register tests.
- `crates/app/src/app/tests/app_lifecycle_tests.rs`: real control pointer events, rendered fills, repeated frames, save/reload.
- `crates/app/src/palette/palette_surface_view.rs`: shrinking results and fixed DialogFrame footer.
- `crates/app/src/palette/tests/palette_surface_view_tests.rs`: keyboard help stays inside the visible dialog.
- `crates/app/src/workspace/workspace_actions.rs`: light Agent capture mode.
- Existing plan's PLAN/CHECKLIST/FINDINGS/VERIFICATION/AGENT_EVIDENCE and `docs/plans/STATUS.md`: current evidence and limitations.
- `evidence/2026-10-19/`: source manifest, initial attempt metadata/logs, and final 36 light/dark native captures.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | Baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus uncommitted manifest-bound patch |
| Verdict | n/a — implementation self-verification is not independent approval |
| P0 / P1 / P2 counts | Requested-path introduced open: 0 / 0 / 0 observed; baseline issues fixed: 4 P2 (separator, theme, search sizing, footer clipping). Wider-repository failures not independently classified. |
| Findings | Failed workspace test/Clippy and clipped-height gate remain recorded; no suppression or unrelated cleanup |
| CI disposition | CI not run; see local command evidence |
| Next tasks unblocked | Independent review and full-height/native-pointer verification |

## 5. Research / audit handoff

- Source date: 2026-10-19.
- References: manifest-owned files above; installed egui/eframe 0.36.2 `context.rs`, `ui.rs`, `containers/panel.rs`, `native/epi_integration.rs`; RULES.md; feature lifecycle; native migration visual gate.
- Baseline mechanisms: `set_visuals` does not choose egui context theme; eframe root Ui precedes App logic; centered unconstrained `with_layout` consumes remaining height; panel idle separators paint independently of frames.
- Measured: search first-result y=796 before fix; footer regression fails before/passes after; final pointer-driven theme and owned suites pass; final native matrix 36/36 exit 0.
- Inference explicitly rejected: settings did not overwrite the runtime theme each frame; applying saved settings happens during restore.
- Inference/limitation: exact relative contribution of egui theme/root snapshot/repaint issues to the original user session is not separately isolated; the regression exercises the complete repaired path.
- PostgreSQL: n/a presentation change. SQLite: n/a presentation change. No provider success inferred from fixtures or the other provider.
- Unresolved: larger requested heights, independent review, true OS hover/resize; saved-connection and sample-seed failures outside this task.
- Downstream issues activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã bỏ separator đen dư, sửa Light/Dark đồng bộ toolbar–Settings và style của egui, thu gọn ô tìm kiếm Quick Open và giữ hướng dẫn phím ở footer. Test app/UI cuối cùng 1299/0/0, build release thành công, đã xem 36 ảnh native Light/Dark. Gate workspace còn lỗi ở sample SQLite/Clippy ngoài phạm vi; ảnh rộng bị giới hạn chiều cao 838, chưa có review độc lập và thao tác resize/hover native nên plan vẫn RUNTIME_VERIFY.

### Border follow-up

Owner rejected the retained explicit frame strokes. At baseline
`95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus manifest-bound uncommitted follow-up,
outer topbar/statusbar/output/Agent/activity boundaries are now borderless.
Rendered-shell regression: 1/0/0; capture build passes; 12 native PNGs exit 0 and
were inspected. Evidence and exact source hashes are under
`evidence/2026-10-19/border-followup/`. Prior full-height/native-pointer limitations remain.
Đã bỏ cả viền frame còn sót, không chỉ separator egui; giữ nguyên focus và resize.

