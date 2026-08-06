import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaIndexDto } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface IndexListProps {
  indexes: SchemaIndexDto[];
}

export function IndexList({ indexes }: IndexListProps) {
  const { t } = useTranslation();

  if (indexes.length === 0) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        {t("schema.noIndexes")}
      </div>
    );
  }

  const headerClass = "px-3 py-2 font-medium text-muted-foreground border-b border-border";

  return (
    <table className="w-full text-sm">
      <thead>
        <tr>
          <th className={cn(headerClass, "text-left")}>{t("schema.indexName")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.indexColumns")}</th>
          <th className={cn(headerClass, "text-center")}>{t("schema.indexUnique")}</th>
        </tr>
      </thead>
      <tbody>
        {indexes.map((idx) => (
          <tr
            key={idx.name}
            className="transition-colors hover:bg-background"
          >
            <td className="px-3 py-1.5 font-mono text-sm">{idx.name}</td>
            <td className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {idx.columns.join(", ")}
            </td>
            <td className="px-3 py-1.5 text-center text-muted-foreground">
              {idx.unique ? "YES" : "NO"}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
