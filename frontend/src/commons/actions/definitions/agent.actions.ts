import { z } from "zod";

import { defineAction } from "../registry";
import { useShellStore } from "@/commons/stores/shell.store";

import type { ActionResult } from "../types";

// ─── agent.open ──────────────────────────────────────────────

export const openAgentAction = defineAction<
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  {},
  { open: boolean }
>({
  id: "agent.open",
  title: "Open agent panel",
  description: "Open the AI agent side panel.",
  category: "agent",
  inputSchema: z.object({}),
  risk: "read",

  async execute() {
    useShellStore.getState().setAgentOpen(true);

    return {
      status: "success",
      data: { open: true },
      effects: [{ type: "agent.panel.opened" }],
    } satisfies ActionResult<{ open: boolean }>;
  },
});

// ─── agent.close ─────────────────────────────────────────────

export const closeAgentAction = defineAction<
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  {},
  { open: boolean }
>({
  id: "agent.close",
  title: "Close agent panel",
  description: "Close the AI agent side panel.",
  category: "agent",
  inputSchema: z.object({}),
  risk: "read",

  async execute() {
    useShellStore.getState().setAgentOpen(false);

    return {
      status: "success",
      data: { open: false },
      effects: [{ type: "agent.panel.closed" }],
    } satisfies ActionResult<{ open: boolean }>;
  },
});
