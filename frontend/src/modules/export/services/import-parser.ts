import type { ImportColumnMapping, ImportFormat, ImportPreview } from "../types/import.types";

const PREVIEW_ROWS = 10;

/**
 * Parse a CSV string into headers + rows.
 * Handles quoted fields, escaped quotes, and newlines within quotes.
 */
export function parseCsv(
  text: string,
  delimiter = ",",
): { headers: string[]; rows: string[][] } {
  const rows: string[][] = [];
  let current: string[] = [];
  let field = "";
  let inQuotes = false;
  let i = 0;

  while (i < text.length) {
    const ch = text[i];

    if (inQuotes) {
      if (ch === '"') {
        if (i + 1 < text.length && text[i + 1] === '"') {
          field += '"';
          i += 2;
          continue;
        }
        inQuotes = false;
        i++;
        continue;
      }
      field += ch;
      i++;
      continue;
    }

    if (ch === '"') {
      inQuotes = true;
      i++;
      continue;
    }

    if (ch === delimiter) {
      current.push(field);
      field = "";
      i++;
      continue;
    }

    if (ch === "\r") {
      i++;
      continue;
    }

    if (ch === "\n") {
      current.push(field);
      field = "";
      rows.push(current);
      current = [];
      i++;
      continue;
    }

    field += ch;
    i++;
  }

  // Handle last field/row
  if (field.length > 0 || current.length > 0) {
    current.push(field);
    rows.push(current);
  }

  if (rows.length === 0) return { headers: [], rows: [] };

  const headers = rows[0];
  const dataRows = rows.slice(1).filter((r) => r.length > 1 || r[0] !== "");

  return { headers, rows: dataRows };
}

/**
 * Parse JSON content (array of objects).
 */
export function parseJson(text: string): { keys: string[]; rows: Record<string, unknown>[] } {
  const data = JSON.parse(text);
  if (!Array.isArray(data) || data.length === 0) {
    return { keys: [], rows: [] };
  }
  const keys = Object.keys(data[0]);
  return { keys, rows: data };
}

/**
 * Detect whether content is CSV or JSON.
 */
export function detectFormat(content: string): ImportFormat {
  const trimmed = content.trimStart();
  if (trimmed.startsWith("[")) return "json";
  return "csv";
}

/**
 * Build an ImportPreview from raw file content.
 */
export function buildImportPreview(
  content: string,
  format: ImportFormat,
  targetColumns: string[],
): ImportPreview {
  if (format === "csv") {
    const { headers, rows } = parseCsv(content);
    const sampleRows = rows.slice(0, PREVIEW_ROWS).map((row) => {
      const obj: Record<string, string> = {};
      for (const [i, h] of headers.entries()) {
        obj[h] = row[i] ?? "";
      }
      return obj;
    });

    const mappings: ImportColumnMapping[] = headers.map((h, i) => ({
      sourceIndex: i,
      sourceName: h,
      targetColumn: autoMapColumn(h, targetColumns),
    }));

    return {
      format,
      sourceColumns: headers,
      sampleRows,
      totalRowCount: rows.length,
      mappings,
    };
  }

  // JSON
  const { keys, rows } = parseJson(content);
  const sampleRows = rows.slice(0, PREVIEW_ROWS).map((row) => {
    const obj: Record<string, string> = {};
    for (const k of keys) {
      obj[k] = row[k] == null ? "" : String(row[k]);
    }
    return obj;
  });

  const mappings: ImportColumnMapping[] = keys.map((k, i) => ({
    sourceIndex: i,
    sourceName: k,
    targetColumn: autoMapColumn(k, targetColumns),
  }));

  return {
    format,
    sourceColumns: keys,
    sampleRows,
    totalRowCount: rows.length,
    mappings,
  };
}

/**
 * Try to auto-map a source column to a target column by case-insensitive match.
 */
function autoMapColumn(sourceName: string, targetColumns: string[]): string | null {
  const lower = sourceName.toLowerCase();
  const match = targetColumns.find((t) => t.toLowerCase() === lower);
  return match ?? null;
}

/**
 * Parse all rows from file content for import execution.
 */
export function parseAllRows(
  content: string,
  format: ImportFormat,
): Record<string, string>[] {
  if (format === "csv") {
    const { headers, rows } = parseCsv(content);
    return rows.map((row) => {
      const obj: Record<string, string> = {};
      for (const [i, h] of headers.entries()) {
        obj[h] = row[i] ?? "";
      }
      return obj;
    });
  }

  const { keys, rows } = parseJson(content);
  return rows.map((row) => {
    const obj: Record<string, string> = {};
    for (const k of keys) {
      obj[k] = row[k] == null ? "" : String(row[k]);
    }
    return obj;
  });
}
