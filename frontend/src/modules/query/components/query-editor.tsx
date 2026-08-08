import Editor, { type OnMount } from "@monaco-editor/react";
import { useCallback, useEffect, useRef } from "react";

import { onQueryAction } from "@/commons/commands/query-dispatch";
import { useQueryEditorContextStore } from "@/commons/stores/query-editor-context.store";
import { registerSqlProviders } from "../services/sql-providers";
import { resolveRunTarget } from "../services/statement-splitter";

type MonacoEditorInstance = Parameters<OnMount>[0];

/**
 * Extract tabId from the Monaco editor path.
 * Path format: `dbpro://query/<tabId>.sql`
 */
function extractTabId(path: string | undefined): string | null {
  if (!path) return null;
  const match = path.match(/dbpro:\/\/query\/(.+?)\.sql$/);
  return match?.[1] ?? null;
}

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  path?: string;
  onExecute?: (sql: string) => void;
  onExecuteAll?: () => void;
  onCancel?: () => void;
}

export function QueryEditor({
  value,
  onChange,
  readOnly = false,
  path,
  onExecute,
  onExecuteAll,
  onCancel,
}: QueryEditorProps) {
  const onExecuteRef = useRef(onExecute);
  const onExecuteAllRef = useRef(onExecuteAll);
  const onCancelRef = useRef(onCancel);
  onExecuteRef.current = onExecute;
  onExecuteAllRef.current = onExecuteAll;
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
      }
      if (onExecuteAllRef.current) {
        editor.addCommand(
          monacoInstance.KeyMod.CtrlCmd | monacoInstance.KeyMod.Shift | monacoInstance.KeyCode.Enter,
          () => {
            onExecuteAllRef.current?.();
          },
        );
        editor.addCommand(monacoInstance.KeyCode.F5, () => {
          onExecuteAllRef.current?.();
        });
      }

      // Escape → cancel (independent of onExecute so it always works)
      if (onCancelRef.current) {
        editor.addCommand(monacoInstance.KeyCode.Escape, () => {
          onCancelRef.current?.();
        });
      }

      // Track cursor/selection state for the Action Platform.
      // Command Palette and Keyboard read from this store to resolve
      // "current statement" instead of hardcoding cursorOffset=0.
      const tabId = extractTabId(path);
      if (tabId) {
        const updateContext = () => {
          const model = editor.getModel();
          if (!model) return;
          const pos = editor.getPosition();
          const offset = pos ? model.getOffsetAt(pos) : 0;
          const sel = editor.getSelection();
          const selectionRange =
            sel && !sel.isEmpty()
              ? {
                  start: model.getOffsetAt(sel.getStartPosition()),
                  end: model.getOffsetAt(sel.getEndPosition()),
                }
              : null;
          useQueryEditorContextStore
            .getState()
            .setEditorContext(tabId, { cursorOffset: offset, selection: selectionRange });
        };

        editor.onDidChangeCursorPosition(updateContext);
        editor.onDidChangeCursorSelection(updateContext);

        // Seed initial state.
        updateContext();
      }
    },
    [runTargetSql, runAllSql, path],
  );

  // Listen for "executeCurrent" dispatch from toolbar
  useEffect(() => {
    return onQueryAction("executeCurrent", () => {
      if (!editorRef.current) return;
      const sql = runTargetSql(editorRef.current);
      if (sql) onExecuteRef.current?.(sql);
    });
  }, [runTargetSql]);

  // Clean up editor context when the component unmounts (tab close).
  useEffect(() => {
    const tabId = extractTabId(path);
    if (!tabId) return;
    return () => {
      useQueryEditorContextStore.getState().removeEditorContext(tabId);
    };
  }, [path]);

  const editorRef = useRef<MonacoEditorInstance | null>(null);

  const handleMountWrapped: OnMount = useCallback(
    (editor, monacoInstance) => {
      editorRef.current = editor;
      handleMount(editor, monacoInstance);
    },
    [handleMount],
  );

  return (
    <div className="h-full w-full bg-[var(--app-editor-bg,var(--app-surface-3))]">
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
          lineHeight: 22,
          wordWrap: "on",
          readOnly,
          automaticLayout: true,
          padding: { top: 12, bottom: 12 },
          renderLineHighlight: "line",
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
