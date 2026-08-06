export interface SqlTemplate {
  label: string;
  sql: string;
}

export const SQL_TEMPLATES: SqlTemplate[] = [
  {
    label: "SELECT",
    sql: "SELECT * FROM table_name\nWHERE condition\nLIMIT 100;",
  },
  {
    label: "SELECT with JOIN",
    sql: "SELECT t1.*, t2.*\nFROM table1 t1\nJOIN table2 t2 ON t1.id = t2.table1_id\nWHERE t1.condition;",
  },
  {
    label: "INSERT",
    sql: "INSERT INTO table_name (column1, column2)\nVALUES ('value1', 'value2');",
  },
  {
    label: "UPDATE",
    sql: "UPDATE table_name\nSET column1 = 'value1'\nWHERE id = 1;",
  },
  {
    label: "DELETE",
    sql: "DELETE FROM table_name\nWHERE id = 1;",
  },
  {
    label: "CREATE TABLE",
    sql: "CREATE TABLE table_name (\n  id SERIAL PRIMARY KEY,\n  name VARCHAR(255) NOT NULL,\n  created_at TIMESTAMP DEFAULT NOW()\n);",
  },
  {
    label: "ALTER TABLE — add column",
    sql: "ALTER TABLE table_name\nADD COLUMN new_column VARCHAR(255);",
  },
  {
    label: "DROP TABLE",
    sql: "DROP TABLE IF EXISTS table_name;",
  },
  {
    label: "CREATE INDEX",
    sql: "CREATE INDEX idx_table_column\nON table_name (column_name);",
  },
  {
    label: "CREATE VIEW",
    sql: "CREATE VIEW view_name AS\nSELECT column1, column2\nFROM table_name\nWHERE condition;",
  },
  {
    label: "COUNT with GROUP BY",
    sql: "SELECT column_name, COUNT(*) AS cnt\nFROM table_name\nGROUP BY column_name\nORDER BY cnt DESC;",
  },
  {
    label: "Subquery",
    sql: "SELECT *\nFROM table_name\nWHERE id IN (\n  SELECT table_name_id\n  FROM other_table\n  WHERE condition\n);",
  },
];
