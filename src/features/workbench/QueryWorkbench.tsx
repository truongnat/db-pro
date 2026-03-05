import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import { Circle, FlaskConical, GripHorizontal, Play, Sparkles, Square, X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { ConnectionListItem, DbEngine, QueryResult } from "@/types";
import type { QueryHistoryEntry } from "@/features/query/history";

import { QueryHistoryBar } from "../query/QueryHistoryBar";
import { QueryResultGrid, type QueryGridModifiers } from "../query/QueryResultGrid";
import { SqlTemplateBar } from "../sql-editor/SqlTemplateBar";
import type { SqlCompletionCatalog, SqlEditorSelection } from "../sql-editor/types";

const SqlEditorPane = lazy(() => import("../sql-editor/SqlEditorPane"));
const DEFAULT_EDITOR_RATIO = 0.58;
const MIN_EDITOR_RATIO = 0.3;
const MAX_EDITOR_RATIO = 0.8;

type QueryWorkbenchProps = {
  selectedConnection: ConnectionListItem | null;
  statusText: string;
  errorText: string | null;
  busyAction: string | null;
  cancelRequested: boolean;
  sqlText: string;
  queryResult: QueryResult | null;
  queryHistory: QueryHistoryEntry[];
  queryPageSize: number;
  gridModifiers: QueryGridModifiers;
  sqlEngine: DbEngine | undefined;
  sqlCompletionCatalog: SqlCompletionCatalog;
  onSqlTextChange: (nextText: string) => void;
  onSqlSelectionChange: (selection: SqlEditorSelection) => void;
  onQueryPageSizeChange: (nextPageSize: number) => void;
  onGridModifiersChange: (next: QueryGridModifiers) => void;
  onApplyQueryHistory: (historyId: string) => void;
  onRunQueryHistory: (historyId: string) => void;
  onClearQueryHistory: () => void;
  onApplySqlTemplate: (sql: string, label: string) => void;
  onFormatSql: () => void | Promise<void>;
  onCopyResultPage: (columns: string[], rows: string[][]) => void;
  onExportResultCsv: (columns: string[], rows: string[][]) => void;
  onGridCopyFeedback: (message: string) => void;
  onClearError: () => void;
  onTestConnection: () => void;
  onCancelQuery: () => void;
  onRunQuery: () => void;
  onPreviousPage: () => void;
  onNextPage: () => void;
};

export function QueryWorkbench({
  selectedConnection,
  statusText,
  errorText,
  busyAction,
  cancelRequested,
  sqlText,
  queryResult,
  queryHistory,
  queryPageSize,
  gridModifiers,
  sqlEngine,
  sqlCompletionCatalog,
  onSqlTextChange,
  onSqlSelectionChange,
  onQueryPageSizeChange,
  onGridModifiersChange,
  onApplyQueryHistory,
  onRunQueryHistory,
  onClearQueryHistory,
  onApplySqlTemplate,
  onFormatSql,
  onCopyResultPage,
  onExportResultCsv,
  onGridCopyFeedback,
  onClearError,
  onTestConnection,
  onCancelQuery,
  onRunQuery,
  onPreviousPage,
  onNextPage,
}: QueryWorkbenchProps) {
  const [editorRatio, setEditorRatio] = useState(DEFAULT_EDITOR_RATIO);
  const [resizing, setResizing] = useState(false);
  const layoutRef = useRef<HTMLDivElement | null>(null);
  const resizeStartRef = useRef<{ y: number; ratio: number } | null>(null);

  const clampRatio = useCallback((value: number) => {
    return Math.min(MAX_EDITOR_RATIO, Math.max(MIN_EDITOR_RATIO, value));
  }, []);

  useEffect(() => {
    if (!resizing) {
      return;
    }

    const onPointerMove = (event: PointerEvent) => {
      const start = resizeStartRef.current;
      const container = layoutRef.current;
      if (!start || !container) {
        return;
      }

      const height = container.clientHeight;
      if (height <= 0) {
        return;
      }

      const deltaY = event.clientY - start.y;
      setEditorRatio(clampRatio(start.ratio + deltaY / height));
    };

    const onPointerUp = () => {
      setResizing(false);
      resizeStartRef.current = null;
    };

    document.body.style.cursor = "row-resize";
    document.body.style.userSelect = "none";

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    return () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
  }, [clampRatio, resizing]);

  const startResize = useCallback(
    (clientY: number) => {
      resizeStartRef.current = {
        y: clientY,
        ratio: editorRatio,
      };
      setResizing(true);
    },
    [editorRatio],
  );

  const adjustEditorRatio = useCallback(
    (delta: number) => {
      setEditorRatio((previous) => clampRatio(previous + delta));
    },
    [clampRatio],
  );

  return (
    <Card className="flex min-h-0 flex-col border-white/70 bg-card/90 shadow-xl backdrop-blur-xl">
      <CardHeader className="border-b border-border/60 pb-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-2">
            <CardTitle className="text-base">
              {selectedConnection?.name ?? "No active connection"}
            </CardTitle>
            <CardDescription>
              {selectedConnection?.target ?? "Pick or add a connection"}
            </CardDescription>
            <div className="flex items-center gap-2">
              <Circle className="h-2.5 w-2.5 fill-emerald-500 text-emerald-500" />
              <span className="text-xs text-muted-foreground">
                Passwords are stored in system keychain
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="secondary"
              onClick={onTestConnection}
              disabled={!selectedConnection || !!busyAction}
            >
              <FlaskConical className="mr-1.5 h-4 w-4" />
              {busyAction === "test-connection" ? "Testing..." : "Test"}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={onCancelQuery}
              disabled={!selectedConnection || busyAction !== "run-query"}
            >
              <Square className="mr-1.5 h-4 w-4" />
              {cancelRequested ? "Cancelling..." : "Stop"}
            </Button>
            <Button
              type="button"
              onClick={onRunQuery}
              disabled={!selectedConnection || !!busyAction}
            >
              <Play className="mr-1.5 h-4 w-4" />
              {busyAction === "run-query" ? "Running..." : "Run Query"}
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="flex min-h-0 flex-1 flex-col overflow-hidden pt-4">
        {busyAction && (
          <div className="mb-2 rounded-md border border-blue-500/30 bg-blue-500/10 px-3 py-2 text-xs text-blue-700">
            Working: {busyAction.replace(/-/g, " ")}...
          </div>
        )}

        {errorText && (
          <div className="mb-2 flex items-start justify-between gap-2 rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
            <span>{errorText}</span>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-5 w-5 shrink-0"
              onClick={onClearError}
            >
              <X className="h-3.5 w-3.5" />
              <span className="sr-only">Dismiss error</span>
            </Button>
          </div>
        )}

        <div ref={layoutRef} className={cn("flex min-h-0 flex-1 flex-col", resizing && "select-none")}>
          <div
            className="flex min-h-[180px] min-w-0 flex-col overflow-hidden pb-2"
            style={{ flex: `0 0 ${Math.round(editorRatio * 1000) / 10}%` }}
          >
            <div className="mb-2">
              <QueryHistoryBar
                items={queryHistory}
                busy={!!busyAction}
                onApply={onApplyQueryHistory}
                onRun={onRunQueryHistory}
                onClear={onClearQueryHistory}
              />
            </div>
            <div className="mb-2">
              <SqlTemplateBar
                engine={sqlEngine}
                busy={!!busyAction}
                onApplyTemplate={onApplySqlTemplate}
              />
            </div>
            <div className="mb-2 flex items-center justify-between text-xs text-muted-foreground">
              <span>Query text</span>
              <div className="flex items-center gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 text-[11px]"
                  onClick={() => {
                    void onFormatSql();
                  }}
                  disabled={!!busyAction}
                >
                  <Sparkles className="mr-1 h-3.5 w-3.5" />
                  Format
                </Button>
                <Badge variant="outline">Page size {queryPageSize} rows</Badge>
              </div>
            </div>
            <div className="min-h-0 flex-1 overflow-hidden rounded-md border bg-background">
              <Suspense
                fallback={
                  <div className="flex h-full min-h-[180px] items-center justify-center text-xs text-muted-foreground">
                    Loading SQL editor...
                  </div>
                }
              >
                <SqlEditorPane
                  value={sqlText}
                  engine={sqlEngine}
                  completionCatalog={sqlCompletionCatalog}
                  busy={!!busyAction}
                  onRunQuery={onRunQuery}
                  onFormatSql={onFormatSql}
                  onValueChange={onSqlTextChange}
                  onSelectionChange={onSqlSelectionChange}
                />
              </Suspense>
            </div>
          </div>

          <button
            type="button"
            className={cn(
              "group flex h-4 items-center justify-center rounded-sm border-y bg-muted/35 text-muted-foreground transition hover:bg-muted/55",
              resizing && "bg-muted/65",
            )}
            onPointerDown={(event) => startResize(event.clientY)}
            onKeyDown={(event) => {
              if (event.key === "ArrowUp") {
                event.preventDefault();
                adjustEditorRatio(-0.03);
              }
              if (event.key === "ArrowDown") {
                event.preventDefault();
                adjustEditorRatio(0.03);
              }
            }}
            aria-label="Resize editor and result panels"
          >
            <GripHorizontal className="h-3.5 w-3.5 transition group-hover:text-foreground" />
          </button>

          <div className="flex min-h-[160px] min-w-0 flex-1 flex-col gap-3 overflow-hidden pt-2">
            <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              Result grid
            </div>
            <div className="min-h-0 flex-1 overflow-hidden">
              <QueryResultGrid
                queryResult={queryResult}
                loadingPage={busyAction === "run-query"}
                queryPageSize={queryPageSize}
                gridModifiers={gridModifiers}
                onQueryPageSizeChange={onQueryPageSizeChange}
                onGridModifiersChange={onGridModifiersChange}
                onCopyResultPage={onCopyResultPage}
                onExportResultCsv={onExportResultCsv}
                onGridCopyFeedback={onGridCopyFeedback}
                onPreviousPage={onPreviousPage}
                onNextPage={onNextPage}
              />
            </div>
          </div>
        </div>
      </CardContent>

      <CardFooter className="border-t border-border/60 py-3 text-xs text-muted-foreground">
        {statusText}
      </CardFooter>
    </Card>
  );
}
