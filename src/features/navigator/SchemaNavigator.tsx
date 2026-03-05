import { useEffect, useMemo, useRef, useState } from "react";
import {
  Braces,
  Columns3,
  Copy,
  DatabaseZap,
  Eye,
  FilePlus2,
  FolderTree,
  PencilLine,
  Play,
  RefreshCw,
  Search,
  Table2,
} from "lucide-react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type { NavigatorObject, NavigatorSchema, NavigatorTree } from "@/types";

import {
  type NavigatorActionRequest,
  type NavigatorObjectAction,
} from "./actions";

type SchemaNavigatorProps = {
  selectedConnection: boolean;
  tree: NavigatorTree | null;
  loading: boolean;
  error: string | null;
  objectCount: number;
  onRefresh: () => void;
  onAction: (request: NavigatorActionRequest) => void;
};

type ContextTarget = Pick<NavigatorActionRequest, "schemaName" | "object">;

type ContextMenuState = ContextTarget & {
  x: number;
  y: number;
};

type ObjectGroupProps = {
  label: string;
  schema: NavigatorSchema;
  kind: "table" | "view";
  onAction: (request: NavigatorActionRequest) => void;
  onOpenContextMenu: (
    x: number,
    y: number,
    target: ContextTarget,
  ) => void;
};

type ContextMenuItem = {
  action: NavigatorObjectAction;
  label: string;
  description: string;
  Icon: typeof Braces;
};

const CONTEXT_MENU_ITEMS: ContextMenuItem[] = [
  {
    action: "generate_select",
    label: "Generate SELECT",
    description: "Insert SELECT query into editor",
    Icon: Braces,
  },
  {
    action: "open_data",
    label: "Open Data",
    description: "Generate and run SELECT immediately",
    Icon: Play,
  },
  {
    action: "copy_name",
    label: "Copy Qualified Name",
    description: "Copy schema.object identifier",
    Icon: Copy,
  },
  {
    action: "generate_insert",
    label: "Generate INSERT",
    description: "Insert row template using object columns",
    Icon: FilePlus2,
  },
  {
    action: "generate_update",
    label: "Generate UPDATE",
    description: "Update template with WHERE clause",
    Icon: PencilLine,
  },
  {
    action: "generate_ddl",
    label: "Generate DDL",
    description: "Create table/view DDL template",
    Icon: DatabaseZap,
  },
];

function boundMenuPosition(x: number, y: number): { x: number; y: number } {
  if (typeof window === "undefined") {
    return { x, y };
  }

  const menuWidth = 280;
  const menuHeight = 290;
  const offset = 8;

  return {
    x: Math.max(offset, Math.min(x, window.innerWidth - menuWidth - offset)),
    y: Math.max(offset, Math.min(y, window.innerHeight - menuHeight - offset)),
  };
}

function NavigatorColumns({ object }: { object: NavigatorObject }) {
  if (object.columns.length === 0) {
    return <p className="text-xs text-muted-foreground">No column metadata.</p>;
  }

  return (
    <>
      {object.columns.map((column) => (
        <div
          key={`${object.name}-${column.name}`}
          className="flex items-center justify-between rounded border bg-muted/30 px-2 py-1 text-xs"
        >
          <span className="inline-flex items-center gap-1">
            <Columns3 className="h-3 w-3 text-muted-foreground" />
            {column.name}
          </span>
          <span className="inline-flex items-center gap-1">
            <Badge variant="outline">{column.dataType}</Badge>
            <Badge variant={column.nullable ? "secondary" : "outline"}>
              {column.nullable ? "NULL" : "NOT NULL"}
            </Badge>
          </span>
        </div>
      ))}
    </>
  );
}

