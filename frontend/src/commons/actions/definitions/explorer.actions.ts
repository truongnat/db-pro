import { z } from "zod";

import { defineAction } from "../registry";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import { useConnectionStore } from "@/commons/stores/connection.store";

import type { ActionResult } from "../types";

// ─── explorer.refresh ────────────────────────────────────────

export const refreshExplorerAction = defineAction<{ connectionId?: string }, void>({
  id: "explorer.refresh",
  title: "Refresh explorer",
  description: "Refresh the schema tree for the active or specified connection.",
  category: "explorer",
  inputSchema: z.object({ connectionId: z.string().optional() }),
  risk: "read",

  availability(ctx) {
    if (!ctx.connectionId) {
      return { status: "unavailable", reason: "connection_required" };
    }
    return { status: "available" };
  },

  async execute(input, ctx) {
    const connectionId = input.connectionId ?? ctx.connectionId;
    if (!connectionId) {
      return {
        status: "error",
        error: { code: "connection_required", message: "No connection specified" },
      };
    }

    // Trigger schema re-introspection via the schema service.
    // The actual service call is delegated here; the schema service
    // will invalidate its cache and re-fetch.
    // TODO: wire to ISchemaService.invalidateCache once available via DI.

    return {
      status: "success",
      effects: [{ type: "explorer.refreshed", connectionId }],
    };
  },
});

// ─── explorer.toggleNode ─────────────────────────────────────

export const toggleNodeAction = defineAction<{ path: string }, { path: string; expanded: boolean }>(
  {
    id: "explorer.toggleNode",
    title: "Toggle explorer node",
    description: "Expand or collapse a node in the explorer tree.",
    category: "explorer",
    inputSchema: z.object({ path: z.string().min(1) }),
    risk: "read",

    async execute(input) {
      const { toggleNode } = useExplorerStore.getState();
      toggleNode(input.path);

      const isNowExpanded = useExplorerStore.getState().expandedNodes.includes(input.path);

      return {
        status: "success",
        data: { path: input.path, expanded: isNowExpanded },
        effects: [{ type: "explorer.node.toggled", path: input.path, expanded: isNowExpanded }],
      } satisfies ActionResult<{ path: string; expanded: boolean }>;
    },
  },
);

// ─── explorer.expandNode ─────────────────────────────────────

export const expandNodeAction = defineAction<{ path: string }, { path: string }>({
  id: "explorer.expandNode",
  title: "Expand explorer node",
  description: "Expand a specific node in the explorer tree.",
  category: "explorer",
  inputSchema: z.object({ path: z.string().min(1) }),
  risk: "read",

  async execute(input) {
    useExplorerStore.getState().expandNode(input.path);

    return {
      status: "success",
      data: { path: input.path },
      effects: [{ type: "explorer.node.expanded", path: input.path }],
    } satisfies ActionResult<{ path: string }>;
  },
});

// ─── explorer.openObject ─────────────────────────────────────

export const openObjectAction = defineAction<
  {
    connectionId: string;
    schema: string;
    name: string;
    objectType?: string;
    section?: string;
  },
  { tabId: string }
>({
  id: "explorer.openObject",
  title: "Open database object",
  description: "Open a table, view, function, or other database object in a new tab.",
  category: "explorer",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    name: z.string().min(1),
    objectType: z.string().optional(),
    section: z.string().optional(),
  }),
  risk: "read",

  async execute(input) {
    // Import dynamically to avoid circular deps at module init.
    const { createDbObjectTab } = await import("@/commons/factories/tab-factories");
    const { useWorkspaceStore } = await import("@/commons/stores/workspace.store");

    const tab = createDbObjectTab(
      input.connectionId,
      input.schema,
      input.name,
      (input.objectType as "table" | "view" | "function" | "sequence" | "type") ?? "table",
      (input.section as "data" | "columns" | "indexes" | "relations" | "ddl" | "triggers") ??
        "columns",
    );

    useWorkspaceStore.getState().openDbObject(tab);

    return {
      status: "success",
      data: { tabId: tab.id },
      effects: [
        {
          type: "workspace.tab.opened",
          tabId: tab.id,
          kind: "db-object",
          objectName: input.name,
          schema: input.schema,
        },
      ],
    } satisfies ActionResult<{ tabId: string }>;
  },
});

// ─── explorer.get_selection (query action) ───────────────────

export const getExplorerSelectionAction = defineAction<
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  {},
  { connectionId: string | null }
>({
  id: "explorer.get_selection",
  title: "Get explorer selection",
  description: "Return the currently selected connection in the explorer.",
  category: "explorer",
  inputSchema: z.object({}),
  risk: "read",

  async execute() {
    const connectionId = useConnectionStore.getState().explorerConnectionId;

    return {
      status: "success",
      data: { connectionId },
    } satisfies ActionResult<{ connectionId: string | null }>;
  },
});
