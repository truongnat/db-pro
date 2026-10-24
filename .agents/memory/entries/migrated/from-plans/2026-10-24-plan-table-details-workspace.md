---
schema_version: 1
id: "MEM-PLAN-table-details-workspace-44cd0bc8257a"
title: "Plan archive: Table Details Workspace"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, table-details-workspace]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "44cd0bc8257a629a1e8b682f8fe50d8a1f676021bcf8eccf2913dd7baa0c3257"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/table-details-workspace/PLAN.md"
  - "docs/plans/completed/table-details-workspace/FINDINGS.md"
  - "docs/plans/completed/table-details-workspace/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/table-details-workspace/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan: Table Details Workspace (Data Grid, Inline Editing, Structure, Query, Context Menu)

State: COMPLETED

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


## Source: `FINDINGS.md`

# Findings: Table Details Workspace

- Previously, selecting a table defaulted to `TableView::Structure` without triggering immediate row data loading.
- Table sub-tabs were rendered using basic `tab_frame` instead of the design system's `SegmentedTabs` / `UnderlineTabs`.
- Column data types and structure were rendered with custom frames rather than the unified `Table` widget.
- Generating SQL templates (SELECT, INSERT, UPDATE, DELETE) and quick query execution directly on a selected table provide immense productivity benefits matching DBeaver / DataGrip.
- The native grid already virtualizes visible rows and stages typed cell updates, but its selection state was single-row only. A separate visible-row selection set and anchor is required so Shift range selection remains correct after filtering or sorting.
- Table refresh previously reset the page and cleared local sort state. Refresh now reloads the current page/query state; filter and sort changes still intentionally return to the first page.
- Table mutations now use a parameterized transaction command shared by PostgreSQL and SQLite. Deletes, merged per-row updates, and inserts are ordered safely; failures retain the UI ChangeSet and surface the provider error. Live PostgreSQL execution remains pending.
- Transaction failure metadata now crosses the full runtime bridge. Statement-level failures map to the staged delete/update/insert target; validation and commit-level failures remain transaction-level and do not falsely highlight a row. Affected-row zero is surfaced as a stale-row conflict while staged values remain available for retry, discard, or reload.
- Cell ranges now retain an anchor/focus pair in visible row and column order, so Shift+Arrow and Shift-click render a rectangle while row-gutter selection remains independent.
- Copy actions use staged cell values and escape identifiers/literals in generated INSERT text. Temporary inserted rows remain open P1 work; multi-column sorting and same-column multiple-filter clauses remain follow-up work.
- Visible selection rendering no longer performs a linear row/column position search for every cell; a lookup map is built once per visible grid slice.
- Rendering moves the table result out temporarily instead of cloning the full QueryResult each frame. The existing visible-row virtualization and selection lookup remain in place; filter/sort/layout memoization and temporary inserted-row rendering are still follow-up work.


## Source: `VERIFICATION.md`

# Verification: Table Details Workspace

## Quality Gates
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace` (offline)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` (offline)
- [x] `cargo test --workspace` (273 core, 62 infrastructure, 32 SQLite integration, 122 UI, 3 native, 4 runtime, 21 tauri tests; 18 PostgreSQL integration tests remain ignored without a live target)
- [x] `cargo build --release --locked -p db-pro-native` (release build)
- [x] Performance scan: native binary 20.9 MB, workspace check/clippy pass
- [x] Runtime verification of the affected Table Data grid recorded at `screenshots/table-data-grid-1280x800.png`.

## Current Slice Evidence

- Selection model tests cover filtered/sorted Shift range selection and non-empty Cmd/Ctrl toggle behavior.
- Focused UI tests cover rectangular cell selection and selecting the complete current grid; the grid also has TSV-with-header, JSON, and INSERT SQL copy actions.
- Native translation test covers comparison and `IS NULL` filter mapping.
- A release binary smoke launch completed schema introspection against the configured local PostgreSQL connection. Interactive grid gestures were not collected because the native app was not accessible to the scripted click harness in this run.
- ChangeSet tests cover final-value merge, update-to-delete supersession, no-op reversion, and cell reversion.
- Core table mutation tests cover parameterized statement construction/order and propagation of a confirmed rollback failure.
- Detailed transaction failures retain `statement_index` through core, runtime, and native UI events; the grid highlights the mapped row/cell, preserves the ChangeSet after rollback, and exposes rollback/error details with reload/discard/keep-local actions.
- Tables without a primary key now show a safety warning and disable row update/delete while leaving inserts available; row identity continues to use original server-side composite PK values.
- The provider transaction path is implemented for PostgreSQL and SQLite with rollback on statement failure, timeout, and affected-rows zero; live PostgreSQL execution remains pending.
- SQLite integration test `sqlite_parameterized_table_mutations_roll_back_and_reject_zero_rows` passes, covering actual rollback and zero-row failure handling.
- The grid now builds a visible row/column coordinate lookup once per render, preserving O(1) selection checks per cell.

