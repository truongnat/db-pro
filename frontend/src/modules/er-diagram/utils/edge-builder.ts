import type { IntrospectResult } from "@/modules/schema/types/schema.types";

export interface EdgeGroup {
  /** Constraint identity (schema + fromTable + constraint name). */
  key: string;
  /** First FK entry (used for source/target table info). */
  fk: IntrospectResult["foreignKeys"][number];
  /** All column pairs belonging to this constraint. */
  columns: { from: string; to: string }[];
}

/**
 * Group foreign keys by constraint identity.
 *
 * Backend now returns composite FKs as single objects with arrays of columns.
 * This function creates EdgeGroups for UI consumption, filtering by visible tables.
 *
 * The grouping key is `${schema}.${fromTable}.${name}` to ensure uniqueness
 * even when constraint names collide across tables (common in SQLite).
 *
 * Only FKs where BOTH `fromTable` and `toTable` are in `visibleTables` are included,
 * preventing dangling edges to hidden tables.
 */
export function groupForeignKeys(
  foreignKeys: IntrospectResult["foreignKeys"],
  visibleTables: Set<string>,
): EdgeGroup[] {
  const groups: EdgeGroup[] = [];

  for (const fk of foreignKeys) {
    const fromKey = `${fk.schema}.${fk.fromTable}`;
    if (!visibleTables.has(fromKey)) continue;

    const toKey = `${fk.toSchema}.${fk.toTable}`;
    if (!visibleTables.has(toKey)) continue;

    const key = `${fk.schema}.${fk.fromTable}.${fk.name}`;
    
    // Build column pairs from the arrays
    const columns = fk.fromColumns.map((fromCol, i) => ({
      from: fromCol,
      to: fk.toColumns[i],
    }));

    groups.push({ key, fk, columns });
  }

  return groups;
}
