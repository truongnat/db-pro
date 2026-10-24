---
schema_version: 1
id: "MEM-SRC-5de66f13f996f2b6"
title: "Source mirror: docs/user-guide/table-workbench.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/user-guide/table-workbench.md"
source_sha256: "5de66f13f996f2b6c7636612f87007f1ad100a7f7d930b777df772cdd061ade1"
mirror_body_sha256: "5de66f13f996f2b6c7636612f87007f1ad100a7f7d930b777df772cdd061ade1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Table Workbench

The Table Workbench is the workspace that opens when you select a table in DB Pro. It bundles a data grid with safe staged editing, structure and metadata browsers, column profiling, DDL inspection, plus three schema-level companions: the ER diagram, Schema Compare, and the Schema Workbench (plan → preview → apply object mutations).

## Opening a table

- Click a table in the Explorer to select it; the table workspace opens as a workspace tab.
- Right-click a table in the Explorer for the full context menu:

| Menu item | What it does |
|---|---|
| View Data | Open the workspace on the **Data** tab |
| View Structure | Open the workspace on the **Structure** tab |
| Modify Table (Alter)... | Open the Schema Workbench with an alter plan for this table |
| Drop Table... | Open a drop-table plan/confirmation flow |
| Generate SQL: SELECT * / INSERT / UPDATE / DELETE | Create a SQL editor tab prefilled with the statement |
| View DDL / CREATE Script | Open the workspace on the **DDL** tab |
| Copy Qualified Name / Copy Table Name | Copy `schema.table` or `table` to the clipboard |
| Ask Agent about table | Open the AI assistant with this table as context |
| Refresh Schema (F5) | Re-introspect the table's schema |

## Workspace header and tabs

The top band shows a breadcrumb — `connection › schema › table` — and a row-count badge (`N rows`) when known. Three buttons sit at the right:

| Button | Tooltip | Effect |
|---|---|---|
| Ask Agent | Open AI Assistant with table context | Opens the agent surface scoped to this table |
| New Query | Open SQL Editor for this table | Creates a query tab for this table |
| Refresh | Reload table metadata and rows | Reloads both the structure and the data page |

Below the header are eight view tabs: **Data · Structure · Profile · Indexes · Foreign Keys · Constraints · Dependencies · DDL**.

## Data tab

### Toolbar

The top toolbar combines editing controls, export, and an inline WHERE editor:

- **Add Row** — opens the Insert Row dialog. Disabled (with a tooltip explaining why) when inserts are unavailable for the current result or connection.
- **Export** — exports the *visible* rows (see below).
- **WHERE** field — a `SELECT` condition editor. Type a `WHERE` clause body (e.g. `amount > 100`); column-name completion pops up automatically (use ↑/↓, Enter or Tab to accept, Esc to dismiss). Press **Run** or Enter to execute `SELECT … WHERE <condition>`; an empty condition selects all rows.
- **Run** is blocked while staged changes exist — a status message tells you to apply or discard them first. Running a destructive statement requires confirmation first.

### Status indicators

| Indicator | Meaning |
|---|---|
| `N pending · +i ~u -d` chip | Staged edits exist; click to open the Pending Changes dialog |
| `Read-only` | Connection or result is not writable |
| `Table has no primary key; safe row editing is unavailable.` | Updates/deletes are disabled; inserts still work. Use the cell menu's **Use column as row id** to pick an alternate identity column |
| `N rows selected` | Multi-row selection is active |
| `CONFLICT · Mutation #n failed` | A mutation failed; hover for the error. Buttons **Reload Row**, **Discard Local Change**, **Resolve Conflict**, and **Retry** appear |

### Active filters

Server-side filters render as chips under the toolbar (`column = value`, `column contains x`, `IS NULL`, …). Each chip has an **×** to remove it; **Clear all** removes every filter. Create a filter via the column-header menu (**Add Filter**, which primes a draft) or a cell's context menu (**Filter by this value**).

### The grid

- Click a column header to sort; right-click it for the header menu: **Sort Ascending/Descending, Clear Sort, Add Filter, Copy Column Name, Copy Column Values, Move Column Left/Right, Reset Column Order, Reset Column Widths, Hide Column, Show Columns, Reset Layout, Auto Size**.
- Drag column edges to resize; reorder/hide state is per-table layout that you can reset from the header menu.
- Multi-select rows with the row gutter; a selection badge appears in the toolbar.

