import Editor, { type OnMount } from "@monaco-editor/react";
import { useCallback, useEffect, useRef } from "react";

import { onQueryAction } from "@/commons/commands/query-dispatch";
import { registerSqlProviders } from "../services/sql-providers";
import { resolveRunTarget } from "../services/statement-splitter";

type MonacoEditorInstance = Parameters<OnMount>[0];

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  path?: string;
  onExecute?: (sql: string) => void;
  onCancel?: () => void;
}

export function QueryEditor({
  value,
  onChange,
  readOnly = false,
  path,
  onExecute,
  onCancel,
}: QueryEditorProps) {
  const onExecuteRef = useRef(onExecute);
  const onCancelRef = useRef(onCancel);
  onExecuteRef.current = onExecute;
  onCancelRef.current = onCancel;

  const runTargetSql = useCallback((editor: MonacoEditorInstance): string | undefined => {
    const model = editor.getModel();
    const position = editor.getPosition();
    if (!model || !position) return undefined;

    const selection = editor.getSelection();
    const selectionRange =
      selection && !selection.isEmpty()
        ? {
            start: model.getOffsetAt(selection.getStartPosition()),
            end: model.getOffsetAt(selection.getEndPosition()),
          }
        : null;

    return resolveRunTarget({
      value: model.getValue(),
      selection: selectionRange,
      cursorOffset: model.getOffsetAt(position),
    });
  }, []);

  const runAllSql = useCallback((editor: MonacoEditorInstance): string | undefined => {
    const value = editor.getModel()?.getValue().trim();
    return value || undefined;
  }, []);

  const handleMount: OnMount = useCallback(
    (editor, monacoInstance) => {
      registerSqlProviders(monacoInstance);

      if (onExecuteRef.current) {
        editor.addCommand(
          monacoInstance.KeyMod.CtrlCmd | monacoInstance.KeyCode.Enter,
          () => {
            const sql = runTargetSql(editor);
            if (sql) onExecuteRef.current?.(sql);
          },
        );
        editor.addCommand(
          monacoInstance.KeyMod.CtrlCmd | monacoInstance.KeyMod.Shift | monacoInstance.KeyCode.Enter,
          () => {
            const sql = runAllSql(editor);
            if (sql) onExecuteRef.current?.(sql);
          },
        );
        editor.addCommand(monacoInstance.KeyCode.F5, () => {
          const sql = runAllSql(editor);
          if (sql) onExecuteRef.current?.(sql);
        });
      }

      // Escape → cancel (independent of onExecute so it always works)
      if (onCancelRef.current) {
        editor.addCommand(monacoInstance.KeyCode.Escape, () => {
          onCancelRef.current?.();
        });
      }
    },
    [runTargetSql, runAllSql],
  );

  // Listen for "executeCurrent" dispatch from toolbar
  useEffect(() => {
    return onQueryAction("executeCurrent", () => {
      if (!editorRef.current) return;
      const sql = runTargetSql(editorRef.current);
      if (sql) onExecuteRef.current?.(sql);
    });
  }, [runTargetSql]);

  const editorRef = useRef<MonacoEditorInstance | null>(null);

  const handleMountWrapped: OnMount = useCallback(
    (editor, monacoInstance) => {
      editorRef.current = editor;
      handleMount(editor, monacoInstance);
    },
    [handleMount],
  );

  return (
    <div className="h-full w-full border border-border">
      <Editor
        height="100%"
        language="sql"
        theme="vs-dark"
        path={path}
        value={value}
        onChange={(v) => onChange(v ?? "")}
        onMount={handleMountWrapped}
        options={{
          minimap: { enabled: false },
          lineNumbers: "on",
          scrollBeyondLastLine: false,
          fontSize: 14,
          wordWrap: "on",
          readOnly,
          automaticLayout: true,
          padding: { top: 8 },
          suggest: {
            showKeywords: true,
            showWords: false,
            preview: true,
          },
          quickSuggestions: {
            other: true,
            comments: false,
            strings: false,
          },
        }}
      />
    </div>
  );
}
