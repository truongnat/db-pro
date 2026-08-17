import type { AgentMessage, SchemaContext } from "../types/agent.types";

function columnsForTable(ctx: SchemaContext, tableName: string) {
  for (const [key, cols] of ctx.columns) {
    if (key.endsWith(`.${tableName}`) || key === tableName) return cols;
  }
  return undefined;
}

function findTable(ctx: SchemaContext, hint: string | null): string | null {
  if (!hint) {
    if (ctx.activeTable) return ctx.activeTable;
    if (ctx.tables.length === 1) return ctx.tables[0].name;
    return null;
  }
  const lower = hint.toLowerCase();
  const found = ctx.tables.find((t) => t.name.toLowerCase() === lower);
  if (found) return found.name;
  const partial = ctx.tables.find((t) => t.name.toLowerCase().includes(lower));
  return partial?.name ?? null;
}

function extractTableName(prompt: string): string | null {
  const patterns = [/(?:from|table|of|for)\s+["`]?(\w+)["`]?/i, /^(\w+)$/i];
  for (const p of patterns) {
    const m = prompt.match(p);
    if (m) return m[1];
  }
  return null;
}

function generateSelectAll(
  ctx: SchemaContext,
  tableName: string,
): { content: string; sql: string } {
  const cols = columnsForTable(ctx, tableName);
  const colStr = cols ? cols.map((c) => c.name).join(", ") : "*";
  const sql = `SELECT ${colStr}\nFROM ${tableName}\nLIMIT 100;`;
  return {
    content: `Here's a SELECT query for \`${tableName}\`. Adjust the LIMIT or add WHERE clauses as needed.`,
    sql,
  };
}

function generateSelectWithJoin(
  ctx: SchemaContext,
  tableA: string,
  tableB: string,
): { content: string; sql: string } {
  const fk = ctx.foreignKeys.find(
    (f) =>
      (f.fromTable === tableA && f.toTable === tableB) ||
      (f.fromTable === tableB && f.toTable === tableA),
  );

  let joinClause: string;
  if (fk) {
    // Build join condition from all column pairs
    const isReversed = fk.fromTable === tableB;
    const fromTable = isReversed ? fk.toTable : fk.fromTable;
    const toTable = isReversed ? fk.fromTable : fk.toTable;
    const fromCols = isReversed ? fk.toColumns : fk.fromColumns;
    const toCols = isReversed ? fk.fromColumns : fk.toColumns;

    const conditions = fromCols.map(
      (fromCol, i) => `${fromTable}.${fromCol} = ${toTable}.${toCols[i]}`,
    );
    joinClause = conditions.join(" AND ");
  } else {
    joinClause = `${tableA}.id = ${tableB}.id`;
  }

  const sql = `SELECT *\nFROM ${tableA}\nJOIN ${tableB} ON ${joinClause}\nLIMIT 100;`;
  return {
    content: `Joined \`${tableA}\` and \`${tableB}\`${fk ? " using the foreign key relationship" : " — you may need to adjust the join condition"}.`,
    sql,
  };
}

function generateExplain(ctx: SchemaContext, tableName: string): { content: string; sql: string } {
  const sql = `EXPLAIN ANALYZE\nSELECT * FROM ${tableName} LIMIT 100;`;
  return {
    content: `This will show the execution plan for querying \`${tableName}\`. Run it in the query editor to see index usage and cost estimates.`,
    sql,
  };
}

function generateInsert(ctx: SchemaContext, tableName: string): { content: string; sql: string } {
  const cols = columnsForTable(ctx, tableName);
  if (!cols || cols.length === 0) {
    return {
      content: `I don't have column info for \`${tableName}\`. Here's a template — fill in the columns:`,
      sql: `INSERT INTO ${tableName} (column1, column2)\nVALUES (value1, value2);`,
    };
  }
  const nonPkCols = cols.filter((c) => !c.isPrimaryKey);
  const insertCols = nonPkCols.length > 0 ? nonPkCols : cols;
  const colNames = insertCols.map((c) => c.name).join(", ");
  const placeholders = insertCols
    .map((c) => {
      if (c.dataType.includes("int")) return "0";
      if (
        c.dataType.includes("text") ||
        c.dataType.includes("varchar") ||
        c.dataType.includes("char")
      )
        return "''";
      if (c.dataType.includes("bool")) return "false";
      if (
        c.dataType.includes("float") ||
        c.dataType.includes("real") ||
        c.dataType.includes("numeric") ||
        c.dataType.includes("decimal")
      )
        return "0.0";
      if (c.dataType.includes("date") || c.dataType.includes("timestamp")) return "NOW()";
      return "NULL";
    })
    .join(", ");

  const sql = `INSERT INTO ${tableName} (${colNames})\nVALUES (${placeholders});`;
  return {
    content: `Insert template for \`${tableName}\` with ${insertCols.length} column(s). Replace the placeholder values.`,
    sql,
  };
}

function generateUpdate(ctx: SchemaContext, tableName: string): { content: string; sql: string } {
  const cols = columnsForTable(ctx, tableName);
  if (!cols || cols.length === 0) {
    return {
      content: `I don't have column info for \`${tableName}\`. Here's a template:`,
      sql: `UPDATE ${tableName}\nSET column1 = value1\nWHERE id = ?;`,
    };
  }
  const nonPkCols = cols.filter((c) => !c.isPrimaryKey);
  const setCols = (nonPkCols.length > 0 ? nonPkCols : cols.slice(1))
    .map((c) => `  ${c.name} = -- TODO`)
    .join(",\n");
  const pkCol = cols.find((c) => c.isPrimaryKey);
  const whereCol = pkCol?.name ?? "id";

  const sql = `UPDATE ${tableName}\nSET\n${setCols}\nWHERE ${whereCol} = ?;`;
  return {
    content: `Update template for \`${tableName}\`. Fill in the SET values and WHERE condition.`,
    sql,
  };
}

function generateDelete(ctx: SchemaContext, tableName: string): { content: string; sql: string } {
  const cols = columnsForTable(ctx, tableName);
  const pkCol = cols?.find((c) => c.isPrimaryKey);
  const whereCol = pkCol?.name ?? "id";

  const sql = `DELETE FROM ${tableName}\nWHERE ${whereCol} = ?;`;
  return {
    content: `Delete template for \`${tableName}\`. **Be careful** — always use a WHERE clause to avoid deleting all rows.`,
    sql,
  };
}

function generateTableOverview(
  ctx: SchemaContext,
  tableName: string,
): { content: string; sql: string } {
  const cols = columnsForTable(ctx, tableName);
  const table = ctx.tables.find((t) => t.name === tableName);
  const fks = ctx.foreignKeys.filter((f) => f.fromTable === tableName || f.toTable === tableName);

  let content = `**${tableName}**`;
  if (table?.rowCount != null) content += ` — ~${table.rowCount} rows`;
  content += "\n\n";

  if (cols && cols.length > 0) {
    content += "Columns:\n";
    for (const c of cols) {
      const pk = c.isPrimaryKey ? " (PK)" : "";
      const nullable = c.nullable ? "" : " NOT NULL";
      content += `- \`${c.name}\` ${c.dataType}${pk}${nullable}\n`;
    }
  }

  if (fks.length > 0) {
    content += "\nRelationships:\n";
    for (const fk of fks) {
      // Show all column pairs for composite FKs
      const colPairs = fk.fromColumns
        .map((fromCol, i) => `${fromCol} → ${fk.toColumns[i]}`)
        .join(", ");
      content += `- ${fk.fromTable}(${colPairs}) → ${fk.toTable}\n`;
    }
  }

  const sql = `SELECT *\nFROM ${tableName}\nLIMIT 100;`;
  return { content, sql };
}

function generateSchemaOverview(ctx: SchemaContext): { content: string; sql: string } {
  let content = `**Schema overview**`;
  if (ctx.connectionName) content += ` — ${ctx.connectionName}`;
  content += `\n\n${ctx.tables.length} table(s) found.\n\n`;

  for (const t of ctx.tables.slice(0, 20)) {
    const cols = columnsForTable(ctx, t.name);
    const colCount = cols?.length ?? "?";
    const rows = t.rowCount != null ? ` (~${t.rowCount} rows)` : "";
    content += `- **${t.schema}.${t.name}** — ${colCount} columns${rows}\n`;
  }

  if (ctx.tables.length > 20) {
    content += `\n_...and ${ctx.tables.length - 20} more tables._\n`;
  }

  const sql =
    ctx.tables.length > 0
      ? `SELECT table_name, column_name, data_type\nFROM information_schema.columns\nWHERE table_schema = '${ctx.activeSchema ?? "public"}'\nORDER BY table_name, ordinal_position;`
      : "-- No tables found";

  return { content, sql };
}

function generateFkRelationships(
  ctx: SchemaContext,
  tableName: string | null,
): { content: string; sql: string } {
  const fks = tableName
    ? ctx.foreignKeys.filter((f) => f.fromTable === tableName || f.toTable === tableName)
    : ctx.foreignKeys;

  if (fks.length === 0) {
    return {
      content: tableName
        ? `No foreign key relationships found for \`${tableName}\`.`
        : "No foreign key relationships found in the schema.",
      sql: "",
    };
  }

  let content = "Relationships:\n\n";
  for (const fk of fks) {
    // Show all column pairs for composite FKs
    const colPairs = fk.fromColumns
      .map(
        (fromCol, i) => `\`${fk.fromTable}.${fromCol}\` → \`${fk.toTable}.${fk.toColumns[i]}\``,
      )
      .join(", ");
    content += `- ${colPairs}\n`;
  }

  const sql = `-- Foreign key joins\n${fks
    .map((fk) => {
      const conditions = fk.fromColumns
        .map(
          (fromCol, i) => `${fk.fromTable}.${fromCol} = ${fk.toTable}.${fk.toColumns[i]}`,
        )
        .join(" AND ");
      return `SELECT *\nFROM ${fk.fromTable}\nJOIN ${fk.toTable} ON ${conditions}\nLIMIT 50;`;
    })
    .join("\n\n")}`;

  return { content, sql };
}

function generateOptimize(
  ctx: SchemaContext,
  tableName: string | null,
): { content: string; sql: string } {
  const table = tableName ?? ctx.activeTable;
  if (!table) {
    return {
      content:
        "To optimize a query, I need to know which table or query you're working with. Try selecting a table first or paste the SQL you want to optimize.",
      sql: "",
    };
  }

  const sql = `EXPLAIN ANALYZE\nSELECT * FROM ${table}\nLIMIT 100;`;
  return {
    content: `Run \`EXPLAIN ANALYZE\` on \`${table}\` to see the execution plan. Look for:\n- **Seq Scan** — may benefit from an index\n- **Nested Loop** with high rows — consider a join optimization\n- **Sort** operations — check if an index can avoid the sort`,
    sql,
  };
}

function detectIntent(prompt: string): string {
  const lower = prompt.toLowerCase();

  if (/^(select|show|get|fetch|list|find|query)/.test(lower) || /select/.test(lower))
    return "select";
  if (/join|relate|connect|link/.test(lower)) return "join";
  if (/explain|plan|performance|slow|index/.test(lower)) return "explain";
  if (/insert|create.*row|add.*row|new.*row/.test(lower)) return "insert";
  if (/update|modify|change|edit|set/.test(lower)) return "update";
  if (/delete|remove|drop.*row/.test(lower)) return "delete";
  if (/overview|describe|structure|schema|tables|columns/.test(lower)) return "overview";
  if (/foreign|relation|fk|reference/.test(lower)) return "fk";
  if (/optim|tune|speed|faster/.test(lower)) return "optimize";

  return "select";
}

export function generateTemplateResponse(prompt: string, ctx: SchemaContext): AgentMessage {
  const trimmed = prompt.trim();

  if (!trimmed) {
    return {
      id: crypto.randomUUID(),
      role: "assistant",
      content:
        'Please describe what you\'d like to do. For example:\n- "Show all products"\n- "Join orders with customers"\n- "Explain the users table"',
      timestamp: Date.now(),
    };
  }

  if (ctx.tables.length === 0) {
    return {
      id: crypto.randomUUID(),
      role: "assistant",
      content:
        "I don't see any tables in the current connection. Make sure you're connected to a database with schema access.",
      timestamp: Date.now(),
    };
  }

  const intent = detectIntent(trimmed);
  const tableHint = extractTableName(trimmed);
  const tableName = findTable(ctx, tableHint);

  switch (intent) {
    case "select": {
      if (!tableName) {
        return {
          id: crypto.randomUUID(),
          role: "assistant",
          content: `Which table would you like to query? Available tables:\n${ctx.tables
            .slice(0, 10)
            .map((t) => `- \`${t.name}\``)
            .join("\n")}`,
          sql: "",
          timestamp: Date.now(),
        };
      }
      const result = generateSelectAll(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "join": {
      const tables = ctx.tables.map((t) => t.name);
      const mentioned = tables.filter((t) => trimmed.toLowerCase().includes(t.toLowerCase()));
      if (mentioned.length >= 2) {
        const result = generateSelectWithJoin(ctx, mentioned[0], mentioned[1]);
        return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
      }
      if (mentioned.length === 1 && tableName) {
        const relatedFks = ctx.foreignKeys.filter(
          (f) => f.fromTable === tableName || f.toTable === tableName,
        );
        if (relatedFks.length > 0) {
          const otherTable =
            relatedFks[0].fromTable === tableName ? relatedFks[0].toTable : relatedFks[0].fromTable;
          const result = generateSelectWithJoin(ctx, tableName, otherTable);
          return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
        }
      }
      return {
        id: crypto.randomUUID(),
        role: "assistant",
        content: "Which tables would you like to join? Mention two table names in your request.",
        timestamp: Date.now(),
      };
    }

    case "explain": {
      if (!tableName) {
        return {
          id: crypto.randomUUID(),
          role: "assistant",
          content: "Which table would you like to analyze? Mention the table name in your request.",
          timestamp: Date.now(),
        };
      }
      const result = generateExplain(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "insert": {
      if (!tableName) {
        return {
          id: crypto.randomUUID(),
          role: "assistant",
          content: "Which table do you want to insert into?",
          timestamp: Date.now(),
        };
      }
      const result = generateInsert(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "update": {
      if (!tableName) {
        return {
          id: crypto.randomUUID(),
          role: "assistant",
          content: "Which table do you want to update?",
          timestamp: Date.now(),
        };
      }
      const result = generateUpdate(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "delete": {
      if (!tableName) {
        return {
          id: crypto.randomUUID(),
          role: "assistant",
          content: "Which table do you want to delete from?",
          timestamp: Date.now(),
        };
      }
      const result = generateDelete(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "overview": {
      if (tableName) {
        const result = generateTableOverview(ctx, tableName);
        return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
      }
      const result = generateSchemaOverview(ctx);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "fk": {
      const result = generateFkRelationships(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    case "optimize": {
      const result = generateOptimize(ctx, tableName);
      return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
    }

    default: {
      if (tableName) {
        const result = generateSelectAll(ctx, tableName);
        return { id: crypto.randomUUID(), role: "assistant", ...result, timestamp: Date.now() };
      }
      return {
        id: crypto.randomUUID(),
        role: "assistant",
        content: `I can help with:\n- **SELECT** — "Show all products"\n- **JOIN** — "Join orders with customers"\n- **INSERT/UPDATE/DELETE** — templates with your columns\n- **EXPLAIN** — query analysis\n- **Overview** — table structure and relationships\n\nTry one of these, or mention a specific table name.`,
        timestamp: Date.now(),
      };
    }
  }
}

export async function generateLlmResponse(
  prompt: string,
  ctx: SchemaContext,
  config: { apiEndpoint: string; apiKey: string; model: string },
  history: { role: string; content: string }[],
): Promise<AgentMessage> {
  const schemaSummary = ctx.tables
    .slice(0, 30)
    .map((t) => {
      const cols = columnsForTable(ctx, t.name);
      const colStr = cols ? cols.map((c) => `${c.name}:${c.dataType}`).join(", ") : "";
      return `${t.schema}.${t.name}(${colStr})`;
    })
    .join("\n");

  const fkSummary = ctx.foreignKeys
    .map((f) => {
      const colPairs = f.fromColumns.map((fromCol, i) => `${fromCol}->${f.toColumns[i]}`).join(",");
      return `${f.fromTable}(${colPairs}) -> ${f.toTable}`;
    })
    .join("\n");

  const systemPrompt = `You are a SQL expert assistant for a database IDE. Generate SQL queries based on user requests.
Current connection: ${ctx.connectionName ?? "unknown"}
Active schema: ${ctx.activeSchema ?? "public"}

Available tables:
${schemaSummary}

Foreign keys:
${fkSummary || "None detected"}

Rules:
- Generate valid SQL for the database type
- Always wrap SQL in a fenced code block with sql language tag
- Be concise in explanations
- If unsure about table/column names, ask for clarification`;

  const messages = [
    { role: "system", content: systemPrompt },
    ...history.map((m) => ({ role: m.role, content: m.content })),
    { role: "user", content: prompt },
  ];

  const response = await fetch(config.apiEndpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({
      model: config.model,
      messages,
      temperature: 0.3,
      max_tokens: 1024,
    }),
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  const content = data.choices?.[0]?.message?.content ?? "No response from API.";

  const sqlMatch = content.match(/```sql\n?([\s\S]*?)```/);
  const sql = sqlMatch ? sqlMatch[1].trim() : undefined;

  return {
    id: crypto.randomUUID(),
    role: "assistant",
    content: sqlMatch
      ? content.replace(/```sql\n?[\s\S]*?```/, "").trim() || "Here's the generated SQL:"
      : content,
    sql,
    timestamp: Date.now(),
  };
}
