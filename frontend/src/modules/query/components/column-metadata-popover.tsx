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
      className="fixed z-[var(--z-popover)] min-w-[200px] rounded-sm border border-border bg-background p-3 shadow-lg"
      style={{
        top: pos.top,
        left: pos.left,
      }}
    >
      <div className="mb-2">
        <p className="text-xs font-medium text-muted-foreground">
          {t("query.metadata.name")}
        </p>
        <p className="text-sm font-mono text-foreground">
          {column.name}
        </p>
      </div>
      <div className="mb-2">
        <p className="text-xs font-medium text-muted-foreground">
          {t("query.metadata.dataType")}
        </p>
        <p className="text-sm text-foreground">
          {column.dataType}
        </p>
      </div>
      <div>
        <p className="text-xs font-medium text-muted-foreground">
          {t("query.metadata.nullable")}
        </p>
        <p className={`text-sm ${column.nullable ? "text-foreground" : "text-muted-foreground"}`}>
          {column.nullable ? t("common.yes") : t("common.no")}
        </p>
      </div>
    </div>
  );
}
