import { useTranslation } from "@/commons/locales/useTranslation";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
}

export function ObjectSectionTabs({ activeSection, onSelect }: ObjectSectionTabsProps) {
  const { t } = useTranslation();
  return (
    <Tabs value={activeSection} onValueChange={(v) => onSelect(v as DbObjectSection)}>
      <TabsList
        variant="line"
        className="h-[34px] w-full justify-start rounded-none border-b border-[var(--border-subtle)] bg-[var(--surface-nav)] px-0"
      >
        {OBJECT_SECTIONS.map((section) => (
          <TabsTrigger
            key={section.id}
            value={section.id}
            className="h-full rounded-none px-3 text-[13px] font-medium"
          >
            {t(section.labelKey)}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
