# Tasks: 100% Core Component Standardization

## Phase 1: Shell & Navigation Chrome

- [x] Task 1: Standardize ActivityBar, TopBar, and StatusBar to Core Workspace Primitives

**Description:** Unify the app's top bar, left activity rail, and bottom status bar with the canonical `components::workspace::{ActivityBar, StatusBar}` and core `Button` / `Badge` primitives. Expand `ActivityBarItemKind` in `components::workspace` to cleanly support all 9 activity tabs (`Explorer`, `Queries`, `Schema`, `Diagram`, `Compare`, `History`, `Audit`, `Agent`, `Settings`) so that `activity_bar_view.rs` and `shell_statusbar_view.rs` stop using ad-hoc frames and buttons.

**Acceptance criteria:**
- [ ] `ActivityBarItemKind` encompasses all active activities with canonical icons and tooltips.
- [ ] `activity_bar_view.rs` uses `ActivityBar::new(...)` or shared rail button primitives with exact theme tokens.
- [ ] `shell_statusbar_view.rs` replaces ad-hoc frame margins and raw layouts with canonical status bar primitives.
- [ ] `shell_topbar_view.rs` uses standard `Button` variants and `Badge` tags for environment and driver labels.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui --test app_tests`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Launch app, verify clicking all activity icons switches views, hover tooltips show correct shortcuts, and status bar displays connection information cleanly.

**Dependencies:** None

**Files likely touched:**
- `crates/ui/src/activity_bar_view.rs`
- `crates/ui/src/shell_statusbar_view.rs`
- `crates/ui/src/shell_topbar_view.rs`
- `crates/ui/src/components/workspace/ui.rs`
- `crates/ui/src/components/workspace/handler.rs`

**Estimated scope:** Medium: 3-5 files

---

- [x] `cargo check --workspace` passes without warnings.
- [x] `cargo test -p db-pro-ui` passes.
- [x] Activity rail and status bar render with unified tokens and responsive layout.

---

## Phase 2: Explorer & Object Tree Surfaces

## Task 2: Standardize Explorer Toolbar, Filter, and Schema Feedback Surfaces [DONE]
- [x] Completed and verified with tests passing

**Description:** Refactor `explorer_toolbar_view.rs` and `explorer_schema_feedback_view.rs` to replace legacy button helpers (`compact_icon_button`, `secondary_button_with_icon`) with canonical `Button::new(theme).size(ButtonSize::IconSm)`. Migrate schema loading feedback from raw labels and spinners to `Spinner::new(theme)` and `EmptyState`.

**Acceptance criteria:**
- [ ] Refresh and filter toggle buttons use `Button::new(theme).size(ButtonSize::IconSm).variant(ButtonVariant::Ghost)`.
- [ ] Context menus and active indicator dots attach seamlessly to canonical `Button` responses.
- [ ] Schema failure and loading states use canonical `Alert` or `Spinner` from `components::feedback`.
- [ ] Zero calls to `legacy::compact_icon_button` or `legacy::secondary_button_with_icon` in these files.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui explorer`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Expand connection in Explorer, click refresh, toggle object filters, verify context menu opens.

**Dependencies:** Task 1

**Files likely touched:**
- `crates/ui/src/explorer_toolbar_view.rs`
- `crates/ui/src/explorer_schema_feedback_view.rs`

**Estimated scope:** Small: 2 files

---

## Task 3: Standardize Object Folders, Agent Context, and Empty States [DONE]
- [x] Completed and verified with tests passing

**Description:** Standardize table folders, files agent context view, and explorer empty states to use canonical `EmptyState`, `Badge`, and `Button` primitives instead of `legacy::empty_state` and `legacy::compact_button_with_icon`.

**Acceptance criteria:**
- [ ] Explorer empty state uses `EmptyState::new("No connections", theme).description(...).action("New connection", ...)` or canonical Button.
- [ ] `files_agent_context_view.rs` uses `Button::new` for action popups.
- [ ] Table folder item counts and type indicators use canonical `Badge`.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Explorer with zero connections displays canonical empty state; adding connection updates badge counts.

**Dependencies:** Task 2

**Files likely touched:**
- `crates/ui/src/explorer_surface_view.rs`
- `crates/ui/src/explorer_table_folder_view.rs`
- `crates/ui/src/files_agent_context_view.rs`

