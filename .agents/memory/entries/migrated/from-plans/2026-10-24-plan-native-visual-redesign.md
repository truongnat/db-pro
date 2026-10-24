---
schema_version: 1
id: "MEM-PLAN-native-visual-redesign-ba3a75f59cbe"
title: "Plan archive: Native Visual Redesign"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-visual-redesign]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "ba3a75f59cbedf6e0384a744d0a02cab4343fbd5971cbb12922bf15b5b36f957"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-visual-redesign/PLAN.md"
  - "docs/plans/active/native-visual-redesign/FINDINGS.md"
  - "docs/plans/active/native-visual-redesign/VERIFICATION.md"
  - "docs/plans/active/native-visual-redesign/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-visual-redesign/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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

### Wave 11 — metadata empty-state composition (completed in prior change)

- keep metadata cards at the full available workspace width instead of collapsing to their text;
- use one shared Lucide/Codex empty-state composition for empty indexes, foreign keys, dependencies
  and constraints;
- distinguish a genuinely empty constraint surface from a table that contains primary-key or
  `NOT NULL` metadata, without changing introspection or provider behavior;
- verify the composed states in both light and dark native runtime modes.

### Wave 12 — query output empty-state composition (completed in prior change)

- replace bare query output placeholders with the shared Lucide/Codex empty-state composition;
- keep Results, Messages, Explain and History output surfaces full-width and visually consistent;
- preserve the existing query editor, execution, explain and history behavior while changing only
  the empty presentation state;
- verify the output states in both dark and light native runtime modes.

### Wave 13 — statement-complete output state (current change)

- use the shared Lucide/Codex empty-state composition when a statement completes without result
  columns;
- remove the redundant `No result` header from an otherwise empty Results surface;
- preserve the existing execution result payload, timing, export behavior and provider routing;
- verify a real isolated SQLite statement-complete flow in the native runtime.

### Wave 14 — Query overflow actions (current change)

- keep Run as the only persistent Query command-bar action;
- move secondary query/editor controls into a Codex-style anchored overflow popover instead of a
  second always-visible toolbar row;
- use the shared native menu-row primitive and Lucide icons for action hierarchy and dark/light
  parity;
- preserve search, font-size, completion, snippets, save, folder, Explain and Agent behavior;
- make Escape close the overflow popover before closing other Query overlays;
- verify the default editor-first layout and overflow interaction in a real native runtime.

### Wave 15 — shared theme alignment

- reconcile the live primitive palette with the already-recorded Codex light/dark reference;
- remove the blue cast from dark neutral surfaces so shell, components and editor surrounds share
  the same neutral hierarchy;
- use role-specific accent foregrounds and accessible text contrast in both themes;
- keep this wave limited to shared theme tokens and theme regression coverage;
- leave native runtime visual verification pending for owner review.

### Wave 16 — Linux window controls

- replace Linux window-manager minimize, maximize/restore and close glyphs with compact Lucide
  controls that match the app's topbar, including focus and close-hover states;
- retain native move, maximize/restore, minimize, close and edge/corner resize behavior through
  egui viewport commands;
- keep native decorations on macOS and Windows;
- keep native runtime verification pending for owner review.

### Follow-up waves

- query/editor toolbar reduction and editor-first layout;
- data-grid interaction polish, including resize smoke coverage after Wave 9;
- ER canvas interaction controls and large-schema runtime smoke coverage;
- native runtime screenshots at all required dimensions and keyboard/DPI/clipboard/file-picker
  smoke coverage.

### Wave 17 — egui renderer diagnostics

- trace and document the locked egui/epaint/Glow pipeline, color/alpha representations, textures,
  framebuffer, filtering, DPI and MSAA;
- record upstream #2071, #7311 and #8283 against the exact dependency version;
- add a Gallery diagnostics page for alpha, physical-pixel strokes, text and overlays;
- keep production renderer changes pending until a fork-level test can exercise the real backend.

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
- Sparse SQLite table/query results retain a stable, legible grid viewport instead of coll

