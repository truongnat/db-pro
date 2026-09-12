# Findings: Table Details Workspace

- Previously, selecting a table defaulted to `TableView::Structure` without triggering immediate row data loading.
- Table sub-tabs were rendered using basic `tab_frame` instead of the design system's `SegmentedTabs` / `UnderlineTabs`.
- Column data types and structure were rendered with custom frames rather than the unified `Table` widget.
- Generating SQL templates (SELECT, INSERT, UPDATE, DELETE) and quick query execution directly on a selected table provide immense productivity benefits matching DBeaver / DataGrip.
- The native grid already virtualizes visible rows and stages typed cell updates, but its selection state was single-row only. A separate visible-row selection set and anchor is required so Shift range selection remains correct after filtering or sorting.
- Table refresh previously reset the page and cleared local sort state. Refresh now reloads the current page/query state; filter and sort changes still intentionally return to the first page.
- The runtime table mutation commands are still one-operation commands. Applying several staged changes sequentially cannot satisfy the transaction rollback contract; a parameterized transaction command is still required before marking Apply transactional.
