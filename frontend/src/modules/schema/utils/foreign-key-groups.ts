import type { SchemaForeignKeyDto } from "../types/schema.types";

export interface ForeignKeyGroup {
  key: string;
  name: string;
  fromTable: string;
  fromColumns: string[];
  toTable: string;
  toColumns: string[];
  schema: string;
  toSchema: string;
}

/**
 * Introspection returns one row per FK column mapping. PostgreSQL preserves the
 * database constraint name; SQLite uses a stable synthetic name per PRAGMA FK
 * id. Group rows by constraint identity so composite foreign keys are treated
 * as one relation throughout the UI.
 */
export function groupForeignKeys(foreignKeys: SchemaForeignKeyDto[]): ForeignKeyGroup[] {
  const groups = new Map<string, ForeignKeyGroup>();

  for (const fk of foreignKeys) {
    const key = [fk.schema, fk.fromTable, fk.name, fk.toSchema, fk.toTable].join("\u0000");
    const existing = groups.get(key);

    if (existing) {
      existing.fromColumns.push(fk.fromColumn);
      existing.toColumns.push(fk.toColumn);
      continue;
    }

    groups.set(key, {
      key,
      name: fk.name,
      fromTable: fk.fromTable,
      fromColumns: [fk.fromColumn],
      toTable: fk.toTable,
      toColumns: [fk.toColumn],
      schema: fk.schema,
      toSchema: fk.toSchema,
    });
  }

  return [...groups.values()];
}
