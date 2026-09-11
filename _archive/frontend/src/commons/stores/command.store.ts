import { create } from "zustand";

import type { Command } from "@/commons/types/command.types";

interface CommandState {
  commands: Command[];
  isOpen: boolean;

  register: (command: Command) => void;
  registerMany: (commands: Command[]) => void;
  unregister: (id: string) => void;
  getCommand: (id: string) => Command | undefined;
  getAvailableCommands: () => Command[];
  executeCommand: (id: string) => void;

  open: () => void;
  close: () => void;
  toggle: () => void;
}

export const useCommandStore = create<CommandState>()((set, get) => ({
  commands: [],
  isOpen: false,

  register: (command) => {
    set((state) => {
      const existing = state.commands.findIndex((c) => c.id === command.id);
      if (existing >= 0) {
        const updated = [...state.commands];
        updated[existing] = command;
        return { commands: updated };
      }
      return { commands: [...state.commands, command] };
    });
  },

  registerMany: (commands) => {
    set((state) => {
      const updated = [...state.commands];
      for (const cmd of commands) {
        const existing = updated.findIndex((c) => c.id === cmd.id);
        if (existing >= 0) {
          updated[existing] = cmd;
        } else {
          updated.push(cmd);
        }
      }
      return { commands: updated };
    });
  },

  unregister: (id) => {
    set((state) => ({
      commands: state.commands.filter((c) => c.id !== id),
    }));
  },

  getCommand: (id) => {
    return get().commands.find((c) => c.id === id);
  },

  getAvailableCommands: () => {
    return get().commands.filter((c) => !c.when || c.when());
  },

  executeCommand: (id) => {
    const cmd = get().commands.find((c) => c.id === id);
    if (!cmd) return;
    if (cmd.when && !cmd.when()) return;
    cmd.execute();
  },

  open: () => set({ isOpen: true }),
  close: () => set({ isOpen: false }),
  toggle: () => set((state) => ({ isOpen: !state.isOpen })),
}));
