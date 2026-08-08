import { create } from "zustand";

/**
 * Lightweight bridge store that tracks Monaco editor cursor/selection state
 * for the active query tab.
 *
 * Monaco updates this on cursor move / selection change.
 * Keyboard, Command Palette, and Action Platform read from it to resolve
 * "current statement" correctly — instead of hardcoding cursorOffset=0.
 */

export interface EditorCursorState {
  cursorOffset: number;
  selection: { start: number; end: number } | null;
}

interface QueryEditorContextState {
  /** Map of tabId → latest cursor/selection state. */
  contexts: Record<string, EditorCursorState>;

  /** Update cursor state for a given tab (called by Monaco). */
  setEditorContext: (
    tabId: string,
    state: EditorCursorState,
  ) => void;

  /** Get cursor state for a tab. Returns default if not tracked. */
  getEditorContext: (tabId: string) => EditorCursorState;

  /** Remove tracking for a tab (on tab close). */
  removeEditorContext: (tabId: string) => void;
}

export const useQueryEditorContextStore = create<QueryEditorContextState>((set, get) => ({
  contexts: {},

  setEditorContext(tabId, state) {
    set((s) => ({
      contexts: { ...s.contexts, [tabId]: state },
    }));
  },

  getEditorContext(tabId) {
    return get().contexts[tabId] ?? { cursorOffset: 0, selection: null };
  },

  removeEditorContext(tabId) {
    set((s) => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { [tabId]: _, ...rest } = s.contexts;
      return { contexts: rest };
    });
  },
}));
