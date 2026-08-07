import { Button } from "@/components/ui/button";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData } from "@/commons/types/workspace.types";
import {
  setTabMultiResultIndex,
  setTabResult,
} from "../controllers/query-workspace.controller";

export function ResultTabs() {
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const tabData = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === s.activeTabId);
    return tab?.kind === "query" ? (tab.data as QueryTabData) : undefined;
  });

  const multiResults = tabData?.multiResults ?? null;
  const multiResultIndex = tabData?.multiResultIndex ?? 0;

  if (!multiResults || multiResults.length <= 1 || !activeTabId) return null;

  return (
    <div className="flex gap-1 border-b border-border bg-card px-3 py-1">
      {multiResults.map((r, idx) => (
        <Button
          key={idx}
          type="button"
          variant="ghost"
          className={`rounded-t px-3 py-1 text-xs transition-colors ${
            idx === multiResultIndex
              ? "bg-background text-foreground"
              : "text-muted-foreground"
          } ${
            idx === multiResultIndex
              ? "border-b-2 border-primary"
              : "border-b-2 border-transparent"
          }`}
          onClick={() => {
            setTabMultiResultIndex(activeTabId, idx);
            setTabResult(activeTabId, multiResults[idx]);
          }}
        >
          Result {idx + 1} ({r.rowCount} rows)
        </Button>
      ))}
    </div>
  );
}
