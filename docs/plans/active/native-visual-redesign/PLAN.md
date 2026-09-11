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
behavior. The native visual contract follows the installed Codex desktop app as the reference
for hierarchy, color, iconography, spacing, typography, states and both appearance modes.

## Codex visual contract

The native surface must be calibrated against the Codex desktop reference in both light and dark
mode. This is an acceptance rule for every remaining wave, not a suggestion to reuse React/Tauri
code.

- use one native token layer for surfaces, borders, text hierarchy, accent and semantic states;
- keep the Codex hierarchy: quiet near-flat surfaces, hairline separators, neutral active rows,
  blue interaction accent, compact rounded controls and generous whitespace;
- use the existing Lucide icon font consistently, with monochrome icons at the same visual weight
  as Codex navigation/actions; do not introduce mixed icon families or emoji substitutes;
- verify hover, active, focus, disabled, selected and destructive states in both light and dark
  mode before calling a surface visually complete;
- keep typography, control height, spacing and corner radii centralized in native components so
  screens cannot drift into bespoke styling.

## Scope

### Wave 1 — shell composition (this change)

- dark-first visual tokens and persisted theme-version reset;
- compact application header with workspace context and icon actions;
- narrow activity rail with grouped navigation and clear active state;
- tree-like Explorer header, connection rows, schema/object groups and compact add action;
- integrated tab strip with quiet inactive tabs and a restrained active indicator;
- native empty/welcome composition that makes the workspace intentional without a database.

### Wave 2 — productive surface polish (completed in prior change)

- keep the native window maximized by default even after eframe restores an older frame, with the
  existing inner-size fallback;
- make the ER empty state read as the same canvas as a populated relationship map;
- wrap Agent context badges so narrow panels do not clip the active database context;
- keep Transfers and Monitor placeholders compact and intentional instead of rendering a full-width
  unfinished-state stripe.

### Wave 3 — full-window data surfaces (completed in prior change)

- keep the default full-window launch while making shared Data Editor and Query Results grids use
  the available viewport instead of collapsing to the content width;
- auto-size untouched columns for the current viewport, preserve manual divider resizing, and keep
  the horizontal-scroll fallback for narrow windows or wide result sets;
- reserve a stable table viewport for sparse result sets and remove implementation-only grid wording
  from the user-facing status copy;
- center native connection/delete/insert dialogs and the welcome workspace content when egui restores
  persisted positions or the app opens on a maximized monitor.

### Wave 4 — context-aware status bar (completed in prior change)

- keep SQL editor metadata (`Ln`, `Col`, encoding) scoped to the Query workspace;
- show a truthful workspace context for Data Editor, Schema Object, ER Diagram and Welcome surfaces;
- preserve connection, provider, duration and runtime-error status reporting without changing commands
  or provider behavior.

### Wave 5 — full ER canvas (completed in prior change)

- make the ER canvas and grid fill the visible native workspace before content overflow requires scrolling;
- keep the existing pan, zoom, fit and node-opening interactions unchanged;
- preserve the bounded render policy for large schemas and avoid introducing provider-specific behavior.

### Wave 6 — live query cursor status (completed in prior change)

- report the actual SQL editor line and column from egui cursor state;
- reset cursor metadata when switching, creating or closing query documents;
- keep the existing status-bar context split and avoid changing query execution behavior.

### Wave 7 — keyboard grid focus (completed in prior change)

- move the selected result-grid cell with Arrow keys and jump within a row with Home/End;
- preserve original result-row identity when filtering or sorting changes the visible projection;
- distinguish the active cell from its selected row without changing copy, edit or mutation semantics.

### Wave 8 — ER search mode recovery (completed in prior change)

- leave explicit large-schema “Show all” mode when the user enters a non-empty search query;
- expose a compact “Focus search” action so users can return to the bounded focused map without
  closing and reopening the ER workspace;
- preserve the existing render limit, table matching and provider-neutral diagram behavior.

### Wave 9 — staged grid interaction correctness (completed)

- commit an active Data Editor cell when Enter is pressed;
- commit the active cell before row/cell selection changes so the editor and selection cannot drift;
- copy staged values in the Data Editor while preserving raw query-result values in the Query
  workspace;
- keep the fix inside the native shared result-grid state path without changing database mutation
  or transaction semantics.

### Wave 10 — Codex visual parity (completed in prior change)

