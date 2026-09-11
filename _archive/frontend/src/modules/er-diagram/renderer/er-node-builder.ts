import type { Node } from "@xyflow/react";

import type {
  IntrospectResult,
  PrimaryKeyDto,
  SchemaColumnDto,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

import type { TableNodeData } from "../components/lod/types";

/**
 * P3.3 Phase A — pre-indexed node building.
 *
 * The component used to run `data.columns.filter(...)` / `data.primaryKeys
 * .filter(...)` / `data.foreignKeys.filter(...)` per table, which is O(T × C)
 * with T=500 tables and C=8,000 columns. These builders index the metadata
 * once (O(C + P + F)) and `buildTableNodes` resolves each table's columns,
 * PK flags and FK flags in O(1) per column via map lookups.
 *
 * Keeping the pipeline pure (no React) makes the real component path
 * unit-testable and benchmarkable — `benchmark.test.ts` exercises exactly the
 * code that runs in the diagram.
 */

export interface ErNodeIndexes {
  /** schema.tableName → columns */
  columnsByTable: Map<string, SchemaColumnDto[]>;
  /** schema.tableName → PK column names */
  primaryKeysByTable: Map<string, Set<string>>;
  /** schema.tableName:columnName → is an FK source column */
  fkColumnSet: Set<string>;
}

export function buildColumnsByTable(columns: SchemaColumnDto[]): Map<string, SchemaColumnDto[]> {
  const map = new Map<string, SchemaColumnDto[]>();
  for (const col of columns) {
    const key = `${col.schema}.${col.tableName}`;
    const list = map.get(key);
    if (list) list.push(col);
    else map.set(key, [col]);
  }
  return map;
}

export function buildPrimaryKeysByTable(primaryKeys: PrimaryKeyDto[]): Map<string, Set<string>> {
  const map = new Map<string, Set<string>>();
  for (const pk of primaryKeys) {
    const key = `${pk.schema}.${pk.tableName}`;
    const existing = map.get(key);
    if (existing) {
      for (const c of pk.columns) existing.add(c);
    } else {
      map.set(key, new Set(pk.columns));
    }
  }
  return map;
}

export function buildFkColumnSet(foreignKeys: SchemaForeignKeyDto[]): Set<string> {
  const set = new Set<string>();
  for (const fk of foreignKeys) {
    for (const col of fk.fromColumns) {
      set.add(`${fk.schema}.${fk.fromTable}:${col}`);
    }
  }
  return set;
}

export interface BuildTableNodesOptions {
  /** Manual compact toggle — caps rendering at summary. */
  compact: boolean;
}

/**
 * Build React Flow table nodes for `tables` using pre-indexed metadata.
 * Never scans `data.columns` / `data.primaryKeys` / `data.foreignKeys`
 * inside the per-table loop — all lookups are O(1) map/set reads.
 */
export function buildTableNodes(
  tables: TableDto[],
  indexes: ErNodeIndexes,
  options: BuildTableNodesOptions,
): Node<TableNodeData>[] {
  return tables.map((table) => {
    const tableKey = `${table.schema}.${table.name}`;
    const cols = indexes.columnsByTable.get(tableKey);
    const pkCols = indexes.primaryKeysByTable.get(tableKey);

    const columnData = (cols ?? []).map((col) => ({
      name: col.name,
      dataType: col.dataType,
      nullable: col.nullable,
      isPrimaryKey: pkCols?.has(col.name) ?? false,
      isForeignKey: indexes.fkColumnSet.has(`${tableKey}:${col.name}`),
    }));

    const nodeData: TableNodeData = {
      label: table.name,
      schema: table.schema,
      columns: columnData,
      compact: options.compact,
      // Initial value; the lod-injection memo refines it per viewport.
      lod: "detail",
    };

    return {
      id: tableKey,
      type: "table",
      position: { x: 0, y: 0 },
      data: nodeData,
    };
  });
}

/** All three indexes in one pass (used by callers that do not memoize parts). */
export function buildErNodeIndexes(data: IntrospectResult): ErNodeIndexes {
  return {
    columnsByTable: buildColumnsByTable(data.columns),
    primaryKeysByTable: buildPrimaryKeysByTable(data.primaryKeys),
    fkColumnSet: buildFkColumnSet(data.foreignKeys),
  };
}
