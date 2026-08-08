import type { ConnectionCatalog } from "../stores/schema-catalog.store";
import type { SchemaColumnDto } from "@/modules/schema/types/schema.types";
import { extractTableRefs, type AliasInfo } from "./sql-context-parser";

/* ------------------------------------------------------------------ */
/*  Symbol kinds produced by the resolver                              */
/* ------------------------------------------------------------------ */

export type ResolvedSymbolKind =
  | "column"
  | "table"
  | "schema"
  | "unknown";

export interface ResolvedSymbol {
  kind: ResolvedSymbolKind;
  schema?: string;
  table?: string;
  column?: SchemaColumnDto;
  aliasInfo?: AliasInfo;
}

/* ------------------------------------------------------------------ */
/*  SQL keyword set – used to avoid resolving keywords as symbols      */
/* ------------------------------------------------------------------ */

const SQL_KEYWORD_SET = new Set([
  "SELECT", "FROM", "WHERE", "JOIN", "INNER", "LEFT", "RIGHT", "FULL",
  "CROSS", "ON", "ORDER", "GROUP", "BY", "HAVING", "INSERT", "INTO",
  "UPDATE", "SET", "DELETE", "TRUNCATE", "CREATE", "ALTER", "DROP",
  "TABLE", "VIEW", "SCHEMA", "DATABASE", "INDEX", "AS", "AND", "OR",
  "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "ILIKE", "IS", "NULL",
  "CASE", "WHEN", "THEN", "ELSE", "END", "DISTINCT", "ALL", "UNION",
  "INTERSECT", "EXCEPT", "LIMIT", "OFFSET", "FETCH", "ASC", "DESC",
  "WITH", "RECURSIVE", "RETURNING", "VALUES", "LATERAL", "OVER",
  "PARTITION", "ROWS", "RANGE", "TRUE", "FALSE", "BEGIN", "COMMIT",
  "ROLLBACK", "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT",
  "DEFAULT", "CHECK", "UNIQUE", "CASCADE", "RESTRICT", "IF", "REPLACE",
  "TEMPORARY", "TEMP", "MATERIALIZED", "CONCURRENTLY", "ONLY",
  "USING", "EXPLAIN", "ANALYZE", "VERBOSE", "COSTS", "BUFFERS",
  "FORMAT", "LANGUAGE", "FUNCTION", "PROCEDURE", "TRIGGER", "TYPE",
  "EXTENSION", "GRANT", "REVOKE", "TO", "ROLE", "OWNER",
]);

/* ------------------------------------------------------------------ */
/*  Word / identifier helpers (text-based, no Monaco dependency)       */
/* ------------------------------------------------------------------ */

function isIdentChar(ch: string): boolean {
  return /[\w$]/.test(ch);
}

/** Find the word boundaries at a given offset in plain text. */
export function getWordAtOffset(
  text: string,
  offset: number,
): { word: string; start: number; end: number } | null {
  if (offset < 0 || offset >= text.length) return null;

  // The character at offset must be an identifier character
  if (!isIdentChar(text[offset])) return null;

  let start = offset;
  while (start > 0 && isIdentChar(text[start - 1])) start--;

  let end = offset;
  while (end < text.length - 1 && isIdentChar(text[end + 1])) end++;
  end++;

  return { word: text.slice(start, end), start, end };
}

/** Check whether the character immediately before `start` is a dot. */
function hasDotBefore(text: string, start: number): boolean {
  let i = start - 1;
  while (i >= 0 && (text[i] === " " || text[i] === "\t")) i--;
  return i >= 0 && text[i] === ".";
}

/** Check whether the character immediately after `end` is a dot. */
function hasDotAfter(text: string, end: number): boolean {
  let i = end;
  while (i < text.length && (text[i] === " " || text[i] === "\t")) i++;
  return i < text.length && text[i] === ".";
}

/** Get the identifier after the dot that follows `end`. */
function getIdentifierAfterDot(text: string, end: number): string | null {
  let dotPos = end;
  while (dotPos < text.length && (text[dotPos] === " " || text[dotPos] === "\t")) dotPos++;
  if (dotPos >= text.length || text[dotPos] !== ".") return null;

  let pos = dotPos + 1;
  while (pos < text.length && (text[pos] === " " || text[pos] === "\t")) pos++;
  if (pos >= text.length || !isIdentChar(text[pos])) return null;

  const wordStart = pos;
  while (pos < text.length && isIdentChar(text[pos])) pos++;
  return text.slice(wordStart, pos);
}

/** Extract the identifier before the dot that precedes `start`. */
function getQualifierBeforeDot(text: string, start: number): string | null {
  let dotPos = start - 1;
  while (dotPos >= 0 && (text[dotPos] === " " || text[dotPos] === "\t")) dotPos--;
  if (dotPos < 0 || text[dotPos] !== ".") return null;

  let pos = dotPos - 1;
  while (pos >= 0 && (text[pos] === " " || text[pos] === "\t")) pos--;
  if (pos < 0 || !isIdentChar(text[pos])) return null;

  const wordEnd = pos + 1;
  while (pos > 0 && isIdentChar(text[pos - 1])) pos--;

  return text.slice(pos, wordEnd);
}

/**
 * Extract the full statement containing the given offset.
 * Scans backward for the previous `;` and forward for the next `;`.
 */
