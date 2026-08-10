import { useCallback, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
} from "@/components/ui/alert-dialog";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useTranslation } from "@/commons/locales/useTranslation";
import { MoreHorizontal, Trash2, Plus, Copy, Check } from "lucide-react";

import { useExecuteDdl } from "../queries/schema.queries";
import type { SchemaColumnDto, SchemaIndexDto } from "../types/schema.types";
import { buildCreateIndex, buildDropIndex } from "../services/ddl-builder";
import { cn } from "@/lib/utils";

interface IndexManagerProps {
  connectionId: string;
  schema: string;
  table: string;
  columns: SchemaColumnDto[];
  indexes: SchemaIndexDto[];
}

export function IndexManager({ connectionId, schema, table, columns, indexes }: IndexManagerProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);

  const [showCreate, setShowCreate] = useState(false);
  const [droppingIndex, setDroppingIndex] = useState<string | null>(null);
  const [copiedIdx, setCopiedIdx] = useState<string | null>(null);

  const handleCopyName = async (name: string) => {
    await navigator.clipboard.writeText(name).catch(() => {});
    setCopiedIdx(name);
    setTimeout(() => setCopiedIdx(null), 1500);
  };

  const handleConfirmDrop = useCallback(() => {
    if (!droppingIndex) return;
    const sql = buildDropIndex(schema, droppingIndex);
    executeDdl.mutate(sql, {
      onSuccess: () => setDroppingIndex(null),
    });
  }, [droppingIndex, schema, executeDdl]);

  const headerClass =
    "px-3 py-2 font-medium text-[12.5px] text-[var(--app-text-muted)] border-b border-[var(--app-border-subtle)]";

  return (
    <>
      <div className="flex min-h-0 flex-col overflow-auto">
        <div className="flex items-center justify-between px-3 py-2">
          <span className="text-xs font-semibold text-foreground">
            {indexes.length} {indexes.length === 1 ? "index" : "indexes"}
          </span>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-7 gap-1.5 text-[12px]"
            onClick={() => setShowCreate(true)}
          >
            <Plus className="h-3 w-3" />
            {t("schema.createIndex")}
          </Button>
        </div>

        {indexes.length === 0 ? (
          <div className="px-3 py-8 text-center text-xs italic text-[var(--app-text-muted)]">
            {t("schema.noIndexes")}
          </div>
        ) : (
          <Table className="w-full text-[13px]">
            <TableHeader>
              <TableRow>
                <TableHead className={cn(headerClass, "text-left")}>
                  {t("schema.ddlIndexName")}
                </TableHead>
                <TableHead className={cn(headerClass, "text-left")}>
                  {t("schema.ddlIndexColumns")}
                </TableHead>
                <TableHead className={cn(headerClass, "text-center")}>Unique</TableHead>
                <TableHead className={cn(headerClass, "w-10")} />
              </TableRow>
            </TableHeader>
            <TableBody>
              {indexes.map((idx) => (
                <TableRow
                  key={idx.name}
                  className="group transition-colors hover:bg-[var(--app-hover)]"
                >
                  <TableCell className="px-3 py-1.5 font-mono text-[13px]">
                    <div className="flex items-center gap-1.5">
                      <span className="select-text">{idx.name}</span>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="h-5 w-5 shrink-0 p-0 opacity-0 transition-opacity group-hover:opacity-60 hover:!opacity-100 hover:bg-transparent"
                            onClick={() => handleCopyName(idx.name)}
                          >
                            {copiedIdx === idx.name ? (
                              <Check className="h-3 w-3 text-emerald-500" />
                            ) : (
                              <Copy className="h-3 w-3" />
                            )}
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Copy index name</TooltipContent>
                      </Tooltip>
                    </div>
                  </TableCell>
                  <TableCell className="px-3 py-1.5 font-mono text-[11px] text-[var(--app-text-muted)] select-text">
                    {idx.columns.join(", ")}
                  </TableCell>
                  <TableCell className="px-3 py-1.5 text-center">
                    {idx.unique && <Badge variant="info">UNIQUE</Badge>}
                  </TableCell>
                  <TableCell className="px-1 py-1.5 text-center">
                    <DropdownMenu>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <DropdownMenuTrigger asChild>
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              className="h-6 w-6 p-0 opacity-0 transition-opacity group-hover:opacity-60 hover:!opacity-100"
                            >
                              <MoreHorizontal className="h-3.5 w-3.5" />
                            </Button>
                          </DropdownMenuTrigger>
                        </TooltipTrigger>
                        <TooltipContent>Actions</TooltipContent>
                      </Tooltip>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem
                          className="text-destructive focus:text-destructive"
                          onClick={() => setDroppingIndex(idx.name)}
                        >
                          <Trash2 className="size-3.5" />
                          Drop index
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>

      {/* Create Index Dialog */}
      {showCreate && (
        <CreateIndexDialog
          schema={schema}
          table={table}
          columns={columns}
          connectionId={connectionId}
          onClose={() => setShowCreate(false)}
        />
      )}

      {/* Confirm Drop Index */}
      <AlertDialog
        open={!!droppingIndex}
        onOpenChange={(open) => {
          if (!open) setDroppingIndex(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Drop Index</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to drop index{" "}
              <code className="rounded bg-muted px-1 font-mono">{droppingIndex}</code>? This action
              cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.actions.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmDrop}>
              {executeDdl.isPending ? t("common.states.loading") : "Drop Index"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

// ── Create Index Dialog ──────────────────────────────────────────────

interface CreateIndexDialogProps {
  schema: string;
  table: string;
  columns: SchemaColumnDto[];
  connectionId: string;
  onClose: () => void;
}

function CreateIndexDialog({
  schema,
  table,
  columns,
  connectionId,
  onClose,
}: CreateIndexDialogProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);

  const [indexName, setIndexName] = useState("");
  const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
  const [unique, setUnique] = useState(false);

  const toggleColumn = useCallback((col: string) => {
    setSelectedColumns((prev) =>
      prev.includes(col) ? prev.filter((c) => c !== col) : [...prev, col],
    );
  }, []);

  const sql = (() => {
    if (!indexName.trim() || selectedColumns.length === 0) return null;
    return buildCreateIndex(schema, table, indexName.trim(), selectedColumns, unique);
  })();

  const handleCreate = useCallback(() => {
    if (!sql) return;
    executeDdl.mutate(sql, { onSuccess: () => onClose() });
  }, [sql, executeDdl, onClose]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter" && sql) {
        e.preventDefault();
        handleCreate();
      }
    },
    [handleCreate, sql],
  );

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent className="sm:max-w-md" onKeyDown={handleKeyDown}>
        <DialogHeader>
          <DialogTitle>{t("schema.createIndex")}</DialogTitle>
          <DialogDescription>
            {schema}.{table}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-4">
          <div>
            <Label htmlFor="index-name" className="mb-1 block text-xs text-[var(--app-text-muted)]">
              {t("schema.ddlIndexName")}
            </Label>
            <Input
              id="index-name"
              value={indexName}
              onChange={(e) => setIndexName(e.target.value)}
              className="w-full font-mono text-sm"
              placeholder="idx_table_columns"
              autoFocus
            />
          </div>

          <div>
            <Label className="mb-1.5 block text-xs text-[var(--app-text-muted)]">
              {t("schema.ddlIndexColumns")}
            </Label>
            <div className="flex flex-wrap gap-1.5">
              {columns.map((col) => (
                <Button
                  key={col.name}
                  type="button"
                  size="sm"
                  variant={selectedColumns.includes(col.name) ? "default" : "outline"}
                  className="h-7 text-[12px]"
                  onClick={() => toggleColumn(col.name)}
                >
                  {col.name}
                </Button>
              ))}
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Checkbox
              id="idx-unique"
              checked={unique}
              onCheckedChange={(checked) => setUnique(checked === true)}
            />
            <Label htmlFor="idx-unique" className="text-sm font-normal">
              {t("schema.ddlUnique")}
            </Label>
          </div>

          {/* SQL Preview */}
          {sql && (
            <pre className="overflow-x-auto rounded bg-muted p-2 font-mono text-[11px] leading-relaxed text-foreground">
              {sql}
            </pre>
          )}

          {executeDdl.isError && (
            <div className="rounded-sm bg-destructive/10 px-3 py-2 text-xs text-destructive">
              {(executeDdl.error as Error)?.message ?? "Failed to create index"}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="button"
            disabled={!indexName.trim() || selectedColumns.length === 0 || executeDdl.isPending}
            onClick={handleCreate}
          >
            {executeDdl.isPending ? t("common.states.loading") : t("schema.createIndex")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
