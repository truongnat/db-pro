---
schema_version: 1
id: "MEM-SRC-54baa712c11ff542"
title: "Source mirror: docs/user-guide/query-workspace.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/user-guide/query-workspace.md"
source_sha256: "54baa712c11ff5429827a04bb18638f6a167e90ae1484c8912e006df2d7d4d0e"
mirror_body_sha256: "54baa712c11ff5429827a04bb18638f6a167e90ae1484c8912e006df2d7d4d0e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Query Workspace

The Query workspace is where you write, run, and inspect SQL. It combines a SQL
editor, a connection/schema picker, an output dock (results, chart, messages,
explain plan, history), and transaction controls.

Open it with the **Query** tab in the workspace, or press `Ctrl/Cmd+T` for a new
query document. `Ctrl/Cmd+N` opens the New Connection dialog instead if you have
no connection yet.

## Screen layout

Top to bottom:

1. **File breadcrumb** — shown when the document was loaded from a `.sql` file.
2. **Context strip** — connection/schema chip, Run/Stop, Explain, Format, row
   limit, and the "More actions" (…) menu.
3. **Visual query builder** — optional panel toggled from the actions menu.
4. **Transaction bar** — appears when a transaction is open or toggled on.
5. **Parameters panel** — appears when the SQL contains bind parameters.
6. **Editor** — the SQL text editor with its own status bar at the bottom.
7. **Output dock** — Results / Chart / Messages / Explain / History tabs. It can
   be resized by dragging its divider, docked to the bottom or the right, and
   maximized.

## Context strip

| Control | What it does |
|---|---|
| Connection/schema chip | Shows `connection / schema`. Click it to open the context picker and switch connection or schema. Shows a green dot when connected, a red dot for `production`/`prod` environments, and a warning icon when nothing is connected. |
| **Run** (▶) | Runs the statement under the cursor, or the selected text if you have a selection. Becomes **Stop** (■) while a query runs if the driver supports cancellation; otherwise it shows **Running**. |
| **Explain** | Runs `EXPLAIN` on the current statement and opens the Explain tab. |
| **Format** | Reformats the SQL in place (also `Ctrl/Cmd+Shift+F`). |
| Row limit | Dropdown (`100 / 500 / 1,000 / 100,000 rows`) that caps the rows fetched by the next editor run. |
| **⋯** (More query actions) | Opens the Query actions menu described below. |

### Query actions menu (⋯)

- **Run query** / **Run selection** — same as the Run button.
- **Run script** — executes every statement in the document in order.
- **Format SQL** — same as the Format button.
- **Explain** — same as the Explain button.
- **Ask agent** — hands the query to the Agent surface.
- **Save query** (`Ctrl/Cmd+S`) / **Save query as** (`Ctrl/Cmd+Shift+S`) — saves
  to the query library; "Save as" asks for a name and optional folder.
- **Visual builder / Hide visual builder** — toggles the visual SELECT builder.
- **Find in SQL** — opens the editor find overlay (also `Ctrl/Cmd+F`).
- **Show/Hide transaction controls** — toggles the transaction bar.
- **Decrease / Increase font size** — adjusts the editor font (shown in px).
- **Generate SQL Prediction** — requests an AI completion on demand.
- **AI prediction** — `Off`, `Subtle` (default), or `Eager`. The note under it
  is accurate: it sends the SQL around your cursor plus schema context to your
  configured AI provider. With no provider key configured, nothing is sent.
- **SQL snippets** — opens the snippet panel (see below).
- **New folder** — creates a folder in the query library from the text field
  next to the button.

## The SQL editor

- **Syntax highlighting** follows the active driver dialect (PostgreSQL or
  SQLite).
- **Auto-pairing** of brackets and quotes; `Backspace` deletes a pair.
- **`Ctrl/Cmd+/`** toggles `--` line comments on the selected lines.
- **Find overlay** (`Ctrl/Cmd+F`): a floating "Search…" box over the editor.
  Typing jumps to the first match; `Enter` next match, `Shift+Enter` previous,
  `Esc` or the ✕ button closes. Shows `n/total`, or `0/0` in red when there are
  no matches.
- **Completion popup**: opens automatically ~150 ms after you type an identifier
  character (outside strings/comments), on `.`, or manually with `Ctrl/Cmd+Space`.
  Navigate with ↑/↓ or PageUp/PageDown, accept with `Enter` or `Tab`, dismiss
  with `Esc`. Items carry a kind tag (table, column, keyword, snippet…).
- **Hover** over a table or column name shows a card with schema details.
  `Ctrl/Cmd+click` an identifier opens the referenced table or schema object.
- **Signature help** pops up inside function calls showing parameters.
- **Snippets**: type a trigger like `sel*`, `upd*`, `ins*`, `del*`, `exp*`,
  `idx*` and accept the suggestion; `Tab`/`Shift+Tab` move between `${n:…}`
  placeholders, `Esc` ends the session.
