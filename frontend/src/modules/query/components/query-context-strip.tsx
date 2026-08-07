import { Database } from "lucide-react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTranslation } from "@/commons/locales/useTranslation";
import type { QueryContext } from "@/commons/types/workspace.types";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";

import { setQueryTabConnection, setQueryTabSchema } from "../controllers/query-workspace.controller";

const DEFAULT_SCHEMA = "__default__";

interface QueryContextStripProps {
  tabId: string;
  connectionId: string | null;
  context: QueryContext;
}

export function QueryContextStrip({ tabId, connectionId, context }: QueryContextStripProps) {
  const { t } = useTranslation();
  const { data: connections } = useConnectionList();
  const catalogs = useSchemaCatalogStore((s) => s.catalogs);

  const connection = connections?.find((c) => c.id === connectionId);
  const database = context.database ?? connection?.database ?? null;
  const schemas = catalogs.get(connectionId ?? "")?.schemas ?? [];

  const handleConnectionChange = (nextId: string) => {
    if (!nextId) return;
    const next = connections?.find((c) => c.id === nextId);
    if (!next || next.id === connectionId) return;
    setQueryTabConnection(tabId, next.id, { database: next.database, schema: null });
  };

  return (
    <div className="flex items-center gap-2 border-b border-border bg-card px-3 py-1">
      <Database className="h-3.5 w-3.5 shrink-0 text-muted-foreground" aria-hidden />
      <span className="text-[11px] text-muted-foreground">{t("query.context")}</span>
      <Select value={connectionId ?? ""} onValueChange={handleConnectionChange}>
        <SelectTrigger className="h-6 w-auto max-w-[180px] rounded-sm border px-2 py-0 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {connections?.map((conn) => (
            <SelectItem key={conn.id} value={conn.id}>
              {conn.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {database && (
        <span className="truncate text-[11px] text-muted-foreground">
          {database}
          {context.schema ? `.${context.schema}` : ""}
        </span>
      )}
      {schemas.length > 0 && (
        <Select
          value={context.schema ?? DEFAULT_SCHEMA}
          onValueChange={(value) =>
            setQueryTabSchema(tabId, value === DEFAULT_SCHEMA ? null : value)
          }
        >
          <SelectTrigger className="h-6 w-auto max-w-[180px] rounded-sm border px-2 py-0 text-xs">
            <SelectValue placeholder={t("query.contextDefaultSchema")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={DEFAULT_SCHEMA}>{t("query.contextDefaultSchema")}</SelectItem>
            {schemas.map((schema) => (
              <SelectItem key={schema.name} value={schema.name}>
                {schema.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
      <div className="flex-1" />
    </div>
  );
}