… [truncated for memory entry; see source file]


## Source: `FINDINGS.md`

# Native Visual Redesign — Findings

## Baseline finding — structural visual mismatch (UX-P1)

### Evidence

- Baseline native screenshot: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/7debe2b1-2127-4613-8c73-6b0d88ef3254-screenshot.png`.
- The baseline is light-first despite the native theme code having dark tokens.
- The topbar, workspace tabs, connection row and table controls are each rendered as separate
  outlined/filled controls, so the workbench hierarchy is weak.
- The Explorer keeps `Edit`, delete and `New connection` visible as routine buttons.
- The current frame therefore fails the explicit `goal-2.md` requirement that the redesign be
  immediately obvious before interaction.

### Failure scenario

Launch the native binary with an existing light theme preference and open a table. The first
frame still reads as the previous prototype: pale surfaces, button-heavy chrome, and a wide
sidebar consuming workspace area. A small token or spacing adjustment would not satisfy the
redesign requirement.

### Scope decision

Wave 1 changes only the native presentation layer and theme persistence version. Runtime state,
database commands, DTOs and provider behavior remain out of scope.

## Open risks

- egui does not expose the same accessibility tree as the React/Tauri surface; visual runtime
  screenshots are required for the acceptance decision.
- Native text input, clipboard, DPI scaling and close/reopen behavior need separate runtime
  evidence after the structural shell is in place.
- Existing persisted eframe state can hide the new default until the theme storage version is
  bumped and tested.

## Wave 15 — P2 shared palette drift

The recorded Codex contract calls for near-neutral light/dark surfaces and a blue interaction
accent. The live dark tokens had blue-gray neutrals (`#17191c` canvas, `#1d2024` panel,
`#303946` selected state) and a separate workstation-blue accent; light panel and primary text
also differed from the recorded reference. Since every UI surface reads these semantic roles,
the drift made component, shell and editor surroundings feel like separate visual systems.

Wave 15 realigns the shared light/dark neutrals and accent foregrounds at the primitive/semantic
token layer. Contrast is checked in source tests. Native visual confirmation remains pending.

## Wave 1 evidence

- Native screenshot at 1280×800 content: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bc48a216-856d-4464-80a3-d1290fa72785-screenshot.png`.
- Native screenshot at 1440×900 content: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/7676317c-9b76-4b65-a7a5-dd5898392cb5-screenshot.png`.
- Fresh launch after the native default-maximized change reported a top-aligned 1280×832
  window in the macOS harness, with the 1280×800 content viewport intact.

The 1280×832 result was traced to eframe restoring its persisted non-maximized frame after the
initial viewport request. The native entrypoint now sends `ViewportCommand::Maximized(true)` after
that restore. A fresh launch of the rebuilt binary reported a full-monitor 2473×1409 window.

## Wave 2 audit findings

### P2 — ER empty canvas lost the graph-grid language

The populated ER canvas paints a grid, but the empty/search states rendered only a flat editor
surface. This made the empty state look disconnected from the relationship-map workspace. The
smallest fix is presentation-only: paint the same grid behind the existing empty-state copy.

### P2 — Agent context badges clipped in narrow panels

The Agent context row used a horizontal scroll region. At the runtime panel width, the screenshot
showed the database/provider/table badges continuing past the visible edge, so the active context
was not fully legible. Wrapping the badges preserves all information without changing Agent
behavior.

### P2 — activity placeholders looked like unfinished full-width stripes

Transfers and Monitor intentionally have no provider implementation yet, but their `COMING SOON`
badge expanded across the centered column in the native screenshot. Centering the intrinsic badge
keeps the state honest while making it read as a compact status marker.

## Wave 3 audit findings

### P2 — persisted egui window positions displaced native dialogs

