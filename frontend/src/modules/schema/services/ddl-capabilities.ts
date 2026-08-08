import type { SqlDialect } from "@/modules/query/sql/dialect";
import type { DriverType } from "@/modules/connection/types/connection.types";

/* ------------------------------------------------------------------ */
/*  DDL Capability Descriptor                                          */
/* ------------------------------------------------------------------ */

/**
 * Describes what DDL operations a dialect supports natively and what
 * warnings or workarounds are required.
 */
export interface DdlCapabilities {
  /** ALTER TABLE … DROP COLUMN works natively. */
  supportsDropColumn: boolean;
  /** ALTER TABLE … ALTER COLUMN works natively. */
  supportsAlterColumn: boolean;
  /** ALTER TABLE … RENAME TO works natively. */
  supportsRenameTable: boolean;
  /** ALTER TABLE … RENAME COLUMN works natively. */
  supportsRenameColumn: boolean;
  /** ALTER TABLE … ADD COLUMN with CHECK / UNIQUE works natively. */
  supportsAddColumnWithConstraint: boolean;
  /** DDL statements can be wrapped in BEGIN/COMMIT transactions. */
  supportsTransactionalDdl: boolean;
  /** GENERATED / IDENTITY column syntax is supported. */
  supportsIdentity: boolean;
  /** CREATE TRIGGER is supported. */
  supportsTriggers: boolean;
  /** Operations that require rebuilding the table (SQLite pattern). */
  requiresTableRebuild: TableRebuildOp[];
}

export type TableRebuildOp =
  | "dropColumn"
  | "alterColumn"
  | "addForeignKey";

/* ------------------------------------------------------------------ */
/*  Per-dialect capabilities                                           */
/* ------------------------------------------------------------------ */

const postgresCapabilities: DdlCapabilities = {
  supportsDropColumn: true,
  supportsAlterColumn: true,
  supportsRenameTable: true,
  supportsRenameColumn: true,
  supportsAddColumnWithConstraint: true,
  supportsTransactionalDdl: true,
  supportsIdentity: true,
  supportsTriggers: true,
  requiresTableRebuild: [],
};

const sqliteCapabilities: DdlCapabilities = {
  // SQLite 3.35+ supports DROP COLUMN, but we flag it as limited
  supportsDropColumn: true,
  supportsAlterColumn: false,
  supportsRenameTable: true,
  supportsRenameColumn: true,
  // SQLite ADD COLUMN cannot have PRIMARY KEY or UNIQUE
  supportsAddColumnWithConstraint: false,
  supportsTransactionalDdl: false,
  supportsIdentity: false,
  supportsTriggers: true,
  requiresTableRebuild: ["alterColumn", "addForeignKey"],
};

const capabilitiesMap: Record<DriverType, DdlCapabilities> = {
  postgres: postgresCapabilities,
  sqlite: sqliteCapabilities,
};

/* ------------------------------------------------------------------ */
/*  Public API                                                         */
/* ------------------------------------------------------------------ */

export function getDdlCapabilities(driver: DriverType): DdlCapabilities {
  return capabilitiesMap[driver];
}

export function getDdlCapabilitiesForDialect(dialect: SqlDialect): DdlCapabilities {
  return capabilitiesMap[dialect.driver];
}

/* ------------------------------------------------------------------ */
/*  Capability checks                                                  */
/* ------------------------------------------------------------------ */

export type CapabilityCheckResult =
  | { supported: true }
  | { supported: false; reason: string; requiresRebuild?: boolean };

/**
 * Check whether a DDL operation is supported by the given dialect.
 * Returns a result indicating support status and a human-readable reason.
 */
