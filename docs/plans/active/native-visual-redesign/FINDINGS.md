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

## Codex visual parity requirement

The installed Codex desktop app is now the visual reference for all remaining native work. The
current native surface is structurally close in its dark token direction, but parity is not yet
proven: the native screenshot still has denser database-IDE chrome and stronger grid emphasis than
the Codex reference, and a native light-mode comparison has not been captured. Wave 10 remains
open for colors, icon weight, spacing, typography, radii and interaction states in both modes.

## Wave 3 runtime evidence

- Fresh maximized native launch and centered welcome: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/2f485840-c8ff-4106-8731-45fe03ea1ad8-screenshot.png`.
- Centered SQLite connection dialog with a real fixture path: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/18b43f96-421c-423a-bea0-3ce3ee2839b7-screenshot.png`.
- SQLite schema loaded from the fixture: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/17360595-801c-40cb-bd92-6f78a0d44c59-screenshot.png`.
- Data Editor before full-width grid sizing: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/9a0ea9f3-b66f-4b58-a583-c73e9ded9673-screenshot.png`.
- Data Editor after full-width grid sizing: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/c3fb448e-d9a1-4f7f-b3ff-bd0dae3dd867-screenshot.png`.
- Query Results after full-width grid sizing: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/f2e70db6-7005-478d-a877-8213785ba81c-screenshot.png`.
- Centered Insert row dialog over the real SQLite table: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bdb43a73-40fa-4d56-8c1c-e60ebc2ede5d-screenshot.png`.
