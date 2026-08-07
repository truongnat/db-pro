import type * as monaco from "monaco-editor";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData } from "@/commons/types/workspace.types";

import { useSchemaCatalogStore } from "../stores/schema-catalog.store";
import { parseSqlContext } from "./sql-context-parser";
import { SQL_KEYWORDS } from "./sql-keywords";

function getConnectionIdForModel(modelUri: string): string | null {
  const match = modelUri.match(/dbpro:\/\/query\/(.+)\.sql/);
  if (!match) return null;
  const tabId = match[1];
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

function positionToOffset(model: monaco.editor.ITextModel, position: monaco.Position): number {
  return model.getOffsetAt(position);
}

export function createSqlCompletionProvider(
  monacoInstance: typeof monaco,
): monaco.languages.CompletionItemProvider {
  return {
    triggerCharacters: [".", " "],
    provideCompletionItems: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
    ): Promise<monaco.languages.CompletionList> => {
      const tabId = modelUriToTabId(model.uri.toString());
      if (!tabId) return { suggestions: [] };

      const tab = useWorkspaceStore.getState().tabs.find(
        (t) => t.id === tabId && t.kind === "query",
      );
      if (!tab || tab.kind !== "query") return { suggestions: [] };

      const connectionId = tab.connectionId;
      if (!connectionId) return { suggestions: [] };

      const sql = model.getValue();
      const offset = positionToOffset(model, position);
      const ctx = parseSqlContext(sql, offset);

      const word = model.getWordUntilPosition(position);
      const range: monaco.IRange = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      const catalog = useSchemaCatalogStore.getState();

      switch (ctx.kind) {
        case "keyword":
          return {
            suggestions: SQL_KEYWORDS.map((kw) => ({
              label: kw.label,
              kind: monacoInstance.languages.CompletionItemKind.Keyword,
              detail: kw.detail,
              insertText: kw.label,
              range,
            })),
          };

        case "table": {
          await catalog.ensureLoaded(connectionId);
          const cat = catalog.getCatalog(connectionId);
          if (!cat) return { suggestions: [] };

          const suggestions: monaco.languages.CompletionItem[] = [];

          for (const schema of cat.schemas) {
            suggestions.push({
              label: schema.name,
              kind: monacoInstance.languages.CompletionItemKind.Folder,
              detail: "Schema",
              insertText: schema.name,
              range,
            });
          }

          for (const obj of cat.objects) {
            const insertText = obj.schema === "public"
              ? obj.name
              : `${obj.schema}.${obj.name}`;
            suggestions.push({
              label: obj.name,
              kind: obj.kind === "table"
                ? monacoInstance.languages.CompletionItemKind.Struct
                : monacoInstance.languages.CompletionItemKind.Interface,
              detail: `${obj.kind} · ${obj.schema}`,
              insertText,
              range,
            });
          }

          return { suggestions };
        }

        case "column": {
          await catalog.ensureLoaded(connectionId);
          const suggestions: monaco.languages.CompletionItem[] = [];
          const seen = new Set<string>();

          for (const ref of ctx.tableRefs) {
            const columns = await catalog.ensureTableColumns(connectionId, ref.schema, ref.table);
            for (const col of columns) {
              if (seen.has(col.name)) continue;
              seen.add(col.name);
              suggestions.push({
                label: col.name,
                kind: monacoInstance.languages.CompletionItemKind.Field,
                detail: `${col.dataType}${col.nullable ? "" : " NOT NULL"}`,
                insertText: col.name,
                range,
              });
            }
          }

          if (suggestions.length === 0) {
            return {
              suggestions: SQL_KEYWORDS.map((kw) => ({
                label: kw.label,
                kind: monacoInstance.languages.CompletionItemKind.Keyword,
                detail: kw.detail,
                insertText: kw.label,
                range,
              })),
            };
          }

          return { suggestions };
        }

        case "qualifiedColumn": {
          if (!ctx.qualifier) return { suggestions: [] };

          await catalog.ensureLoaded(connectionId);
          const cat = catalog.getCatalog(connectionId);
          if (!cat) return { suggestions: [] };

          const aliasRef = ctx.tableRefs.find((r) => r.alias === ctx.qualifier);
          let schema: string;
          let table: string;

          if (aliasRef) {
            schema = aliasRef.schema;
            table = aliasRef.table;
          } else {
            const objMatch = cat.objects.find(
              (o) => o.name === ctx.qualifier || `${o.schema}.${o.name}` === ctx.qualifier,
            );
            if (!objMatch) return { suggestions: [] };
            schema = objMatch.schema;
            table = objMatch.name;
          }

          const columns = await catalog.ensureTableColumns(connectionId, schema, table);
          return {
            suggestions: columns.map((col) => ({
              label: col.name,
              kind: monacoInstance.languages.CompletionItemKind.Field,
              detail: `${col.dataType}${col.nullable ? "" : " NOT NULL"}`,
              insertText: col.name,
              range,
            })),
          };
        }

        case "schema": {
          await catalog.ensureLoaded(connectionId);
          const cat = catalog.getCatalog(connectionId);
          if (!cat) return { suggestions: [] };
          return {
            suggestions: cat.schemas.map((s) => ({
              label: s.name,
              kind: monacoInstance.languages.CompletionItemKind.Folder,
              detail: "Schema",
              insertText: s.name,
              range,
            })),
          };
        }

        default:
          return { suggestions: [] };
      }
    },
  };
}
