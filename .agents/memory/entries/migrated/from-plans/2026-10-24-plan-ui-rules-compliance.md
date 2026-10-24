---
schema_version: 1
id: "MEM-PLAN-ui-rules-compliance-3cea63130eac"
title: "Plan archive: Ui Rules Compliance"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-rules-compliance]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3cea63130eacf0446645ee0a347ed478e667186dd55e11cd6a735652f99a804f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-rules-compliance/PLAN.md"
  - "docs/plans/active/ui-rules-compliance/FINDINGS.md"
  - "docs/plans/active/ui-rules-compliance/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-rules-compliance/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# PLAN — ui-rules-compliance

## Goal

Make `crates/ui` conform to `RULES.md` (component-first UI rules) at HEAD
`445bb165ed0d2c507c226574658185a21df2f5cc`, and make the repo's own declared gate
(`RULES.md` §6) pass instead of the narrower `scripts/check-ui-architecture.sh`.

## Non-goals

- No visual redesign. Every replacement MUST render the same surface, same strings,
  same widget ids, same order. Any intentional visual delta must be listed in
  VERIFICATION.md with a screenshot.
- No change to `components/` public API beyond adding a missing primitive when no
  existing component fits a pattern.
- No `RULES.md` text change, no weakening of the rule.
- §3 (hard-coded colours/metrics) is a separate wave — see CHECKLIST phase W2.

## Scope (frozen)

`docs/plans/active/ui-rules-compliance/FINDINGS.md` — 88 non-test §1 sites:

| bucket | count | action |
|---|---:|---|
| `frame (styled)` | 61 | replace with `components::legacy::*_frame` helper / `components::card` / `components::badge` / `components::hover_card` |
| raw widget (`ui.small_button`, `egui::Button`, `ui.radio_value`, `egui::Area`, `CollapsingHeader`, `ui.checkbox`, `egui::ScrollArea`) | 23 | replace with the matching core component |
| `frame (layout-only)` | 4 | keep — §2 allows `ui.allocate_*`/layout-only frames |

Plus: `RULES.md` §6 gate command must return empty output, which also requires the
test-file and `component_gallery_rendering.rs` usages to carry an inline
`// ui-rules: <reason>` (same line) instead of relying on the script's path exemptions.

## Fix policy (binding for every agent)

1. If a core component exists for the pattern, it MUST be used. Reuse order:
   `components::legacy::{panel_frame, sidebar_frame, activity_bar_frame, toolbar_frame, tab_frame,
   card_frame, agent_message_frame, editor_frame, grid_frame}` →
   `components::card` / `components::badge` / `components::hover_card` / `components::button` /
   `components::selection` / `components::collapsible` / `components::overlay` / `components::tabs`.
2. If no existing component fits, add the smallest missing primitive under
   `crates/ui/src/components/` (theme-aware, light+dark, all five states) and use it.
3. A `// ui-rules: <reason>` comment may only be added when the site is genuinely one of
   §5's exemptions (inside `components/`, `dev_tools/`, `components::scroll_area` wrapper,
   editor/canvas paint, dev-only gallery reference baseline, test harness). Adding a comment
   where a component replacement exists is a REJECT and will be reverted.
4. Behaviour-preserving: same label text, same `Id`, same call order, same callbacks.
5. Minimal diff. Do not reformat unrelated lines. Do not touch files outside your slice.

## Evidence

Source: `FINDINGS.md` inventory (file:line). Automated: `RULES.md` §6 command empty,
`scripts/check-ui-architecture.sh` PASS, `cargo fmt/check/clippy/test`. Runtime: `capture`
feature screenshots light+dark at 1280×800 / 1440×900 / 1920×1080 for the affected surfaces
(recorded in VERIFICATION.md).


## Source: `FINDINGS.md`

# FINDINGS — ui-rules-compliance

Frozen inventory of `RULES.md` §1 violations in `crates/ui` view code at HEAD `445bb165ed0d2c507c226574658185a21df2f5cc`.

Scanner: RULES.md §1 forbidden-pattern list; `components/` and `dev_tools/` excluded; `*_tests.rs`/`app_tests.rs` excluded; lines already carrying `ui-rules:` excluded. Frames are split by whether they carry `fill`/`stroke`/`corner_radius`/`shadow`: `frame (styled)` = §1 violation; `frame (layout-only)` = pure layout wrapper, allowed by §2, no change.

Total non-test §1 sites: **88** (61 styled frames, 4 layout-only, 23 raw widgets).

## agent (4)

| file:line | pattern | snippet |
|---|---|---|
| `agent_settings_view.rs:46` | frame (styled) | `egui::Frame {` |
| `agent_surface_view.rs:32` | frame (styled) | `.frame(egui::Frame {` |
| `agent_thread_surface_view.rs:108` | frame (styled) | `egui::Frame {` |
| `agent_thread_surface_view.rs:122` | frame (styled) | `egui::Frame {` |

## component gallery (7)

