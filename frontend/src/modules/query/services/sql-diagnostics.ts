import type * as monaco from "monaco-editor";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData } from "@/commons/types/workspace.types";

import { useSchemaCatalogStore, type ConnectionCatalog } from "../stores/schema-catalog.store";
import { extractTableRefs, type AliasInfo } from "./sql-context-parser";

/* ------------------------------------------------------------------ */
/*  Helpers                                                            */
/* ------------------------------------------------------------------ */

function getConnectionIdForTab(tabId: string): string | null {
  const tab = useWorkspaceStore.getState().tabs.find(
    (t) => t.id === tabId && t.kind === "query",
  );
  if (!tab || tab.kind !== "query") return null;
  return (tab.data as QueryTabData).sql !== undefined ? tab.connectionId : null;
}

function modelUriToTabId(modelUri: string): string | null {
  const match = modelUri.match(/dbpro:\/\/query\/(.+)\.sql/);
  return match?.[1] ?? null;
}

/** Strip string literals and comments so we don't analyse their content. */
function stripStringsAndComments(text: string): string {
  let result = "";
  let i = 0;
  while (i < text.length) {
    if (text[i] === "'") {
      i++;
      while (i < text.length && text[i] !== "'") {
        if (text[i] === "'" && i + 1 < text.length && text[i + 1] === "'") i += 2;
        else i++;
      }
      i++;
      result += " ";
    } else if (text[i] === "-" && i + 1 < text.length && text[i + 1] === "-") {
      while (i < text.length && text[i] !== "\n") i++;
      result += " ";
    } else if (text[i] === "/" && i + 1 < text.length && text[i + 1] === "*") {
      i += 2;
      while (i < text.length && !(text[i] === "*" && i + 1 < text.length && text[i + 1] === "/")) i++;
      i += 2;
      result += " ";
    } else {
      result += text[i];
      i++;
    }
  }
  return result;
}

function getCurrentStatementRange(
  cleaned: string,
  offset: number,
): { start: number; text: string } {
  let lastSemi = -1;
  for (let i = offset - 1; i >= 0; i--) {
    if (cleaned[i] === ";") { lastSemi = i; break; }
  }
  let nextSemi = cleaned.length;
  for (let i = offset; i < cleaned.length; i++) {
    if (cleaned[i] === ";") { nextSemi = i; break; }
  }
  return { start: lastSemi + 1, text: cleaned.slice(lastSemi + 1, nextSemi) };
}

/* ------------------------------------------------------------------ */
/*  Core validation                                                    */
/* ------------------------------------------------------------------ */

interface DiagnosticRef {
  qualifier: string;
  column: string;
  /** Offset of the column name in the ORIGINAL sql (not cleaned). */
  columnOffset: number;
  columnLength: number;
}

/**
 * Find all `qualifier.column` references in the statement and return
 * those whose qualifier resolves to a known table but whose column
 * does NOT exist in that table.
 */
function findUnknownQualifiedColumns(
  sql: string,
  cleaned: string,
  stmtStart: number,
  stmtText: string,
  tableRefs: AliasInfo[],
  catalog: ConnectionCatalog | undefined,
): DiagnosticRef[] {
  if (!catalog) return [];

  const diagnostics: DiagnosticRef[] = [];
  // Match `word.word` patterns in the statement
  const pattern = /\b(\w+)\s*\.\s*(\w+)/g;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(stmtText)) !== null) {
    const qualifier = match[1];
    const columnName = match[2];

    // Skip if qualifier is a SQL keyword
    if (/^(SELECT|FROM|WHERE|JOIN|INNER|LEFT|RIGHT|FULL|CROSS|ON|ORDER|GROUP|BY|HAVING|INSERT|INTO|UPDATE|SET|DELETE|AND|OR|NOT|IN|EXISTS|BETWEEN|LIKE|ILIKE|IS|NULL|AS|CASE|WHEN|THEN|ELSE|END|DISTINCT|UNION|ALL|LIMIT|OFFSET|WITH|RETURNING|VALUES|TRUNCATE|CREATE|ALTER|DROP|TABLE|VIEW|SCHEMA|INDEX|GRANT|REVOKE|TO|ROLE|OWNER)$/i.test(qualifier)) {
      continue;
    }

    // Resolve qualifier → table
    const lowerQualifier = qualifier.toLowerCase();
    const aliasRef = tableRefs.find((r) => r.alias.toLowerCase() === lowerQualifier);

    let schema: string;
    let table: string;

    if (aliasRef) {
      schema = aliasRef.schema;
      table = aliasRef.table;
    } else {
      const obj = catalog.objects.find(
        (o: { name: string; schema: string }) =>
          o.name.toLowerCase() === lowerQualifier ||
          `${o.schema}.${o.name}`.toLowerCase() === lowerQualifier,
      );
      if (!obj) continue; // qualifier doesn't resolve to a known object — skip
      schema = obj.schema;
      table = obj.name;
    }

    // Check if column exists
    const columns = catalog.columnsByTable.get(`${schema}.${table}`);
    if (!columns) continue; // columns not loaded — can't validate

    const colExists = columns.some((c: { name: string }) => c.name.toLowerCase() === columnName.toLowerCase());
    if (colExists) continue;

    // Compute offset in original SQL
    const matchOffsetInStmt = match.index + match[0].indexOf(columnName);
    const columnOffset = stmtStart + matchOffsetInStmt;

    diagnostics.push({
      qualifier,
      column: columnName,
      columnOffset,
      columnLength: columnName.length,
    });
  }

  return diagnostics;
}