- **AI ghost text**: in Subtle or Eager mode a suggestion appears after ~300 ms
  of idle editing. `Tab` accepts all, `Alt+→` accepts one word, `Alt+↓` accepts
  one line, `Esc` dismisses. `Ctrl/Cmd+Alt+Space` requests one manually.
- **Undo/redo**: `Ctrl/Cmd+Z` / `Ctrl/Cmd+Shift+Z`. `Ctrl/Cmd+A` selects all;
  `Tab`/`Shift+Tab` indent/unindent a selection.

### Editor status bar

`Ln n, Col m`, selected-line range, driver name, current schema, a transaction
indicator (`Auto-commit` / `Manual` / `Transaction · n pending`), `n parameters`
(click to edit bind values), `n diagnostics` (click to see SQL warnings), and a
"Show output" button when the output dock is closed.

## Running queries

- **Run** (`F5` by default; the Run button tooltip shows `Ctrl/Cmd+Enter`) runs
  the statement at the cursor or the current selection. The app picks the
  statement containing the cursor — you don't have to select it.
- **Run script** (`Ctrl/Cmd+Shift+Enter`) runs every statement; each statement
  produces a result tab (`Result 1`, `Result 2`, …).
- While a query runs, the button shows **Stop** (press it or `Esc` to cancel).
  On drivers without cancellation support it shows "Running" and explains why
  in the tooltip.
- Running with no connection shows "Connect to a database before running";
  running empty SQL shows "Query is empty".
- The row-limit dropdown applies only to interactive editor runs — saved tasks
  and other callers keep their exact SQL.

### Destructive-statement guard

`DROP`, `TRUNCATE`, `DELETE` without `WHERE`, `DO`, `CALL`, `EXECUTE`, and
comparable script contents are held before anything reaches the server. A
"Run Destructive Statement?" dialog shows the exact SQL (truncated at 600
characters) with **Run Destructive Statement** and **Cancel**. Nothing runs
until you confirm.

## Bind parameters

When your SQL uses placeholders (`$1`, `$2`, … on PostgreSQL-style drivers; `?`
where the driver supports it — `?` is not treated as a placeholder on
PostgreSQL or SQL Server), a parameters panel lists each one with an input and
a secret toggle. The status bar shows `n parameters`. Values are kept in memory
per document only; secret values are never persisted with drafts.

## Transactions

Toggle the transaction bar from the status bar transaction label or the actions
menu ("Show transaction controls").

- **Auto-commit active** is the default. Switch to manual mode to get a
  **Begin Transaction** button.
- Inside a transaction the bar shows `Uncommitted Transaction` or
  `Transaction Active`, the isolation level (`[READ COMMITTED]`), a pending
  change count, and **Commit** / **Rollback** buttons.
- An open transaction blocks disconnecting: you'll see "Open transaction blocks
  disconnect — Commit or Rollback first" with a Dismiss button.

## Output dock

Five tabs — **Results**, **Chart**, **Messages**, **Explain**, **History** —
with a row-count badge on Results and a message count on Messages. When the
strip is narrow (docked right) the tabs collapse to icons. Chrome buttons:
**Close output** (✕), **Maximize/Restore output**, **Dock to bottom / Dock to
right** (when docked), and a drag handle to resize. `Ctrl/Cmd+Shift+R` opens
the dock on the Results tab and moves keyboard focus into the grid.

### Results

Each run shows `{n} rows · {ms} ms`, a pin button, **Export**, and an
"Open results in a workspace tab" button. Multi-statement scripts get a row of
result tabs; right-click a result tab for **Pin/Unpin Tab**, **Close**, **Close
Other Tabs**.

Empty states: "Statement completed" for commands that return no rows, and a
"No results" prompt when nothing has run yet.

**Grid toolbar** (query results): a "Filter visible rows…" box with a live
`n rows` counter and a clear (✕) button; copy buttons; a **Copy as ▾** menu
(visible rows as CSV, JSON, Markdown, or SQL INSERT); a record/value inspector
toggle; an advanced cell-inspector button (shown when a cell is selected); and
a help (ⓘ) tooltip. Read-only results (e.g. views or statements without a
usable primary key) show a warning notice instead of edit affordances.

**Grid interactions:**

