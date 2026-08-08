import { useState } from "react";
import { MessageSquare, Plus, Send, Sparkles, X } from "lucide-react";

import { cn } from "@/lib/utils";

interface AgentPanelProps {
  open: boolean;
  onClose: () => void;
  width?: number;
  className?: string;
}

export function AgentPanel({ open, onClose, width, className }: AgentPanelProps) {
  const [input, setInput] = useState("");

  if (!open) return null;

  return (
    <aside
      className={cn(
        "flex min-h-0 flex-col border-l border-[var(--app-border-subtle)] bg-sidebar",
        className,
      )}
      style={width ? { width: `${width}px`, minWidth: `${width}px`, maxWidth: `${width}px` } : undefined}
    >
      {/* Header */}
      <div className="flex shrink-0 items-center justify-between border-b border-[var(--app-border-subtle)] px-3 py-2">
        <div className="flex items-center gap-2">
          <Sparkles className="h-3.5 w-3.5 text-primary" />
          <span className="text-[11px] font-semibold uppercase tracking-[0.04em] text-[var(--app-text-dim)]">
            AI Assistant
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            className="flex h-6 w-6 items-center justify-center rounded text-[var(--app-text-muted)] transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
            title="New Chat"
          >
            <Plus className="h-3.5 w-3.5" />
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
        <div className="flex flex-1 flex-col items-center justify-center px-4 py-8">
          <div className="mb-4 grid h-10 w-10 place-items-center rounded-xl bg-primary/10">
            <MessageSquare className="h-5 w-5 text-primary" />
          </div>
          <p className="mb-1 text-sm font-medium text-foreground">How can I help with your database?</p>
          <p className="mb-6 text-center text-xs text-[var(--app-text-dim)]">
            Ask about schemas, queries, optimization, and more.
          </p>

          {/* Quick actions */}
          <div className="grid w-full grid-cols-2 gap-2">
            {[
              { label: "Write SQL", icon: "SELECT" },
              { label: "Explain schema", icon: "▦" },
              { label: "Analyze query", icon: "▶" },
              { label: "Find slow query", icon: "⏱" },
            ].map((action) => (
              <button
                key={action.label}
                type="button"
                className="flex items-center gap-2 rounded-lg border border-[var(--app-border)] px-3 py-2 text-xs text-[var(--app-text-muted)] transition-colors hover:border-[var(--app-border-strong)] hover:bg-[var(--app-hover)] hover:text-foreground"
              >
                <span className="text-[var(--app-text-dim)]">{action.icon}</span>
                {action.label}
              </button>
            ))}
          </div>
        </div>

        {/* Input */}
        <div className="shrink-0 border-t border-[var(--app-border-subtle)] px-3 py-2">
          <div className="flex items-center gap-2 rounded-lg border border-[var(--app-border)] bg-background px-3 py-2 transition-colors focus-within:border-[var(--app-border-strong)]">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Ask anything about this database..."
              className="flex-1 bg-transparent text-xs text-foreground placeholder:text-[var(--app-text-dim)] focus:outline-none"
            />
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
    </aside>
  );
}
