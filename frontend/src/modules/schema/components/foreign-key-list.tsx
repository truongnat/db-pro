import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaForeignKeyDto } from "../types/schema.types";

interface ForeignKeyListProps {
  foreignKeys: SchemaForeignKeyDto[];
}

export function ForeignKeyList({ foreignKeys }: ForeignKeyListProps) {
  const { t } = useTranslation();

  if (foreignKeys.length === 0) {
    return (
      <div
        className="p-4 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("schema.noForeignKeys")}
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
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.fkName")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.fkFromColumn")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.fkToTable")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.fkToColumn")}</th>
        </tr>
      </thead>
      <tbody>
        {foreignKeys.map((fk) => (
          <tr
            key={fk.name}
            className="transition-colors hover:bg-[var(--color-bg)]"
          >
            <td className="px-3 py-1.5 font-mono text-sm">{fk.name}</td>
            <td className="px-3 py-1.5 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {fk.fromColumn}
            </td>
            <td className="px-3 py-1.5 font-mono text-xs">
              {fk.toSchema !== fk.schema ? `${fk.toSchema}.` : ""}{fk.toTable}
            </td>
            <td className="px-3 py-1.5 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {fk.toColumn}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