**Estimated scope:** Small: 3 files

---

## Checkpoint: Phase 2
- [x] All tests in `db-pro-ui` pass.
- [x] Explorer tree navigation, filter workbench, and empty states render strictly with core components.

---

## Phase 3: Query Workspace & Editor Surfaces

## Task 4: Standardize Query Editor Actions, Run Controls, and Context Header [DONE]
- [x] Completed and verified with tests passing

**Description:** Standardize all query editor header bars, context pickers, and run controls (`query_actions_surface_view.rs`, `query_run_control_view.rs`, `query_context_view.rs`, `query_status_bar_surface_view.rs`). Ensure all buttons use canonical `Button::new` with explicit variants (`Default` for Run, `Secondary` for Stop/Explain, `Ghost` for icons).

**Acceptance criteria:**
- [ ] Run, Stop, Explain, Format buttons use canonical `Button` with correct loading states and icons.
- [ ] Query status bar and prediction mode selectors use canonical `Button` / `SegmentedTabs`.
- [ ] Cancellation capability and tooltip helpers preserve accessibility labels and shortcuts.
- [ ] Eliminate `compact_button` and raw `ui.button` invocations in these views.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui query`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Run a query (⌘↵), observe button state transition to Running/Stop, test Explain and Format.

**Dependencies:** Task 1

**Files likely touched:**
- `crates/ui/src/query_actions_surface_view.rs`
- `crates/ui/src/query_run_control_view.rs`
- `crates/ui/src/query_context_view.rs`
- `crates/ui/src/query_status_bar_surface_view.rs`

**Estimated scope:** Medium: 4 files

---

## Task 5: Standardize Query Output Actions, Tabs, and Results Surface Controls [DONE]
- [x] Completed and verified with tests passing

**Description:** Replace `compact_button`, `compact_icon_button`, and raw `ui.button` in `query_output_actions_view.rs`, `query_output_tabs_view.rs`, and `query_results_surface_view.rs` with canonical `Button`, `Badge`, and `UnderlineTabs` / `SegmentedTabs`.

**Acceptance criteria:**
- [ ] Result tab headers, pin/unpin toggles, close buttons use canonical `Button::new(theme).size(ButtonSize::IconSm)`.
- [ ] Context menus on tabs (Pin, Close, Close Others) preserve click handling and menu closing.
- [ ] Explain plan copy and history clear buttons use canonical `Button`.
- [ ] Row counts and execution times render with canonical `Badge`.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui query`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Execute multiple queries, switch result tabs, test pin/unpin, verify export and copy buttons.

**Dependencies:** Task 4

**Files likely touched:**
- `crates/ui/src/query_output_actions_view.rs`
- `crates/ui/src/query_output_tabs_view.rs`
- `crates/ui/src/query_results_surface_view.rs`

**Estimated scope:** Medium: 3 files

---

## Task 6: Standardize Query Dialogs (Save, Destructive Run, Folder Delete) [DONE]
- [x] Completed and verified with tests passing

**Description:** Refactor `query_dialog_surface_view.rs`, `query_save_dialog_surface_view.rs`, and `query_folder_delete_dialog.rs` to use canonical `Dialog`, `AlertDialog`, or `DestructiveOperationDialog` from `components::dialog` and `components::transaction`.

