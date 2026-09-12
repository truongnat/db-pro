# Plan: Table Details Workspace (Data Grid, Inline Editing, Structure, Query, Context Menu)

## Product Vision & Goals
When a user clicks on a table in the database navigator (Codex / DBeaver style tree), DB Pro opens a dedicated, rich, highly-functional Table Workspace using the unified common component design system (`Button`, `Badge`, `Card`, `SegmentedTabs`, `Table`, `Input`, `Dialog`, `ContextMenu`, etc.).

## Key Capabilities:
1. **Immediate Table Data View (Default on Click)**:
   - Automatically loads and displays table rows when selecting a table.
   - Shows rich table metadata in the header: breadcrumb (`connection > database > schema.table`), table type, row count badge, and quick action toolbar.
2. **Modern Sub-Tab Navigation (`components::tabs::SegmentedTabs`)**:
   - `Data` (Interactive data grid with pagination, sorting, search, filtering).
   - `Structure` (Clean column table with types, nullable, PK/FK badges, defaults, comments).
   - `Indexes` (Declared indexes, uniqueness, indexed columns).
   - `Foreign Keys` (Relationships, source and target columns).
   - `DDL / SQL` (Generated/Introspected CREATE TABLE DDL in syntax-highlighted `CodeBlock`).
   - `Query` (Integrated SQL console pre-populated for this table with execute & limit).
3. **Interactive Table Data Grid & Editing**:
   - In-place cell editing with validation and staged changes indicator (amber highlight).
   - Staged mutations action bar: `Apply Changes`, `Discard Changes`, `View SQL Diff`.
   - New row insertion (`+ Add Row`), row deletion (`Delete Selected Row` with confirmation).
   - Search across columns & column-specific filter.
   - Pagination controls (`Previous`, `Next`, Page numbers, Rows per page selector).
4. **Comprehensive Context Actions**:
   - Sidebar right-click & table toolbar context menu:
     - View Data / Edit Structure / New SQL Query
     - Generate SQL (SELECT, INSERT, UPDATE, DELETE templates)
     - Export Data (CSV, JSON, SQL)
     - Ask AI Agent to analyze schema / optimize queries
     - Truncate / Drop table with safety confirmation dialog.
