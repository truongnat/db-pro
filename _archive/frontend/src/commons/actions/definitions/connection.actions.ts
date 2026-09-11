import { z } from "zod";

import { defineAction } from "../registry";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useRecentStore } from "@/commons/stores/recent.store";

import type { ActionResult } from "../types";

// ─── connection.new ──────────────────────────────────────────

export const newConnectionAction = defineAction<
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  {},
  void
>({
  id: "connection.new",
  title: "New connection",
  description: "Open the connection dialog to create or edit a connection.",
  category: "connection",
  inputSchema: z.object({}),
  risk: "read",

  async execute() {
    useRecentStore.getState().openConnectionDialog();

    return {
      status: "success",
      effects: [{ type: "connection.dialog.opened" }],
    };
  },
});

// ─── connection.setActive ────────────────────────────────────

export const setActiveConnectionAction = defineAction<
  { connectionId: string },
  { connectionId: string }
>({
  id: "connection.setActive",
  title: "Set active connection",
  description: "Set the active connection in the explorer.",
  category: "connection",
  inputSchema: z.object({ connectionId: z.string().min(1) }),
  risk: "read",

  availability() {
    const { connections } = useConnectionStore.getState();
    if (connections.length === 0) {
      return { status: "unavailable", reason: "no_connections" };
    }
    return { status: "available" };
  },

  async execute(input) {
    const { connections } = useConnectionStore.getState();
    if (!connections.some((c) => c.id === input.connectionId)) {
      return {
        status: "error",
        error: {
          code: "connection_not_found",
          message: `Connection "${input.connectionId}" not found`,
        },
      };
    }

    useConnectionStore.getState().setExplorerConnection(input.connectionId);

    return {
      status: "success",
      data: { connectionId: input.connectionId },
      effects: [{ type: "connection.activated", connectionId: input.connectionId }],
    } satisfies ActionResult<{ connectionId: string }>;
  },
});

// ─── connection.list ─────────────────────────────────────────

export const listConnectionsAction = defineAction<
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  {},
  {
    connections: Array<{
      id: string;
      name: string;
      driver: string;
      database: string;
    }>;
  }
>({
  id: "connection.list",
  title: "List connections",
  description: "Return all configured connections.",
  category: "connection",
  inputSchema: z.object({}),
  risk: "read",

  async execute() {
    const { connections } = useConnectionStore.getState();

    return {
      status: "success",
      data: {
        connections: connections.map((c) => ({
          id: c.id,
          name: c.name,
          driver: c.driver,
          database: c.database,
        })),
      },
    } satisfies ActionResult;
  },
});
