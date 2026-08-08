import { z } from "zod";

import { defineAction } from "../registry";
import { createConnectionService } from "@/modules/connection/services/connection.service";

import type { ActionResult } from "../types";

// ─── database.connect ────────────────────────────────────────

export const connectDatabaseAction = defineAction<
  { connectionId: string },
  void
>({
  id: "database.connect",
  title: "Connect to database",
  description: "Establish a connection to the database.",
  category: "connection",
  inputSchema: z.object({ connectionId: z.string().min(1) }),
  risk: "read",

  async execute(input) {
    const service = createConnectionService();
    await service.connect(input.connectionId);

    return {
      status: "success",
      effects: [
        { type: "connection.connected", connectionId: input.connectionId },
      ],
    };
  },
});

// ─── database.disconnect ─────────────────────────────────────

export const disconnectDatabaseAction = defineAction<
  { connectionId: string },
  void
>({
  id: "database.disconnect",
  title: "Disconnect from database",
  description: "Close the active database connection.",
  category: "connection",
  inputSchema: z.object({ connectionId: z.string().min(1) }),
  risk: "read",

  async execute(input) {
    const service = createConnectionService();
    await service.disconnect(input.connectionId);

    return {
      status: "success",
      effects: [
        { type: "connection.disconnected", connectionId: input.connectionId },
      ],
    };
  },
});

// ─── database.reconnect ──────────────────────────────────────

export const reconnectDatabaseAction = defineAction<
  { connectionId: string },
  void
>({
  id: "database.reconnect",
  title: "Reconnect to database",
  description: "Disconnect and re-establish the database connection.",
  category: "connection",
  inputSchema: z.object({ connectionId: z.string().min(1) }),
  risk: "read",

  async execute(input) {
    const service = createConnectionService();
    await service.disconnect(input.connectionId);
    await service.connect(input.connectionId);

    return {
      status: "success",
      effects: [
        { type: "connection.reconnected", connectionId: input.connectionId },
      ],
    };
  },
});