The New/Edit connection, delete confirmation and Insert row windows inherited their old egui
positions. On the maximized runtime they opened toward the upper-left instead of the active
workspace center, weakening the modal focus and making the first-run flow look broken. Center
anchoring is presentation-only and leaves the existing form and command flow unchanged.

### P2 — welcome content did not center in a top-down egui layout

The welcome view attempted to center its content with `ui.add_space`, but the parent layout was
vertical, so the offset only consumed height and left the card at the left edge of the maximized
workspace. A horizontal layout now owns the width offset and keeps the existing content intact.

### P2 — shared result grids collapsed to content width and height

With three SQLite rows, the Data Editor and Query Results cards rendered a narrow five-column strip
inside a full monitor and then collapsed below the rows. This left most of the productive surface
visually empty and made the grid read like a debug preview. Untouched columns now size to the
available viewport, manual divider resizing is preserved, and the table area reserves a stable
height for sparse result sets.

### P2 — SQL editor status leaked into non-editor workspaces

The native status bar always rendered `Ln 1, Col 1 · UTF-8`, including Data Editor, Schema Object,
ER Diagram and Welcome. On a real SQLite Data Editor screenshot this presented stale editor metadata
while the user was working with rows, which weakens context and makes the shell look unfinished.
The smallest safe fix is presentation-only: keep the editor metadata on Query and use a workspace
context label everywhere else.

### P2 — ER canvas did not fill the maximized workspace

The populated SQLite relationship map rendered only its node-derived content rectangle, leaving most
of the maximized central workspace as an unowned black area. This made the ER surface look like a
small preview even though the native window was full-size. The smallest safe fix is to size the
canvas to at least the current viewport while retaining the existing scroll overflow for larger
schemas.

### P2 — Query status bar used a hard-coded cursor position

After the context split, the Query workspace still showed `Ln 1, Col 1` regardless of where the
cursor was. This was directly observable in the native editor and made the status bar unreliable
for keyboard-first work. The smallest fix is to read egui's primary paragraph/offset cursor and
reset it when the active query document changes.

### P2 — Grid selection did not expose keyboard focus

The shared result grid supported click selection and clipboard actions, but no Arrow/Home/End
navigation. In addition, selecting a cell painted every cell in the row with the same strong
selected treatment, so the focused column was not visually identifiable. The smallest fix is a
pure navigation mapping over the existing filtered/sorted row indexes plus a separate active-cell
accent, with text inputs left to their normal egui keyboard handling.

### P2 — Large-schema “Show all” made ER search inert

The large-schema ER view kept the search field visible after the user selected “Show all”, but the
render path ignored the query while that mode was active. There was also no visible control to
return to the bounded focused map, so the user had to close and reopen the ER workspace to recover
search-first navigation. The smallest fix is to make a non-empty query leave explicit show-all mode
and add a “Focus search” action; table matching, render limits and provider behavior remain unchanged.

## Wave 8 runtime evidence

- Large-schema SQLite fixture: `/tmp/db-pro-native-er-search-XXXXXX.sqlite`, 202 tables and one
  relationship.
- Show-all state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/4bf1bff5-1d5f-48a8-b321-9926f845212b-screenshot.png`.
- Focused search after entering `order_items`: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/f32295e6-4ce3-410c-a9e7-6848de3200e7-screenshot.png`.

## Wave 9 audit findings

### P1 — Data Editor Enter did not commit the active cell

The active Data Editor input used `lost_focus && Enter` as its commit condition. In the real
native SQLite window, pressing Enter left the editor open and then clicking another cell changed
selection while the old editor remained visible. This made the displayed edit and active selection
disagree and could leave a user believing a change had been staged when it had not.

The focused fix is to commit on the Enter key event and commit the existing editor before row or
cell selection changes. Database mutation, transaction and provider paths remain unchanged.

### P2 — Data Editor copy actions ignored staged values

