/**
 * Column mutation risk classification and SQL generation.
 *
 * Every schema mutation is classified into a risk level before execution.
 * The UI shows the generated SQL, risk badge, and warnings — and requires
 * explicit confirmation for anything above "low" risk.
 */

// ── Risk levels ──────────────────────────────────────────────────────

export type MutationRiskLevel = "low" | "medium" | "high" | "destructive";

export interface RiskClassification {
  level: MutationRiskLevel;
  label: string;
  warning: string | null;
  /** Whether the UI must show a confirmation dialog before applying. */
  requiresConfirmation: boolean;
}

const RISK_META: Record<MutationRiskLevel, { label: string; warning: string | null }> = {
  low: {
    label: "Low Risk",
    warning: null,
  },
  medium: {
    label: "Medium Risk",
    warning: "This operation may lock the table briefly on large datasets.",
  },
  high: {
    label: "High Risk",
    warning:
      "Changing the column type may rewrite the entire table. Existing values may not be castable and data could be lost.",
  },
  destructive: {
    label: "Destructive",
    warning:
      "This operation is irreversible. Data in this column will be permanently lost.",
  },
};

// ── Type-change risk matrix ──────────────────────────────────────────

/**
 * Broad categories for classifying type-change risk.
 * We normalise PostgreSQL type names into these families.
 */
function typeFamily(dtype: string): string {
  const d = dtype.toLowerCase().replace(/\(.*\)/, "").trim();
  if (d.startsWith("varchar") || d.startsWith("character varying") || d === "text" || d === "char" || d.startsWith("char") || d === "bpchar" || d === "name") return "text";
  if (d === "int2" || d === "smallint" || d === "integer" || d === "int" || d === "int4" || d === "bigint" || d === "int8" || d === "serial" || d === "bigserial") return "integer";
  if (d === "float4" || d === "real" || d === "float8" || d === "double precision" || d === "numeric" || d === "decimal" || d.startsWith("numeric") || d.startsWith("decimal")) return "numeric";
  if (d === "bool" || d === "boolean") return "boolean";
  if (d === "date") return "date";
  if (d.startsWith("timestamp") || d === "timestamptz") return "timestamp";
  if (d === "time" || d === "timetz") return "time";
  if (d === "uuid") return "uuid";
  if (d === "json" || d === "jsonb") return "json";
  if (d.startsWith("bytea") || d === "bytea") return "binary";
  return d;
}

/**
 * Safe widening pairs within the same type family.
 * Key = source type, Value = set of types it can safely widen to.
 */
const SAFE_WIDENING: Record<string, Set<string>> = {
  int2: new Set(["int4", "integer", "int8", "bigint"]),
  smallint: new Set(["int4", "integer", "int8", "bigint", "int2"]),
  int4: new Set(["int8", "bigint"]),
  integer: new Set(["int8", "bigint"]),
  int8: new Set(),
  bigint: new Set(),
  float4: new Set(["float8", "real", "double precision"]),
  real: new Set(["float8", "double precision"]),
  varchar: new Set(["text"]),
  "character varying": new Set(["text"]),
  char: new Set(["text", "varchar"]),
};

function classifyTypeChange(fromType: string, toType: string): MutationRiskLevel {
  const fromNorm = fromType.toLowerCase().replace(/\(.*\)/, "").trim();
  const toNorm = toType.toLowerCase().replace(/\(.*\)/, "").trim();

  if (fromNorm === toNorm) return "low";

  const fromFamily = typeFamily(fromType);
  const toFamily = typeFamily(toType);

  if (fromFamily === toFamily) {
    const safeSet = SAFE_WIDENING[fromNorm];
    if (safeSet && safeSet.has(toNorm)) return "low";
    return "medium";
  }

  // text → integer/numeric/boolean/timestamp/uuid = high risk (needs cast, may fail)
  if (fromFamily === "text") return "high";

  // integer → text = medium (safe cast but rewrites)
  if (fromFamily === "integer" && toFamily === "text") return "medium";

  // date → timestamp = low (safe widening)
  if (fromFamily === "date" && toFamily === "timestamp") return "low";

  // numeric → integer = high (may lose precision)
  if (fromFamily === "numeric" && toFamily === "integer") return "high";

  // timestamp → date = medium (loses time component)
  if (fromFamily === "timestamp" && toFamily === "date") return "medium";

  // Anything involving json/binary = high
  if (toFamily === "json" || toFamily === "binary" || fromFamily === "json" || fromFamily === "binary") return "high";

  // Default: high risk for cross-family conversions
  return "high";
}

// ── Column mutation operations ───────────────────────────────────────

export interface ColumnMutationDraft {
  /** Original column definition. */
  original: {
    name: string;
    dataType: string;
    nullable: boolean;
    defaultValue: string | null;
  };
  /** Proposed new values (only changed fields need to be set). */
  newName: string;
  newDataType: string;
  newNullable: boolean;
  newDefaultValue: string | null;
}

