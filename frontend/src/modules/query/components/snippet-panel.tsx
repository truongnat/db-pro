import { useCallback, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTranslation } from "@/commons/locales/useTranslation";

import { expandSnippet, useSnippetStore } from "../stores/snippet.store";
import type { Snippet } from "../types/snippet.types";

interface SnippetPanelProps {
  /** Called when user selects a snippet — inserts expanded SQL into the editor. */
  onInsertSnippet: (sql: string) => void;
}

export function SnippetPanel({ onInsertSnippet }: SnippetPanelProps) {
  const { t } = useTranslation();
  const search = useSnippetStore((s) => s.search);
  const addSnippet = useSnippetStore((s) => s.addSnippet);
  const removeSnippet = useSnippetStore((s) => s.removeSnippet);

  const [query, setQuery] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [newTrigger, setNewTrigger] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [newBody, setNewBody] = useState("");

  const results = search(query);

  const handleAdd = useCallback(() => {
    if (!newTrigger.trim() || !newBody.trim()) return;
    addSnippet(newTrigger.trim(), newLabel.trim() || newTrigger.trim(), newBody.trim());
    setNewTrigger("");
    setNewLabel("");
    setNewBody("");
    setShowAddForm(false);
  }, [newTrigger, newLabel, newBody, addSnippet]);

  return (
    <div className="flex h-full flex-col">
      {/* Search & add toolbar */}
      <div className="flex items-center gap-1 border-b border-[var(--border-subtle)] p-2">
        <Input
          className="flex-1 text-xs"
          placeholder={t("query.searchSnippets")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="shrink-0 text-xs"
          onClick={() => setShowAddForm(!showAddForm)}
        >
          + {t("query.addSnippet")}
        </Button>
      </div>

      {/* Add snippet form */}
      {showAddForm && (
        <div className="space-y-2 border-b border-[var(--border-subtle)] p-2">
          <div className="flex gap-2">
            <Input
              className="flex-1 text-xs"
              placeholder={t("query.snippetTrigger")}
              value={newTrigger}
              onChange={(e) => setNewTrigger(e.target.value)}
            />
            <Input
              className="flex-1 text-xs"
              placeholder={t("query.snippetLabel")}
              value={newLabel}
              onChange={(e) => setNewLabel(e.target.value)}
            />
          </div>
          <textarea
            className="w-full rounded-sm border border-[var(--border-subtle)] bg-background p-2 font-mono text-xs text-foreground outline-none placeholder:text-[var(--text-tertiary)]"
            rows={3}
            placeholder={"SELECT * FROM $cursor;"}
            value={newBody}
            onChange={(e) => setNewBody(e.target.value)}
          />
          <div className="flex justify-end gap-1">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="text-xs"
              onClick={() => setShowAddForm(false)}
            >
              {t("common.actions.cancel")}
            </Button>
            <Button
              type="button"
              variant="default"
              size="sm"
              className="text-xs"
              onClick={handleAdd}
            >
              {t("common.actions.save")}
            </Button>
          </div>
        </div>
      )}

      {/* Snippet list */}
      <div className="flex-1 overflow-auto">
        {results.length === 0 && (
          <div className="flex items-center justify-center py-8">
            <p className="text-sm text-[var(--text-secondary)]">{t("common.states.empty")}</p>
          </div>
        )}
        {results.map((snippet) => (
          <SnippetRow
            key={snippet.trigger}
            snippet={snippet}
            onSelect={() => onInsertSnippet(expandSnippet(snippet.body))}
            onDelete={() => removeSnippet(snippet.trigger)}
          />
        ))}
      </div>
    </div>
  );
}

/* ─── Snippet Row ───────────────────────────────────────────────── */

function SnippetRow({
  snippet,
  onSelect,
  onDelete,
}: {
  snippet: Snippet;
  onSelect: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div
      className="group flex cursor-pointer items-start gap-2 border-b border-[var(--border-subtle)] px-3 py-2 transition-colors hover:bg-background"
      onClick={onSelect}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="shrink-0 px-1.5 py-0 font-mono text-[11px]">
            {snippet.trigger}
          </Badge>
          <span className="truncate text-xs text-foreground">{snippet.label}</span>
          {snippet.builtIn && (
            <Badge variant="secondary" className="shrink-0 px-1 py-0 text-[11px]">
              {t("query.builtIn")}
            </Badge>
          )}
        </div>
        <pre
          className="mt-1 overflow-hidden text-ellipsis whitespace-pre-wrap text-[11px] text-[var(--text-secondary)]"
          style={{ maxHeight: "2.5em" }}
        >
          {snippet.body}
        </pre>
      </div>
      {!snippet.builtIn && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="shrink-0 rounded px-1 text-xs text-[var(--text-secondary)] opacity-0 transition-opacity group-hover:opacity-100"
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          title={t("common.actions.delete")}
        >
          ×
        </Button>
      )}
    </div>
  );
}
