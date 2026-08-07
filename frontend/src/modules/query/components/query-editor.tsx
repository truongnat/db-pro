import Editor from "@monaco-editor/react";

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  path?: string;
}

export function QueryEditor({
  value,
  onChange,
  readOnly = false,
  path,
}: QueryEditorProps) {
  return (
    <div className="h-full w-full border border-border">
      <Editor
        height="100%"
        language="sql"
        theme="vs-dark"
        path={path}
        value={value}
        onChange={(v) => onChange(v ?? "")}
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
