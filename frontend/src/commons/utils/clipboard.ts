export async function copyToClipboard(
  rows: Record<string, unknown>[],
  columns?: string[],
): Promise<boolean> {
  try {
    const lines: string[] = [];

    if (columns && columns.length > 0) {
      lines.push(columns.join("\t"));
    }

    for (const row of rows) {
      const keys = columns ?? Object.keys(row);
      const values = keys.map((key) => {
        const value = row[key];
        if (value === null || value === undefined) {
          return "";
        }
        return String(value);
      });
      lines.push(values.join("\t"));
    }

    const text = lines.join("\n");
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

export async function copyWithHeaders(
  rows: Record<string, unknown>[],
  columns: string[],
): Promise<boolean> {
  try {
    const lines: string[] = [columns.join(" | ")];
    lines.push(columns.map(() => "---").join(" | "));

    for (const row of rows) {
      const values = columns.map((key) => {
        const value = row[key];
        return value === null || value === undefined ? "NULL" : String(value);
      });
      lines.push(values.join(" | "));
    }

    await navigator.clipboard.writeText(lines.join("\n"));
    return true;
  } catch {
    return false;
  }
}

export async function copyAsMarkdown(
  rows: Record<string, unknown>[],
  columns: string[],
): Promise<boolean> {
  try {
    const lines: string[] = [];
    lines.push(`| ${columns.join(" | ")} |`);
    lines.push(`| ${columns.map(() => "---").join(" | ")} |`);

    for (const row of rows) {
      const values = columns.map((key) => {
        const value = row[key];
        return value === null || value === undefined ? "NULL" : String(value);
      });
      lines.push(`| ${values.join(" | ")} |`);
    }

    await navigator.clipboard.writeText(lines.join("\n"));
    return true;
  } catch {
    return false;
  }
}
