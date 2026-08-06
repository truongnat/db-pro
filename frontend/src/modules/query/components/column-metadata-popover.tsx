import { useEffect, useRef, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import type { ColumnMeta } from "../types/query.types";

interface ColumnMetadataPopoverProps {
  column: ColumnMeta;
  anchorEl: HTMLElement;
  onClose: () => void;
}

export function ColumnMetadataPopover({ column, anchorEl, onClose }: ColumnMetadataPopoverProps) {
  const { t } = useTranslation();
  const popoverRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ top: 0, left: 0 });

  useEffect(() => {
    const rect = anchorEl.getBoundingClientRect();
    setPos({ top: rect.bottom + 4, left: rect.left });
  }, [anchorEl]);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [onClose]);

  return (
    <div
      ref={popoverRef}
      className="fixed z-50 min-w-[200px] rounded-[var(--radius-sm)] border p-3 shadow-lg"
      style={{
        top: pos.top,
        left: pos.left,
        backgroundColor: "var(--color-bg)",
        borderColor: "var(--color-border)",
      }}
    >
      <div className="mb-2">
        <p className="text-xs font-medium" style={{ color: "var(--color-text-secondary)" }}>
          {t("query.metadata.name")}
        </p>
        <p className="text-sm font-mono" style={{ color: "var(--color-text)" }}>
          {column.name}
        </p>
      </div>
      <div className="mb-2">
        <p className="text-xs font-medium" style={{ color: "var(--color-text-secondary)" }}>
          {t("query.metadata.dataType")}
        </p>
        <p className="text-sm" style={{ color: "var(--color-text)" }}>
          {column.dataType}
        </p>
      </div>
      <div>
        <p className="text-xs font-medium" style={{ color: "var(--color-text-secondary)" }}>
          {t("query.metadata.nullable")}
        </p>
        <p className="text-sm" style={{ color: column.nullable ? "var(--color-text)" : "var(--color-text-secondary)" }}>
          {column.nullable ? t("common.yes") : t("common.no")}
        </p>
      </div>
    </div>
  );
}
