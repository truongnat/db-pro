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
