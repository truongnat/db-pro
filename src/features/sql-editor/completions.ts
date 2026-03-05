import type { Completion, CompletionContext } from "@codemirror/autocomplete";

import type { NavigatorTree } from "@/types";

import type { SqlCompletionCatalog } from "./types";

const SQL_KEYWORDS = [
  "SELECT",
  "FROM",
  "WHERE",
  "JOIN",
  "LEFT JOIN",
  "RIGHT JOIN",
  "INNER JOIN",
  "GROUP BY",
  "ORDER BY",
  "LIMIT",
  "OFFSET",
  "INSERT INTO",
  "UPDATE",
  "DELETE FROM",
  "CREATE TABLE",
  "ALTER TABLE",
  "DROP TABLE",
  "CREATE VIEW",
  "DROP VIEW",
  "WITH",
  "UNION",
  "DISTINCT",
  "HAVING",
  "VALUES",
  "AS",
  "AND",
  "OR",
  "NOT",
] as const;

const COMPLETION_CACHE_LIMIT = 8;
const completionCatalogCacheByTree = new WeakMap<NavigatorTree, SqlCompletionCatalog>();
const completionCatalogCacheByKey = new Map<string, SqlCompletionCatalog>();

const keywordCompletions: Completion[] = SQL_KEYWORDS.map((keyword) => ({
  label: keyword,
  type: "keyword",
  detail: "SQL keyword",
  apply: `${keyword} `,
  boost: 90,
}));

const baseCatalog: SqlCompletionCatalog = {
  global: keywordCompletions,
  columnsByScope: new Map<string, Completion[]>(),
};

