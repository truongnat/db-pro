import { useState } from "react";

interface JsonCellRendererProps {
  value: unknown;
}

export function JsonCellRenderer({ value }: JsonCellRendererProps) {
  const [expanded, setExpanded] = useState(false);

  const preview = typeof value === "string" ? value : JSON.stringify(value);
  const shortPreview =
    preview && preview.length > 40 ? `${preview.slice(0, 40)}...` : preview;

  if (!expanded) {
    return (
      <button
        type="button"
        className="cursor-pointer text-left font-mono text-xs"
        style={{ color: "var(--color-primary, #3b82f6)" }}
        onClick={() => setExpanded(true)}
        title={preview}
      >
        {"{ } "}
        <span style={{ color: "var(--color-text-secondary)" }}>{shortPreview}</span>
      </button>
    );
  }

  return (
    <div className="font-mono text-xs">
      <button
        type="button"
        className="mb-1 text-xs underline"
        style={{ color: "var(--color-primary, #3b82f6)" }}
        onClick={() => setExpanded(false)}
      >
        collapse
      </button>
      <pre
        className="max-h-[200px] max-w-[400px] overflow-auto whitespace-pre-wrap rounded p-1 text-xs"
        style={{
          backgroundColor: "var(--color-bg)",
          color: "var(--color-text)",
        }}
      >
        {JSON.stringify(value, null, 2)}
      </pre>
    </div>
  );
}
