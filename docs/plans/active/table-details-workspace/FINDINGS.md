# Findings: Table Details Workspace

- Previously, selecting a table defaulted to `TableView::Structure` without triggering immediate row data loading.
- Table sub-tabs were rendered using basic `tab_frame` instead of the design system's `SegmentedTabs` / `UnderlineTabs`.
- Column data types and structure were rendered with custom frames rather than the unified `Table` widget.
- Generating SQL templates (SELECT, INSERT, UPDATE, DELETE) and quick query execution directly on a selected table provide immense productivity benefits matching DBeaver / DataGrip.
- The native grid already virtualizes visible rows and stages typed cell updates, but its selection state was single-row only. A separate visible-row selection set and anchor is required so Shift range selection remains correct after filtering or sorting.
- Table refresh previously reset the page and cleared local sort state. Refresh now reloads the current page/query state; filter and sort changes still intentionally return to the first page.
- Table mutations now use a parameterized transaction command shared by PostgreSQL and SQLite. Deletes, merged per-row updates, and inserts are ordered safely; failures retain the UI ChangeSet and surface the provider error. Live PostgreSQL execution remains pending.
- Cell ranges now retain an anchor/focus pair in visible row and column order, so Shift+Arrow and Shift-click render a rectangle while row-gutter selection remains independent.
- Copy actions use staged cell values and escape identifiers/literals in generated INSERT text. Temporary inserted rows remain open P1 work; multi-column sorting and same-column multiple-filter clauses remain follow-up work.
- Visible selection rendering no longer performs a linear row/column position search for every cell; a lookup map is built once per visible grid slice.