function getFullStatementAtOffset(sql: string, offset: number): string {
  let lastSemi = -1;
  for (let i = offset - 1; i >= 0; i--) {
    if (sql[i] === ";") {
      lastSemi = i;
      break;
    }
  }
  let nextSemi = sql.length;
  for (let i = offset; i < sql.length; i++) {
    if (sql[i] === ";") {
      nextSemi = i;
      break;
    }
  }
  return sql.slice(lastSemi + 1, nextSemi);
}

/* ------------------------------------------------------------------ */
/*  Catalog lookup helpers                                             */
/* ------------------------------------------------------------------ */

function findAlias(
  tableRefs: AliasInfo[],
  name: string,
): AliasInfo | undefined {
  const lower = name.toLowerCase();
  return tableRefs.find((r) => r.alias.toLowerCase() === lower);
}

function findObject(
  catalog: ConnectionCatalog,
  name: string,
) {
  const lower = name.toLowerCase();
  return catalog.objects.find(
    (o) =>
      o.name.toLowerCase() === lower ||
      `${o.schema}.${o.name}`.toLowerCase() === lower,
  );
}

function findColumn(
  columns: SchemaColumnDto[],
  name: string,
): SchemaColumnDto | undefined {
  const lower = name.toLowerCase();
  return columns.find((c) => c.name.toLowerCase() === lower);
}

/* ------------------------------------------------------------------ */
/*  Main resolver                                                      */
/* ------------------------------------------------------------------ */

/**
 * Resolve the SQL symbol at the given cursor offset.
 *
 * Used by the hover provider and (potentially) other intelligence features.
 * The completion provider has its own inline logic via `parseSqlContext`.
 */
export function resolveSymbolAtOffset(
  sql: string,
  offset: number,
  catalog: ConnectionCatalog,
): ResolvedSymbol | null {
  const wordInfo = getWordAtOffset(sql, offset);
  if (!wordInfo) return null;

  const { word, start, end } = wordInfo;
  if (SQL_KEYWORD_SET.has(word.toUpperCase())) return null;

  const stmt = getFullStatementAtOffset(sql, start);
  const tableRefs = extractTableRefs(stmt);

  const dotBefore = hasDotBefore(sql, start);
  const dotAfter = hasDotAfter(sql, end);

  /* ---- qualified: qualifier.word (dot before) ---- */
  if (dotBefore) {
    const qualifier = getQualifierBeforeDot(sql, start);
    if (!qualifier) return null;

    // schema.table ?
    const schemaQualifiedName = `${qualifier}.${word}`;
    const objMatch = findObject(catalog, schemaQualifiedName);
    if (objMatch) {
      return {
        kind: "table",
        schema: objMatch.schema,
        table: objMatch.name,
      };
    }

    // alias.column or table.column ?
    return resolveQualifiedColumn(qualifier, word, tableRefs, catalog);
  }

  /* ---- word.schema: word followed by dot (could be schema.table) ---- */
  if (dotAfter) {
    const afterDot = getIdentifierAfterDot(sql, end);
    if (afterDot) {
      const combined = `${word}.${afterDot}`;
      const objMatch = findObject(catalog, combined);
      if (objMatch) {
        return {
          kind: "table",
          schema: objMatch.schema,
          table: objMatch.name,
        };
      }
    }
  }

  /* ---- unqualified word ---- */

  // Table name?
  const tableMatch = findObject(catalog, word);
  if (tableMatch) {
    return {
      kind: "table",
      schema: tableMatch.schema,
      table: tableMatch.name,
    };
  }

  // Schema name?
  const schemaMatch = catalog.schemas.find(
    (s) => s.name.toLowerCase() === word.toLowerCase(),
  );
  if (schemaMatch) {
    return { kind: "schema", schema: schemaMatch.name };
  }

  // Unqualified column from any referenced table?
  return resolveUnqualifiedColumn(word, tableRefs, catalog);
}

/* ------------------------------------------------------------------ */
/*  Internal resolution helpers                                        */
/* ------------------------------------------------------------------ */

function resolveQualifiedColumn(
  qualifier: string,
  columnName: string,
  tableRefs: AliasInfo[],
  catalog: ConnectionCatalog,
): ResolvedSymbol | null {
  const aliasRef = findAlias(tableRefs, qualifier);

  let schema: string;
  let table: string;

  if (aliasRef) {
    schema = aliasRef.schema;
    table = aliasRef.table;
  } else {
    const obj = findObject(catalog, qualifier);
    if (!obj) return null;
    schema = obj.schema;
    table = obj.name;
  }

  const columns = catalog.columnsByTable.get(`${schema}.${table}`);
  if (!columns) return null;

  const col = findColumn(columns, columnName);
  if (!col) return null;

  return { kind: "column", schema, table, column: col, aliasInfo: aliasRef };
}

function resolveUnqualifiedColumn(
  name: string,
  tableRefs: AliasInfo[],
  catalog: ConnectionCatalog,
): ResolvedSymbol | null {
  for (const ref of tableRefs) {
    const key = `${ref.schema}.${ref.table}`;
    const columns = catalog.columnsByTable.get(key);
    if (!columns) continue; // columns not loaded yet — skip, don't bail

    const col = findColumn(columns, name);
    if (col) {
      return { kind: "column", schema: ref.schema, table: ref.table, column: col, aliasInfo: ref };
    }
  }
  return null;
}