/* ------------------------------------------------------------------ */
/*  Monaco integration                                                 */
/* ------------------------------------------------------------------ */

const DIAGNOSTIC_SOURCE = "dbpro-sql";
const DIAGNOSTIC_MARKER_TAG = "unknown-column";

function offsetToPosition(
  model: monaco.editor.ITextModel,
  offset: number,
): { lineNumber: number; column: number } {
  const clamped = Math.max(0, Math.min(offset, model.getValue().length));
  return model.getPositionAt(clamped);
}

async function validateModel(
  monacoInstance: typeof monaco,
  model: monaco.editor.ITextModel,
): Promise<void> {
  const tabId = modelUriToTabId(model.uri.toString());
  if (!tabId) return;

  const connectionId = getConnectionIdForTab(tabId);
  if (!connectionId) return;

  const catalog = useSchemaCatalogStore.getState();
  await catalog.ensureLoaded(connectionId);
  const connCatalog = catalog.getCatalog(connectionId);
  if (!connCatalog) return;

  const sql = model.getValue();
  const cleaned = stripStringsAndComments(sql);

  // Clear existing markers from our diagnostics
  const existingMarkers = monacoInstance.editor.getModelMarkers({ owner: DIAGNOSTIC_SOURCE });
  monacoInstance.editor.setModelMarkers(model, DIAGNOSTIC_SOURCE, []);

  const markers: monaco.editor.IMarkerData[] = [];

  // Validate each statement in the SQL
  let searchFrom = 0;
  while (searchFrom < cleaned.length) {
    const { start, text } = getCurrentStatementRange(cleaned, searchFrom + 1);
    if (!text.trim()) break;

    const tableRefs = extractTableRefs(text);
    const unknowns = findUnknownQualifiedColumns(sql, cleaned, start, text, tableRefs, connCatalog);

    for (const diag of unknowns) {
      const startPos = offsetToPosition(model, diag.columnOffset);
      const endPos = offsetToPosition(model, diag.columnOffset + diag.columnLength);

      markers.push({
        severity: monacoInstance.MarkerSeverity.Warning,
        message: `Unknown column '${diag.column}' in '${diag.qualifier}'`,
        startLineNumber: startPos.lineNumber,
        startColumn: startPos.column,
        endLineNumber: endPos.lineNumber,
        endColumn: endPos.column,
        source: DIAGNOSTIC_SOURCE,
        tags: [monacoInstance.MarkerTag.Unnecessary],
      });
    }

    // Advance past this statement
    const nextSemi = cleaned.indexOf(";", searchFrom);
    if (nextSemi === -1) break;
    searchFrom = nextSemi + 1;
  }

  monacoInstance.editor.setModelMarkers(model, DIAGNOSTIC_SOURCE, markers);
}

/**
 * Register the SQL diagnostics provider.
 * Call once during app initialisation (inside `registerSqlProviders`).
 */
export function registerSqlDiagnostics(monacoInstance: typeof monaco): void {
  // Validate on model content change (debounced)
  const timers = new Map<string, ReturnType<typeof setTimeout>>();

  function attachDiagnostics(model: monaco.editor.ITextModel) {
    if (model.getLanguageId() !== "sql") return;
    if (!model.uri.toString().startsWith("dbpro://query/")) return;

    // Initial validation
    setTimeout(() => {
      validateModel(monacoInstance, model).catch(() => {});
    }, 300);

    // Debounced re-validation on content change
    model.onDidChangeContent(() => {
      const uri = model.uri.toString();
      const existing = timers.get(uri);
      if (existing) clearTimeout(existing);

      timers.set(
        uri,
        setTimeout(() => {
          timers.delete(uri);
          validateModel(monacoInstance, model).catch(() => {});
        }, 500),
      );
    });
  }

  // Attach to existing models
  for (const model of monacoInstance.editor.getModels()) {
    attachDiagnostics(model);
  }

  // Attach to newly created models
  monacoInstance.editor.onDidCreateModel((model: monaco.editor.ITextModel) => {
    attachDiagnostics(model);
  });
}
