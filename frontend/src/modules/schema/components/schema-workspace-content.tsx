import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { ErDiagram } from "@/modules/er-diagram/components/er-diagram";
import { cn } from "@/lib/utils";
import type { SchemaWorkspaceSection } from "@/commons/types/workspace.types";

const SCHEMA_SECTIONS: { id: SchemaWorkspaceSection; labelKey: string }[] = [
  { id: "diagram", labelKey: "schemaWorkspace.sections.diagram" },
  { id: "overview", labelKey: "schemaWorkspace.sections.overview" },
];

interface SchemaWorkspaceContentProps {
  tabId: string;
  connectionId: string;
  schema: string;
}

function SchemaSectionTabs({
  activeSection,
  onSelect,
}: {
  activeSection: SchemaWorkspaceSection;
  onSelect: (section: SchemaWorkspaceSection) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex h-[34px] items-center border-b border-[var(--border-subtle)] bg-[var(--surface-nav)]">
      <div className="flex flex-1 overflow-x-auto">
        {SCHEMA_SECTIONS.map((section) => (
          <button
            key={section.id}
            type="button"
            className={cn(
              "relative h-full shrink-0 px-3 text-[13px] font-medium transition-colors hover:text-foreground",
              activeSection === section.id ? "text-foreground" : "text-[var(--text-secondary)]",
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

export function SchemaWorkspaceContent({
  tabId,
  connectionId,
  schema,
}: SchemaWorkspaceContentProps) {
  const { t } = useTranslation();
  const activeSection = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === tabId);
    return tab?.kind === "schema-workspace" ? tab.data.activeSection : "diagram";
  });
  const setSection = useWorkspaceStore((s) => s.setSchemaWorkspaceSection);

  const introspect = useIntrospect(connectionId);

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <SchemaSectionTabs
        activeSection={activeSection}
        onSelect={(section) => setSection(tabId, section)}
      />
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        {activeSection === "diagram" && (
          <>
            {introspect.isLoading && (
              <div className="flex h-full items-center justify-center p-4">
                <span className="text-[13px] text-[var(--text-secondary)]">
                  {t("common.states.loading")}
                </span>
              </div>
            )}
            {introspect.isError && (
              <div className="flex h-full items-center justify-center p-4">
                <span className="text-[13px] text-destructive">{t("common.states.error")}</span>
              </div>
            )}
            {introspect.data && (
              <ErDiagram connectionId={connectionId} schema={schema} data={introspect.data} />
            )}
          </>
        )}
        {activeSection === "overview" && (
          <div className="flex h-full items-center justify-center p-8">
            <p className="text-[13px] text-[var(--text-secondary)]">
              {t("schemaWorkspace.overviewComingSoon")}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