export interface ClassifiedMutation {
  /** Individual operation descriptions. */
  operations: string[];
  /** Generated ALTER TABLE SQL statements. */
  sql: string[];
  /** Overall risk (worst of all operations). */
  risk: RiskClassification;
  /** Combined warning messages. */
  warnings: string[];
}

const RISK_ORDER: Record<MutationRiskLevel, number> = {
  low: 0,
  medium: 1,
  high: 2,
  destructive: 3,
};

function worstRisk(a: MutationRiskLevel, b: MutationRiskLevel): MutationRiskLevel {
  return RISK_ORDER[a] >= RISK_ORDER[b] ? a : b;
}

function quoteIdent(name: string): string {
  return `"${name.replace(/"/g, '""')}"`;
}

function quoteLiteral(val: string): string {
  return `'${val.replace(/'/g, "''")}'`;
}

/**
 * Format a column default value for SQL.
 *
 * SQL expressions (function calls, keywords, numbers, already-quoted strings)
 * pass through with validation. Plain unquoted strings are treated as literals.
 */
function formatDefault(value: string): string {
  const trimmed = value.trim();

  if (/^'(?:''|[^'])*'$/.test(trimmed)) return trimmed;

  if (/^-?\d+(\.\d+)?$/.test(trimmed)) return trimmed;

  const upper = trimmed.toUpperCase();
  if (
    upper === "NULL" ||
    upper === "TRUE" ||
    upper === "FALSE" ||
    upper === "CURRENT_TIMESTAMP" ||
    upper === "CURRENT_DATE" ||
    upper === "CURRENT_TIME" ||
    upper === "CURRENT_USER" ||
    upper === "LOCALTIMESTAMP" ||
    upper === "LOCALTIME" ||
    upper === "SESSION_USER"
  ) {
    return trimmed;
  }

  if (trimmed.includes("(")) {
    validateExpression(trimmed);
    return trimmed;
  }

  return quoteLiteral(trimmed);
}

/**
 * Validate that a SQL expression is safe for embedding in DDL.
 * Rejects statement separators, comments, and DDL/DML keywords.
 */
function validateExpression(expr: string): void {
  if (expr.includes(";")) {
    throw new Error("Default expression must not contain statement separators (';')");
  }
  if (expr.includes("--") || expr.includes("/*")) {
    throw new Error("Default expression must not contain SQL comments");
  }
  const upper = expr.toUpperCase().trim();
  const forbidden = /^(DROP|CREATE|ALTER|DELETE|INSERT|UPDATE|TRUNCATE|EXEC|EXECUTE)\b/;
  if (forbidden.test(upper)) {
    throw new Error("Default expression must not contain DDL/DML statements");
  }
}

/**
 * Validate a PostgreSQL data type string for safe embedding in DDL.
 *
 * Accepts: `integer`, `varchar(255)`, `numeric(10,2)`,
 *          `timestamp with time zone`, `text[]`, etc.
 *
 * Rejects: semicolons, comments, SQL keywords (ALTER, DROP, USING, …),
 *          commas outside parameter clauses, and anything that is not
 *          a valid type name.
 */
export function validateDataType(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) {
    throw new Error("Data type must not be empty");
  }

  if (trimmed.includes(";")) {
    throw new Error("Data type must not contain statement separators (';')");
  }
  if (trimmed.includes("--") || trimmed.includes("/*")) {
    throw new Error("Data type must not contain SQL comments");
  }

  const upper = trimmed.toUpperCase();
  const forbiddenKeywords = [
    "ALTER", "DROP", "CREATE", "DELETE", "INSERT", "UPDATE", "TRUNCATE",
    "SELECT", "FROM", "WHERE", "SET", "TABLE", "COLUMN", "INDEX",
    "USING", "DEFAULT", "CONSTRAINT", "ADD", "RENAME", "GRANT", "REVOKE",
    "EXEC", "EXECUTE", "INTO", "VALUES",
  ];
  for (const kw of forbiddenKeywords) {
    const re = new RegExp(`\\b${kw}\\b`, "i");
    if (re.test(upper)) {
      throw new Error(`Data type must not contain SQL keyword "${kw}"`);
    }
  }

  let work = trimmed;

  // Strip array suffixes: text[], integer[][], etc.
  work = work.replace(/(\[\s*\])+$/, "");

  // Extract and validate parameter clause: varchar(255), numeric(10,2)
  const parenMatch = work.match(/^(.+?)\((.+)\)$/s);
  if (parenMatch) {
    const params = parenMatch[2];
    if (!/^[\d,\s]+$/.test(params)) {
      throw new Error("Data type parameters must be numeric (e.g. varchar(255), numeric(10,2))");
    }
    work = parenMatch[1].trim();
  }

  // Remaining must be a valid type name: letters, digits, underscores, spaces, dots
  if (!/^[a-zA-Z_][a-zA-Z0-9_\s.]*$/.test(work)) {
    throw new Error(`Invalid data type: "${trimmed}"`);
  }

  return trimmed;
}

