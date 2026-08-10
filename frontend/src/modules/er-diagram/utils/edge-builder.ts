import type { IntrospectResult } from "@/modules/schema/types/schema.types";

export interface EdgeGroup {
  /** Constraint name (qualified with schema). */
  key: string;
  /** First FK entry (used for source/target table info). */
  fk: IntrospectResult["foreignKeys"][number];
  /** All column pairs belonging to this constraint. */
  columns: { from: string; to: string }[];
}

/**
 * Group foreign-key entries by constraint name.
 *
 * Composite FKs like `(tenant_id, parent_id) REFERENCES parent(tenant_id, id)`
 * produce multiple `ForeignKey` rows sharing the same constraint `name`.
 * This function merges them into one `EdgeGroup` per logical constraint.
 *
 * Only FKs whose `fromTable` exists in the provided `visibleTables` set are included.
 */
export function groupForeignKeys(
  foreignKeys: IntrospectResult["foreignKeys"],
  visibleTables: Set<string>,
): EdgeGroup[] {
  const groups = new Map<string, EdgeGroup>();

  for (const fk of foreignKeys) {
    const fromKey = `${fk.schema}.${fk.fromTable}`;
    if (!visibleTables.has(fromKey)) continue;

    const key = `${fk.schema}.${fk.name}`;
    const existing = groups.get(key);
    if (existing) {
      existing.columns.push({ from: fk.fromColumn, to: fk.toColumn });
    } else {
      groups.set(key, { key, fk, columns: [{ from: fk.fromColumn, to: fk.toColumn }] });
    }
  }

  return [...groups.values()];
}
