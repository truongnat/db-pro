import { useCallback, useMemo, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaColumnDto } from "../types/schema.types";

interface GenerateCrudProps {
  schema: string;
  table: string;
  columns: SchemaColumnDto[];
}

type CrudType = "select" | "insert" | "update" | "delete";

function quoteId(name: string): string {
  return `"${name.replace(/"/g, '""')}"`;
}

function qualify(s: string, t: string): string {
  return `${quoteId(s)}.${quoteId(t)}`;
}

function generateSelect(schema: string, table: string, columns: SchemaColumnDto[]): string {
  const q = qualify(schema, table);
  const cols = columns.map((c) => `  ${quoteId(c.name)}`).join(",\n");
  return `SELECT\n${cols}\nFROM ${q};`;
}

function generateInsert(schema: string, table: string, columns: SchemaColumnDto[]): string {
  const q = qualify(schema, table);
  const colNames = columns.map((c) => quoteId(c.name)).join(", ");
  const placeholders = columns.map((c) => {
    if (c.isPrimaryKey && c.defaultValue) return "DEFAULT";
    if (!c.nullable && !c.defaultValue) return `'<${c.name}>'`;
    if (c.defaultValue) return c.defaultValue;
    return "NULL";
  });
  return `INSERT INTO ${q} (${colNames})\nVALUES (${placeholders.join(", ")});`;
}

function generateUpdate(schema: string, table: string, columns: SchemaColumnDto[]): string {
  const q = qualify(schema, table);
  const pkCols = columns.filter((c) => c.isPrimaryKey);
  const nonPkCols = columns.filter((c) => !c.isPrimaryKey);

  const setClauses = nonPkCols.map((c) => `  ${quoteId(c.name)} = <${c.name}>`).join(",\n");
  const whereClauses = pkCols.map((c) => `${quoteId(c.name)} = <${c.name}>`).join(" AND ");

  return `UPDATE ${q}\nSET\n${setClauses}\nWHERE ${whereClauses || "1 = 1 /* no PK */"};`;
}

function generateDelete(schema: string, table: string, columns: SchemaColumnDto[]): string {
  const q = qualify(schema, table);
  const pkCols = columns.filter((c) => c.isPrimaryKey);
  const whereClauses = pkCols.map((c) => `${quoteId(c.name)} = <${c.name}>`).join(" AND ");
  return `DELETE FROM ${q}\nWHERE ${whereClauses || "1 = 1 /* no PK */"};`;
}

export function GenerateCrud({ schema, table, columns }: GenerateCrudProps) {
  const { t } = useTranslation();
  const [activeType, setActiveType] = useState<CrudType>("select");
  const [copied, setCopied] = useState(false);

  const sql = useMemo(() => {
    switch (activeType) {
      case "select":
        return generateSelect(schema, table, columns);
      case "insert":
        return generateInsert(schema, table, columns);
      case "update":
        return generateUpdate(schema, table, columns);
      case "delete":
        return generateDelete(schema, table, columns);
    }
  }, [activeType, schema, table, columns]);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(sql);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [sql]);

  const types: { key: CrudType; label: string }[] = [
    { key: "select", label: "SELECT" },
    { key: "insert", label: "INSERT" },
    { key: "update", label: "UPDATE" },
    { key: "delete", label: "DELETE" },
  ];

  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="flex gap-2">
        {types.map((type) => (
          <button
            key={type.key}
            type="button"
            onClick={() => setActiveType(type.key)}
            className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs font-medium transition-colors"
            style={{
              borderColor: activeType === type.key ? "var(--color-primary, #3b82f6)" : "var(--color-border)",
              backgroundColor: activeType === type.key ? "var(--color-primary, #3b82f6)" : "transparent",
              color: activeType === type.key ? "white" : "var(--color-text)",
            }}
          >
            {type.label}
          </button>
        ))}
      </div>

      <pre
        className="overflow-auto rounded-[var(--radius-sm)] border p-3 font-mono text-xs leading-relaxed"
        style={{
          borderColor: "var(--color-border)",
          color: "var(--color-text)",
          backgroundColor: "var(--color-bg-secondary, var(--color-bg))",
          maxHeight: "300px",
        }}
      >
        <code>{sql}</code>
      </pre>

      <button
        type="button"
        onClick={handleCopy}
        className="self-start rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
        style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
      >
        {copied ? t("schema.copied") : t("schema.copyDdl")}
      </button>
    </div>
  );
}