The grid rendered the staged value from its local change list, but `Copy cell` and `Copy row` read
the original query-result payload. A user could therefore copy a value different from the one on
screen. The fix resolves staged values only for the Data Editor and deliberately keeps Query Results
on the raw result payload.

## Wave 9 runtime evidence

- Rebuilt native binary: `target/debug/db-pro-native` from the Wave 9 source.
- SQLite fixture: `/tmp/db-pro-native-ui-audit-20260911.sqlite`, connection `AuditSQLite`, table
  `customers`.
- Pressing Enter after changing row 1 `name` to `Alice Updated` closes the editor while retaining
  the visible staged value and `pending changes` state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/f1886941-6af1-4d6c-8dd3-b79e4e7e4b93-screenshot.png`.
- Clicking row 2 then moves the active cell without reopening or leaving the row 1 editor behind:
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/586187f8-61be-44eb-9213-590b3ffa9fc6-screenshot.png`.
- Selecting the staged row 1 value and pressing `Copy cell` produced the clipboard payload
  `Alice Updated` via `pbpaste`.

## Wave 10 visual audit

The installed Codex desktop app is the visual reference for all remaining native work. Its local
bundle defines the calibrated palette used in this wave: light main/sidebar surfaces `#ffffff` and
`#f9f9f9`, dark main/sidebar surfaces `#181818` and `#212121`, primary text `#1a1c1f`/`#dfdfdf`,
blue interaction accent `#0285ff`/`#339cff`, and neutral active/hover surfaces. Wave 15 aligns the
shared surfaces and dark accent to this reference. The light accent is darkened to `#006fcc` so
normal text and white button labels meet 4.5:1 contrast; dark accent controls use a dark foreground
for the same reason.

The full requirement is not closed yet: every native workspace still needs a light/dark traversal,
and any database-IDE-specific density or grid treatment must be documented as an intentional
deviation before the baseline visual P1 can close.

## Wave 10 runtime evidence

- Dark native shell after the token/component rebuild: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/b98d3253-74ab-4828-9d07-402f09baaa64-screenshot.png`.
- Dark native Data Editor over the real `AuditSQLite` fixture, including the shared grid and
  active Data tab: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/ba0878ec-c0cf-448f-85b9-dd4b6551952b-screenshot.png`.
- Light native Data Editor over the real `AuditSQLite` fixture, including the shared grid and
  active Data tab: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/26a5b6fc-111e-4153-abfd-481cbfe4b4f2-screenshot.png`.
- Light Settings/Appearance surface confirms the Light control and shared component treatment:
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/679b00a3-1603-449f-8c48-209f065fa8be-screensho

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Native Visual Redesign — Verification

> **[CORRECTED 2026-09-14 — V01-06 evidence audit]** Audited verdict for this workstream: **`EVIDENCE_GAP`** — the runtime `PASS` claims in this file are not supported by retrievable evidence; see `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md` §2.

## Current state

Wave 1–10 implemented slices and runtime screenshot passes are recorded on
`feature/native-visual-redesign`; Wave 10 currently covers the calibrated shell and Data Editor
surfaces in both appearance modes. No overall feature completion claim has been made: all native
workspace traversal, independent review and any provider/runtime regression follow-up remain open.
The baseline screenshot and source evidence are recorded in `FINDINGS.md`.

## Commands

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | 2026-09-11, after Wave 1 implementation |
| `cargo check --workspace --offline` | PASS | 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; provider integration suites ran independently where configured, PostgreSQL live tests remain ignored without fixture env |
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11; rebuilt after default-maximized launch change |
| Native launch screenshot at 1280×800 | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bc48a216-856d-4464-80a3-d1290fa72785-screenshot.png`; window 1280×832 including titlebar, content 1280×800 |
| Native launch screenshot at 1440×900 | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/7676317c-9b76-4b65-a7a5-dd5898392cb5-screenshot.png`; window 1440×932 including titlebar, content 1440×900 |

## Runtime observations

- Fresh native process opened dark-first with the redesigned Explorer/welcome shell.
- Settings → Dark and activity-rail → Explorer were exercised through the real macOS window;
  the resulting screenshots show the selected state, grid canvas and context-driven connection actions.
- The native entrypoint now requests a maximized window by default with a 1280×800 inner-size
  fallback for environments that do not honor the maximize request, and reapplies maximize after
  eframe restores persisted window geometry.
- Fresh launch after the reapply fix reported a full-monitor 2473×1409 window in the macOS harness;
  Orca capture: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/1e95a1ff-8f08-4952-8613-7ee3c17684ba-screenshot.png`.
