import { useState } from "react";
import { MessageSquare, Plus, Send, Sparkles, Table2, Code2, Network, X } from "lucide-react";

import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";

interface AgentPanelProps {
  open: boolean;
  onClose: () => void;
  width?: number;
  className?: string;
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

  const contextItems: string[] = [];
  if (connectionName) contextItems.push(connectionName);
  if (activeTab?.kind === "query") {
    const ctx = activeTab.data.context;
    if (ctx.database) contextItems.push(ctx.database);
    if (ctx.schema) contextItems.push(ctx.schema);
  } else if (activeTab?.kind === "db-object") {
    contextItems.push(activeTab.data.schema);
    contextItems.push(activeTab.data.objectName);
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
      <div className="flex h-[36px] shrink-0 items-center justify-between border-b border-[var(--app-border-subtle)] px-3">
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-primary" />
          <span className="text-[13px] font-semibold text-foreground">Agent</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            className="flex h-6 items-center gap-1 rounded-md px-1.5 text-[11px] text-[var(--app-text-muted)] transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
            title="New Chat"
          >
            <Plus className="h-3 w-3" />
            New
          </button>
          <button
            type="button"
            className="flex h-6 w-6 items-center justify-center rounded text-[var(--app-text-muted)] transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
            onClick={onClose}
            title="Close Panel"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex min-h-0 flex-1 flex-col">
        {/* Welcome state */}
        <div className="flex flex-1 flex-col items-center justify-center px-4 py-6">
          <div className="mb-3 grid h-10 w-10 place-items-center rounded-xl bg-primary/10">
            <MessageSquare className="h-5 w-5 text-primary" />
          </div>
          <p className="mb-1 text-[13px] font-medium text-foreground">How can I help with your database?</p>
          <p className="mb-4 text-center text-[12px] text-[var(--app-text-muted)]">
            Ask about schemas, queries, optimization, and more.
          </p>

          {/* Context chips */}
          {contextItems.length > 0 && (
            <div className="mb-4 flex w-full flex-wrap justify-center gap-1.5">
              {contextItems.map((item) => (
                <span
                  key={item}
                  className="rounded-md bg-primary/8 px-2 py-0.5 text-[11px] font-medium text-primary"
                >
                  {item}
                </span>
              ))}
            </div>
          )}

          {/* Starter suggestions */}
          <div className="grid w-full grid-cols-2 gap-1.5">
            {[
              { label: "Explain this table", icon: Table2 },
              { label: "Write SELECT", icon: Code2 },
              { label: "Find relations", icon: Network },
              { label: "Optimize query", icon: Sparkles },
            ].map((action) => {
              const Icon = action.icon;
              return (
                <button
                  key={action.label}
                  type="button"
                  className="flex items-center gap-2 rounded-md border border-[var(--app-border-subtle)] px-2.5 py-2 text-[12px] text-[var(--app-text-muted)] transition-colors hover:border-[var(--app-border)] hover:bg-[var(--app-hover)] hover:text-foreground"
                >
                  <Icon className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-dim)]" />
                  <span className="text-[12px]">{action.label}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Composer */}
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
                  // submit
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
              >
                <Send className="h-3 w-3" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </aside>
  );
}
