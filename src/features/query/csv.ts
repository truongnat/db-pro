function escapeCsvValue(value: string): string {
  if (/[",\r\n]/.test(value)) {
    return `"${value.replace(/"/g, "\"\"")}"`;
  }
  return value;
}

export function toCsv(columns: string[], rows: string[][]): string {
  const lines: string[] = [];

  lines.push(columns.map((item) => escapeCsvValue(item)).join(","));
  for (const row of rows) {
    lines.push(row.map((item) => escapeCsvValue(item)).join(","));
  }

  return lines.join("\r\n");
}

export function downloadCsv(filename: string, csv: string): void {
  if (typeof document === "undefined") {
    throw new Error("Document API is not available for export.");
  }

  const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.style.display = "none";
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}

export function buildResultCsvFilename(connectionName: string | undefined): string {
  const safeName = (connectionName ?? "connection")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  return `db-pro-${safeName || "connection"}-${timestamp}.csv`;
}
