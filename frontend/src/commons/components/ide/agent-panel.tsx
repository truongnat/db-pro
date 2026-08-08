import { useState } from "react";
import {
  MessageSquare,
  Plus,
  Send,
  Sparkles,
  Table2,
  Code2,
  Network,
  X,
  Database,
  Layers,
  TableIcon,
  ChevronRight,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";

interface AgentPanelProps {
  open: boolean;
  onClose: () => void;
  width?: number;
  className?: string;
}

interface ContextChip {
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}

export function AgentPanel({ open, onClose, width, className }: AgentPanelProps) {
  const [input, setInput] = useState("");

  // Gather current context for context chips
  const activeTab = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === s.activeTabId);
    return tab;
  });
  const { data: connections } = useConnectionList();

  const connectionName = activeTab?.connectionId
    ? connections?.find((c) => c.id === activeTab.connectionId)?.name
    : null;

  const contextChips: ContextChip[] = [];
  if (connectionName) contextChips.push({ label: connectionName, icon: Database });
  if (activeTab?.kind === "query") {
    const ctx = activeTab.data.context;
    if (ctx.database) contextChips.push({ label: ctx.database, icon: Database });
    if (ctx.schema) contextChips.push({ label: ctx.schema, icon: Layers });
  } else if (activeTab?.kind === "db-object") {
    contextChips.push({ label: activeTab.data.schema, icon: Layers });
    contextChips.push({ label: activeTab.data.objectName, icon: TableIcon });
  }

  if (!open) return null;

  return (
    <aside
      className={cn(
        "flex min-h-0 flex-col border-l border-[var(--app-border-subtle)] bg-[var(--app-surface-2)]",
        className,
      )}
      style={width ? { width: `${width}px`, minWidth: `${width}px`, maxWidth: `${width}px` } : undefined}
    >
      {/* Header */}
      <div className="flex h-[38px] shrink-0 items-center justify-between border-b border-[var(--app-border-subtle)] px-3">
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-primary" />
          <span className="text-[13px] font-semibold text-foreground">Agent</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            className="flex h-7 items-center gap-1 rounded-md px-2 text-[12px] text-[var(--app-text-muted)] opacity-50 cursor-not-allowed"
            title="Coming soon"
            disabled
          >
            <Plus className="h-3.5 w-3.5" />
            New
          </button>
          <button
            type="button"
            className="flex h-7 w-7 items-center justify-center rounded text-[var(--app-text-muted)] transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
            onClick={onClose}
            title="Close Panel"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      {/* Content — top-aligned, no vertical centering */}
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        <div className="flex flex-col px-4 pb-4 pt-5">
          {/* Context chips — stronger readability */}
          {contextChips.length > 0 && (
            <div className="mb-5 flex w-full flex-wrap gap-1.5">
              {contextChips.map((chip) => {
                const Icon = chip.icon;
                return (
                  <span
                    key={chip.label}
                    className="flex items-center gap-1.5 rounded-md border border-primary/15 bg-primary/6 px-2.5 py-1 text-[12px] font-medium text-primary"
                  >
                    <Icon className="h-3 w-3" />
                    {chip.label}
                  </span>
                );
              })}
            </div>
          )}

          {/* Welcome heading */}
          <div className="mb-4 flex items-center gap-2.5">
            <div className="grid h-8 w-8 place-items-center rounded-lg bg-primary/10">
              <MessageSquare className="h-4 w-4 text-primary" />
            </div>
            <div>
              <p className="text-[13px] font-semibold leading-tight text-foreground">How can I help with this database?</p>
              <p className="text-[12px] text-[var(--app-text-muted)]">Ask about schemas, queries, optimization</p>
            </div>
          </div>

          {/* Starter suggestions — 1-column, 36px height */}
          <div className="flex w-full flex-col gap-1">
            {[
              { label: "Explain this table", icon: Table2 },
              { label: "Write a SELECT query", icon: Code2 },
              { label: "Find relations", icon: Network },
              { label: "Optimize current query", icon: Sparkles },
            ].map((action) => {
              const Icon = action.icon;
              return (
                <button
                  key={action.label}
                  type="button"
                  className="flex h-[36px] w-full items-center gap-2.5 rounded-md border border-[var(--app-border-subtle)] px-3 text-[13px] text-[var(--app-text-muted)] transition-colors hover:border-[var(--app-border)] hover:bg-[var(--app-hover)] hover:text-foreground"
                  onClick={() => setInput(action.label === "Explain this table" ? "Explain the current table structure" : action.label === "Write a SELECT query" ? "SELECT * FROM " : action.label === "Find relations" ? "Show foreign key relationships for " : "")}
                >
                  <Icon className="h-4 w-4 shrink-0 text-[var(--app-text-dim)]" />
                  <span className="flex-1 text-left">{action.label}</span>
                  <ChevronRight className="h-3 w-3 text-[var(--app-text-dim)]" />
                </button>
              );
            })}
          </div>
        </div>
      </div>

      {/* Composer — pinned to bottom */}
      <div className="shrink-0 border-t border-[var(--app-border-subtle)] px-3 py-2.5">
        <div className="flex items-center gap-2 rounded-lg border border-[var(--app-border)] bg-background px-3 py-2 transition-colors focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/20">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask about this database..."
            className="flex-1 bg-transparent text-[13px] text-foreground placeholder:text-[var(--app-text-dim)] focus:outline-none"
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && input.trim()) {
                setInput("");
              }
            }}
          />
          <div className="flex items-center gap-1.5">
            <kbd className="text-[11px] text-[var(--app-text-dim)]">⌘↵</kbd>
            <button
              type="button"
              className={cn(
                "flex h-6 w-6 items-center justify-center rounded transition-colors",
                input.trim()
                  ? "bg-primary text-primary-foreground"
                  : "text-[var(--app-text-dim)]",
              )}
              disabled={!input.trim()}
              onClick={() => { if (input.trim()) setInput(""); }}
            >
              <Send className="h-3 w-3" />
            </button>
          </div>
        </div>
      </div>
    </aside>
  );
}
