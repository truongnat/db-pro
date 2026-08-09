import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";
import type { DbObjectSection } from "@/commons/types/workspace.types";

export const OBJECT_SECTIONS: { id: DbObjectSection; labelKey: string }[] = [
  { id: "data", labelKey: "dbObject.sections.data" },
  { id: "columns", labelKey: "dbObject.sections.columns" },
  { id: "indexes", labelKey: "dbObject.sections.indexes" },
  { id: "relations", labelKey: "dbObject.sections.relations" },
  { id: "triggers", labelKey: "dbObject.sections.triggers" },
  { id: "diagram", labelKey: "dbObject.sections.diagram" },
  { id: "ddl", labelKey: "dbObject.sections.ddl" },
];

interface ObjectSectionTabsProps {
  activeSection: DbObjectSection;
  onSelect: (section: DbObjectSection) => void;
}

export function ObjectSectionTabs({ activeSection, onSelect }: ObjectSectionTabsProps) {
  const { t } = useTranslation();
  return (
    <div className="flex h-[34px] items-center border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-1)]">
      <div className="flex flex-1 overflow-x-auto">
        {OBJECT_SECTIONS.map((section) => (
          <button
            key={section.id}
            type="button"
            className={cn(
              "relative h-full shrink-0 px-3 text-[13px] font-medium transition-colors hover:text-foreground",
              activeSection === section.id ? "text-foreground" : "text-[var(--app-text-muted)]",
            )}
            onClick={() => onSelect(section.id)}
          >
            {t(section.labelKey)}
            {activeSection === section.id && (
              <span className="absolute inset-x-3 bottom-0 h-[2px] bg-primary" />
            )}
          </button>
        ))}
      </div>
    </div>
  );
}
