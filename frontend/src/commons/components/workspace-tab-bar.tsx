import { X } from "lucide-react";

import type { WorkspaceTab } from "@/commons/types/workspace.types";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface WorkspaceTabBarProps {
  className?: string;
}

function TabItem({
  tab,
  isActive,
  onActivate,
  onClose,
}: {
  tab: WorkspaceTab;
  isActive: boolean;
  onActivate: () => void;
  onClose: () => void;
}) {
  return (
    <div
      className={cn(
        "group flex shrink-0 cursor-pointer items-center gap-1.5 border-r border-border px-3 py-1.5 text-xs transition-colors",
        isActive
          ? "bg-background text-foreground"
          : "text-muted-foreground hover:bg-muted/50",
      )}
      onClick={onActivate}
      role="tab"
      aria-selected={isActive}
      tabIndex={isActive ? 0 : -1}
    >
      {tab.dirty && (
        <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-primary" aria-label="Unsaved changes" />
      )}
      {tab.pinned && (
        <span className="text-[10px]" aria-label="Pinned">📌</span>
      )}
      <span className="max-w-[140px] truncate">{tab.title}</span>
      {!tab.pinned && (
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-4 w-4 shrink-0 rounded px-0 text-muted-foreground opacity-0 transition-opacity hover:bg-border hover:text-foreground group-hover:opacity-100"
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          aria-label={`Close ${tab.title}`}
        >
          <X className="h-3 w-3" />
        </Button>
      )}
    </div>
  );
}

export function WorkspaceTabBar({ className }: WorkspaceTabBarProps) {
  const tabs = useWorkspaceStore((s) => s.tabs);
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const activateTab = useWorkspaceStore((s) => s.activateTab);
  const closeTab = useWorkspaceStore((s) => s.closeTab);

  if (tabs.length === 0) {
    return null;
  }

  return (
    <div
      className={cn("flex items-center gap-0 overflow-x-auto border-b border-border bg-card", className)}
      role="tablist"
      aria-label="Workspace tabs"
    >
      {tabs.map((tab) => (
        <TabItem
          key={tab.id}
          tab={tab}
          isActive={tab.id === activeTabId}
          onActivate={() => activateTab(tab.id)}
          onClose={() => closeTab(tab.id)}
        />
      ))}
    </div>
  );
}
