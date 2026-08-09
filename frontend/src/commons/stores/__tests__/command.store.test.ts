import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useCommandStore } from "@/commons/stores/command.store";
import type { Command } from "@/commons/types/command.types";

function resetStore() {
  useCommandStore.setState({
    commands: [],
    isOpen: false,
  });
}

function createCommand(overrides: Partial<Command> = {}): Command {
  return {
    id: "test.command",
    labelKey: "test.command",
    execute: vi.fn(),
    ...overrides,
  };
}

describe("CommandStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("register", () => {
    it("adds a command to the list", () => {
      const cmd = createCommand({ id: "cmd-1" });
      useCommandStore.getState().register(cmd);

      const state = useCommandStore.getState();
      expect(state.commands).toHaveLength(1);
      expect(state.commands[0].id).toBe("cmd-1");
    });

    it("replaces existing command with same id", () => {
      const cmd1 = createCommand({ id: "cmd-1", labelKey: "test.first" });
      const cmd2 = createCommand({ id: "cmd-1", labelKey: "test.second" });

      useCommandStore.getState().register(cmd1);
      useCommandStore.getState().register(cmd2);

      const state = useCommandStore.getState();
      expect(state.commands).toHaveLength(1);
      expect(state.commands[0].labelKey).toBe("test.second");
    });
  });

  describe("registerMany", () => {
    it("adds multiple commands", () => {
      const cmds = [
        createCommand({ id: "cmd-1" }),
        createCommand({ id: "cmd-2" }),
        createCommand({ id: "cmd-3" }),
      ];

      useCommandStore.getState().registerMany(cmds);

      const state = useCommandStore.getState();
      expect(state.commands).toHaveLength(3);
    });

    it("replaces existing commands with same id", () => {
      const cmd1 = createCommand({ id: "cmd-1", labelKey: "test.first" });
      useCommandStore.getState().register(cmd1);

      const cmds = [
        createCommand({ id: "cmd-1", labelKey: "test.updated" }),
        createCommand({ id: "cmd-2" }),
      ];
      useCommandStore.getState().registerMany(cmds);

      const state = useCommandStore.getState();
      expect(state.commands).toHaveLength(2);
      expect(state.commands[0].labelKey).toBe("test.updated");
    });
  });

  describe("unregister", () => {
    it("removes a command by id", () => {
      const cmd1 = createCommand({ id: "cmd-1" });
      const cmd2 = createCommand({ id: "cmd-2" });
      useCommandStore.getState().registerMany([cmd1, cmd2]);

      useCommandStore.getState().unregister("cmd-1");

      const state = useCommandStore.getState();
      expect(state.commands).toHaveLength(1);
      expect(state.commands[0].id).toBe("cmd-2");
    });

    it("does nothing if id not found", () => {
      const cmd = createCommand({ id: "cmd-1" });
      useCommandStore.getState().register(cmd);

      useCommandStore.getState().unregister("nonexistent");

      const state = useCommandStore.getState();
      expect(state.commands).toHaveLength(1);
    });
  });

  describe("getCommand", () => {
    it("returns the correct command", () => {
      const cmd1 = createCommand({ id: "cmd-1", labelKey: "test.first" });
      const cmd2 = createCommand({ id: "cmd-2", labelKey: "test.second" });
      useCommandStore.getState().registerMany([cmd1, cmd2]);

      const result = useCommandStore.getState().getCommand("cmd-2");
      expect(result?.labelKey).toBe("test.second");
    });

    it("returns undefined if not found", () => {
      const result = useCommandStore.getState().getCommand("nonexistent");
      expect(result).toBeUndefined();
    });
  });

  describe("getAvailableCommands", () => {
    it("returns all commands without when predicate", () => {
      const cmds = [createCommand({ id: "cmd-1" }), createCommand({ id: "cmd-2" })];
      useCommandStore.getState().registerMany(cmds);

      const available = useCommandStore.getState().getAvailableCommands();
      expect(available).toHaveLength(2);
    });

    it("filters out commands where when() returns false", () => {
      const cmds = [
        createCommand({ id: "cmd-1", when: () => true }),
        createCommand({ id: "cmd-2", when: () => false }),
        createCommand({ id: "cmd-3" }),
      ];
      useCommandStore.getState().registerMany(cmds);

      const available = useCommandStore.getState().getAvailableCommands();
      expect(available).toHaveLength(2);
      expect(available.map((c) => c.id)).toEqual(["cmd-1", "cmd-3"]);
    });
  });

  describe("executeCommand", () => {
    it("calls the command's execute function", () => {
      const execute = vi.fn();
      const cmd = createCommand({ id: "cmd-1", execute });
      useCommandStore.getState().register(cmd);

      useCommandStore.getState().executeCommand("cmd-1");

      expect(execute).toHaveBeenCalledOnce();
    });

    it("does nothing if command not found", () => {
      expect(() => {
        useCommandStore.getState().executeCommand("nonexistent");
      }).not.toThrow();
    });

    it("does not execute if when() returns false", () => {
      const execute = vi.fn();
      const cmd = createCommand({ id: "cmd-1", execute, when: () => false });
      useCommandStore.getState().register(cmd);

      useCommandStore.getState().executeCommand("cmd-1");

      expect(execute).not.toHaveBeenCalled();
    });

    it("executes if when() returns true", () => {
      const execute = vi.fn();
      const cmd = createCommand({ id: "cmd-1", execute, when: () => true });
      useCommandStore.getState().register(cmd);

      useCommandStore.getState().executeCommand("cmd-1");

      expect(execute).toHaveBeenCalledOnce();
    });
  });

  describe("open/close/toggle", () => {
    it("open sets isOpen to true", () => {
      useCommandStore.getState().open();
      expect(useCommandStore.getState().isOpen).toBe(true);
    });

    it("close sets isOpen to false", () => {
      useCommandStore.getState().open();
      useCommandStore.getState().close();
      expect(useCommandStore.getState().isOpen).toBe(false);
    });

    it("toggle flips isOpen", () => {
      expect(useCommandStore.getState().isOpen).toBe(false);
      useCommandStore.getState().toggle();
      expect(useCommandStore.getState().isOpen).toBe(true);
      useCommandStore.getState().toggle();
      expect(useCommandStore.getState().isOpen).toBe(false);
    });
  });
});
