import type { DbEngine } from "@/types";

let formatterModulePromise: Promise<typeof import("sql-formatter")> | null = null;

function toSqlFormatterDialect(engine: DbEngine | undefined) {
  if (engine === "postgres") {
    return "postgresql";
  }
  if (engine === "mysql") {
    return "mysql";
  }
  return "sqlite";
}

async function loadFormatterModule() {
  if (!formatterModulePromise) {
    formatterModulePromise = import("sql-formatter");
  }
  return formatterModulePromise;
}

export async function formatSqlText(
  sql: string,
  engine: DbEngine | undefined,
): Promise<string> {
  const trimmed = sql.trim();
  if (!trimmed) {
    return sql;
  }

  try {
    const formatter = await loadFormatterModule();
    return formatter.format(sql, {
      language: toSqlFormatterDialect(engine),
      keywordCase: "upper",
      linesBetweenQueries: 1,
      tabWidth: 2,
    });
  } catch {
    return sql;
  }
}