- Orca's macOS accessibility snapshot exposes only the native window controls for this egui
  surface, so widget-level verification is screenshot-based rather than accessibility-tree-based.

## Provider matrix

| Provider | UI behavior affected by Wave 1 | Automated evidence | Runtime evidence |
|---|---|---|---|
| PostgreSQL | None; presentation-only shell | Existing native/runtime suites | Existing walkthrough remains valid; rerun if connection interaction changes |
| SQLite | None; presentation-only shell | Existing native/runtime suites | Existing walkthrough remains valid; rerun if connection interaction changes |

## Wave 2 evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | 2026-09-11, after Wave 2 edits |
| `cargo check --workspace --offline` | PASS | 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 54 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; macOS xargs compatibility warnings are emitted by the existing script |
| Native ER empty state | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/0c439307-0f22-4749-8151-e9d9ddade3c9-screenshot.png`; grid is visible behind the empty state |
| Native Agent context | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/81be016b-5cb5-47f7-a543-b434bd652deb-screenshot.png`; Preview, provider, connection, driver, table count and schema badges remain legible |
| Native Transfers placeholder | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/6a932db9-1e89-4a2e-b712-daeb8830603e-screenshot.png`; status badge stays compact |
| Native Monitor placeholder | PASS | Orca capture `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/fbcd36fb-b737-46d9-8ce2-0c232d35bb36-screenshot.png`; status badge stays compact |

## Wave 3 evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11, rebuilt and relaunched native binary |
| `cargo fmt --all -- --check` | PASS | 2026-09-11, final Wave 3 check |
| `cargo check --workspace --offline` | PASS | 2026-09-11, final Wave 3 check |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11, final Wave 3 check |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 55 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; existing macOS xargs compatibility warnings emitted |
| SQLite connection test/save/connect | PASS | Real temporary SQLite fixture; Orca screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/17360595-801c-40cb-bd92-6f78a0d44c59-screenshot.png` shows connected status and loaded schema |
| SQLite Data Editor | PASS | Three rows, five typed columns and null/JSON/decimal values visible in `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/c3fb448e-d9a1-4f7f-b3ff-bd0dae3dd867-screenshot.png` |
| SQLite Query Results | PASS | Three rows rendered through the shared result grid in `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/f2e70db6-7005-478d-a877-8213785ba81c-screenshot.png` |
| Centered native Insert row dialog | PASS | `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bdb43a73-40fa-4d56-8c1c-e60ebc2ede5d-screenshot.png` |
| Full-window default | PASS | Fresh native window reported `2473×1409` at `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/4ff65e9a-5dcb-4267-8790-68c1a695b33f-screenshot.png` |

The provider matrix remains asymmetric by evidence: SQLite has the Wave 3 runtime walkthrough;
PostgreSQL remains presentation-only for this wave because no live fixture was authorized/configured.
The overall plan remains `IMPLEMENTING`: query/editor toolbar polish, keyboard/DPI/clipboard/file-picker
smoke coverage, wider provider walkthroughs and independent review are still open.