/**
 * Classify all mutations in a draft and produce SQL + risk + warnings.
 */
export function classifyColumnMutation(
  draft: ColumnMutationDraft,
  schemaName: string,
  tableName: string,
): ClassifiedMutation {
  const { original } = draft;
  const operations: string[] = [];
  const sql: string[] = [];
  const warnings: string[] = [];
  let overallRisk: MutationRiskLevel = "low";

  const tableRef = `${quoteIdent(schemaName)}.${quoteIdent(tableName)}`;

  // 1. Rename
  if (draft.newName !== original.name) {
    operations.push(`Rename column "${original.name}" → "${draft.newName}"`);
    sql.push(
      `ALTER TABLE ${tableRef} RENAME COLUMN ${quoteIdent(original.name)} TO ${quoteIdent(draft.newName)};`,
    );
    // Rename is "dependency impact" — views, functions, FK references may break
    overallRisk = worstRisk(overallRisk, "medium");
    warnings.push(
      `Renaming column "${original.name}" may break views, functions, foreign keys, or application queries that reference it by name.`,
    );
  }

  // 2. Type change
  if (draft.newDataType.toLowerCase().replace(/\s+/g, " ").trim() !== original.dataType.toLowerCase().replace(/\s+/g, " ").trim()) {
    const validatedType = validateDataType(draft.newDataType);
    const typeRisk = classifyTypeChange(original.dataType, draft.newDataType);
    operations.push(`Change type "${original.dataType}" → "${draft.newDataType}"`);
    sql.push(
      `ALTER TABLE ${tableRef} ALTER COLUMN ${quoteIdent(draft.newName || original.name)} TYPE ${validatedType};`,
    );
    overallRisk = worstRisk(overallRisk, typeRisk);
    if (typeRisk === "high") {
      warnings.push(
        `Changing type from "${original.dataType}" to "${draft.newDataType}" may rewrite the table. Existing values may not be castable.`,
      );
    } else if (typeRisk === "medium") {
      warnings.push(
        `Type change from "${original.dataType}" to "${draft.newDataType}" may lock the table briefly.`,
      );
    }
  }

  // 3. Nullable change
  if (draft.newNullable !== original.nullable) {
    const colName = draft.newName || original.name;
    if (draft.newNullable) {
      // NOT NULL → nullable: low risk (relaxing constraint)
      operations.push(`Allow NULL on "${colName}"`);
      sql.push(
        `ALTER TABLE ${tableRef} ALTER COLUMN ${quoteIdent(colName)} DROP NOT NULL;`,
      );
      // stays at current risk — low
    } else {
      // nullable → NOT NULL: requires data validation
      operations.push(`Set NOT NULL on "${colName}"`);
      sql.push(
        `ALTER TABLE ${tableRef} ALTER COLUMN ${quoteIdent(colName)} SET NOT NULL;`,
      );
      overallRisk = worstRisk(overallRisk, "medium");
      warnings.push(
        `Setting NOT NULL on "${colName}" will fail if any existing rows contain NULL values. Ensure data is validated first.`,
      );
    }
  }

  // 4. Default value change
  if ((draft.newDefaultValue || null) !== (original.defaultValue ?? null)) {
    const colName = draft.newName || original.name;
    if (draft.newDefaultValue === null || draft.newDefaultValue === "") {
      operations.push(`Remove default from "${colName}"`);
      sql.push(
        `ALTER TABLE ${tableRef} ALTER COLUMN ${quoteIdent(colName)} DROP DEFAULT;`,
      );
    } else {
      operations.push(`Set default of "${colName}" to ${draft.newDefaultValue}`);
      sql.push(
        `ALTER TABLE ${tableRef} ALTER COLUMN ${quoteIdent(colName)} SET DEFAULT ${formatDefault(draft.newDefaultValue)};`,
      );
    }
    // Default changes are low risk (doesn't affect existing data)
  }

  const riskMeta = RISK_META[overallRisk];

  return {
    operations,
    sql,
    risk: {
      level: overallRisk,
      label: riskMeta.label,
      warning: riskMeta.warning,
      requiresConfirmation: overallRisk !== "low",
    },
    warnings,
  };
}

/**
 * Check if a draft has any actual changes compared to the original.
 */
export function hasChanges(draft: ColumnMutationDraft): boolean {
  return (
    draft.newName !== draft.original.name ||
    draft.newDataType.toLowerCase().replace(/\s+/g, " ").trim() !== draft.original.dataType.toLowerCase().replace(/\s+/g, " ").trim() ||
    draft.newNullable !== draft.original.nullable ||
    (draft.newDefaultValue || null) !== (draft.original.defaultValue ?? null)
  );
}
