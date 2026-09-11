# Native Visual Redesign — Plan

Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`. Current: `IMPLEMENTING`.
Branch: `feature/native-visual-redesign`. Source brief: [`goal-2.md`](../../../goal-2.md).

## Evidence and decision

The latest native chain (`1946d59`, followed by the provider-status cleanup commits) completed
`native-ui-foundation` and `native-ide-redesign`. A fresh native screenshot still showed a light
shell with browser-like tabs, permanently visible connection actions, a wide activity/sidebar
composition, and a large button-heavy table workspace. `goal-2.md` explicitly rejects another
polish pass and requires an immediately visible structural redesign.

## Goal

Recompose the native egui presentation into a dark-first database workbench with a narrow
activity rail, calm tree explorer, integrated IDE tab strip, compact workspace chrome, and
stronger productive surfaces, while preserving the existing runtime/task-bridge/database
behavior.

## Scope

### Wave 1 — shell composition (this change)

- dark-first visual tokens and persisted theme-version reset;
- compact application header with workspace context and icon actions;
- narrow activity rail with grouped navigation and clear active state;
- tree-like Explorer header, connection rows, schema/object groups and compact add action;
- integrated tab strip with quiet inactive tabs and a restrained active indicator;
- native empty/welcome composition that makes the workspace intentional without a database.

### Wave 2 — productive surface polish (current change)

- keep the native window maximized by default even after eframe restores an older frame, with the
  existing inner-size fallback;
- make the ER empty state read as the same canvas as a populated relationship map;
- wrap Agent context badges so narrow panels do not clip the active database context;
- keep Transfers and Monitor placeholders compact and intentional instead of rendering a full-width
  unfinished-state stripe.

### Follow-up waves

- query/editor toolbar reduction and editor-first layout;
- data-grid/status-bar visual hierarchy;
- query/result grid/status-bar polish;
- ER canvas interaction controls;
- native runtime screenshots at all required dimensions and keyboard/DPI/clipboard/file-picker
  smoke coverage.

## Non-goals

- no React/Tauri UI work;
- no database service, DTO, provider, or task-bridge behavior changes;
- no autonomous Agent execution or new database capabilities;
- no removal of the existing native screens until parity and runtime evidence are proven.

## Acceptance for Wave 1

- Fresh native launch is dark by default and visibly differs from the baseline screenshot.
- Header, activity rail, Explorer and tabs read as one desktop workbench rather than stacked
  form controls.
- Existing connection selection, schema selection, table opening, query opening, Agent toggle,
  palette actions and theme/settings actions retain their current behavior.
- Native UI tests cover theme-version reset and the new shell invariants.
- Rust fmt/check/clippy/tests pass for the changed workspace.
- Runtime screenshots are captured at 1280×800 and 1440×900 before Wave 1 can leave
  `RUNTIME_VERIFY`.

## Acceptance for Wave 2

- The default native launch requests and reapplies a maximized window after persisted eframe state,
  while retaining a usable 1280×800 inner-size fallback when the host does not honor maximization.
- Empty ER, Agent context, Transfers and Monitor surfaces remain legible at the current narrow
  native panel width and at the captured wide viewport.
- Rust fmt/check/clippy/tests pass for the changed workspace and fresh runtime screenshots confirm
  the corrected surfaces.