## Wave 4 evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | 2026-09-11, after Wave 4 edits |
| `cargo check --workspace --offline` | PASS | 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 56 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11, rebuilt native binary for runtime verification |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; existing macOS xargs compatibility warnings emitted |
| Native Welcome status context | PASS | Fresh full-window screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/ea9729df-8832-45f4-b8b5-d149cebd0d92-screenshot.png`; bottom-right shows `Workspace` |
| Native Table Structure status context | PASS | Fresh full-window screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/d65f21eb-62c2-4859-84d5-4fbed683fad0-screenshot.png`; bottom-right shows `Table Structure` |
| Native Data Editor status context | PASS | Fresh SQLite screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/2b40685c-dbc7-4c63-a3d0-e440e3402868-screenshot.png`; bottom-right shows `Data Editor` and no SQL line/encoding metadata |

Wave 4 is presentation-only and applies independently of the connected provider. PostgreSQL
runtime remains pending for the existing provider matrix; no PostgreSQL fixture was changed or
assumed for this status-bar check.

## Wave 5 evidence

| Check | Result | Evidence |
|---|---|---|
| Native ER canvas viewport fill | PASS | Fresh SQLite screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/a828ae55-27c1-48e0-b8f6-4915aad426db-screenshot.png`; relationship map grid fills the maximized central workspace at `2473×1409` |
| Native ER node and relationship rendering | PASS | Same screenshot shows `main.customers`, `main.orders`, one `customer_id` relationship and zoom controls after the canvas sizing change |
| `cargo fmt --all -- --check` | PASS | Run after Wave 5 source edits on 2026-09-11 |
| `cargo check --workspace --offline` | PASS | Run after Wave 5 source edits on 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | Run after Wave 5 source edits on 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 57 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11, rebuilt native binary for the fresh ER runtime check |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; existing macOS xargs compatibility warnings emitted |

Wave 5 is presentation-only. The canvas retains its existing scroll surface for content larger
than the viewport, and no provider-specific behavior was changed or inferred from SQLite.

## Wave 6 evidence

