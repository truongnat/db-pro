import Editor, { type OnMount } from "@monaco-editor/react";
import { useCallback, useRef } from "react";

import { registerSqlProviders } from "../services/sql-providers";

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  path?: string;
  onExecute?: () => void;
}

export function QueryEditor({
  value,
  onChange,
  readOnly = false,
  path,
  onExecute,
}: QueryEditorProps) {
  const onExecuteRef = useRef(onExecute);
  onExecuteRef.current = onExecute;

  const handleMount: OnMount = useCallback((editor, monacoInstance) => {
    registerSqlProviders(monacoInstance);

    if (onExecuteRef.current) {
      editor.addCommand(
        monacoInstance.KeyMod.CtrlCmd | monacoInstance.KeyCode.Enter,
        () => onExecuteRef.current?.(),
      );
      editor.addCommand(monacoInstance.KeyCode.F5, () => onExecuteRef.current?.());
    }
  }, []);

  return (
    <div className="h-full w-full border border-border">
      <Editor
        height="100%"
        language="sql"
        theme="vs-dark"
        path={path}
        value={value}
        onChange={(v) => onChange(v ?? "")}
        onMount={handleMount}
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
