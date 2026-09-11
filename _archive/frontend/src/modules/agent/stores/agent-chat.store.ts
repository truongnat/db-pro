import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { AgentConfig, AgentMessage } from "../types/agent.types";

interface AgentChatState {
  messages: AgentMessage[];
  config: AgentConfig;
  isProcessing: boolean;
  addMessage: (msg: AgentMessage) => void;
  clearMessages: () => void;
  setProcessing: (v: boolean) => void;
  setConfig: (config: Partial<AgentConfig>) => void;
}

export const useAgentChatStore = create<AgentChatState>()(
  persist(
    (set) => ({
      messages: [],
      config: {
        apiEndpoint: "https://api.openai.com/v1/chat/completions",
        apiKey: "",
        model: "gpt-4o-mini",
      },
      isProcessing: false,

      addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),

      clearMessages: () => set({ messages: [] }),

      setProcessing: (v) => set({ isProcessing: v }),

      setConfig: (partial) => set((s) => ({ config: { ...s.config, ...partial } })),
    }),
    {
      name: "db-pro-agent-chat",
      partialize: (s) => ({ config: s.config }),
    },
  ),
);
