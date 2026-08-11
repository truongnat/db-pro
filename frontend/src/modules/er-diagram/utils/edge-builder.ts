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
 * Group foreign-key entries by constraint identity.
 *
 * Composite FKs like `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)`
 * produce multiple `ForeignKey` rows sharing the same constraint `name`.
 * This function merges them into one `EdgeGroup` per logical constraint.
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
  const groups = new Map<string, EdgeGroup>();

  for (const fk of foreignKeys) {
    const fromKey = `${fk.schema}.${fk.fromTable}`;
    if (!visibleTables.has(fromKey)) continue;

    const toKey = `${fk.toSchema}.${fk.toTable}`;
    if (!visibleTables.has(toKey)) continue;

    const key = `${fk.schema}.${fk.fromTable}.${fk.name}`;
    const existing = groups.get(key);
    if (existing) {
      existing.columns.push({ from: fk.fromColumn, to: fk.toColumn });
    } else {
      groups.set(key, { key, fk, columns: [{ from: fk.fromColumn, to: fk.toColumn }] });
    }
  }

  return [...groups.values()];
}
