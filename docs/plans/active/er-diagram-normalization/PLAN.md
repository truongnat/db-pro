# S6 — ER Diagram Normalization

## Goal
Normalize the existing ER diagram implementation to correctly handle composite foreign keys, edge identity, and cross-schema relations without rewriting from scratch.

## Problem
The ER diagram creates one edge per `ForeignKey` entry. Since composite FKs produce multiple `ForeignKey` rows (one per column pair) sharing the same constraint name, a single logical FK becomes multiple parallel edges. This violates the invariant that one constraint = one visual edge.

## Current Behavior
- `ForeignKey` domain struct uses singular `from_column`/`to_column`
- PostgreSQL introspection emits N rows per composite FK (via `unnest(con.conkey)`)
- SQLite PRAGMA `foreign_key_list` emits N rows per composite FK (via `seq`)
- ER diagram maps each FK entry to a separate edge with `id: edge-${index}`
- Result: composite FK `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)` → 2 parallel edges

## Target Behavior
- Edges are grouped by constraint name + from_table + to_table
- Each composite FK produces ONE edge with a label showing the constraint name
- Edge IDs use constraint name instead of sequential index
- Column handles still connect to the correct PK/FK columns

## Invariants
- One FK constraint = one visual edge (regardless of column count)
- Edge identity is stable (based on constraint name, not array index)
- Node identity remains `${schema}.${table}`
- Self-relations work correctly
- Provider behavior is identical (both use same IntrospectResult)

## Scope
- Edge grouping logic in `er-diagram.tsx`
- Edge ID generation (constraint name based)
- Edge label (show constraint name)
- Tests for composite FK grouping
- Tests for edge identity stability

## Out of Scope
- Multi-schema visualization (only first schema shown — deferred)
- Edge routing/curving for parallel edges between same tables
- Performance virtualization for large schemas
- Backend FK model change (keeping singular from_column/to_column)

## Test Strategy
- Unit test: group composite FKs into single edge
- Unit test: edge IDs use constraint name
- Unit test: single-column FK still produces one edge
- Unit test: self-referencing FK works
- Layout test: verify no regression in dagre layout
