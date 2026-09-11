import type { IntrospectResult } from "@/modules/schema/types/schema.types";
import type { ErGraphModel, ErGraphRelation, ErGraphTable, TableId } from "./types";

/**
 * Pure domain builder — the renderer-agnostic `ErGraphModel` consumed by every
 * `ErRenderer` implementation (P1.9). Pre-indexes columns once, computes FK
 * degrees, and builds the adjacency index, all in O(T + C + F).
 *
 * Only relations whose both endpoints are tables of `schema` are included —
 * same rule the React Flow edge builder applies.
 */
export function buildErGraphModel(data: IntrospectResult, schema: string): ErGraphModel {
  const schemaKeys = new Set<TableId>();
  for (const table of data.tables) {
    if (table.schema === schema) schemaKeys.add(`${table.schema}.${table.name}`);
  }

  // Column counts — O(C) once.
  const columnCounts = new Map<TableId, number>();
  for (const col of data.columns) {
    if (col.schema !== schema) continue;
    const key = `${col.schema}.${col.tableName}`;
    columnCounts.set(key, (columnCounts.get(key) ?? 0) + 1);
  }

  // FK relations + outgoing FK degree — O(F) once.
  const relations: ErGraphRelation[] = [];
  const fkCounts = new Map<TableId, number>();
  for (const fk of data.foreignKeys) {
    const source = `${fk.schema}.${fk.fromTable}`;
    const target = `${fk.toSchema}.${fk.toTable}`;
    if (!schemaKeys.has(source) || !schemaKeys.has(target)) continue;
    relations.push({
      id: `fk:${fk.schema}.${fk.fromTable}.${fk.name}`,
      source,
      target,
      name: fk.name,
    });
    fkCounts.set(source, (fkCounts.get(source) ?? 0) + 1);
  }

  const tables: ErGraphTable[] = data.tables
    .filter((table) => table.schema === schema)
    .map((table) => {
      const id = `${table.schema}.${table.name}`;
      return {
        id,
        label: table.name,
        schema: table.schema,
        columnCount: columnCounts.get(id) ?? 0,
        fkCount: fkCounts.get(id) ?? 0,
      };
    });

  const adjacency = new Map<TableId, Set<TableId>>();
  for (const rel of relations) {
    let from = adjacency.get(rel.source);
    if (!from) adjacency.set(rel.source, (from = new Set()));
    from.add(rel.target);
    let to = adjacency.get(rel.target);
    if (!to) adjacency.set(rel.target, (to = new Set()));
    to.add(rel.source);
  }

  const totalColumns = tables.reduce((sum, t) => sum + t.columnCount, 0);

  return {
    tables,
    relations,
    adjacency,
    stats: {
      tables: tables.length,
      relations: relations.length,
      columns: totalColumns,
    },
  };
}

/**
 * Schema complexity score (locked P1 hard rule #5): thresholds are complexity
 * scores, not hardcoded table counts.
 *
 *   complexity = tableCount + relationCount * 0.7 + totalColumnCount * 0.08
 *
 * Columns dominate real-world schemas (each table card renders one DOM row per
 * column in React Flow), so they weigh in even at a small per-column factor.
 */
export function computeSchemaComplexity(stats: {
  tables: number;
  relations: number;
  columns: number;
}): number {
  return stats.tables + stats.relations * 0.7 + stats.columns * 0.08;
}

export type SchemaComplexityTier = "S" | "M" | "L" | "XL";

/**
 * Tier boundaries tuned from the P1.8 runtime benchmarks (Chrome 150, real
 * renderers) against the locked fixture scores:
 *
 *   A100  = 310.4  → M  (React Flow full graph: 60 fps, 1,143 DOM — no UX)
 *   A500  = 1,802.5 → L (full graph on canvas immediately — UX pivot: opass-style
 *                       search focus + click neighborhood highlight)
 *   A1000 = 3,765.2 → XL (full-graph React Flow not viable: 122 s layout)
 *
 * The M/L boundary sits at 700 — headroom above A100 (310.4) so dense
 * 100–200-table schemas stay on the full React Flow graph, well before the
 * A500 (1,802.5) degradation point. Do not re-tune it back to the plan's
 * literal ranges without new benchmark evidence.
 *
 * `L` and `XL` both trigger the exploration UX (`isLargeSchema`); `XL` is
 * reserved for the scale where a full-graph React Flow overview is outright
 * rejected (no distinct behavior yet — see VERIFICATION.md).
 */
export function classifySchemaComplexity(complexity: number): SchemaComplexityTier {
  if (complexity < 100) return "S";
  if (complexity < 700) return "M";
  if (complexity < 2000) return "L";
  return "XL";
}