function NavigatorObjectGroup({
  label,
  schema,
  kind,
  onAction,
  onOpenContextMenu,
}: ObjectGroupProps) {
  const objects = kind === "table" ? schema.tables : schema.views;
  const icon = kind === "table" ? Table2 : Eye;

  if (objects.length === 0) {
    return null;
  }

  return (
    <div className="space-y-1">
      <p className="text-[11px] uppercase tracking-wide text-muted-foreground">{label}</p>
      <Accordion type="multiple" className="w-full">
        {objects.map((object) => {
          const Icon = icon;
          const actionTarget = { schemaName: schema.name, object };

          return (
            <AccordionItem
              key={`${schema.name}-${kind}-${object.name}`}
              value={`${schema.name}-${kind}-${object.name}`}
              className="border-b-0"
            >
              <AccordionTrigger className="py-1.5 text-xs hover:no-underline">
                <div
                  className="flex w-full items-center justify-between pr-2"
                  onContextMenu={(event) => {
                    event.preventDefault();
                    onOpenContextMenu(event.clientX, event.clientY, actionTarget);
                  }}
                >
                  <span className="inline-flex items-center gap-1">
                    <Icon className="h-3.5 w-3.5 text-muted-foreground" />
                    {object.name}
                  </span>
                  <Badge variant="secondary">{object.columns.length}</Badge>
                </div>
              </AccordionTrigger>
              <AccordionContent className="space-y-1 pb-2">
                <div className="mb-1 rounded border border-dashed bg-muted/30 px-2 py-1 text-[10px] text-muted-foreground">
                  Right-click object name for actions.
                </div>
                <div className="mb-1 flex flex-wrap items-center gap-1">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-6 text-[10px]"
                    onClick={() => onAction({ ...actionTarget, action: "generate_select" })}
                  >
                    <Braces className="mr-1 h-3 w-3" />
                    Generate SELECT
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-6 text-[10px]"
                    onClick={() => onAction({ ...actionTarget, action: "open_data" })}
                  >
                    <Play className="mr-1 h-3 w-3" />
                    Open Data
                  </Button>
                </div>
                <NavigatorColumns object={object} />
              </AccordionContent>
            </AccordionItem>
          );
        })}
      </Accordion>
    </div>
  );
}