| Action | Effect |
|---|---|
| Click a cell | Select it. Click the `#` gutter to select whole rows. |
| Right-click a cell | Context menu: Copy Cell Value `Ctrl/Cmd+C`, Copy Row (TSV) `Ctrl/Cmd+Shift+C`, Copy Selected Rows (/with Headers/as JSON/as Markdown/as INSERT SQL), Copy Row as JSON/CSV; when editable: Edit Cell `Enter`/`F2`, Set to NULL, Revert Cell, Revert Row / Undo Delete, Duplicate Row, Delete Row `Delete`/`Backspace`; plus Use column as row id, Open referenced row (foreign keys), Filter by this value, Sort Ascending/Descending. |
| Right-click a column header | Sort Ascending (A → Z) / Descending (Z → A) / Clear Sort, Copy Column Name, Copy Column Values, Add Filter, Hide Column / Show Columns, Move Column Left/Right, Reset Column Order, Reset Column Widths, Auto Size, Reset Layout. |
| Click a column header | Cycle sort ascending → descending → none. (In the table-data surface, `Shift+click` adds a multi-column sort.) |
| Drag a column divider | Resize the column. |
| `Ctrl/Cmd+A` | Select all rows. `Esc` clears the selection. |
| `Ctrl/Cmd+S` / `Ctrl/Cmd+Z` in the grid | Apply / discard staged edits. Paste (`Ctrl/Cmd+V`) into a selected cell stages an edit on editable results. |
| `Tab` while editing | Commit the cell and move to the next. |

Editable cells open an in-place editor matched to the column type: a text field,
a dropdown for booleans and enum-like columns, or a date picker for
date/datetime columns. Staged changes highlight the row until you apply
(`Ctrl/Cmd+S`) or discard (`Ctrl/Cmd+Z`) them.

The **record inspector** (panel toggle in the toolbar) shows the selected row
as a Field/Value table; click a value to copy it. The **cell inspector**
(🔍 button) opens a larger viewer for long values.

### Chart

Turns the result set into a chart with pickers for **Type** (Bar, Line, Area,
Scatter, Pie), **X** column, **Y** column (numeric columns only), **Agg**
(None, Count, Sum, Average, Min, Max), and an optional **Series** column.
The footer reports the plotted point count, the cap, and skipped
null/non-numeric Y values. With no rows it shows "No data to chart".

### Messages

Notices, warnings, and execution details returned by the server, deduplicated.
Empty state: "Query notices and execution details will appear here."

### Explain

**Explain** and **Explain analyze** buttons, a **Raw JSON** toggle, and
**Copy plan** once a plan exists. Parsed PostgreSQL plans render as a plan tree
with planning time, runtime stats (for ANALYZE), and up to 8 advisor findings
(hotspots/warnings); unparseable plans fall back to raw text.

`EXPLAIN ANALYZE` actually executes the statement. On drivers where ANALYZE
executes, the pane first shows a warning and requires ticking "I understand
this will execute the query" before **Run EXPLAIN ANALYZE** becomes clickable.

### History

Recent executions for the session, newest first, filtered by a search box and
capped at 50 shown rows (`n total` on the right). Each entry shows a ✓ OK /
✕ FAIL badge, duration in ms, row or affected-row count, and a two-line SQL
preview. Click the preview to open it in a new editor; **Replay** re-runs it.

The **History** section in the sidebar shows the persisted execution log
(`{connection} · {schema} · {outcome} · {time} · {ms}`), with outcome filters
(ok / error / cancelled), per-day grouping (Today / Yesterday / date), and an
"Open as new query" action.

### Export

**Export** on the Results pane opens a dialog: pick a format (CSV, TSV, JSON,
Markdown, INSERT, COPY, Excel), enter an output path, and confirm. Exporting
writes only the rows the result actually holds — if the row cap truncated the
result, the message tells you how many rows matched versus how many were
written. If the file already exists you get **Overwrite** / **Keep existing
file**; a failed export leaves no partial file.

## Visual query builder

Opened from the actions menu. It builds a `SELECT` from picked tables/columns
against the active dialect, shows a SQL preview, and offers **Apply to editor**
(copies the generated SQL into the document without running it), **Import from
editor** (parses a supported `SELECT` back into the builder — unsupported SQL is
refused with an error and your text is untouched), and **Clear**.

## Keyboard shortcuts

All of these are remappable in **Settings → Keybindings**; `mod` means
`Ctrl` on Windows/Linux and `Cmd` on macOS.

| Command | Default |
|---|---|
| New query | `mod+T` |
| Run query | `F5` |
| Run script (all statements) | `mod+Shift+Enter` |
| Format query | `mod+Shift+F` |
| Save query | `mod+S` |
| Save query as | `mod+Shift+S` |
| Find in editor | `mod+F` |
| Toggle comment | `mod+/` |
| Focus results grid | `mod+Shift+R` |
| Quick open | `mod+P` or `mod+K` |
| Command palette | `mod+Shift+P` |
| Stop running query | `Esc` (where supported) |

## Notable states and limits

- **Disconnected**: the chip turns into a warning "No connection" and Run is
  disabled with a tooltip telling you to connect first.
- **Row cap**: editor runs are capped at the selected limit (max 100,000);
  `EXPLAIN`, exports, and history are unaffected — export only writes the rows
  that were fetched.
- **Completion** is suppressed inside strings and comments.
- **AI prediction** requires a configured provider; the egress note in the
  actions menu states exactly what leaves the machine.
