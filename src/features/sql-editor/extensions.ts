import {
  acceptCompletion,
  autocompletion,
  startCompletion,
  type CompletionContext,
} from "@codemirror/autocomplete";
import { Prec, type Extension } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { MySQL, PostgreSQL, SQLite, sql } from "@codemirror/lang-sql";

import type { DbEngine } from "@/types";

import { resolveSqlCompletions } from "./completions";
import { sqlEditorStyling } from "./theme";
import type { SqlCompletionCatalog } from "./types";

type BuildSqlEditorExtensionsInput = {
  engine: DbEngine | undefined;
  catalog: SqlCompletionCatalog;
  busy: boolean;
  onRunQuery: () => void;
  onFormatSql: () => void | Promise<void>;
};

function resolveDialect(engine: DbEngine | undefined) {
  if (engine === "postgres") {
    return PostgreSQL;
  }
  if (engine === "mysql") {
    return MySQL;
  }
  return SQLite;
}

export function buildSqlEditorExtensions({
  engine,
  catalog,
  busy,
  onRunQuery,
  onFormatSql,
}: BuildSqlEditorExtensionsInput): Extension[] {
  return [
    sql({ dialect: resolveDialect(engine) }),
    ...sqlEditorStyling,
    autocompletion({
      activateOnTyping: true,
      maxRenderedOptions: 18,
      override: [
        (context: CompletionContext) => resolveSqlCompletions(context, catalog),
      ],
    }),
    Prec.high(
      keymap.of([
        {
          key: "Mod-Enter",
          run: () => {
            if (busy) {
              return true;
            }
            onRunQuery();
            return true;
          },
        },
        {
          key: "Shift-Mod-f",
          run: () => {
            void onFormatSql();
            return true;
          },
        },
        {
          key: "Alt-Shift-f",
          run: () => {
            void onFormatSql();
            return true;
          },
        },
        {
          key: "Mod-Space",
          run: startCompletion,
        },
        {
          key: "Tab",
          run: acceptCompletion,
        },
      ]),
    ),
  ];
}
