export type QueryErrorCategory =
  | "timeout"
  | "cancelled"
  | "authentication"
  | "network"
  | "connection"
  | "unsupported"
  | "sql"
  | "unknown";

export type ClassifiedQueryError = {
  category: QueryErrorCategory;
  headline: string;
  details: string;
  action: string;
};

export function formatError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected error";
}

export function isTimeoutError(error: unknown): boolean {
  const message = formatError(error).toLowerCase();
  return message.includes("timed out") || message.includes("timeout");
}

export function classifyQueryError(error: unknown): ClassifiedQueryError {
  const rawMessage = formatError(error).trim();
  const normalized = rawMessage.toLowerCase();

  if (normalized.includes("cancelled")) {
    return {
      category: "cancelled",
      headline: "Query cancelled",
      details: rawMessage || "Query cancelled.",
      action: "Adjust SQL or rerun when ready.",
    };
  }

  if (isTimeoutError(rawMessage)) {
    return {
      category: "timeout",
      headline: "Query timeout",
      details: rawMessage || "Query timed out.",
      action: "Reduce dataset, add LIMIT, or increase timeout.",
    };
  }

  if (
    normalized.includes("password authentication failed") ||
    normalized.includes("authentication failed") ||
    normalized.includes("access denied")
  ) {
    return {
      category: "authentication",
      headline: "Authentication failed",
      details: rawMessage,
      action: "Verify username/password and role permissions.",
    };
  }

  if (
    normalized.includes("connection refused") ||
    normalized.includes("no route to host") ||
    normalized.includes("could not connect") ||
    normalized.includes("failed to lookup address")
  ) {
    return {
      category: "network",
      headline: "Network connection issue",
      details: rawMessage,
      action: "Check host/port, VPN, firewall, and server availability.",
    };
  }

  if (normalized.includes("connection failed")) {
    return {
      category: "connection",
      headline: "Connection failed",
      details: rawMessage,
      action: "Validate connection profile and database endpoint.",
    };
  }

  if (
    normalized.includes("pushdown requires") ||
    normalized.includes("wrapped pagination failed")
  ) {
    return {
      category: "unsupported",
      headline: "Filter/sort pushdown unavailable",
      details: rawMessage,
      action: "Run SELECT/WITH statements when using quick filter or sort.",
    };
  }

  if (
    normalized.includes("syntax error") ||
    normalized.includes("execution failed") ||
    normalized.includes("query failed")
  ) {
    return {
      category: "sql",
      headline: "SQL execution error",
      details: rawMessage,
      action: "Review SQL syntax and object names.",
    };
  }

  return {
    category: "unknown",
    headline: "Unexpected error",
    details: rawMessage || "Unexpected error.",
    action: "Review logs and retry.",
  };
}
