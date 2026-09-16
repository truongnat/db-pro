# DB Pro Native UI Audit Baseline — 2026-09-16

Baseline SHA: `e3f477b71c105e5b27ba217ef2c73c38edc2146c`

## Why this audit exists

Recent product work has expanded DB Pro very quickly across Query, Files/IDE, Schema Workbench, Data, Compare, Routines, Agent and additional activities. Source-level feature completion has outpaced visual/runtime quality control. Several surfaces can therefore be functionally present while still feeling unfinished, visually inconsistent, cramped, confusing or low quality when used as a real desktop app.

The new UI program treats runtime visual quality as a first-class acceptance condition. No UI ticket should close only because it compiles or because helper/component tests pass.

## Source-level findings

### 1. Design-system migration is incomplete

The repository already has a modular native component system under `crates/ui/src/components/`, but legacy helpers remain widely used and a `legacy.rs` compatibility layer still exports primitives such as `compact_button`. This creates two visual dialects and makes surface-level polish inconsistent.

### 2. Raw egui controls remain in product surfaces

Examples on the baseline SHA include raw `ui.button(...)` usage in:

- `result_grid_edit.rs` — Apply / Cancel
- `schema_workbench.rs` — Generate Markdown / Generate HTML
- `schema_workbench_form.rs` — Apply / Cancel
- `explorer_view.rs` context menu actions

These bypass the canonical component language and often produce visibly different height, padding, font, hover and focus treatment.

### 3. Visual semantics are still hard-coded outside the theme

`explorer_folders.rs` embeds RGB values for object categories such as Views and Functions. `chart_view.rs` owns a separate fixed palette. These may be intentional semantic colors, but they are not governed by one token/contrast contract and can diverge between light/dark themes.

### 4. Existing UI plans already identified missing/polish gaps

`docs/plans/active/native-core-ui-modernization/PLAN.md` identifies missing or incomplete coverage for rich tooltips, skeleton/loading states, switch/segmented controls, keyboard chips, status dots, toasts, search inputs, richer empty states and sidebar interaction polish.

`docs/plans/active/native-shadcn-ui-system/FINDINGS.md` also records design consistency and component discoverability as unresolved concerns.

### 5. Recent fixes show source review alone is insufficient

Recent commits had to repair real UX regressions after runtime use, including query editor layout/chrome and file-backed SQL execution (`1024952`), stale Explorer schema state and palette navigation (`e3f477b`). This reinforces the requirement for interactive visual traversal rather than accepting source-level implementation as UI completion.

## Audit standard

Every UI hardening ticket created from this baseline should verify, where applicable:

- dark + light theme;
- 1280×800, 1440×900 and 1920×1080;
- empty, loading, success, error and disabled states;
- long names / overflow / truncation;
- keyboard-only navigation;
- mouse hover, pressed and focus states;
- scroll behavior;
- resize behavior;
- provider/context badges;
- no raw provider SQL/business logic added to renderers;
- no duplicate local styling system;
- screenshot evidence from the actual native app.

## Closure rule

A UI ticket is not DONE unless actual native runtime evidence has been reviewed. Compilation/tests are necessary but not sufficient.