### Editing

Rows are edited by *staging* changes — nothing writes to the database until you press **Save**.

- Double-click a cell (or press **Enter/F2**, or choose **Edit Cell** from the context menu) to edit inline. The editor adapts to the column type: plain input, a dropdown for booleans/enums, and a date picker for date/time columns. **Enter** commits the cell, **Tab** commits and moves right, **Esc** cancels.
- Read-only columns (e.g. generated columns) show **Read-only Column** in the menu with the reason.
- **Set to NULL**, **Revert Cell**, **Revert Row / Undo Delete**, **Duplicate Row**, and **Delete Row** round out the cell menu.
- Changes accumulate in the ChangeSet. Save with the **Save** button or `Ctrl+S` (`⌘S` on macOS); discard with **Discard** or `Ctrl+Z`. Discarding more than one change asks for confirmation.

### Cell context menu (right-click a cell)

| Group | Items |
|---|---|
| Copy | Copy Cell Value (`Ctrl+C`), Copy Row (TSV) (`Ctrl+Shift+C`), Copy Selected Rows, Copy Selected Rows with Headers, Copy Selected Rows as JSON / Markdown / INSERT SQL, Copy Row as JSON, Copy Row as CSV |
| Edit (writable results) | Edit Cell, Set to NULL, Revert Cell, Revert Row / Undo Delete, Duplicate Row, Delete Row (Delete/Backspace) |
| Navigate & refine | Use column as row id, Open referenced row (follows the FK to the referenced table, filtered to the row), Filter by this value, Sort Ascending, Sort Descending |

### Footer

The footer holds **Save** (`Ctrl+S`), **Discard** (`Ctrl+Z`), **Refresh** (`F5`), the page-range label, pagination (**First · ‹ · › · Last**), and a page-size selector (50/100/250/500/1000 per page — changing it resets to the first page). Navigation is disabled while staged changes exist.

### Dialogs

- **Insert Row** — per-column fields with PK/FK badges, data types, `*required`/`nullable` markers and default expressions. Quick actions: **Clear All**, **Fill Required**, **Generate All** (sample mock values), and per-field **NULL**, **UUID/Now/Gen**, and clear buttons. **Stage Insert** adds the row to the ChangeSet; it is inserted on Save. Columns with a write restriction show `Read-only — <reason>`.
- **Pending changes** — lists every staged change grouped by row (`Row PK: […]` or `New row · temporary identity #n`), with **Revert Cell**, **Revert Row**, **Remove Insert**, and **Discard All**.
- **Unapplied Changes** — shown when navigation would lose staged edits: **Apply Changes**, **Discard Changes**, or **Cancel**.
- **Row Conflict Resolution** — appears when the database row changed underneath your staged edit. Shows primary key, then a 3-way diff per changed column (Original Baseline / Local Staged (Mine) / Database Current / Status). Resolve with **Keep Mine (Overwrite DB)**, **Use Database (Discard Mine)**, **Reload & Retry**, or **Discard Local Change**. Delete conflicts offer **Discard Staged Delete** and **Reload & Retry Delete**.
- **Export Results** — pick a format (CSV, TSV, JSON, Markdown, INSERT, COPY, Excel), set the output path, and Export. Only the currently loaded page is exported. Existing files prompt an **Overwrite / Keep existing file** choice.
- **Value inspector** — opened via the record view's **Inspect** button per field. Modes: Raw / Pretty / Tree for JSON, Raw / Hex / Base64 for bytea, Raw / Pretty otherwise. Writable cells can **Apply to ChangeSet**; bytes can be saved via **Export bytes…**.
- **Record view** — switch the presentation to see a single selected row as a field list (`Record · row N`), with per-field **Inspect**.

### Empty and error states

`Loading data for <table>…` while fetching; `<table> has no rows` (with a hint to Add Row) when empty; `Data for <table> could not be loaded` with the error and a **Retry** button on failure.

## Structure tab

Shows metric badges (column count, `PK: …`, index count, foreign-key count, check constraints, dependencies, row count) and a searchable **Columns** table (`Search columns or types…` filters name, type, and default):