| file:line | pattern | snippet |
|---|---|---|
| `component_gallery_inputs.rs:593` | frame (styled) | `egui::Frame::NONE` |
| `component_gallery_rendering.rs:33` | raw checkbox | `ui.checkbox(` |
| `component_gallery_rendering.rs:74` | raw scroll | `egui::ScrollArea::horizontal()` |
| `component_gallery_rendering.rs:100` | raw checkbox | `ui.checkbox(` |
| `component_gallery_surfaces.rs:289` | frame (styled) | `let tree_frame = egui::Frame::NONE` |
| `component_gallery_view.rs:273` | frame (styled) | `egui::Frame::NONE` |
| `component_gallery_view.rs:342` | frame (styled) | `egui::Frame::NONE` |

## diagram (5)

| file:line | pattern | snippet |
|---|---|---|
| `diagram_canvas_view.rs:22` | frame (styled) | `egui::Frame {` |
| `diagram_canvas_view.rs:94` | frame (styled) | `egui::Frame {` |
| `diagram_canvas_view.rs:538` | frame (styled) | `egui::Frame {` |
| `diagram_view.rs:522` | raw button | `egui::Button::new(egui::RichText::new(toggle.label).size(12.0).color(color))` |
| `diagram_view.rs:1082` | frame (styled) | `egui::Frame {` |

## explorer / sidebar (5)

| file:line | pattern | snippet |
|---|---|---|
| `explorer_schema_feedback_view.rs:32` | frame (styled) | `egui::Frame {` |
| `explorer_schema_feedback_view.rs:76` | frame (styled) | `egui::Frame {` |
| `explorer_schema_object_row_view.rs:121` | raw button | `ui.add_enabled(false, egui::Button::new(format!("Enable/Disable unavailable: {reason}")));` |
| `explorer_toolbar_view.rs:95` | raw radio | `ui.radio_value(&mut filter.mode, mode, mode.label());` |
| `sidebar_surface_view.rs:52` | frame (styled) | `.frame(egui::Frame {` |

## files / tasks ide (3)

| file:line | pattern | snippet |
|---|---|---|
| `files_agent_context_view.rs:36` | frame (styled) | `egui::Frame {` |
| `files_surface_view.rs:120` | frame (styled) | `egui::Frame {` |
| `files_surface_view.rs:311` | frame (styled) | `let response = egui::Frame::NONE` |

## management / admin (1)

| file:line | pattern | snippet |
|---|---|---|
| `tools_hub_surface_view.rs:222` | frame (styled) | `let card_frame = egui::Frame {` |

## monitoring (1)

| file:line | pattern | snippet |
|---|---|---|
| `monitoring_header_view.rs:21` | frame (styled) | `egui::Frame {` |

## query editor surface (24)

| file:line | pattern | snippet |
|---|---|---|
| `query_completion_popup_view.rs:76` | raw overlay | `let area_resp = egui::Area::new(egui::Id::new("floating_completion_popup"))` |
| `query_completion_popup_view.rs:80` | frame (styled) | `egui::Frame {` |
| `query_context_picker_view.rs:27` | raw overlay | `let menu = egui::Area::new(egui::Id::new("query_context_picker"))` |
| `query_context_picker_view.rs:32` | frame (styled) | `egui::Frame {` |
| `query_context_view.rs:222` | frame (styled) | `let response = egui::Frame::NONE` |
| `query_dialog_surface_view.rs:47` | frame (styled) | `egui::Frame {` |
| `query_editor_support.rs:30` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:38` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:107` | raw overlay | `egui::Area::new(egui::Id::new(("sql_rich_hover", token_range)))` |
| `query_editor_support.rs:112` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:143` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:232` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:251` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:265` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:274` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:313` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:369` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:398` | frame (styled) | `egui::Frame::NONE` |
| `query_editor_support.rs:436` | raw overlay | `egui::Area::new(egui::Id::new("sql_signature_help"))` |
| `query_editor_support.rs:441` | frame (styled) | `egui::Frame::NONE` |
| `query_search_view.rs:28` | raw overlay | `egui::Area::new(egui::Id::new("query_find_overlay"))` |
| `query_search_view.rs:32` | frame (styled) | `egui::Frame {` |
| `query_shell_surface_view.rs:12` | raw disclosure | `egui::CollapsingHeader::new("Visual query builder")` |
| `query_view.rs:31` | frame (layout-only) | `egui::Frame::NONE` |

## result grid / output (8)

| file:line | pattern | snippet |
|---|---|---|
| `query_actions_surface_view.rs:40` | raw overlay | `let menu = egui::Area::new(egui::Id::new("query_actions_menu"))` |
| `query_actions_surface_view.rs:45` | frame (styled) | `egui::Frame {` |
| `query_output_actions_view.rs:220` | frame (styled) | `egui::Frame::NONE` |
| `query_output_tabs_view.rs:144` | frame (styled) | `let response = egui::Frame::NONE` |
| `query_results_surface_view.rs:35` | frame (styled) | `egui::Frame {` |
| `result_grid_cell_editor_surface_view.rs:96` | frame (layout-only) | `.frame(egui::Frame::NONE)` |
| `result_grid_cell_editor_surface_view.rs:168` | raw button | `egui::Button::new(icon_text(Icon::Calendar, "", theme.text_muted)).frame(false),` |
| `result_grid_cell_editor_surface_view.rs:189` | raw overlay | `let popup = egui::Area::new(popup_id.with("area"))` |

