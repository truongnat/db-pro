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
