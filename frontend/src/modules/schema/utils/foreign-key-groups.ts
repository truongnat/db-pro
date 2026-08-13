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
 * Backend now returns composite foreign keys as single objects with arrays.
 * This function normalizes the data structure for UI consumption.
 */
export function groupForeignKeys(foreignKeys: SchemaForeignKeyDto[]): ForeignKeyGroup[] {
  return foreignKeys.map((fk) => ({
    key: [fk.schema, fk.fromTable, fk.name, fk.toSchema, fk.toTable].join("\u0000"),
    name: fk.name,
    fromTable: fk.fromTable,
    fromColumns: fk.fromColumns,
    toTable: fk.toTable,
    toColumns: fk.toColumns,
    schema: fk.schema,
    toSchema: fk.toSchema,
  }));
}