## schema workbench / compare (3)

| file:line | pattern | snippet |
|---|---|---|
| `schema_workbench_form.rs:717` | frame (styled) | `egui::Frame::NONE` |
| `schema_workbench_form.rs:904` | frame (styled) | `egui::Frame::NONE` |
| `schema_workbench_secondary_view.rs:156` | frame (styled) | `egui::Frame::NONE` |

## settings (2)

| file:line | pattern | snippet |
|---|---|---|
| `settings_surface_view.rs:42` | frame (styled) | `egui::Frame {` |
| `settings_surface_view.rs:73` | frame (styled) | `egui::Frame {` |

## shell / workspace (7)

| file:line | pattern | snippet |
|---|---|---|
| `activity_bar_view.rs:30` | frame (styled) | `.frame(egui::Frame {` |
| `shell_chrome_view.rs:164` | frame (styled) | `.frame(egui::Frame {` |
| `shell_frame_view.rs:17` | frame (styled) | `.frame(egui::Frame {` |
| `shell_statusbar_view.rs:31` | frame (styled) | `.frame(egui::Frame {` |
| `shell_topbar_view.rs:93` | raw overlay | `egui::Area::new(egui::Id::new(("frameless_window_resize", index)))` |
| `shell_topbar_view.rs:160` | frame (styled) | `.frame(egui::Frame {` |
| `workspace_tabs_surface_view.rs:58` | frame (styled) | `egui::Frame {` |

## table data editor (18)

| file:line | pattern | snippet |
|---|---|---|
| `table_data_filter_view.rs:35` | raw button | `if ui.small_button("×").on_hover_text("Remove filter").clicked() {` |
| `table_data_filter_view.rs:39` | raw button | `if ui.small_button("Clear all").clicked() {` |
| `table_data_placeholder_view.rs:22` | frame (styled) | `egui::Frame {` |
| `table_data_toolbar_surface_view.rs:221` | frame (styled) | `egui::Frame::NONE` |
| `table_data_toolbar_surface_view.rs:253` | frame (layout-only) | `.frame(egui::Frame::NONE)` |
| `table_data_toolbar_surface_view.rs:523` | raw overlay | `egui::Area::new(egui::Id::new("table_data_column_suggestions"))` |
| `table_data_toolbar_surface_view.rs:527` | frame (styled) | `egui::Frame {` |
| `table_data_toolbar_surface_view.rs:547` | frame (styled) | `let row = egui::Frame::NONE` |
| `table_ddl_surface_view.rs:33` | frame (styled) | `egui::Frame {` |
| `table_ddl_surface_view.rs:100` | frame (styled) | `egui::Frame {` |
| `table_mutation_dialog_surface.rs:102` | raw button | `if ui.small_button("Discard All").clicked() {` |
| `table_mutation_dialog_surface.rs:140` | raw button | `if ui.small_button("Revert Row").clicked() {` |
| `table_mutation_dialog_surface.rs:144` | raw button | `if ui.small_button("Remove Insert").clicked() {` |
| `table_mutation_dialog_surface.rs:168` | raw button | `if ui.small_button("Revert Cell").clicked() {` |
| `table_profile_surface_view.rs:12` | frame (styled) | `egui::Frame {` |
| `table_structure_surface_view.rs:30` | frame (styled) | `egui::Frame {` |
| `table_structure_surface_view.rs:72` | frame (styled) | `egui::Frame {` |
| `table_workspace_surface_view.rs:147` | frame (layout-only) | `pub(super) fn table_band_frame(theme: DbProTheme) -> egui::Frame {` |


## Source: `VERIFICATION.md`

# VERIFICATION — ui-rules-compliance

## W1 — §1 component-first

| Gate | Command | Result |
|---|---|---|
| `RULES.md` §6 | `grep -rn 'ui\.checkbox\|ui\.button(\|ui\.radio\|ui\.toggle_value\|egui::ComboBox\|egui::Slider\|ui\.text_edit_' crates/ui/src --include='*.rs' \| grep -v 'components/\|dev_tools/\|ui-rules:'` | PENDING |
| Architecture script | `bash scripts/check-ui-architecture.sh` | PENDING |
| Format | `cargo fmt --all -- --check` | PENDING |
| Check | `cargo check --workspace` | PENDING |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| Tests | `cargo test --workspace` | PENDING |
| Release build | `cargo build --release --locked -p db-pro-native` | PENDING |
| Extended §1 scan | deterministic scanner over all clusters | PENDING |

## Runtime evidence

| Surface | Sizes | Light | Dark | Notes |
|---|---|---|---|---|
| PENDING | 1280×800 / 1440×900 / 1920×1080 | PENDING | PENDING | `DB_PRO_CAPTURE_TO=... DB_PRO_WINDOW_SIZE=... ./target/debug/db-pro-native` |

## Behaviour-preservation review

PENDING — list any intentional visual delta with before/after screenshots; otherwise state
"no visual delta observed".

