/**
 * SQL Snippet — a short trigger that expands to a SQL template.
 *
 * Built-in snippets ship with the app and cannot be deleted.
 * Custom snippets are persisted in localStorage.
 */
export interface Snippet {
  /** Short trigger text, e.g. "sel", "ct". */
  trigger: string;
  /** Human-readable label shown in the snippet list. */
  label: string;
  /** SQL template. May contain `$cursor` placeholder. */
  body: string;
  /** Whether this is a built-in (non-deletable) snippet. */
  builtIn: boolean;
}

export const BUILT_IN_SNIPPETS: Snippet[] = [
  {
    trigger: "sel",
    label: "SELECT *",
    body: "SELECT * FROM $cursor;",
    builtIn: true,
  },
  {
    trigger: "selw",
    label: "SELECT … WHERE",
    body: "SELECT * FROM $cursor\nWHERE ",
    builtIn: true,
  },
  {
    trigger: "ins",
    label: "INSERT INTO",
    body: "INSERT INTO $cursor (\n  \n) VALUES (\n  \n);",
    builtIn: true,
  },
  {
    trigger: "upd",
    label: "UPDATE … SET",
    body: "UPDATE $cursor\nSET \nWHERE ;",
    builtIn: true,
  },
  {
    trigger: "del",
    label: "DELETE FROM",
    body: "DELETE FROM $cursor\nWHERE ;",
    builtIn: true,
  },
  {
    trigger: "ct",
    label: "CREATE TABLE",
    body: "CREATE TABLE $cursor (\n  id SERIAL PRIMARY KEY,\n  \n);",
    builtIn: true,
  },
  {
    trigger: "ati",
    label: "ALTER TABLE ADD",
    body: "ALTER TABLE $cursor ADD COLUMN ",
    builtIn: true,
  },
  {
    trigger: "dti",
    label: "DROP TABLE",
    body: "DROP TABLE IF EXISTS $cursor;",
    builtIn: true,
  },
  {
    trigger: "ci",
    label: "CREATE INDEX",
    body: "CREATE INDEX idx_$cursor ON $cursor ();",
    builtIn: true,
  },
  {
    trigger: "cnt",
    label: "COUNT",
    body: "SELECT COUNT(*) FROM $cursor;",
    builtIn: true,
  },
  {
    trigger: "grp",
    label: "GROUP BY",
    body: "SELECT $cursor, COUNT(*)\nFROM \nGROUP BY ;",
    builtIn: true,
  },
  {
    trigger: "joi",
    label: "JOIN … ON",
    body: "JOIN $cursor ON ",
    builtIn: true,
  },
];
