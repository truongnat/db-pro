import { useEffect, useRef, useState } from "react";

import { Input } from "@/components/ui/input";

import type { CellValue } from "../types/data-grid.types";
import { renderCellValue } from "@/modules/query/types/query.types";

interface CellEditorProps {
  value: CellValue;
  onSave: (value: CellValue) => void;
  onCancel: () => void;
  columnType?: string;
}

export function CellEditor({ value, onSave, onCancel, columnType }: CellEditorProps) {
  const [text, setText] = useState(renderCellValue(value));
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const effectiveType = columnType ? normalizeColumnType(columnType) : value.type;

  const commit = () => {
    if (text === renderCellValue(value)) {
      onCancel();
      return;
    }
    if (text.toUpperCase() === "NULL") {
      onSave({ type: "null" });
      return;
    }
    switch (effectiveType) {
      case "bool": {
        const lower = text.toLowerCase();
        if (lower === "true" || lower === "t" || lower === "1" || lower === "yes") {
          onSave({ type: "bool", value: true });
        } else if (lower === "false" || lower === "f" || lower === "0" || lower === "no") {
          onSave({ type: "bool", value: false });
        } else {
          setError("Enter true/false");
        }
        return;
      }
      case "int64": {
        const n = Number(text);
        if (Number.isInteger(n)) {
          onSave({ type: "int64", value: n });
        } else {
          setError("Enter a valid integer");
        }
        return;
      }
      case "float64": {
        const n = Number(text);
        if (!Number.isNaN(n)) {
          onSave({ type: "float64", value: n });
        } else {
          setError("Enter a valid number");
        }
        return;
      }
      case "json": {
        try {
          const parsed = JSON.parse(text);
          onSave({ type: "json", value: parsed });
        } catch {
          setError("Invalid JSON");
        }
        return;
      }
      case "uuid":
        onSave({ type: "uuid", value: text });
        return;
      case "datetime":
        onSave({ type: "datetime", value: text });
        return;
      case "bytes":
        onSave({ type: "text", value: text });
        return;
      default:
        onSave({ type: "text", value: text });
    }
  };

  if (effectiveType === "bool") {
    const checked = text.toLowerCase() === "true" || text === "1";
    return (
      <div className="absolute inset-0 z-20 flex items-center justify-center bg-background">
        <input
          type="checkbox"
          className="h-4 w-4 accent-primary"
          checked={checked}
          onChange={(e) => {
            const newVal = e.target.checked;
            if (newVal !== (value.type === "bool" ? value.value : false)) {
              onSave({ type: "bool", value: newVal });
            } else {
              onCancel();
            }
          }}
          onKeyDown={(e) => {
            if (e.key === "Escape") onCancel();
          }}
          autoFocus
        />
      </div>
    );
  }

  return (
    <div className="absolute inset-0 z-20 flex flex-col">
      <Input
        ref={inputRef}
        className={`h-full w-full rounded-none border-2 bg-background px-2 text-xs shadow-none focus-visible:ring-0 ${
          error ? "border-destructive" : "border-primary"
        }`}
        value={text}
        onChange={(e) => {
          setText(e.target.value);
          if (error) setError(null);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          else if (e.key === "Escape") onCancel();
        }}
        onBlur={commit}
      />
      {error && (
        <span className="absolute -bottom-4 left-0 whitespace-nowrap rounded bg-destructive px-1.5 py-0.5 text-[10px] text-white">
          {error}
        </span>
      )}
    </div>
  );
}

function normalizeColumnType(dataType: string): CellValue["type"] {
  const dt = dataType.toLowerCase();
  if (dt === "boolean" || dt === "bool") return "bool";
  if (dt === "uuid") return "uuid";
  if (dt === "json" || dt === "jsonb") return "json";
  if (dt.includes("int")) return "int64";
  if (dt.includes("float") || dt.includes("double") || dt.includes("numeric") || dt.includes("decimal") || dt.includes("real")) return "float64";
  if (dt.includes("timestamp") || dt.includes("date") || dt.includes("time")) return "datetime";
  if (dt.includes("bytea") || dt.includes("blob")) return "bytes";
  return "text";
}
