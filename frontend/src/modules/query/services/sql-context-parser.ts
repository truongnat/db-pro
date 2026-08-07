export type SqlContextKind = "keyword" | "table" | "column" | "qualifiedColumn" | "schema";

export interface AliasInfo {
  alias: string;
  schema: string;
  table: string;
}

export interface SqlContext {
  kind: SqlContextKind;
  prefix: string;
  qualifier?: string;
  tableRefs: AliasInfo[];
}

const CLAUSE_KEYWORDS = [
  "SELECT", "FROM", "WHERE", "JOIN", "INNER JOIN", "LEFT JOIN", "RIGHT JOIN",
  "FULL JOIN", "CROSS JOIN", "ON", "ORDER BY", "GROUP BY", "HAVING",
  "INSERT INTO", "UPDATE", "SET", "DELETE FROM", "TRUNCATE",
];

const TABLE_CLAUSES = new Set([
  "FROM", "JOIN", "INNER JOIN", "LEFT JOIN", "RIGHT JOIN",
  "FULL JOIN", "CROSS JOIN", "INSERT INTO", "UPDATE", "DELETE FROM", "TRUNCATE",
]);

const COLUMN_CLAUSES = new Set([
  "SELECT", "WHERE", "ON", "ORDER BY", "GROUP BY", "HAVING", "SET",
]);

function stripStringsAndComments(text: string): string {
  let result = "";
  let i = 0;
  while (i < text.length) {
    if (text[i] === "'") {
      i++;
      while (i < text.length && text[i] !== "'") {
        if (text[i] === "'" && i + 1 < text.length && text[i + 1] === "'") {
          i += 2;
        } else {
          i++;
        }
      }
      i++;
      result += " ";
    } else if (text[i] === "-" && i + 1 < text.length && text[i + 1] === "-") {
      while (i < text.length && text[i] !== "\n") i++;
      result += " ";
    } else if (text[i] === "/" && i + 1 < text.length && text[i + 1] === "*") {
      i += 2;
      while (i < text.length && !(text[i] === "*" && i + 1 < text.length && text[i + 1] === "/")) i++;
      i += 2;
      result += " ";
    } else {
      result += text[i];
      i++;
    }
  }
  return result;
}

function getCurrentStatement(cleaned: string, cursorOffset: number): string {
  const before = cleaned.slice(0, cursorOffset);
  let lastSemi = -1;
  for (let i = before.length - 1; i >= 0; i--) {
    if (before[i] === ";") {
      lastSemi = i;
      break;
    }
  }
  return before.slice(lastSemi + 1);
}

function findCurrentClause(stmt: string): string {
  const upper = stmt.toUpperCase();
  let bestClause = "";
  let bestPos = -1;

  for (const clause of CLAUSE_KEYWORDS) {
    const searchStr = clause.toUpperCase();
    let searchFrom = 0;
    while (true) {
      const idx = upper.indexOf(searchStr, searchFrom);
      if (idx === -1) break;
      const before = idx > 0 ? upper[idx - 1] : " ";
      const after = idx + searchStr.length < upper.length ? upper[idx + searchStr.length] : " ";
      if (!/\w/.test(before) && !/\w/.test(after) && idx > bestPos) {
        bestPos = idx;
        bestClause = clause;
      }
      searchFrom = idx + 1;
    }
  }

  return bestClause;
}

function extractTableRefs(stmt: string): AliasInfo[] {
  const refs: AliasInfo[] = [];
  const pattern = /(?:FROM|JOIN)\s+([\w.]+)(?:\s+(?:AS\s+)?(\w+))?/gi;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(stmt)) !== null) {
    const fullName = match[1];
    const alias = match[2];

    let schema = "public";
    let table = fullName;
    if (fullName.includes(".")) {
      const parts = fullName.split(".");
      schema = parts[0];
      table = parts[1];
    }

    const refAlias = alias && !CLAUSE_KEYWORDS.some((k) => k.split(" ").pop() === alias.toUpperCase())
      ? alias
      : table;

    refs.push({ alias: refAlias, schema, table });
  }

  return refs;
}

function extractPrefix(text: string): { prefix: string; qualifier?: string } {
  const trimmed = text.trimEnd();

  const dotMatch = trimmed.match(/([\w]+)\.\s*([\w]*)$/);
  if (dotMatch) {
    return { prefix: dotMatch[2] ?? "", qualifier: dotMatch[1] };
  }

  const wordMatch = trimmed.match(/(\w*)$/);
  return { prefix: wordMatch?.[1] ?? "" };
}

export function parseSqlContext(sql: string, cursorOffset: number): SqlContext {
  const cleaned = stripStringsAndComments(sql);
  const stmt = getCurrentStatement(cleaned, cursorOffset);
  const textUpToCursor = stmt;

  const { prefix, qualifier } = extractPrefix(textUpToCursor);

  if (qualifier) {
    const tableRefs = extractTableRefs(stmt);
    return {
      kind: "qualifiedColumn",
      prefix,
      qualifier,
      tableRefs,
    };
  }

  const clause = findCurrentClause(textUpToCursor);

  if (!clause) {
    return { kind: "keyword", prefix, tableRefs: [] };
  }

  if (TABLE_CLAUSES.has(clause)) {
    const tableRefs = extractTableRefs(stmt);
    return { kind: "table", prefix, tableRefs };
  }

  if (COLUMN_CLAUSES.has(clause)) {
    const tableRefs = extractTableRefs(stmt);
    return { kind: "column", prefix, tableRefs };
  }

  return { kind: "keyword", prefix, tableRefs: [] };
}