**Acceptance criteria:**
- [ ] Destructive SQL warning modal uses canonical `DestructiveOperationDialog` or `AlertDialog` with `Destructive` variant.
- [ ] Save Query dialog uses canonical `Dialog` with header, body, footer, and standard input fields.
- [ ] Folder delete confirmation uses canonical `AlertDialog`.
- [ ] All action buttons use canonical `Button` with appropriate primary, outline, and ghost variants.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui query`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Attempt to run `DROP TABLE` to verify safety dialog; test Save Query dialog layout and cancellation.

**Dependencies:** Task 5

**Files likely touched:**
- `crates/ui/src/query_dialog_surface_view.rs`
- `crates/ui/src/query_save_dialog_surface_view.rs`
- `crates/ui/src/query_folder_delete_dialog.rs`

**Estimated scope:** Medium: 3 files

---

## Checkpoint: Phase 3
- [x] All query tests pass (`cargo test -p db-pro-ui query`).
- [x] Entire Query workspace (Editor, Run Controls, Result Grid, Output Panes, Dialogs) is 100% compliant with core components.

---

## Phase 4: Table Workstation Surfaces

## Task 7: Standardize Table Structure, Indexes, Metadata, and Relations Panels [DONE]
- [x] Completed and verified with tests passing

**Description:** Modernize table inspection panels (`table_structure_surface_view.rs`, `table_indexes_surface_view.rs`, `table_metadata_surface_view.rs`, `table_relations_surface_view.rs`). Ensure all data lists use canonical `Table::new(&columns, theme)` and cell badges use canonical `Badge::new(theme)`.

**Acceptance criteria:**
- [ ] Header metrics (column count, PK, FK, index counts) use canonical `Badge` with semantic variants (`Warning` for PK, `Secondary` for counts).
- [ ] Grid headers and rows use canonical `Table` with consistent row heights and selection behavior.
- [ ] Section labels use canonical `SectionHeader` instead of `legacy::section_label`.
- [ ] Copy and edit action triggers use canonical `Button::new`.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui table`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Navigate to Table Workstation, inspect Structure, Indexes, Foreign Keys, and Dependencies tabs.

**Dependencies:** Task 1

**Files likely touched:**
- `crates/ui/src/table_structure_surface_view.rs`
- `crates/ui/src/table_indexes_surface_view.rs`
- `crates/ui/src/table_metadata_surface_view.rs`
- `crates/ui/src/table_relations_surface_view.rs`

**Estimated scope:** Medium: 4 files

---

## Task 8: Standardize Table Profile, DDL, and Mutation Toolbars [DONE]
- [x] Completed and verified with tests passing

**Description:** Standardize `table_profile_surface_view.rs`, `table_ddl_surface_view.rs`, and `table_data_toolbar_surface_view.rs`. Ensure DDL script copy buttons, profile metric tables, and inline WHERE filter inputs use canonical `Button`, `Badge`, `Input`, and `Table` primitives.

**Acceptance criteria:**
- [ ] DDL toolbar uses canonical `Badge` for `CREATE TABLE` and `Button` for Copy / Refresh.
- [ ] Table Profile numerical summaries render using canonical `Table` and `Progress` bars.
- [ ] Data grid toolbar WHERE badge and condition input use canonical `Badge` and `Input`.
- [ ] Add Row, Delete Row, and Discard changes buttons use canonical `Button` variants (`Default`, `Destructive`, `Ghost`).

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui table`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Open DDL tab, copy script, switch to Profile tab, edit WHERE clause in Data view.

**Dependencies:** Task 7

**Files likely touched:**
- `crates/ui/src/table_profile_surface_view.rs`
- `crates/ui/src/table_ddl_surface_view.rs`
- `crates/ui/src/table_data_toolbar_surface_view.rs`
- `crates/ui/src/table_data_mutation_toolbar_view.rs`

**Estimated scope:** Medium: 4 files

---

## Checkpoint: Phase 4
- [x] All table tests pass (`cargo test -p db-pro-ui table`).
- [x] All 6 table tabs and the data grid toolbar use 100% canonical components.

---

## Phase 5: Specialized Workstations

## Task 9: Standardize ER Diagram Toolbar, Controls, and Design Panels [DONE]
- [x] Completed and verified with tests passing

**Description:** Replace legacy buttons (`compact_button`, `compact_icon_button`, `secondary_button`) in `diagram_view.rs`, `diagram_canvas_view.rs`, and `diagram_design_panel_view.rs` with canonical `Button::new(theme)`. Replace empty states with `EmptyState`.

**Acceptance criteria:**
- [ ] Zoom in/out, reset zoom, auto-fit, and minimap toggle buttons use canonical `Button::new(theme).size(ButtonSize::IconSm)`.
- [ ] Neighborhood hop toggles and "Show all N tables" buttons use canonical `Button`.
- [ ] Design Mode draft inputs and "Add draft table" / "Apply" buttons use canonical `Input` and `Button`.
- [ ] Diagram empty search results use canonical `EmptyState`.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui diagram`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Open ER Diagram, test zoom in/out, fit to viewport, toggle Design Mode, search for table.

**Dependencies:** Task 1

**Files likely touched:**
- `crates/ui/src/diagram_view.rs`
- `crates/ui/src/diagram_canvas_view.rs`
- `crates/ui/src/diagram_design_panel_view.rs`

