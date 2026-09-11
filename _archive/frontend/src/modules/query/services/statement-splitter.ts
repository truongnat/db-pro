export interface StatementRange {
  start: number;
  end: number;
  sql: string;
}

function isDollarQuoteStart(sql: string, i: number): string | null {
  if (sql[i] !== "$") return null;
  let j = i + 1;
  while (j < sql.length && /[A-Za-z0-9_]/.test(sql[j])) j++;
  if (sql[j] !== "$") return null;
  return sql.slice(i, j + 1);
}

function isDollarQuoteEnd(sql: string, i: number, tag: string): boolean {
  return sql.startsWith(tag, i);
}

export function splitStatementsWithRanges(sql: string): StatementRange[] {
  const statements: StatementRange[] = [];
  let currentStart = -1;
  let i = 0;

  const pushStatement = (rawStart: number, rawEnd: number) => {
    const stmt = sql.slice(rawStart, rawEnd).trim();
    if (stmt) {
      const start = rawStart + sql.slice(rawStart, rawEnd).indexOf(stmt);
      statements.push({ start, end: start + stmt.length, sql: stmt });
    }
    currentStart = -1;
  };

  while (i < sql.length) {
    const ch = sql[i];

    if (ch === "'") {
      if (currentStart < 0) currentStart = i;
      i++;
      while (i < sql.length) {
        if (sql[i] === "'") {
          if (sql[i + 1] === "'") {
            i += 2;
          } else {
            i++;
            break;
          }
        } else {
          i++;
        }
      }
      continue;
    }

    if (ch === '"') {
      if (currentStart < 0) currentStart = i;
      i++;
      while (i < sql.length) {
        if (sql[i] === '"') {
          if (sql[i + 1] === '"') {
            i += 2;
          } else {
            i++;
            break;
          }
        } else {
          i++;
        }
      }
      continue;
    }

    if (ch === "$") {
      const tag = isDollarQuoteStart(sql, i);
      if (tag) {
        if (currentStart < 0) currentStart = i;
        i += tag.length;
        while (i < sql.length) {
          if (isDollarQuoteEnd(sql, i, tag)) {
            i += tag.length;
            break;
          }
          i++;
        }
        continue;
      }
    }

    if (ch === "-" && sql[i + 1] === "-") {
      if (currentStart < 0) currentStart = i;
      while (i < sql.length && sql[i] !== "\n") i++;
      continue;
    }

    if (ch === "/" && sql[i + 1] === "*") {
      if (currentStart < 0) currentStart = i;
      let depth = 1;
      i += 2;
      while (i < sql.length && depth > 0) {
        if (sql[i] === "/" && sql[i + 1] === "*") {
          depth++;
          i += 2;
        } else if (sql[i] === "*" && sql[i + 1] === "/") {
          depth--;
          i += 2;
        } else {
          i++;
        }
      }
      continue;
    }

    if (ch === ";") {
      if (currentStart >= 0) {
        pushStatement(currentStart, i);
      }
      i++;
      continue;
    }

    if (currentStart < 0) currentStart = i;
    i++;
  }

  if (currentStart >= 0) {
    pushStatement(currentStart, sql.length);
  }

  return statements;
}

export function findStatementAt(sql: string, offset: number): StatementRange | undefined {
  const statements = splitStatementsWithRanges(sql);
  for (const stmt of statements) {
    if (offset >= stmt.start && offset <= stmt.end) return stmt;
  }
  for (let i = statements.length - 1; i >= 0; i--) {
    if (statements[i].start <= offset) return statements[i];
  }
  return statements[0];
}

export interface EditorRunContext {
  value: string;
  selection: { start: number; end: number } | null;
  cursorOffset: number;
}

export function resolveRunTarget(ctx: EditorRunContext): string | undefined {
  const trimmed = ctx.value.trim();
  if (!trimmed) return undefined;

  if (ctx.selection) {
    const selected = ctx.value.slice(ctx.selection.start, ctx.selection.end).trim();
    if (selected) return selected;
  }

  return findStatementAt(ctx.value, ctx.cursorOffset)?.sql;
}