| Column | Notes |
|---|---|
| # | Ordinal position |
| Column Name | Icon marks PK (key), FK (link), or plain column; hover for `IDENTITY`/`GENERATED`/`UNIQUE`/collation |
| Data Type | Monospace; hover for the full database type |
| Nullable | `NULL` / `NOT NULL` badge |
| Key | `PK` / `FK` / `UNIQUE` badge |
| Default Expression | Default value; hover when truncated |
| Actions | **Alter** — opens the ALTER COLUMN planner in the Schema Workbench; disabled with a reason tooltip when the driver doesn't support it |

Loading shows `Loading table structure…`; failures show `Table structure could not be loaded` with the error.

## Profile tab

Profiles the **currently loaded page** (badge: `Page sample`) — it is a bounded sample, not a full-table scan. If no rows are loaded, it tells you to open the Data tab first. Per column it shows: **Completeness** (% full + NULL rate), **Distinct** (count · %), **Pattern** (classification badge), **Range · Min / Max**, and **Avg · Sum** when the column is fully numeric.

## Indexes tab

Searchable list (`Filter indexes…`) of indexes: **Index Name** (unique indexes get a checkmark icon; hover for the full DDL definition), **Indexed Columns**, **Method**, **INCLUDE**, **Predicate**, and **Status** (`PRIMARY KEY` / `Enforces uniqueness` / `Active index`).

## Foreign Keys tab

Searchable list of outgoing foreign keys: **Constraint Name**, **Source Columns**, **Target Table**, **Target Columns**, and **Action** (`UPDATE <rule> · DELETE <rule>` plus `DEFERRABLE`/`DEFERRABLE INITIALLY DEFERRED`; hover shows the `MATCH` option). The link icon **opens the referenced table** in its own workspace.

## Constraints tab

Constraint rows grouped and filterable by type — tab buttons **All / Primary Key / Unique / Check / Not Null** plus a text filter. Table columns: **Type** (badge), **Constraint Name**, **Target Columns / Expression**, **Details**. Covers PRIMARY KEY, UNIQUE indexes, CHECK constraints (with their SQL expression), and per-column NOT NULL.

## Dependencies tab

Incoming/outgoing dependency relations with direction filter tabs (**All / Depends On (Outgoing) / Depended By (Incoming)**), a search box, and a `matching of total` counter. Columns: **Direction** (`DEPENDS ON` / `DEPENDED BY`), **Kind** (`TABLE`, `VIEW`, `FK`, `TRIGGER`, `FUNCTION`, `SEQUENCE`), **Object Name** (with an external-link button opening table dependencies in a workspace), **Relationship Details**.

## DDL tab

- Toolbar: `DDL SCRIPT · CREATE TABLE` badge with **Refresh** (re-generate from latest schema), **Open in Query** (open the script in the SQL editor to run it), and **Copy** (copy to clipboard).
- The script editor is an editable preview, but **Apply DDL** is intentionally disabled (`Not enabled in v0.1 — run DDL with Open in Query`) — to execute modified DDL, use Open in Query.
- Loading/failed states mirror the other tabs (`Loading DDL for <table>…`, `DDL for <table> could not be loaded`).

## ER Diagram

Open the **Diagram** activity/tab to see the relationship map for the loaded schema.

- **Canvas**: scroll wheel zooms toward the pointer (pinch supported), drag empty space to pan, drag a table card to reposition it (positions persist per connection). Clicking a node highlights its relationships and dims the rest.
- **Toolbar**: `N tables` / `N relationships` badges, an `Arranging…` badge while layout computes.
- **Large schemas**: a search box (`Find table or column…`) builds a focused neighborhood map instead of rendering everything. Toggle **1 hop / 2 hops** for neighborhood depth, or press **Show all N tables** / **Focus search** to switch modes. Non-matching search shows a **Clear search** button.
- **Floating zoom strip** (top-right): `−`, zoom % (click to reset to 100%), `+`, reset view, **Fit** (fit the whole graph or the active subset).
- **Minimap** (bottom-right): overview of the whole world; click or drag to recenter.
- **Toolbar icons**: **Design Mode** (pencil-ruler) toggles the draft panel; the reset-layout icon clears saved manual positions and rebuilds the grid.
- Empty states: `No map yet` until a schema loads; `Arranging N tables…` while computing; `No matching tables` for an empty search result.

