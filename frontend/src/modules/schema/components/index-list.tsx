import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaIndexDto } from "../types/schema.types";

interface IndexListProps {
  indexes: SchemaIndexDto[];
}

export function IndexList({ indexes }: IndexListProps) {
  const { t } = useTranslation();

  if (indexes.length === 0) {
    return (
      <div
        className="p-4 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("schema.noIndexes")}
      </div>
    );
  }

  const headerStyle: React.CSSProperties = {
    color: "var(--color-text-secondary)",
    borderBottom: "1px solid var(--color-border)",
  };

  return (
    <table className="w-full text-sm">
      <thead>
        <tr>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.indexName")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.indexColumns")}</th>
          <th className="px-3 py-2 text-center font-medium" style={headerStyle}>{t("schema.indexUnique")}</th>
        </tr>
      </thead>
      <tbody>
        {indexes.map((idx) => (
          <tr
            key={idx.name}
            className="transition-colors hover:bg-[var(--color-bg)]"
          >
            <td className="px-3 py-1.5 font-mono text-sm">{idx.name}</td>
            <td className="px-3 py-1.5 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {idx.columns.join(", ")}
            </td>
            <td className="px-3 py-1.5 text-center" style={{ color: "var(--color-text-secondary)" }}>
              {idx.unique ? "YES" : "NO"}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
