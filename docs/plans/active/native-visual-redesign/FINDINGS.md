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
blue interaction accent `#0285ff`/`#339cff`, and neutral active/hover surfaces. The native shared
theme now uses these values and maps the Codex light/dark editor colors for SQL tokens.

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
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/679b00a3-1603-449f-8c48-209f065fa8be-screenshot.png`.
- The post-calibration restart after the final sidebar-radius/selection pass captured the full-window
  dark Welcome shell at `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/e9828e88-d95d-406c-97bd-e2a2cdb89749-screenshot.png`.
- The same rebuilt binary in Light mode captured the flat Welcome and Settings surfaces at
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/3f323363-de84-4199-a969-b90e8ce1fb73-screenshot.png`.
- The Welcome canvas is intentionally flat like Codex; the ER Diagram canvas keeps its own
  database-IDE grid as an intentional workspace affordance.
- The post-fix dark Monitor placeholder is captured at
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/2ff42d1d-12f1-48c1-8532-fe53e98b1154-screenshot.png`:
  icon, title, badge and description now form one compact vertical stack instead of the badge
  expanding into the remaining sidebar height.
- Light and dark Agent panels were traversed at
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/5c93c894-7585-4f9d-ac8e-a4f524495b69-screenshot.png`
  and `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/1d6c3719-bce3-417d-a2b3-bb5ea4fa00a1-screenshot.png`.
- Light and dark command palettes were traversed at
  `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/edd0c806-5d3a-4f40-a47b-1a07d36bda76-screenshot.png`
  and `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/b39a68e5-7686-46fa-b3cc-5cd1965cac36-screenshot.png`.

## Wave 3 runtime evidence

- Fresh maximized native launch and centered welcome: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/2f485840-c8ff-4106-8731-45fe03ea1ad8-screenshot.png`.
- Centered SQLite connection dialog with a real fixture path: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/18b43f96-421c-423a-bea0-3ce3ee2839b7-screenshot.png`.
- SQLite schema loaded from the fixture: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/17360595-801c-40cb-bd92-6f78a0d44c59-screenshot.png`.
- Data Editor before full-width grid sizing: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/9a0ea9f3-b66f-4b58-a583-c73e9ded9673-screenshot.png`.
- Data Editor after full-width grid sizing: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/c3fb448e-d9a1-4f7f-b3ff-bd0dae3dd867-screenshot.png`.
- Query Results after full-width grid sizing: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/f2e70db6-7005-478d-a877-8213785ba81c-screenshot.png`.
- Centered Insert row dialog over the real SQLite table: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bdb43a73-40fa-4d56-8c1c-e60ebc2ede5d-screenshot.png`.

## Wave 11 audit finding

### P2 — Empty metadata cards collapsed to content width

Indexes, foreign keys/dependencies and empty constraints used a shared card frame without a minimum
width. In the native runtime this left a small label-sized card in the top-left of the workspace,
which read as unfinished output and did not match the full-surface Codex composition. The focused fix
keeps the existing metadata rows, makes the card span the available width and adds a shared centered
Lucide icon/title/description stack for genuinely empty metadata states. Constraint detection now
tracks primary-key and `NOT NULL` rows explicitly so a table with metadata never shows the empty state.

The change is native presentation only: introspection, provider routing, database commands and
mutation behavior are unchanged.

## Wave 11 runtime evidence

- Rebuilt native binary: `target/debug/db-pro-native` after the metadata composition change.
- SQLite fixture: `/tmp/db-pro-native-ui-empty-state.sqlite`, connection `AuditEmptyState`; it
  contains populated table/relationship/trigger metadata alongside an empty-index surface.
- Dark Indexes surface: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/dc5b5da0-a94f-430e-bd93-a522f8e551f2-screenshot.png`.
- Dark Constraints surface with populated primary-key/`NOT NULL` rows: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/9238363c-69c8-4523-85e7-32a7969541b0-screenshot.png`.
- Light Indexes surface: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/fe376acf-c3b4-400b-b3aa-bed364fff931-screenshot.png`.

These screenshots show the full-width rounded neutral surface, centered Lucide treatment and
Codex-aligned light/dark tokens. The broader all-workspace light/dark traversal, provider review
and independent review remain open under the plan.

## Wave 12 audit finding

### P2 — Query output placeholders were bare labels inside oversized cards

The empty Results, Messages, Explain and History surfaces showed only a muted line of text. On the
maximized native window this created large low-information panels that did not share the icon/title/
description language already used by the Codex-aligned shell. Messages, Explain and History also
allowed their cards to collapse to content width. The focused fix reuses the shared empty-state
component, gives each output surface a semantic Lucide icon and explanatory copy, and makes the
secondary output cards span the available workspace width.

The query editor, execution, explain, history and populated result paths remain unchanged.

## Wave 12 runtime evidence

- Dark Results empty state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/5f356a79-c102-4fc0-a5e2-db3f2f8dcdb2-screenshot.png`.
- Dark Messages empty state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/c682d9f7-fff1-4292-9691-a40fe3efb132-screenshot.png`.
- Dark Explain empty state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/d9ca8f40-4ca6-4531-8b50-977dfb52b8fc-screenshot.png`.
- Dark History empty state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/e4fd58d7-362b-4c68-9dbf-8d1d08b63b9e-screenshot.png`.
- Light Results empty state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/81825589-8d70-40f4-b8c0-d669babe7052-screenshot.png`.
- Light Messages empty state: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/0d6cfb87-3898-4cd1-a8fa-f1b261eda267-screenshot.png`.

The screenshots show the same shared composition over Codex-aligned dark and light surfaces. The
broader all-workspace traversal, provider review and independent review remain open under the plan.