**Estimated scope:** Medium: 3 files

---

## Task 10: Standardize Schema Compare, Data Diff, and Sync Preview Surfaces [DONE]
- [x] Completed and verified with tests passing

**Description:** Refactor `schema_compare_view.rs` from legacy button/input helpers (`compact_button_with_icon`, `secondary_button_with_icon`, `input_full_width`, `section_label`) to canonical `Button`, `Input`, `Badge`, `Card`, and `SectionHeader` primitives.

**Acceptance criteria:**
- [ ] "Diff vs snapshot" and "Snapshot" actions use canonical `Button::new`.
- [ ] Difference count summaries (Added, Removed, Changed) use canonical `Badge` with semantic variants.
- [ ] Data diff target connection, schema, and table inputs use canonical `Input::new`.
- [ ] Filter pills ("all", "added", "removed", "changed") use canonical `Button` / `SegmentedTabs`.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Open Schema Compare, take snapshot, run diff, check styling of change summary chips.

**Dependencies:** Task 1

**Files likely touched:**
- `crates/ui/src/schema_compare_view.rs`

**Estimated scope:** Small: 1 file

---

## Task 11: Standardize Audit, FDW, and Event Trigger Management Surfaces [DONE]
- [x] Completed and verified with tests passing

**Description:** Modernize secondary database management views (`audit_surface_view.rs`, `fdw_surface_view.rs`, `event_trigger_surface_view.rs`) to use canonical `SectionHeader`, `Button`, `Table`, and `AlertDialog` primitives.

**Acceptance criteria:**
- [ ] Section headers use canonical `SectionHeader::new`.
- [ ] Refresh, Export, Create, Drop, and Close preview buttons use canonical `Button`.
- [ ] Creation and Drop confirmation modals use canonical `AlertDialog` / `Dialog`.
- [ ] Inventory and audit logs render using canonical `Table` or `LogViewer`.

**Verification:**
- [ ] Tests pass: `cargo test -p db-pro-ui`
- [ ] Build succeeds: `cargo check -p db-pro-native`
- [ ] Manual check: Navigate to Audit and FDW tabs, verify headers, refresh actions, and modal layout.

**Dependencies:** Task 1

**Files likely touched:**
- `crates/ui/src/audit_surface_view.rs`
- `crates/ui/src/fdw_surface_view.rs`
- `crates/ui/src/event_trigger_surface_view.rs`

**Estimated scope:** Medium: 3 files

---

## Checkpoint: Phase 5
- [x] All specialized workstation suites pass.
- [x] ER Diagram, Schema Compare, Audit, and FDW views use 100% canonical components.

---

## Phase 6: Legacy Codebase Elimination

## Task 12: Audit Call-sites and Deprecate/Remove `crates/ui/src/components/legacy.rs` [DONE]
- [x] Completed and verified with tests passing

**Description:** Conduct a complete workspace audit to ensure zero remaining references to `crates/ui/src/components/legacy.rs` helpers across all views. Remove obsolete functions from `legacy.rs` (or delete the file if completely superseded), clean up wildcards in `app.rs`, and verify code quality gates.

**Acceptance criteria:**
- [x] Zero references to `compact_button`, `compact_icon_button`, `secondary_button`, `primary_button`, `input_full_width`, `section_label`, `panel_frame`, etc. across `crates/ui/src/`.
- [x] `app.rs` imports only canonical component symbols.
- [x] `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` all pass cleanly.
- [x] Locked native release builds cleanly: `cargo build --release --locked -p db-pro-native`.

**Verification:**
- [ ] Tests pass: `cargo test --workspace`
- [ ] Build succeeds: `cargo build --release --locked -p db-pro-native`
- [ ] Lint check: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Code search: `grep -rn "compact_button\|input_full_width\|section_label" crates/ui/src/` returns zero matches outside tests.

**Dependencies:** Tasks 1-11

**Files likely touched:**
- `crates/ui/src/components/legacy.rs`
- `crates/ui/src/components/mod.rs`
- `crates/ui/src/app.rs`

**Estimated scope:** Small: 3 files

---

## Checkpoint: Complete
- [x] 100% of UI components across DB Pro use canonical Design System primitives.
- [x] Zero regressions in functionality, shortcuts, or database safety.
- [x] Full workspace quality gates pass without warnings.