export function checkOperationSupported(
  driver: DriverType,
  operation: string,
): CapabilityCheckResult {
  const caps = getDdlCapabilities(driver);

  switch (operation) {
    case "dropColumn":
      if (!caps.supportsDropColumn) {
        return {
          supported: false,
          reason: `${driver} does not support DROP COLUMN.`,
          requiresRebuild: caps.requiresTableRebuild.includes("dropColumn"),
        };
      }
      return { supported: true };

    case "alterColumn":
      if (!caps.supportsAlterColumn) {
        return {
          supported: false,
          reason: `${driver} does not support ALTER COLUMN. This change requires rebuilding the table.`,
          requiresRebuild: caps.requiresTableRebuild.includes("alterColumn"),
        };
      }
      return { supported: true };

    case "renameTable":
      if (!caps.supportsRenameTable) {
        return {
          supported: false,
          reason: `${driver} does not support RENAME TABLE.`,
        };
      }
      return { supported: true };

    case "renameColumn":
      if (!caps.supportsRenameColumn) {
        return {
          supported: false,
          reason: `${driver} does not support RENAME COLUMN.`,
        };
      }
      return { supported: true };

    case "addForeignKey":
      if (caps.requiresTableRebuild.includes("addForeignKey")) {
        return {
          supported: false,
          reason: `${driver} cannot add foreign keys to existing tables. This requires rebuilding the table.`,
          requiresRebuild: true,
        };
      }
      return { supported: true };

    case "addConstraint":
      if (!caps.supportsAddColumnWithConstraint) {
        return {
          supported: false,
          reason: `${driver} ADD COLUMN does not support PRIMARY KEY or UNIQUE constraints.`,
        };
      }
      return { supported: true };

    default:
      return { supported: true };
  }
}

/* ------------------------------------------------------------------ */
/*  SQLite table-rebuild SQL generation                                */
/* ------------------------------------------------------------------ */

/**
 * Generate the SQL statements needed to rebuild a SQLite table.
 * This is the 12-step process:
 * 1. BEGIN TRANSACTION
 * 2. CREATE TABLE with new schema (temp name)
 * 3. Copy data from old to new
 * 4. DROP old table
 * 5. RENAME new to original name
 * 6. Recreate indexes, triggers, FK constraints
 * 7. COMMIT
 */
export interface TableRebuildPlan {
  /** SQL statements to execute in order. */
  statements: string[];
  /** Human-readable description of what's happening. */
  description: string;
}

export interface TableRebuildInput {
  schema: string;
  table: string;
  /** Current column definitions. */
  currentColumns: { name: string; dataType: string; nullable: boolean; defaultValue: string | null; isPk: boolean }[];
  /** New column definitions (after the change). */
  newColumns: { name: string; dataType: string; nullable: boolean; defaultValue: string | null; isPk: boolean }[];
  /** Existing indexes to recreate. */
  indexes?: { name: string; columns: string[]; unique: boolean }[];
}

export function buildSqliteTableRebuild(input: TableRebuildInput, dialect: SqlDialect): TableRebuildPlan {
  const qualified = dialect.qualify(input.schema, input.table);
  const tempName = `_rebuild_${input.table}`;
  const tempQualified = dialect.qualify(input.schema, tempName);

  const q = dialect.quoteIdentifier.bind(dialect);

  // Build CREATE TABLE for new schema
  const pkCols = input.newColumns.filter((c) => c.isPk);
  const lines = input.newColumns.map((col) => {
    let line = `    ${q(col.name)} ${col.dataType}`;
    if (!col.nullable) line += " NOT NULL";
    if (col.defaultValue) line += ` DEFAULT ${col.defaultValue}`;
    return line;
  });
  if (pkCols.length > 0) {
    lines.push(`    PRIMARY KEY (${pkCols.map((c) => q(c.name)).join(", ")})`);
  }
  const createSql = `CREATE TABLE ${tempQualified} (\n${lines.join(",\n")}\n)`;

  // Copy data — only columns that exist in both old and new
  const oldNames = new Set(input.currentColumns.map((c) => c.name));
  const commonCols = input.newColumns.filter((c) => oldNames.has(c.name));
  const colList = commonCols.map((c) => q(c.name)).join(", ");

  const copySql = colList
    ? `INSERT INTO ${tempQualified} (${colList}) SELECT ${colList} FROM ${qualified}`
    : `INSERT INTO ${tempQualified} SELECT * FROM ${qualified}`;

  const dropSql = `DROP TABLE ${qualified}`;
  const renameSql = `ALTER TABLE ${tempQualified} RENAME TO ${q(input.table)}`;

  // Recreate indexes
  const indexSqls = (input.indexes ?? []).map((idx) => {
    const cols = idx.columns.map(q).join(", ");
    const u = idx.unique ? "UNIQUE " : "";
    return `CREATE ${u}INDEX ${q(idx.name)} ON ${q(input.table)} (${cols})`;
  });

  const statements = [
    "BEGIN TRANSACTION",
    createSql,
    copySql,
    dropSql,
    renameSql,
    ...indexSqls,
    "COMMIT",
  ];

  return {
    statements,
    description: `This change requires rebuilding "${input.table}". The table will be recreated with the new schema, data will be copied, and indexes will be restored.`,
  };
}
