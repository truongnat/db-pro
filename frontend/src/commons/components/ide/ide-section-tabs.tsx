import { cn } from "@/lib/utils";

interface Tab {
  id: string;
  label: string;
}

interface IdeSectionTabsProps {
  tabs: Tab[];
  activeTab: string;
  onSelect: (id: string) => void;
  className?: string;
}

export function IdeSectionTabs({ tabs, activeTab, onSelect, className }: IdeSectionTabsProps) {
  return (
    <div className={cn("flex items-center border-b border-[var(--app-border-subtle)]", className)}>
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          className={cn(
            "relative px-3 py-2 text-xs font-medium transition-colors",
            activeTab === tab.id
              ? "text-foreground"
              : "text-muted-foreground hover:text-foreground",
          )}
          onClick={() => onSelect(tab.id)}
        >
          {tab.label}
          {activeTab === tab.id && (
            <span className="absolute inset-x-2 bottom-0 h-[2px] rounded-t-sm bg-primary" />
          )}
        </button>
      ))}
    </div>
  );
}