function normalizeSqlIdentifier(value: string): string {
  return value.trim().replace(/"/g, "").toLowerCase();
}

function buildCatalogCacheKey(tree: NavigatorTree): string {
  const parts: string[] = [];

  for (const schema of tree.schemas) {
    parts.push(`schema:${schema.name}`);

    for (const table of schema.tables) {
      parts.push(`table:${table.name}`);
      for (const column of table.columns) {
        parts.push(
          `column:${column.name}:${column.dataType}:${column.nullable ? "1" : "0"}`,
        );
      }
    }

    for (const view of schema.views) {
      parts.push(`view:${view.name}`);
      for (const column of view.columns) {
        parts.push(
          `column:${column.name}:${column.dataType}:${column.nullable ? "1" : "0"}`,
        );
      }
    }
  }

  return parts.join("|");
}

function rememberCatalogByKey(key: string, catalog: SqlCompletionCatalog) {
  completionCatalogCacheByKey.set(key, catalog);
  if (completionCatalogCacheByKey.size <= COMPLETION_CACHE_LIMIT) {
    return;
  }

  const oldestKey = completionCatalogCacheByKey.keys().next().value;
  if (oldestKey) {
    completionCatalogCacheByKey.delete(oldestKey);
  }
}

function buildObjectPreview(
  kind: "TABLE" | "VIEW",
  columns: { name: string; dataType: string }[],
): string {
  if (columns.length === 0) {
    return `${kind}(...)`;
  }

  const preview = columns
    .slice(0, 3)
    .map((column) => `${column.name} ${column.dataType || "unknown"}`)
    .join(", ");
  const suffix = columns.length > 3 ? ", ..." : "";

  return `${kind}(${preview}${suffix})`;
}

function dedupeCompletions(items: Completion[]): Completion[] {
  const map = new Map<string, Completion>();
  for (const item of items) {
    const key = `${item.type ?? "item"}::${item.label}::${item.detail ?? ""}`;
    if (!map.has(key)) {
      map.set(key, item);
    }
  }
  return Array.from(map.values());
}

function buildSqlCompletionCatalog(tree: NavigatorTree): SqlCompletionCatalog {
  const global: Completion[] = [...keywordCompletions];
  const columnsByScope = new Map<string, Completion[]>();

  for (const schema of tree.schemas) {
    global.push({
      label: schema.name,
      apply: schema.name,
      detail: "Schema",
      type: "namespace",
      boost: 60,
    });

    const registerObject = (
      object: {
        name: string;
        columns: {
          name: string;
          dataType: string;
          nullable: boolean;
        }[];
      },
      kind: "table" | "view",
    ) => {
      const qualifiedName = `${schema.name}.${object.name}`;
      const objectKindLabel = kind === "table" ? "TABLE" : "VIEW";

      global.push({
        label: object.name,
        apply: object.name,
        detail: buildObjectPreview(objectKindLabel, object.columns),
        type: kind === "table" ? "class" : "interface",
        boost: 70,
      });

      global.push({
        label: qualifiedName,
        apply: qualifiedName,
        detail: buildObjectPreview(objectKindLabel, object.columns),
        type: kind === "table" ? "class" : "interface",
        boost: 72,
      });

      const columnSuggestions: Completion[] = object.columns.map((column) => ({
        label: column.name,
        apply: column.name,
        detail: `${column.dataType || "unknown"}${column.nullable ? "" : " NOT NULL"}`,
        type: "property",
        boost: 80,
      }));

      const scopedKeys = [
        normalizeSqlIdentifier(object.name),
        normalizeSqlIdentifier(qualifiedName),
      ];

      for (const scopedKey of scopedKeys) {
        columnsByScope.set(scopedKey, dedupeCompletions(columnSuggestions));
      }
    };

    for (const table of schema.tables) {
      registerObject(table, "table");
    }
    for (const view of schema.views) {
      registerObject(view, "view");
    }
  }

  return {
    global: dedupeCompletions(global),
    columnsByScope,
  };
}

export function getSqlCompletionCatalog(
  tree: NavigatorTree | null,
): SqlCompletionCatalog {
  if (!tree) {
    return baseCatalog;
  }

  const cachedByTree = completionCatalogCacheByTree.get(tree);
  if (cachedByTree) {
    return cachedByTree;
  }

  const cacheKey = buildCatalogCacheKey(tree);
  const cachedByKey = completionCatalogCacheByKey.get(cacheKey);
  if (cachedByKey) {
    completionCatalogCacheByTree.set(tree, cachedByKey);
    return cachedByKey;
  }

  const catalog = buildSqlCompletionCatalog(tree);
  completionCatalogCacheByTree.set(tree, catalog);
  rememberCatalogByKey(cacheKey, catalog);

  return catalog;
}

function scoreCompletions(items: Completion[], rawPrefix: string): Completion[] {
  const prefix = normalizeSqlIdentifier(rawPrefix);
  if (!prefix) {
    return items.slice(0, 80);
  }

  const startsWith: Completion[] = [];
  const contains: Completion[] = [];

  for (const suggestion of items) {
    const label = normalizeSqlIdentifier(suggestion.label ?? "");
    if (label.startsWith(prefix)) {
      startsWith.push(suggestion);
      continue;
    }
    if (label.includes(prefix)) {
      contains.push(suggestion);
    }
  }

  return [...startsWith, ...contains].slice(0, 80);
}

export function resolveSqlCompletions(
  context: CompletionContext,
  catalog: SqlCompletionCatalog,
) {
  const tokenRange = context.matchBefore(/[A-Za-z0-9_$."]*/);
  if (!tokenRange) {
    return null;
  }

  if (!shouldOpenCompletions(context, tokenRange.from, tokenRange.to)) {
    return null;
  }

  const rawParts = tokenRange.text.split(".");
  const rawScope =
    rawParts.length > 1 ? rawParts.slice(0, rawParts.length - 1).join(".") : "";
  const rawPrefix = rawParts[rawParts.length - 1] ?? "";

  const normalizedScope = normalizeSqlIdentifier(rawScope);
  const scopedSuggestions = normalizedScope
    ? catalog.columnsByScope.get(normalizedScope)
    : undefined;

  let items = scoreCompletions(scopedSuggestions ?? catalog.global, rawPrefix);
  if (items.length === 0 && scopedSuggestions) {
    items = scoreCompletions(catalog.global, tokenRange.text);
  }

  if (items.length === 0) {
    return null;
  }

  const replaceEnd = tokenRange.to;
  const replaceStart = replaceEnd - rawPrefix.length;
  return {
    from: replaceStart,
    to: replaceEnd,
    options: items,
  };
}

function shouldOpenCompletions(
  context: CompletionContext,
  tokenStart: number,
  tokenEnd: number,
): boolean {
  if (context.explicit) {
    return true;
  }

  if (tokenStart !== tokenEnd) {
    return true;
  }

  const previousChar = context.state.sliceDoc(Math.max(0, context.pos - 1), context.pos);
  if (!previousChar) {
    return false;
  }

  return /[\s(,]/.test(previousChar);
}