function NavigatorContextMenu({
  contextMenu,
  onAction,
  onClose,
}: {
  contextMenu: ContextMenuState | null;
  onAction: (request: NavigatorActionRequest) => void;
  onClose: () => void;
}) {
  const menuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!contextMenu) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [contextMenu, onClose]);

  if (!contextMenu) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50"
      onMouseDown={(event) => {
        if (!menuRef.current?.contains(event.target as Node)) {
          onClose();
        }
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        onClose();
      }}
    >
      <div
        ref={menuRef}
        className="absolute w-[280px] rounded-md border bg-popover p-1 text-popover-foreground shadow-2xl"
        style={{ left: contextMenu.x, top: contextMenu.y }}
      >
        {CONTEXT_MENU_ITEMS.map(({ action, label, description, Icon }) => (
          <button
            key={action}
            type="button"
            className="flex w-full items-start gap-2 rounded px-2 py-1.5 text-left transition hover:bg-accent"
            onClick={() => {
              onAction({
                schemaName: contextMenu.schemaName,
                object: contextMenu.object,
                action,
              });
              onClose();
            }}
          >
            <Icon className="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
            <span>
              <span className="block text-xs font-medium">{label}</span>
              <span className="block text-[10px] text-muted-foreground">{description}</span>
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}

export function SchemaNavigator({
  selectedConnection,
  tree,
  loading,
  error,
  objectCount,
  onRefresh,
  onAction,
}: SchemaNavigatorProps) {
  const [filterText, setFilterText] = useState("");
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const warnings = tree?.warnings ?? [];
  const normalizedFilter = filterText.trim().toLowerCase();

  useEffect(() => {
    if (!selectedConnection) {
      setFilterText("");
      setContextMenu(null);
    }
  }, [selectedConnection]);

  const filteredSchemas = useMemo(() => {
    if (!tree) {
      return [];
    }
    if (!normalizedFilter) {
      return tree.schemas;
    }

    const includesValue = (value: string) =>
      value.toLowerCase().includes(normalizedFilter);
    const objectMatches = (object: NavigatorObject) =>
      includesValue(object.name) ||
      object.columns.some(
        (column) =>
          includesValue(column.name) || includesValue(column.dataType),
      );

    return tree.schemas
      .map((schema) => {
        const schemaMatches = includesValue(schema.name);
        const tables = schemaMatches
          ? schema.tables
          : schema.tables.filter(objectMatches);
        const views = schemaMatches ? schema.views : schema.views.filter(objectMatches);

        return {
          ...schema,
          tables,
          views,
        };
      })
      .filter((schema) => schema.tables.length > 0 || schema.views.length > 0);
  }, [normalizedFilter, tree]);

  const filteredObjectCount = useMemo(
    () =>
      filteredSchemas.reduce(
        (acc, schema) => acc + schema.tables.length + schema.views.length,
        0,
      ),
    [filteredSchemas],
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span className="inline-flex items-center gap-1">
          <FolderTree className="h-3.5 w-3.5" />
          Schema Navigator
        </span>
        <div className="flex items-center gap-1">
          <Badge variant="secondary">
            {normalizedFilter ? `${filteredObjectCount}/${objectCount}` : objectCount}
          </Badge>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            disabled={!selectedConnection || loading}
            onClick={onRefresh}
          >
            <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
            <span className="sr-only">Refresh navigator</span>
          </Button>
        </div>
      </div>

      {selectedConnection && (
        <div className="relative">
          <Search className="pointer-events-none absolute left-2 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
          <Input
            value={filterText}
            onChange={(event) => setFilterText(event.target.value)}
            placeholder="Filter schemas, objects, columns..."
            className="h-8 pl-7 text-xs"
          />
        </div>
      )}

      {!selectedConnection && (
        <div className="rounded-md border border-dashed bg-muted/50 p-3 text-xs text-muted-foreground">
          Select a connection to inspect schemas.
        </div>
      )}

      {selectedConnection && error && (
        <div className="rounded-md border border-dashed border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive">
          <div>{error}</div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="mt-2 h-7 text-xs"
            onClick={onRefresh}
          >
            Retry
          </Button>
        </div>
      )}

      {selectedConnection && !error && warnings.length > 0 && (
        <div className="rounded-md border border-dashed border-amber-500/40 bg-amber-500/10 p-3 text-xs text-amber-800">
          {warnings[0]}
          {warnings.length > 1 ? ` (+${warnings.length - 1} warning(s))` : ""}
        </div>
      )}

      {selectedConnection && loading && !tree && (
        <div className="rounded-md border border-dashed bg-muted/50 p-3 text-xs text-muted-foreground">
          Loading schema metadata...
        </div>
      )}

      {selectedConnection && !error && tree && tree.schemas.length === 0 && !loading && (
        <div className="rounded-md border border-dashed bg-muted/50 p-3 text-xs text-muted-foreground">
          No tables or views found.
        </div>
      )}

      {selectedConnection && !error && tree && filteredSchemas.length === 0 && !loading && (
        <div className="rounded-md border border-dashed bg-muted/50 p-3 text-xs text-muted-foreground">
          No navigator objects match your filter.
        </div>
      )}

      {selectedConnection && !error && tree && filteredSchemas.length > 0 && (
        <ScrollArea className="min-h-0 flex-1 pr-2">
          <Accordion type="multiple" className="w-full space-y-2">
            {filteredSchemas.map((schema) => (
              <AccordionItem
                key={schema.name}
                value={`schema-${schema.name}`}
                className="rounded-lg border bg-background/70 px-3"
              >
                <AccordionTrigger className="py-2 text-xs hover:no-underline">
                  <div className="flex w-full items-center justify-between pr-2">
                    <span className="font-medium">{schema.name}</span>
                    <Badge variant="outline">
                      {schema.tables.length + schema.views.length}
                    </Badge>
                  </div>
                </AccordionTrigger>
                <AccordionContent className="space-y-2 pb-3">
                  <NavigatorObjectGroup
                    label="Tables"
                    schema={schema}
                    kind="table"
                    onAction={onAction}
                    onOpenContextMenu={(x, y, target) => {
                      const position = boundMenuPosition(x, y);
                      setContextMenu({ ...target, ...position });
                    }}
                  />
                  <NavigatorObjectGroup
                    label="Views"
                    schema={schema}
                    kind="view"
                    onAction={onAction}
                    onOpenContextMenu={(x, y, target) => {
                      const position = boundMenuPosition(x, y);
                      setContextMenu({ ...target, ...position });
                    }}
                  />
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </ScrollArea>
      )}

      <NavigatorContextMenu
        contextMenu={contextMenu}
        onClose={() => setContextMenu(null)}
        onAction={onAction}
      />
    </div>
  );
}
