import { useCallback } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute: () => void;
  readOnly?: boolean;
}

export function QueryEditor({
  value,
  onChange,
  onExecute,
  readOnly = false,
}: QueryEditorProps) {
  const handleMount: OnMount = useCallback(
    (editor, monaco) => {
      editor.addAction({
        id: "execute-query",
        label: "Execute Query",
        keybindings: [
          monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter,
        ],
        run: () => {
          onExecute();
        },
      });
    },
    [onExecute],
  );

  return (
    <div className="h-full w-full" style={{ border: "1px solid var(--color-border)" }}>
      <Editor
        height="100%"
        language="sql"
        theme="vs-dark"
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
        }}
      />
    </div>
  );
}
