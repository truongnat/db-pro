import type { Completion } from "@codemirror/autocomplete";

export type SqlRunSource = "selection" | "current" | "all";

export type SqlStatement = {
  sql: string;
  start: number;
  end: number;
};

export type SqlRunTarget = {
  sql: string;
  source: SqlRunSource;
};

export type SqlEditorSelection = {
  anchor: number;
  head: number;
};

export type SqlCompletionCatalog = {
  global: Completion[];
  columnsByScope: Map<string, Completion[]>;
};
