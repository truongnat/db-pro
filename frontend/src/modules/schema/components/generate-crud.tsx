import { useCallback, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";
import { getDialectForConnection } from "@/modules/query/sql/dialect";
import {
  generateDeleteSQL,
  generateInsertSQL,
  generateSelectSQL,
  generateUpdateSQL,
} from "@/modules/query/sql/generators";
import type { SchemaColumnDto } from "../types/schema.types";

interface GenerateCrudProps {
  connectionId: string;
  schema: string;
  table: string;
  columns: SchemaColumnDto[];
}

type CrudType = "select" | "insert" | "update" | "delete";

export function GenerateCrud({ connectionId, schema, table, columns }: GenerateCrudProps) {
  const { t } = useTranslation();
  const [activeType, setActiveType] = useState<CrudType>("select");
  const [copied, setCopied] = useState(false);

  const dialect = useMemo(() => getDialectForConnection(connectionId), [connectionId]);

  const sql = useMemo(() => {
    switch (activeType) {
      case "select":
        return generateSelectSQL(dialect, schema, table, columns);
      case "insert":
        return generateInsertSQL(dialect, schema, table, columns);
      case "update":
        return generateUpdateSQL(dialect, schema, table, columns);
      case "delete":
        return generateDeleteSQL(dialect, schema, table, columns);
    }
  }, [activeType, dialect, schema, table, columns]);

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
          <Button
            key={type.key}
            type="button"
            size="sm"
            onClick={() => setActiveType(type.key)}
            variant={activeType === type.key ? "default" : "outline"}
          >
            {type.label}
          </Button>
        ))}
      </div>

      <pre
        className="overflow-auto rounded-sm border border-[var(--app-border-subtle)] p-3 font-mono text-xs leading-relaxed text-foreground bg-muted"
        style={{ maxHeight: "300px" }}
      >
        <code>{sql}</code>
      </pre>

      <Button
        type="button"
        variant="outline"
        size="sm"
        className="self-start"
        onClick={handleCopy}
      >
        {copied ? t("schema.copied") : t("schema.copyDdl")}
      </Button>
    </div>
  );
}
