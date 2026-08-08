import type * as monaco from "monaco-editor";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData } from "@/commons/types/workspace.types";

import { useSchemaCatalogStore } from "../stores/schema-catalog.store";
import { resolveSymbolAtOffset } from "./sql-symbol-resolver";

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

function formatColumnHover(
  col: { name: string; dataType: string; nullable: boolean; isPrimaryKey: boolean; defaultValue: string | null },
  schema?: string,
  table?: string,
): string {
  const lines: string[] = [];
  lines.push(`**${col.name}**`);
  lines.push("");
  lines.push(`Type: \`${col.dataType}\``);
  if (!col.nullable) lines.push("NOT NULL");
  if (col.isPrimaryKey) lines.push("PRIMARY KEY");
  if (col.defaultValue != null) lines.push(`Default: \`${col.defaultValue}\``);
  if (schema && table) lines.push(`\nFrom: ${schema}.${table}`);
  return lines.join("\n");
}

function formatTableHover(
  obj: { name: string; schema: string; kind: string; rowCount: number | null },
): string {
  const lines: string[] = [];
  lines.push(`**${obj.schema}.${obj.name}**`);
  lines.push("");
  lines.push(`Kind: ${obj.kind}`);
  if (obj.rowCount != null) lines.push(`Rows: ~${obj.rowCount.toLocaleString()}`);
  return lines.join("\n");
}

export function createSqlHoverProvider(
  _monacoInstance: typeof monaco,
): monaco.languages.HoverProvider {
  return {
    provideHover: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
    ): Promise<monaco.languages.Hover | null> => {
      const tabId = modelUriToTabId(model.uri.toString());
      if (!tabId) return null;

      const connectionId = getConnectionIdForTab(tabId);
      if (!connectionId) return null;

      const sql = model.getValue();
      const offset = model.getOffsetAt(position);

      const catalog = useSchemaCatalogStore.getState();
      await catalog.ensureLoaded(connectionId);
      const connCatalog = catalog.getCatalog(connectionId);
      if (!connCatalog) return null;

      const resolved = resolveSymbolAtOffset(sql, offset, connCatalog);
      if (!resolved) return null;

      const wordInfo = getWordRangeAtPosition(model, position);
      if (!wordInfo) return null;

      let contents: string[] = [];

      switch (resolved.kind) {
        case "column": {
          if (resolved.column) {
            contents = [formatColumnHover(resolved.column, resolved.schema, resolved.table)];
          }
          break;
        }
        case "table": {
          if (resolved.schema && resolved.table) {
            const obj = connCatalog.objects.find(
              (o) => o.schema === resolved.schema && o.name === resolved.table,
            );
            if (obj) {
              contents = [formatTableHover(obj)];
            }
          }
          break;
        }
        case "schema": {
          contents = [`**Schema:** ${resolved.schema}`];
          break;
        }
      }

      if (contents.length === 0) return null;

      return {
        contents: contents.map((value) => ({ value })),
        range: wordInfo,
      };
    },
  };
}

function getWordRangeAtPosition(
  model: monaco.editor.ITextModel,
  position: monaco.Position,
): monaco.IRange | undefined {
  const word = model.getWordAtPosition(position);
  if (!word) return undefined;
  return {
    startLineNumber: position.lineNumber,
    endLineNumber: position.lineNumber,
    startColumn: word.startColumn,
    endColumn: word.endColumn,
  };
}
