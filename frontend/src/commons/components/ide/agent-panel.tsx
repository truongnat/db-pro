import { useCallback, useEffect, useRef, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import {
  MessageSquare,
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
  Settings,
  Trash2,
  Copy,
  Check,
  Loader2,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";
import { setTabSql } from "@/modules/query/controllers/query-workspace.controller";
import { useAgentChatStore } from "@/modules/agent/stores/agent-chat.store";
import {
  generateTemplateResponse,
  generateLlmResponse,
} from "@/modules/agent/services/template-agent";
import type { AgentMessage, SchemaContext } from "@/modules/agent/types/agent.types";
import type { SchemaForeignKeyDto } from "@/modules/schema/types/schema.types";

interface AgentPanelProps {
  open: boolean;
  onClose: () => void;
  width?: number;
  className?: string;
}

function buildSchemaContext(
  connectionId: string | null | undefined,
  connectionName: string | null,
  activeSchema: string | null,
  activeTable: string | null,
): SchemaContext {
  const catalog = connectionId
    ? useSchemaCatalogStore.getState().getCatalog(connectionId)
    : undefined;

  const tables =
    catalog?.objects
      .filter((o) => o.kind === "table")
      .map((o) => ({
        name: o.name,
        schema: o.schema,
        rowCount: o.rowCount,
      })) ?? [];

  const columns = new Map<
    string,
    { name: string; dataType: string; nullable: boolean; isPrimaryKey: boolean }[]
  >();
  if (catalog) {
    for (const [key, cols] of catalog.columnsByTable) {
      columns.set(
        key,
        cols.map((c) => ({
          name: c.name,
          dataType: c.dataType,
          nullable: c.nullable,
          isPrimaryKey: c.isPrimaryKey,
        })),
      );
    }
  }

  const foreignKeys: SchemaForeignKeyDto[] = [];

  return {
    tables,
    columns,
    foreignKeys,
    connectionName,
    activeSchema,
    activeTable,
  };
}

export function AgentPanel({ open, onClose, width, className }: AgentPanelProps) {
  const [input, setInput] = useState("");
  const [showSettings, setShowSettings] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const { messages, config, isProcessing, addMessage, clearMessages, setProcessing, setConfig } =
    useAgentChatStore(
      useShallow((s) => ({
        messages: s.messages,
        config: s.config,
        isProcessing: s.isProcessing,
        addMessage: s.addMessage,
        clearMessages: s.clearMessages,
        setProcessing: s.setProcessing,
        setConfig: s.setConfig,
      })),
    );

  const activeTabMeta = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === s.activeTabId);
    if (!tab) return null;
    return {
      connectionId: tab.connectionId,
      kind: tab.kind,
      database: tab.kind === "query" ? tab.data.context.database : null,
      schema:
        tab.kind === "query"
          ? tab.data.context.schema
          : tab.kind === "db-object" || tab.kind === "schema-workspace"
            ? tab.data.schema
            : null,
      objectName: tab.kind === "db-object" ? tab.data.objectName : null,
    };
  });
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const { data: connections } = useConnectionList();

  const connectionName = activeTabMeta?.connectionId
    ? connections?.find((c) => c.id === activeTabMeta.connectionId)?.name
    : null;

  const activeSchema = activeTabMeta?.schema ?? null;
  const activeTable = activeTabMeta?.objectName ?? null;

  useEffect(() => {
    if (activeTabMeta?.connectionId) {
      useSchemaCatalogStore.getState().ensureLoaded(activeTabMeta.connectionId);
    }
  }, [activeTabMeta?.connectionId]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = useCallback(async () => {
    const trimmed = input.trim();
    if (!trimmed || isProcessing) return;

    const userMsg: AgentMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content: trimmed,
      timestamp: Date.now(),
    };
    addMessage(userMsg);
    setInput("");
    setProcessing(true);

    try {
      const ctx = buildSchemaContext(
        activeTabMeta?.connectionId,
        connectionName ?? null,
        activeSchema,
        activeTable,
      );

      let response: AgentMessage;

      if (config.apiKey && config.apiEndpoint) {
        const history = messages.map((m) => ({ role: m.role, content: m.content }));
        response = await generateLlmResponse(trimmed, ctx, config, history);
      } else {
        response = generateTemplateResponse(trimmed, ctx);
      }

      addMessage(response);
    } catch (err) {
      addMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Error: ${err instanceof Error ? err.message : "Unknown error"}. Falling back to template mode.`,
        timestamp: Date.now(),
      });
    } finally {
      setProcessing(false);
    }
  }, [
    input,
    isProcessing,
    addMessage,
    setProcessing,
    activeTabMeta,
    connectionName,
    activeSchema,
    activeTable,
    config,
    messages,
  ]);

  const handleInsertSql = useCallback(
    (sql: string) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, sql);
    },
    [activeTabId],
  );

  if (!open) return null;

  const hasMessages = messages.length > 0;

  return (
    <aside
      className={cn(
        "flex min-h-0 flex-col border-l border-[var(--border-subtle)] bg-[var(--surface-panel)]",
        className,
      )}
      style={
        width ? { width: `${width}px`, minWidth: `${width}px`, maxWidth: `${width}px` } : undefined
      }
    >
      {/* Header */}
      <div className="flex h-[38px] shrink-0 items-center justify-between border-b border-[var(--border-subtle)] px-3">
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-primary" />
          <span className="text-[13px] font-semibold text-foreground">Agent</span>
          {!config.apiKey && (
            <span className="rounded bg-[var(--surface-hover)] px-1.5 py-0.5 text-[10px] text-[var(--text-tertiary)]">
              template
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {hasMessages && (
            <button
              type="button"
              className="flex h-7 w-7 items-center justify-center rounded text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
              onClick={clearMessages}
              title="Clear conversation"
            >
              <Trash2 className="h-3.5 w-3.5" />
            </button>
          )}
          <button
            type="button"
            className={cn(
              "flex h-7 w-7 items-center justify-center rounded transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground",
              showSettings ? "text-primary" : "text-[var(--text-secondary)]",
            )}
            onClick={() => setShowSettings(!showSettings)}
            title="Agent settings"
          >
            <Settings className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            className="flex h-7 w-7 items-center justify-center rounded text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
            onClick={onClose}
            title="Close Panel"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      {/* Settings panel */}
      {showSettings && (
        <div className="shrink-0 border-b border-[var(--border-subtle)] px-3 py-3">
          <div className="flex flex-col gap-2">
            <label className="flex flex-col gap-1">
              <span className="text-[11px] font-medium text-[var(--text-secondary)]">
                API Endpoint
              </span>
              <input
                type="text"
                value={config.apiEndpoint}
                onChange={(e) => setConfig({ apiEndpoint: e.target.value })}
                placeholder="https://api.openai.com/v1/chat/completions"
                className="rounded-md border border-[var(--border-default)] bg-background px-2 py-1 text-[12px] text-foreground placeholder:text-[var(--text-tertiary)] focus:outline-none focus:border-primary/40"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-[11px] font-medium text-[var(--text-secondary)]">API Key</span>
              <input
                type="password"
                value={config.apiKey}
                onChange={(e) => setConfig({ apiKey: e.target.value })}
                placeholder="sk-..."
                className="rounded-md border border-[var(--border-default)] bg-background px-2 py-1 text-[12px] text-foreground placeholder:text-[var(--text-tertiary)] focus:outline-none focus:border-primary/40"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-[11px] font-medium text-[var(--text-secondary)]">Model</span>
              <input
                type="text"
                value={config.model}
                onChange={(e) => setConfig({ model: e.target.value })}
                placeholder="gpt-4o-mini"
                className="rounded-md border border-[var(--border-default)] bg-background px-2 py-1 text-[12px] text-foreground placeholder:text-[var(--text-tertiary)] focus:outline-none focus:border-primary/40"
              />
            </label>
            <p className="text-[11px] text-[var(--text-tertiary)]">
              Leave API key empty to use template-based SQL generation.
            </p>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        {!hasMessages ? (
          <div className="flex flex-col px-4 pb-4 pt-5">
            {/* Context chips */}
            <ContextChips
              connectionId={activeTabMeta?.connectionId}
              connectionName={connectionName}
              activeTabMeta={activeTabMeta}
            />

            {/* Welcome */}
            <div className="mb-4 flex items-center gap-2.5">
              <div className="grid h-8 w-8 place-items-center rounded-lg bg-primary/10">
                <MessageSquare className="h-4 w-4 text-primary" />
              </div>
              <div>
                <p className="text-[13px] font-semibold leading-tight text-foreground">
                  How can I help with this database?
                </p>
                <p className="text-[12px] text-[var(--text-secondary)]">
                  Ask about schemas, queries, optimization
                </p>
              </div>
            </div>

            {/* Starter suggestions */}
            <div className="flex w-full flex-col gap-1">
              {[
                {
                  label: "Explain this table",
                  icon: Table2,
                  prompt: "Explain the current table structure",
                },
                { label: "Write a SELECT query", icon: Code2, prompt: "SELECT * FROM " },
                {
                  label: "Find relations",
                  icon: Network,
                  prompt: "Show foreign key relationships",
                },
                {
                  label: "Optimize current query",
                  icon: Sparkles,
                  prompt: "How can I optimize queries on this schema?",
                },
              ].map((action) => {
                const Icon = action.icon;
                return (
                  <button
                    key={action.label}
                    type="button"
                    className="flex h-[36px] w-full items-center gap-2.5 rounded-md border border-[var(--border-subtle)] px-3 text-[13px] text-[var(--text-secondary)] transition-colors hover:border-[var(--border-default)] hover:bg-[var(--surface-hover)] hover:text-foreground"
                    onClick={() => setInput(action.prompt)}
                  >
                    <Icon className="h-4 w-4 shrink-0 text-[var(--text-tertiary)]" />
                    <span className="flex-1 text-left">{action.label}</span>
                    <ChevronRight className="h-3 w-3 text-[var(--text-tertiary)]" />
                  </button>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-3 px-3 py-3">
            {messages.map((msg) => (
              <MessageBubble
                key={msg.id}
                message={msg}
                onInsertSql={handleInsertSql}
                hasQueryTab={!!activeTabId}
              />
            ))}
            {isProcessing && (
              <div className="flex items-center gap-2 px-2 py-1.5 text-[12px] text-[var(--text-secondary)]">
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                Thinking...
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Composer */}
      <div className="shrink-0 border-t border-[var(--border-subtle)] px-3 py-2.5">
        <div className="flex items-center gap-2 rounded-lg border border-[var(--border-default)] bg-background px-3 py-2 transition-colors focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/20">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask about this database..."
            className="flex-1 bg-transparent text-[13px] text-foreground placeholder:text-[var(--text-tertiary)] focus:outline-none"
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && input.trim()) {
                e.preventDefault();
                void handleSend();
              }
            }}
            disabled={isProcessing}
          />
          <div className="flex items-center gap-1.5">
            <kbd className="text-[11px] text-[var(--text-tertiary)]">⌘↵</kbd>
            <button
              type="button"
              className={cn(
                "flex h-6 w-6 items-center justify-center rounded transition-colors",
                input.trim() && !isProcessing
                  ? "text-primary hover:bg-primary/10"
                  : "text-[var(--text-tertiary)] opacity-50 cursor-not-allowed",
              )}
              disabled={!input.trim() || isProcessing}
              onClick={() => void handleSend()}
              title="Send message"
            >
              <Send className="h-3 w-3" />
            </button>
          </div>
        </div>
      </div>
    </aside>
  );
}

function ContextChips({
  connectionName,
  activeTabMeta,
}: {
  connectionId: string | null | undefined;
  connectionName: string | null | undefined;
  activeTabMeta: {
    connectionId: string | null;
    kind: string;
    database: string | null;
    schema: string | null;
    objectName: string | null;
  } | null;
}) {
  const chips: { label: string; icon: React.ComponentType<{ className?: string }> }[] = [];
  if (connectionName) chips.push({ label: connectionName, icon: Database });
  if (activeTabMeta?.kind === "query") {
    if (activeTabMeta.database) chips.push({ label: activeTabMeta.database, icon: Database });
    if (activeTabMeta.schema) chips.push({ label: activeTabMeta.schema, icon: Layers });
  } else if (activeTabMeta?.kind === "db-object") {
    if (activeTabMeta.schema) chips.push({ label: activeTabMeta.schema, icon: Layers });
    if (activeTabMeta.objectName) chips.push({ label: activeTabMeta.objectName, icon: TableIcon });
  } else if (activeTabMeta?.kind === "schema-workspace") {
    if (activeTabMeta.schema) chips.push({ label: activeTabMeta.schema, icon: Layers });
  }

  if (chips.length === 0) return null;

  return (
    <div className="mb-5 flex w-full flex-wrap gap-1.5">
      {chips.map((chip) => {
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
  );
}

function MessageBubble({
  message,
  onInsertSql,
  hasQueryTab,
}: {
  message: AgentMessage;
  onInsertSql: (sql: string) => void;
  hasQueryTab: boolean;
}) {
  const [copied, setCopied] = useState(false);
  const isUser = message.role === "user";

  const handleCopy = useCallback(() => {
    if (message.sql) {
      void navigator.clipboard.writeText(message.sql);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [message.sql]);

  return (
    <div className={cn("flex flex-col gap-1", isUser ? "items-end" : "items-start")}>
      <div
        className={cn(
          "max-w-full rounded-lg px-3 py-2 text-[13px] leading-relaxed",
          isUser
            ? "bg-primary text-primary-foreground"
            : "bg-[var(--surface-hover)] text-foreground",
        )}
      >
        {message.content}
      </div>

      {message.sql && (
        <div className="w-full rounded-md border border-[var(--border-subtle)] bg-[var(--surface-editor)]">
          <pre className="overflow-x-auto px-3 py-2 text-[12px] leading-relaxed text-foreground">
            <code>{message.sql}</code>
          </pre>
          <div className="flex items-center gap-1 border-t border-[var(--border-subtle)] px-2 py-1">
            {hasQueryTab && (
              <button
                type="button"
                className="flex items-center gap-1 rounded px-2 py-1 text-[11px] text-primary transition-colors hover:bg-primary/10"
                onClick={() => onInsertSql(message.sql!)}
              >
                <Code2 className="h-3 w-3" />
                Insert into editor
              </button>
            )}
            <button
              type="button"
              className="flex items-center gap-1 rounded px-2 py-1 text-[11px] text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)]"
              onClick={handleCopy}
            >
              {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
              {copied ? "Copied" : "Copy"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