### Design Mode

A draft-only schema designer (header: `DESIGN DRAFT · ObjectMutationService plan · fingerprint-gated apply`):

1. Enter `schema` + `table` and click **Add draft table**.
2. For each draft table, add `col`/`type` pairs with **Add col**.
3. Add foreign keys with `fk name`, `from schema.table.col`, `to schema.table.col`, then **Add FK**.
4. **Preview mutation plan** shows the fingerprint, planned effects, and generated SQL. **Apply (confirm)** executes it; **Undo**/**Redo**/**Discard** manage the draft.

If the live schema changes after you opened Design Mode, a warning (`Live schema changed — re-open Design Mode or discard before apply`) blocks applying.

## Schema Compare

Available from the sidebar section and as a workspace view.

- **Take Snapshot** — records the loaded schema. **Diff vs snapshot** / **Diff now** re-introspects and diffs it.
- The diff view summarizes `N total difference(s) · Added · Changed · Removed`, then groups items: tables/views/routines only in snapshot or only in current, and column/type changes. `Schemas are identical. No drift detected.` appears when clean.
- **Migration plan**: **Generate migration plan** produces an operation list keyed to the active `connection · schema` target, with per-operation risk badges (`SAFE`, `MUTATING`, `DESTRUCTIVE`) and warnings. The generated migration DDL can be copied (**Copy SQL**). If a plan contains destructive operations you must tick **Confirm destructive operations (never auto-applied)** before **Apply migration SQL**; if the recorded target no longer matches the active session, a `SAFETY LOCK — plan target mismatch` card disables applying.
- **Data compare**: enter `target connection id`, `schema`, `table`, and `key columns (comma)` and press **Compare rows** for a key-aware sample compare across two connections. Results show counts (`src/tgt · +added -removed ~changed =equal · truncated`), filterable by `all / added / removed / changed`, plus a preview-only `DATA SYNC SQL PREVIEW`.

## Schema Workbench

Opened from a table's context menu (**Modify Table (Alter)…**, **Drop Table…**) or the sidebar. It follows a **plan → preview → apply** flow. The sidebar offers 15 modes:

`Table / columns · Column alter · Views · Indexes · Constraints · Triggers · Sequences · Types / enums · Schema / database · Extensions · Comments · Partitions · Dependencies · Docs export · Execution history`.

- Each mode shows a **DEFINITION** form (schema, object name, mode-specific designer: columns grid with `+ Add Column`/`+ UUID PK`/`+ Timestamps`, trigger presets, sequence defaults, enum presets, view query presets, index/constraint designers).
- Action buttons such as **Plan create**, **Plan drop**, **Plan rename**, **Plan alter**, **Plan refresh**, **Plan comment** generate a live DDL preview (`-- Click 'Plan create', 'Plan drop' or 'Plan rename' above to generate DDL` when empty). **Apply DDL to Database** executes with a confirmation dialog; **Open in SQL Editor** hands the script to a query tab instead.
- **Dependencies** lists object dependency edges (From · Relation · To). **Docs export** generates schema documentation (**Generate Markdown**, **Generate HTML**, **Open as query**). **Execution history** lists past DDL runs with **Clear history**, **Open in SQL editor**, and **Copy SQL** per entry.

## Keyboard shortcuts

| Shortcut | Context | Action |
|---|---|---|
| Enter / F2 | Grid (editable) | Start editing cell; Enter commits edit |
| Tab | Cell editing | Commit and move to next cell |
| Esc | Cell editing / grid | Cancel edit / clear selection |
| Delete / Backspace | Grid (editable) | Stage row delete |
| Ctrl/⌘+C | Grid | Copy cell |
| Ctrl/⌘+Shift+C | Grid | Copy selected rows |
| Ctrl/⌘+A | Grid | Select all |
| Ctrl/⌘+S | Table data | Apply staged changes |
| Ctrl/⌘+Z | Table data | Discard staged changes |
| Ctrl/⌘+V | Grid | Paste into editable cell |
| F5 | Table data / explorer | Refresh table data / schema |
