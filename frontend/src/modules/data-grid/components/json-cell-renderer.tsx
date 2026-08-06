import { useState } from "react";

import { Button } from "@/components/ui/button";

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
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-auto cursor-pointer px-0 py-0 text-left font-mono text-xs text-primary hover:bg-transparent"
        onClick={() => setExpanded(true)}
        title={preview}
      >
        {"{ } "}
        <span className="text-muted-foreground">{shortPreview}</span>
      </Button>
    );
  }

  return (
    <div className="font-mono text-xs">
      <Button
        type="button"
        variant="link"
        size="sm"
        className="mb-1 h-auto px-0 py-0 text-xs underline"
        onClick={() => setExpanded(false)}
      >
        collapse
      </Button>
      <pre className="max-h-[200px] max-w-[400px] overflow-auto whitespace-pre-wrap rounded bg-background p-1 text-xs text-foreground">
        {JSON.stringify(value, null, 2)}
      </pre>
    </div>
  );
}
