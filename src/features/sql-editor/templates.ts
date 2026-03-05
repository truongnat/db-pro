import type { DbEngine } from "@/types";

export type SqlTemplate = {
  id: string;
  label: string;
  description: string;
  buildSql: (engine: DbEngine | undefined) => string;
};

function quoteIdentifier(engine: DbEngine | undefined, identifier: string): string {
  if (engine === "mysql") {
    return `\`${identifier}\``;
  }
  return `"${identifier}"`;
}

export const SQL_TEMPLATES: SqlTemplate[] = [
  {
    id: "select_all",
    label: "SELECT All",
    description: "Select rows from a table with a page-sized LIMIT.",
    buildSql: (engine) => {
      const tableName = quoteIdentifier(engine, "your_table");
      return `SELECT *\nFROM ${tableName}\nLIMIT 100;`;
    },
  },
  {
    id: "insert",
    label: "INSERT",
    description: "Insert one row with placeholder values.",
    buildSql: (engine) => {
      const tableName = quoteIdentifier(engine, "your_table");
      return `INSERT INTO ${tableName} (column_1, column_2)\nVALUES ('value_1', 'value_2');`;
    },
  },
  {
    id: "update",
    label: "UPDATE",
    description: "Update rows with a defensive WHERE clause.",
    buildSql: (engine) => {
      const tableName = quoteIdentifier(engine, "your_table");
      return `UPDATE ${tableName}\nSET column_1 = 'new_value'\nWHERE id = 1;`;
    },
  },
  {
    id: "delete",
    label: "DELETE",
    description: "Delete rows with explicit predicate.",
    buildSql: (engine) => {
      const tableName = quoteIdentifier(engine, "your_table");
      return `DELETE FROM ${tableName}\nWHERE id = 1;`;
    },
  },
  {
    id: "create_table",
    label: "CREATE TABLE",
    description: "Create a starter table schema.",
    buildSql: (engine) => {
      const tableName = quoteIdentifier(engine, "new_table");
      return `CREATE TABLE ${tableName} (\n  id INTEGER PRIMARY KEY,\n  name TEXT NOT NULL,\n  created_at TEXT NOT NULL\n);`;
    },
  },
];
