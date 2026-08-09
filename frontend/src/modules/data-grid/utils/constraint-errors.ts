/**
 * Map PostgreSQL / SQLite error messages to user-friendly constraint descriptions.
 *
 * This helps users understand *why* a mutation failed instead of seeing
 * a raw "Update failed" or an opaque error code.
 */

export interface ConstraintErrorInfo {
  kind:
    | "unique-violation"
    | "foreign-key-violation"
    | "not-null-violation"
    | "check-violation"
    | "invalid-type"
    | "value-too-long"
    | "numeric-out-of-range"
    | "division-by-zero"
    | "serialization-failure"
    | "deadlock"
    | "read-only"
    | "unknown";
  userMessage: string;
  /** The column(s) involved, if detectable. */
  columns?: string[];
  /** The constraint name, if detectable. */
  constraint?: string;
}

/**
 * Classify a database error message into a structured ConstraintErrorInfo.
 * Works for PostgreSQL error messages; falls back gracefully for SQLite.
 */
export function classifyConstraintError(rawError: string): ConstraintErrorInfo {
  const msg = rawError.toLowerCase();

  // Unique violation (PostgreSQL 23505, SQLite UNIQUE constraint)
  if (msg.includes("unique") || msg.includes("23505") || msg.includes("unique constraint")) {
    const constraint = extractQuoted(rawError, 1) ?? undefined;
    const column = extractQuoted(rawError, 3) ?? extractQuoted(rawError, 2) ?? undefined;
    return {
      kind: "unique-violation",
      userMessage: column
        ? `A row with this value for "${column}" already exists.`
        : "A row with these values already exists (unique constraint).",
      columns: column ? [column] : undefined,
      constraint,
    };
  }

  // Foreign key violation (PostgreSQL 23503)
  if (
    msg.includes("foreign key") ||
    msg.includes("23503") ||
    msg.includes("references")
  ) {
    const detail = extractDetail(rawError);
    return {
      kind: "foreign-key-violation",
      userMessage: detail
        ? `This value references a row that does not exist: ${detail}`
        : "This value violates a foreign key constraint — the referenced row does not exist.",
    };
  }

  // Not-null violation (PostgreSQL 23502)
  if (msg.includes("null value") || msg.includes("23502") || msg.includes("not null")) {
    const column = extractQuoted(rawError, 1) ?? extractColumnFromDetail(rawError);
    return {
      kind: "not-null-violation",
      userMessage: column
        ? `Column "${column}" cannot be set to NULL.`
        : "This column cannot be set to NULL.",
      columns: column ? [column] : undefined,
    };
  }

  // Check violation (PostgreSQL 23514)
  if (msg.includes("check") || msg.includes("23514")) {
    const constraint = extractQuoted(rawError, 1) ?? undefined;
    return {
      kind: "check-violation",
      userMessage: `The value does not satisfy the check constraint${constraint ? ` "${constraint}"` : ""}.`,
      constraint,
    };
  }

  // Invalid type / syntax (PostgreSQL 22P02, etc.)
  if (msg.includes("invalid input syntax") || msg.includes("22p02") || msg.includes("22p03")) {
    const type = extractQuoted(rawError, 1) ?? undefined;
    return {
      kind: "invalid-type",
      userMessage: type
        ? `The value is not a valid ${type}.`
        : "The value has an invalid type format.",
    };
  }

  // Value too long (PostgreSQL 22001)
  if (
    msg.includes("too long") ||
    msg.includes("22001") ||
    msg.includes("exceeds")
  ) {
    return {
      kind: "value-too-long",
      userMessage: "The value exceeds the maximum allowed length for this column.",
    };
  }

  // Numeric out of range (PostgreSQL 22003)
  if (msg.includes("out of range") || msg.includes("22003")) {
    return {
      kind: "numeric-out-of-range",
      userMessage: "The numeric value is out of range for this column type.",
    };
  }

  // Division by zero (PostgreSQL 22012)
  if (msg.includes("division by zero") || msg.includes("22012")) {
    return {
      kind: "division-by-zero",
      userMessage: "Division by zero is not allowed.",
    };
  }

  // Serialization failure (PostgreSQL 40001)
  if (msg.includes("serialization") || msg.includes("serialize") || msg.includes("40001")) {
    return {
      kind: "serialization-failure",
      userMessage: "A concurrent transaction modified this row. Please retry.",
    };
  }

  // Deadlock (PostgreSQL 40P01)
  if (msg.includes("deadlock") || msg.includes("40p01")) {
    return {
      kind: "deadlock",
      userMessage: "A deadlock occurred. Please retry the operation.",
    };
  }

  // Read-only
  if (msg.includes("read-only") || msg.includes("read only") || msg.includes("cannot be modified")) {
    return {
      kind: "read-only",
      userMessage: "This table or connection is read-only.",
    };
  }

  return {
    kind: "unknown",
    userMessage: rawError.length > 200 ? rawError.slice(0, 200) + "…" : rawError,
  };
}

/* ---- helpers ---- */

function extractQuoted(text: string, index: number): string | null {
  const matches = text.match(/"([^"]+)"/g);
  if (!matches || matches.length < index) return null;
  return matches[index - 1]?.replace(/"/g, "") ?? null;
}

function extractDetail(text: string): string | null {
  const match = text.match(/DETAIL:\s*(.+)/i);
  return match?.[1]?.trim() ?? null;
}

function extractColumnFromDetail(text: string): string | null {
  const detail = extractDetail(text);
  if (!detail) return null;
  // Try to find column name in "column \"name\"" pattern
  const match = detail.match(/column "([^"]+)"/i);
  return match?.[1] ?? null;
}
