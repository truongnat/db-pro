import type { ReactNode } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";
import type { DbObjectSection } from "@/commons/types/workspace.types";

export const OBJECT_SECTIONS: { id: DbObjectSection; labelKey: string }[] = [
  { id: "data", labelKey: "dbObject.sections.data" },
  { id: "columns", labelKey: "dbObject.sections.columns" },
  { id: "indexes", labelKey: "dbObject.sections.indexes" },
  { id: "relations", labelKey: "dbObject.sections.relations" },
  { id: "triggers", labelKey: "dbObject.sections.triggers" },
  { id: "ddl", labelKey: "dbObject.sections.ddl" },
];

interface ObjectSectionTabsProps {
  activeSection: DbObjectSection;
  onSelect: (section: DbObjectSection) => void;
  trailing?: ReactNode;
}

export function ObjectSectionTabs({ activeSection, onSelect, trailing }: ObjectSectionTabsProps) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center border-b border-border">
      <div className="flex flex-1 overflow-x-auto">
        {OBJECT_SECTIONS.map((section) => (
          <button
            key={section.id}
            type="button"
            className={cn(
              "shrink-0 px-3 py-2 text-xs font-medium transition-colors hover:bg-[var(--app-hover)]",
              activeSection === section.id
                ? "border-b-2 border-primary text-foreground"
                : "text-muted-foreground",
            )}
            onClick={() => onSelect(section.id)}
          >
            {t(section.labelKey)}
          </button>
        ))}
      </div>
      {trailing}
    </div>
  );
}
