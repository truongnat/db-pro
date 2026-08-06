import { useQueryModuleStore } from "../state/query.store";

export function QueryTabs() {
  const tabs = useQueryModuleStore((s) => s.tabs);
  const activeTabId = useQueryModuleStore((s) => s.activeTabId);
  const setActiveTabId = useQueryModuleStore((s) => s.setActiveTabId);
  const addTab = useQueryModuleStore((s) => s.addTab);
  const closeTab = useQueryModuleStore((s) => s.closeTab);

  return (
    <div
      className="flex items-center gap-0 border-b"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: "var(--color-surface)",
      }}
    >
      <div className="flex min-w-0 flex-1 overflow-x-auto">
        {tabs.map((tab) => {
          const isActive = tab.id === activeTabId;
          return (
            <div
              key={tab.id}
              className="group flex shrink-0 cursor-pointer items-center gap-1 border-r px-3 py-1.5 text-xs transition-colors"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: isActive ? "var(--color-bg)" : "transparent",
                color: isActive ? "var(--color-text)" : "var(--color-text-secondary)",
              }}
              onClick={() => setActiveTabId(tab.id)}
            >
              <span className="max-w-[120px] truncate">{tab.title}</span>
              {tabs.length > 1 && (
                <button
                  className="ml-1 rounded px-0.5 opacity-0 transition-opacity hover:bg-[var(--color-border)] group-hover:opacity-100"
                  style={{
                    color: "var(--color-text-secondary)",
                    opacity: isActive ? 0.6 : undefined,
                  }}
                  onClick={(e) => {
                    e.stopPropagation();
                    closeTab(tab.id);
                  }}
                  onMouseEnter={(e) => {
                    (e.target as HTMLElement).style.opacity = "1";
                  }}
                  onMouseLeave={(e) => {
                    (e.target as HTMLElement).style.opacity = isActive ? "0.6" : "0";
                  }}
                >
                  ×
                </button>
              )}
            </div>
          );
        })}
      </div>
      <button
        className="shrink-0 px-2 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
        style={{ color: "var(--color-text-secondary)" }}
        onClick={addTab}
        title="New tab"
      >
        +
      </button>
    </div>
  );
}