| Check | Result | Evidence |
|---|---|---|
| Native query cursor position | PASS | Fresh full-window screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/1f164c92-ff22-4f95-bea9-6328ef41c21f-screenshot.png`; clicking the fourth SQL line reports `Ln 4, Col 11 · UTF-8` |
| Query-document cursor reset | PASS | UI regression test `switching_query_documents_resets_editor_cursor_metadata` |
| `cargo fmt --all -- --check` | PASS | Run after Wave 6 source edits on 2026-09-11 |
| `cargo check --workspace --offline` | PASS | Run after Wave 6 source edits on 2026-09-11 |
| `cargo clippy --workspace --offline --all-targets -- -D warnings` | PASS | Run after Wave 6 source edits on 2026-09-11 |
| `cargo test --workspace --offline` | PASS | 2026-09-11; 186 core, 39 infrastructure, 25 SQLite integration, 58 UI, 21 Tauri, 4 runtime, 1 native; 10 PostgreSQL live tests ignored without fixture env |
| `cargo build --locked -p db-pro-native --offline` | PASS | 2026-09-11, rebuilt native binary for cursor smoke |
| `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` | PASS | 2026-09-11; 33 checks passed, 0 warnings/fails; existing macOS xargs compatibility warnings emitted |

Wave 6 is presentation-only. Query execution, provider capabilities and database state remain
unchanged; PostgreSQL runtime evidence remains pending under the existing provider matrix.

## Wave 7 evidence

| Check | Result | Evidence |
|---|---|---|
| Native grid keyboard focus | PASS | Fresh maximized native SQLite screenshot `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/9bf611e3-ce71-4bf5-b510-66fa0c46134f-screenshot.png`; selecting `name`, pres

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Linux window controls

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native UI implementation |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `0f1b34dbb147ce130a6ae5899c2523f816adb0b1` |
| Branch / PR | `main` / no PR; changes are uncommitted |
| Scope interpretation | Replace Linux window-manager control glyphs with app-owned Lucide controls while preserving window actions and resize behavior. |
| Out of scope | Changing macOS/Windows decorations, launching the app, and the separate pending Explorer scroll change. |

## 2. Progress checkpoint

- Current HEAD: `0f1b34dbb147ce130a6ae5899c2523f816adb0b1`; implementation is in the uncommitted working tree.
- Completed acceptance rows: Linux minimize/maximize/restore/close controls; accessible labels and focus ring; close hover treatment; drag and double-click maximize; edge/corner resize; Windows and macOS keep native decorations.
- Remaining acceptance rows: owner runtime verification; full all-surface review under the existing plan.
- Findings / risks: P0 0, P1 0, P2 1 — actual Linux compositor behavior and visual appearance are not runtime-verified.
- Tests already run: `cargo test -p db-pro-ui --lib` → 938 passed / 0 failed / 0 ignored, exit 0; focused control-click and resize regressions also passed.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `0f1b34dbb147ce130a6ae5899c2523f816adb0b1` (baseline; implementation remains uncommitted) |
| Commit list | none |
| File / surface inventory | `crates/native-app/src/main.rs` — Linux frameless viewport; `crates/ui/src/app_lifecycle.rs` — always show controls in Linux Settings and draw resize hit zones; `crates/ui/src/shell_topbar_view.rs` — Lucide controls, drag/maximize, resize requests and headless test; `PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md` — Wave 16 record. |
| Acceptance mapping | OS frame diagnosis → baseline/source inspection; control actions and accessible names → `draw_window_controls`; buttons through resize overlays → `linux_window_buttons_emit_viewport_commands_through_resize_overlay`; resize boundary → `frameless_window_resize_handles_only_start_native_resize_at_the_edge`; runtime appearance and compositor behavior → pending owner verification. |
| Commands and counts | `cargo fmt --all -- --check` exit 0; `cargo check -p db-pro-ui -p db-pro-native` exit 0; `cargo test -p db-pro-ui --lib` 938/0/0 exit 0; `cargo clippy -p db-pro-ui --all-targets -- -D warnings` exit 0; `cargo build --release --locked -p db-pro-native` exit 0, binary not launched; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` 15 pass / 1 inherited warning / 0 fail; `git diff --check` exit 0. |
| CI run IDs / status | not run |
| Known limitations | No native runtime verification by request; no commit created. The owner should verify hover/focus appearance and drag/minimize/maximize/restore/close/edge resize on the target Linux compositor. |
| Migrations / config implications | Linux window decorations are disabled; application-level controls replace the OS titlebar controls. No persisted data changes. |
| Out-of-scope changes | Pending Explorer mouse-wheel fix remains in the working tree and is not included in this window-controls finding. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review yet |
| Verdict | n/a |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 1 pending runtime verification; inherited: n/a |
| Findings | P2 runtime appearance/compositor behavior pending owner check |
| CI disposition | not run |
| Next task(s) unblocked | Owner runtime verification |

## 5. Research / audit handoff

- Source date: 2026-10-01
- Source references: `crates/native-app/src/main.rs`, `crates/ui/src/shell_topbar_view.rs`, egui 0.29 `ViewportBuilder` and `ViewportCommand` API in the local Cargo registry.
- Factual findings: the baseline configured the native viewport without changing OS decorations; egui provides decoration, drag, resize, minimize, maximize and close commands.
- Inference: the unattractive Linux control icons came from the window manager because DB Pro did not draw these controls.
- Decision / recommendation: use custom Lucide controls on Linux only; retain native controls on macOS and Windows.
- Unresolved questions: how the controls and native resize requests behave on the owner's Linux compositor at runtime.
- Downstream tasks activated: owner runtime verification under Wave 16.

## 6. Tổng kết (Vietnamese summary)

Đã thêm bộ nút cửa sổ Lucide riêng cho Linux, giữ thao tác kéo, phóng to/khôi phục và resize; test UI, Clippy và release build đều qua. Chưa mở app theo yêu cầu, nên giao diện và hành vi trên compositor Linux vẫn cần chủ ứng dụng xác minh.

