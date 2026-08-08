import { z } from "zod";

import { defineAction } from "../registry";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { setTabActivePanel } from "@/modules/query/controllers/query-workspace.controller";

import type { ActionResult } from "../types";
import type { ResultPanelTab, WorkspaceTab } from "@/commons/types/workspace.types";

// ─── workspace.tab.open ──────────────────────────────────────

export const openTabAction = defineAction<
  { tab: unknown },
  { tabId: string }
>({
  id: "workspace.tab.open",
  title: "Open tab",
  description: "Open a new workspace tab (query or db-object).",
  category: "workspace",
  inputSchema: z.object({
    // Accepts a serialisable tab descriptor.
    // The caller is responsible for constructing it via tab factories.
    tab: z.unknown(),
  }),
  risk: "read",

  async execute(input) {
    const tab = input.tab as WorkspaceTab;
    useWorkspaceStore.getState().openTab(tab);

    return {
      status: "success",
      data: { tabId: tab.id },
      effects: [
        { type: "workspace.tab.opened", tabId: tab.id, kind: tab.kind },
      ],
    } satisfies ActionResult<{ tabId: string }>;
  },
});

// ─── workspace.tab.close ─────────────────────────────────────

export const closeTabAction = defineAction<
  { tabId?: string },
  { closedTabId: string }
>({
  id: "workspace.tab.close",
  title: "Close tab",
  description: "Close the active or specified workspace tab.",
  category: "workspace",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  availability(ctx) {
    const { tabs, activeTabId } = useWorkspaceStore.getState();
    const id = ctx.tabId ?? activeTabId;
    if (!id || !tabs.some((t) => t.id === id)) {
      return { status: "unavailable", reason: "no_active_tab" };
    }
    return { status: "available" };
  },

  async execute(input) {
    const { activeTabId, tabs } = useWorkspaceStore.getState();
    const tabId = input.tabId ?? activeTabId;

    if (!tabId) {
      return {
        status: "error",
        error: { code: "no_tab", message: "No tab to close" },
      };
    }

    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) {
      return {
        status: "error",
        error: { code: "tab_not_found", message: `Tab "${tabId}" not found` },
      };
    }

    // Use close-guard for dirty tabs.
    if (tab.dirty) {
      useCloseGuardStore.getState().openDialog([tabId], 1);
      return {
        status: "confirmation_required",
        confirmation: {
          id: `close_guard_${tabId}`,
          actionId: "workspace.tab.close",
          message: `Tab "${tab.title}" has unsaved changes. Close anyway?`,
          risk: "read",
        },
      };
    }

    useWorkspaceStore.getState().closeTab(tabId);

    return {
      status: "success",
      data: { closedTabId: tabId },
      effects: [{ type: "workspace.tab.closed", tabId }],
    } satisfies ActionResult<{ closedTabId: string }>;
  },
});

// ─── workspace.tab.activate ──────────────────────────────────

export const activateTabAction = defineAction<
  { tabId: string },
  { tabId: string }
>({
  id: "workspace.tab.activate",
  title: "Activate tab",
  description: "Switch focus to the specified workspace tab.",
  category: "workspace",
  inputSchema: z.object({ tabId: z.string() }),
  risk: "read",

  async execute(input) {
    const { tabs, activateTab } = useWorkspaceStore.getState();
    if (!tabs.some((t) => t.id === input.tabId)) {
      return {
        status: "error",
        error: { code: "tab_not_found", message: `Tab "${input.tabId}" not found` },
      };
    }

    activateTab(input.tabId);

    return {
      status: "success",
      data: { tabId: input.tabId },
      effects: [{ type: "workspace.tab.activated", tabId: input.tabId }],
    } satisfies ActionResult<{ tabId: string }>;
  },
});

// ─── workspace.tab.pin ───────────────────────────────────────

export const pinTabAction = defineAction<
  { tabId?: string },
  { tabId: string; pinned: boolean }
>({
  id: "workspace.tab.pin",
  title: "Pin / unpin tab",
  description: "Toggle the pinned state of the active or specified tab.",
  category: "workspace",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  async execute(input) {
    const { activeTabId, tabs, toggleTabPinned } = useWorkspaceStore.getState();
    const tabId = input.tabId ?? activeTabId;

    if (!tabId || !tabs.some((t) => t.id === tabId)) {
      return {
        status: "error",
        error: { code: "tab_not_found", message: "No tab to pin" },
      };
    }

    toggleTabPinned(tabId);
    const updated = useWorkspaceStore.getState().tabs.find((t) => t.id === tabId);

    return {
      status: "success",
      data: { tabId, pinned: updated?.pinned ?? false },
      effects: [{ type: "workspace.tab.pinned", tabId, pinned: updated?.pinned ?? false }],
    } satisfies ActionResult<{ tabId: string; pinned: boolean }>;
  },
});

// ─── workspace.panel.set ─────────────────────────────────────

export const setPanelAction = defineAction<
  { tabId?: string; panel: string },
  { panel: string }
>({
  id: "workspace.panel.set",
  title: "Set result panel",
  description:
    "Switch the result panel tab (results, explain, history, local-history, snippets).",
  category: "workspace",
  inputSchema: z.object({
    tabId: z.string().optional(),
    panel: z.enum(["results", "explain", "history", "local-history", "snippets"]),
  }),
  risk: "read",

  async execute(input, ctx) {
    const tabId = input.tabId ?? ctx.tabId;
    if (!tabId) {
      return {
        status: "error",
        error: { code: "no_tab", message: "No active tab" },
      };
    }

    setTabActivePanel(tabId, input.panel as ResultPanelTab);

    return {
      status: "success",
      data: { panel: input.panel },
      effects: [{ type: "workspace.panel.changed", tabId, panel: input.panel }],
    } satisfies ActionResult<{ panel: string }>;
  },
});

// ─── workspace.get_state (query action) ──────────────────────

export const getWorkspaceStateAction = defineAction<
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  {},
  {
    activeTabId: string | null;
    tabCount: number;
    tabs: Array<{
      id: string;
      kind: string;
      title: string;
      connectionId: string | null;
      dirty: boolean;
      pinned: boolean;
    }>;
  }
>({
  id: "workspace.get_state",
  title: "Get workspace state",
  description: "Return semantic workspace state (open tabs, active tab, etc.).",
  category: "workspace",
  inputSchema: z.object({}),
  risk: "read",

  async execute() {
    const { tabs, activeTabId } = useWorkspaceStore.getState();

    return {
      status: "success",
      data: {
        activeTabId,
        tabCount: tabs.length,
        tabs: tabs.map((t) => ({
          id: t.id,
          kind: t.kind,
          title: t.title,
          connectionId: t.connectionId,
          dirty: t.dirty,
          pinned: t.pinned,
        })),
      },
    } satisfies ActionResult;
  },
});
