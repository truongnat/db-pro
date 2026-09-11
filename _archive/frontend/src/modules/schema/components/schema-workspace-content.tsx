import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { ErDiagram } from "@/modules/er-diagram/components/er-diagram";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
  tabId,
  activeSection,
  onSelect,
}: {
  tabId: string;
  activeSection: SchemaWorkspaceSection;
  onSelect: (section: SchemaWorkspaceSection) => void;
}) {
  const { t } = useTranslation();
  return (
    <Tabs value={activeSection} onValueChange={(v) => onSelect(v as SchemaWorkspaceSection)}>
      <TabsList
        variant="line"
        className="h-[34px] w-full justify-start overflow-x-auto rounded-none border-b border-[var(--border-subtle)] bg-[var(--surface-nav)] px-0"
      >
        {SCHEMA_SECTIONS.map((section) => (
          <TabsTrigger
            key={section.id}
            value={section.id}
            className="h-full shrink-0 flex-none rounded-none px-3 text-[13px] font-medium"
            aria-controls={`schema-panel-${tabId}`}
          >
            {t(section.labelKey)}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
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
        tabId={tabId}
        activeSection={activeSection}
        onSelect={(section) => setSection(tabId, section)}
      />
      <div
        id={`schema-panel-${tabId}`}
        role="tabpanel"
        tabIndex={0}
        aria-label={t(`schemaWorkspace.sections.${activeSection}`)}
        className="flex min-h-0 flex-1 flex-col overflow-hidden outline-none"
      >
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
