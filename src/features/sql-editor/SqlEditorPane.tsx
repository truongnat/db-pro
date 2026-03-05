import { useMemo } from "react";

import type { DbEngine } from "@/types";

import { buildSqlEditorExtensions } from "./extensions";
import { SqlCodeEditor } from "./SqlCodeEditor";
import type { SqlCompletionCatalog, SqlEditorSelection } from "./types";

type SqlEditorPaneProps = {
  value: string;
  engine: DbEngine | undefined;
  completionCatalog: SqlCompletionCatalog;
  busy: boolean;
  onRunQuery: () => void;
  onFormatSql: () => void | Promise<void>;
  onValueChange: (nextValue: string) => void;
  onSelectionChange: (selection: SqlEditorSelection) => void;
};

export default function SqlEditorPane({
  value,
  engine,
  completionCatalog,
  busy,
  onRunQuery,
  onFormatSql,
  onValueChange,
  onSelectionChange,
}: SqlEditorPaneProps) {
  const sqlEditorExtensions = useMemo(
    () =>
      buildSqlEditorExtensions({
        engine,
        catalog: completionCatalog,
        busy,
        onRunQuery,
        onFormatSql,
      }),
    [busy, completionCatalog, engine, onFormatSql, onRunQuery],
  );

  return (
    <SqlCodeEditor
      value={value}
      extensions={sqlEditorExtensions}
      onValueChange={onValueChange}
      onSelectionChange={onSelectionChange}
    />
  );
}
