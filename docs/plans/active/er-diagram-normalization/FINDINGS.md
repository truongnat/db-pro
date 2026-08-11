# S6 — ER Diagram Normalization Findings

## Architecture Audit

### Components
- `er-diagram.tsx` — Main ReactFlow component (445 lines)
  - Builds nodes from `data.tables` filtered by first schema
  - Builds edges from `data.foreignKeys` (one per FK entry)
  - dagre layout via `layoutGraph()` utility
  - Position persistence via localStorage
  - Search, compact mode, fit view, reset layout controls

- `table-node.tsx` — Custom ReactFlow node (123 lines)
  - Renders table header + column list
  - PK icon + FK icon per column
  - Source handle on PK columns, target handle on FK columns
  - Column click dispatches custom event for navigation

- `layout.ts` — dagre-based layout (57 lines)
  - LR/TB direction toggle
  - Node height estimated from column count
  - Standard dagre graph setup

### Data Flow
- IntrospectResult → ErDiagram props
- Tables → nodes (filtered by first schema)
- ForeignKeys → edges (one per FK entry)
- PrimaryKeys → PK column indicators
- Columns → column list in table node

## Findings

### F1: Composite FK creates multiple edges (P1 — FIXED)
**Evidence:** `er-diagram.tsx:125-139` maps each FK entry to a separate edge.
**Impact:** A composite FK like `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)` produces 2 parallel edges instead of 1 logical edge.
**Root cause:** FK domain model uses singular `from_column`/`to_column`, so composite FKs are stored as multiple entries sharing the same constraint `name`.
**Fix:** Created `edge-builder.ts` with `groupForeignKeys()` that merges entries by constraint identity (`schema.fromTable.name`). Refactored `er-diagram.tsx` to use it.

### F2: Edge IDs use sequential index (P2 — FIXED)
**Evidence:** `er-diagram.tsx:128` uses `id: edge-${i}`.
**Impact:** Edge IDs change when FK array order changes (e.g., after introspection refresh). This could break edge selection persistence.
**Fix:** Edge IDs now use `fk:${group.key}` where key = `schema.fromTable.constraintName`.

### F3: Only first schema shown (P3 — DEFERRED)
**Evidence:** `er-diagram.tsx:89` filters by `data.schemas[0]?.name`.
**Impact:** Multi-schema databases only show one schema's tables.
**Decision:** Defer — multi-schema visualization is a separate feature.

### F4: Cross-schema edge targets may be orphaned (P3 — FIXED)
**Evidence:** Edge source uses `${fk.toSchema}.${fk.toTable}` but target node may not exist if toSchema differs from the displayed schema.
**Impact:** Edge points to non-existent node.
**Fix:** `groupForeignKeys()` now requires both `fromTable` AND `toTable` to be in the visible set, preventing dangling edges.

## Cubic Review Findings

### CR1 (S6 commit): Grouping key not unique across tables (P1 — FIXED)
**Issue:** `${schema}.${name}` could collapse same-named FKs on different tables (SQLite constraint names aren't database-wide unique).
**Fix:** Key now includes `fromTable`: `${schema}.${fromTable}.${name}`.

### CR2 (S6 commit): Cross-schema dangling edges (P2 — FIXED)
**Issue:** Only `fromTable` visibility was checked; `toTable` could be hidden.
**Fix:** Both `fromTable` and `toTable` must be visible.

### Verified OK
- Node identity: schema-qualified ✅
- Column handles: PK source + FK target ✅
- Self-relations: ReactFlow handles self-edges ✅
- Position persistence: localStorage per connection+schema ✅
- Search: dims non-matching nodes ✅
- Layout: dagre with direction toggle ✅
- Edge highlighting: click to highlight, pane click to clear ✅
- Navigation: node click opens table, column click navigates ✅