- audit every native screen and shared component against the Codex light/dark reference;
- calibrate the native theme tokens, icon sizes/weights, typography, spacing, radii and interaction
  states as one coherent system;
- capture equivalent native light and dark screenshots and record any intentional database-IDE
  deviations explicitly.

### Wave 11 — metadata empty-state composition (current change)

- keep metadata cards at the full available workspace width instead of collapsing to their text;
- use one shared Lucide/Codex empty-state composition for empty indexes, foreign keys, dependencies
  and constraints;
- distinguish a genuinely empty constraint surface from a table that contains primary-key or
  `NOT NULL` metadata, without changing introspection or provider behavior;
- verify the composed states in both light and dark native runtime modes.

### Follow-up waves

- query/editor toolbar reduction and editor-first layout;
- data-grid interaction polish, including resize smoke coverage after Wave 9;
- ER canvas interaction controls and large-schema runtime smoke coverage;
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

## Acceptance for Wave 3

- Shared native result grids use the full available width at the default maximized viewport while
  retaining explicit user resize behavior and a narrow-window overflow path.
- Sparse SQLite table/query results retain a stable, legible grid viewport instead of collapsing to
  a small content-height strip.
- Native dialogs and the welcome content remain centered on a fresh maximized launch.
- Rust fmt/check/clippy/tests pass and runtime evidence covers SQLite schema, Data Editor, Query
  Results and the centered Insert row dialog.

## Acceptance for Wave 4

- SQL editor-only status metadata is absent from non-query workspaces.
- Data Editor, Schema Object, ER Diagram and Welcome surfaces expose a truthful native context label.
- Rust fmt/check/clippy/tests pass and a fresh native screenshot confirms the Data Editor context.

## Acceptance for Wave 5

- ER canvas grid fills the maximized workspace when the rendered schema fits inside the viewport.
- Larger rendered content still overflows through the existing scroll surface.
- Rust fmt/check/clippy/tests pass and a fresh native screenshot confirms the full canvas.

## Acceptance for Wave 6

- Query status bar reports the active cursor position instead of a hard-coded placeholder.
- Switching query documents starts the status position at line 1, column 1 until the editor reports a new cursor.
- Rust fmt/check/clippy/tests pass and a fresh native screenshot confirms a non-default cursor position.

## Acceptance for Wave 7

- Arrow keys move the selected grid cell across visible rows and columns, while Home/End stay within the current row.
- Keyboard navigation does not steal input from the grid filter or active cell editor.
- The active cell has a distinct visual treatment from the selected row.
- Rust fmt/check/clippy/tests pass and a fresh native SQLite screenshot confirms non-default cell focus.

## Acceptance for Wave 8

- A large SQLite schema can enter “Show all” mode, then return to focused search by typing a query
  or activating “Focus search”.
- The focused result uses the existing table/column matching and render policy without changing
  database commands or provider behavior.
- Rust fmt/check/clippy/tests pass and a fresh native SQLite screenshot confirms the full flow.

## Acceptance for Wave 9

- Enter commits a Data Editor edit and the edited value remains visible as a staged change.
- Moving to another row or cell commits the previous editor before changing selection.
- Data Editor `Copy cell` and `Copy row` use the displayed staged values; Query Results continue to
  copy their original result payload.
- Rust fmt/check/clippy/tests pass, the native binary is rebuilt, and a fresh SQLite runtime check
  confirms Enter, selection transition and clipboard behavior.

## Acceptance for Wave 10

- Native `DbProTheme` uses the Codex light/dark neutral surface, text, border, semantic and blue
  accent tokens, with no purple-only active-state dependency.
- Shared native components use the Codex-aligned control heights, typography scale and rounded
  interaction surfaces without introducing a second icon family.
- Fresh native screenshots confirm the calibrated shell and Data Editor in both light and dark mode.
- Rust fmt/check/clippy/tests pass and the native binary is rebuilt; remaining workspace/provider
  review stays explicitly open until every native surface has been traversed.

## Acceptance for Wave 11

- Empty metadata surfaces span the available native workspace instead of rendering as a tiny
  content-width card in the top-left corner.
- Empty-state icon, title and description use the shared Lucide font, Codex-aligned theme tokens,
  typography and spacing in both light and dark modes.
- Populated indexes, relationships and constraints keep their existing metadata rows unchanged.
- Rust fmt/check/clippy/tests, native build and clean-code diff scan pass; fresh SQLite runtime
  screenshots cover the changed empty states in both appearance modes.
