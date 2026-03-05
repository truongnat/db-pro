import type { SqlEditorSelection, SqlRunTarget, SqlStatement } from "./types";

export const starterSql = `SELECT name
FROM sqlite_master
WHERE type = 'table'
ORDER BY name;`;

function isWhitespace(char: string): boolean {
  return /\s/.test(char);
}

function createSqlStatement(
  text: string,
  start: number,
  end: number,
): SqlStatement | null {
  let left = start;
  while (left < end && isWhitespace(text[left])) {
    left += 1;
  }

  let right = end;
  while (right > left && isWhitespace(text[right - 1])) {
    right -= 1;
  }

  if (left >= right) {
    return null;
  }

  return {
    sql: text.slice(left, right),
    start: left,
    end: right,
  };
}

function splitSqlStatements(text: string): SqlStatement[] {
  const statements: SqlStatement[] = [];
  let statementStart = 0;
  let inSingleQuote = false;
  let inDoubleQuote = false;
  let inLineComment = false;
  let inBlockComment = false;

  for (let index = 0; index < text.length; index += 1) {
    const current = text[index];
    const next = text[index + 1] ?? "";

    if (inLineComment) {
      if (current === "\n") {
        inLineComment = false;
      }
      continue;
    }

    if (inBlockComment) {
      if (current === "*" && next === "/") {
        inBlockComment = false;
        index += 1;
      }
      continue;
    }

    if (inSingleQuote) {
      if (current === "'" && next === "'") {
        index += 1;
        continue;
      }
      if (current === "'") {
        inSingleQuote = false;
      }
      continue;
    }

    if (inDoubleQuote) {
      if (current === '"' && next === '"') {
        index += 1;
        continue;
      }
      if (current === '"') {
        inDoubleQuote = false;
      }
      continue;
    }

    if (current === "-" && next === "-") {
      inLineComment = true;
      index += 1;
      continue;
    }

    if (current === "/" && next === "*") {
      inBlockComment = true;
      index += 1;
      continue;
    }

    if (current === "'") {
      inSingleQuote = true;
      continue;
    }

    if (current === '"') {
      inDoubleQuote = true;
      continue;
    }

    if (current === ";") {
      const statement = createSqlStatement(text, statementStart, index + 1);
      if (statement) {
        statements.push(statement);
      }
      statementStart = index + 1;
    }
  }

  const trailingStatement = createSqlStatement(text, statementStart, text.length);
  if (trailingStatement) {
    statements.push(trailingStatement);
  }

  return statements;
}

export function detectSqlTarget(
  text: string,
  selection: SqlEditorSelection | null,
): SqlRunTarget {
  const trimmedAll = text.trim();
  if (!trimmedAll) {
    return { sql: "", source: "all" };
  }

  if (selection) {
    const selectionStart = Math.min(selection.anchor, selection.head);
    const selectionEnd = Math.max(selection.anchor, selection.head);
    if (selectionEnd > selectionStart) {
      const selected = text.slice(selectionStart, selectionEnd).trim();
      if (selected) {
        return { sql: selected, source: "selection" };
      }
    }
  }

  const statements = splitSqlStatements(text);
  if (statements.length === 0) {
    return { sql: trimmedAll, source: "all" };
  }

  if (statements.length === 1) {
    return { sql: statements[0].sql, source: "all" };
  }

  const cursor = selection?.head ?? 0;
  const currentStatement =
    statements.find(
      (statement) => cursor >= statement.start && cursor <= statement.end,
    ) ??
    statements.find((statement) => cursor < statement.start) ??
    statements[statements.length - 1];

  return { sql: currentStatement.sql, source: "current" };
}
