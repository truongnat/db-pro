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

### F1: Composite FK creates multiple edges (P1 — CONFIRMED)
**Evidence:** `er-diagram.tsx:125-139` maps each FK entry to a separate edge.
**Impact:** A composite FK like `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)` produces 2 parallel edges instead of 1 logical edge.
**Root cause:** FK domain model uses singular `from_column`/`to_column`, so composite FKs are stored as multiple entries sharing the same constraint `name`.
**Fix:** Group FK entries by constraint name before creating edges.

### F2: Edge IDs use sequential index (P2 — CONFIRMED)
**Evidence:** `er-diagram.tsx:128` uses `id: edge-${i}`.
**Impact:** Edge IDs change when FK array order changes (e.g., after introspection refresh). This could break edge selection persistence.
**Fix:** Use constraint name + table pair as edge ID.

### F3: Only first schema shown (P3 — DEFERRED)
**Evidence:** `er-diagram.tsx:89` filters by `data.schemas[0]?.name`.
**Impact:** Multi-schema databases only show one schema's tables.
**Decision:** Defer — multi-schema visualization is a separate feature.

### F4: Cross-schema edge targets may be orphaned (P3 — DEFERRED)
**Evidence:** Edge source uses `${fk.toSchema}.${fk.toTable}` but target node may not exist if toSchema differs from the displayed schema.
**Impact:** Edge points to non-existent node.
**Decision:** Defer — related to F3 multi-schema support.

### Verified OK
- Node identity: schema-qualified ✅
- Column handles: PK source + FK target ✅
- Self-relations: ReactFlow handles self-edges ✅
- Position persistence: localStorage per connection+schema ✅
- Search: dims non-matching nodes ✅
- Layout: dagre with direction toggle ✅
- Edge highlighting: click to highlight, pane click to clear ✅
- Navigation: node click opens table, column click navigates ✅
