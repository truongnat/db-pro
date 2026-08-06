import { useEffect, useRef, useState } from "react";

import type { CellValue } from "../types/data-grid.types";
import { renderCellValue } from "@/modules/query/types/query.types";

interface CellEditorProps {
  value: CellValue;
  onSave: (value: CellValue) => void;
  onCancel: () => void;
}

export function CellEditor({ value, onSave, onCancel }: CellEditorProps) {
  const [text, setText] = useState(renderCellValue(value));
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const commit = () => {
    if (text === renderCellValue(value)) {
      onCancel();
      return;
    }
    if (text.toUpperCase() === "NULL") {
      onSave({ type: "null" });
      return;
    }
    switch (value.type) {
      case "int64": {
        const n = Number(text);
        if (!Number.isNaN(n)) onSave({ type: "int64", value: n });
        else onCancel();
        return;
      }
      case "float64": {
        const n = Number(text);
        if (!Number.isNaN(n)) onSave({ type: "float64", value: n });
        else onCancel();
        return;
      }
      case "bool":
        onSave({ type: "bool", value: text.toLowerCase() === "true" });
        return;
      default:
        onSave({ type: "text", value: text });
    }
  };

  return (
    <input
      ref={inputRef}
      className="absolute inset-0 z-20 w-full border-2 px-2 text-xs outline-none"
      style={{
        borderColor: "var(--color-primary, #3b82f6)",
        backgroundColor: "var(--color-surface)",
        color: "var(--color-text)",
      }}
      value={text}
      onChange={(e) => setText(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter") commit();
        else if (e.key === "Escape") onCancel();
      }}
      onBlur={commit}
    />
  );
}
