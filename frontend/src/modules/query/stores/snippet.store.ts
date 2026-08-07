import { create } from "zustand";
import { persist } from "zustand/middleware";

import { BUILT_IN_SNIPPETS, type Snippet } from "../types/snippet.types";

/**
 * Snippet management.
 *
 * Built-in snippets are always available and merged at read time.
 * Custom snippets are persisted in localStorage.
 */
interface SnippetState {
  /** User-defined custom snippets. */
  custom: Snippet[];

  /** Add a custom snippet. */
  addSnippet: (trigger: string, label: string, body: string) => void;
  /** Remove a custom snippet by trigger. */
  removeSnippet: (trigger: string) => void;
  /** Update a custom snippet's body. */
  updateSnippet: (trigger: string, label: string, body: string) => void;
  /** Get all snippets (built-in + custom). */
  getAll: () => Snippet[];
  /** Search snippets by trigger or label. */
  search: (query: string) => Snippet[];
}

export const useSnippetStore = create<SnippetState>()(
  persist(
    (set, get) => ({
      custom: [],

      addSnippet: (trigger, label, body) =>
        set((state) => {
          const filtered = state.custom.filter((s) => s.trigger !== trigger);
          return {
            custom: [...filtered, { trigger, label, body, builtIn: false }],
          };
        }),

      removeSnippet: (trigger) =>
        set((state) => ({
          custom: state.custom.filter((s) => s.trigger !== trigger),
        })),

      updateSnippet: (trigger, label, body) =>
        set((state) => ({
          custom: state.custom.map((s) =>
            s.trigger === trigger ? { ...s, label, body } : s,
          ),
        })),

      getAll: () => {
        return [...BUILT_IN_SNIPPETS, ...get().custom];
      },

      search: (query) => {
        const q = query.toLowerCase().trim();
        const all = [...BUILT_IN_SNIPPETS, ...get().custom];
        if (!q) return all;
        return all.filter(
          (s) =>
            s.trigger.toLowerCase().includes(q) ||
            s.label.toLowerCase().includes(q),
        );
      },
    }),
    {
      name: "db-pro-snippets",
    },
  ),
);

/**
 * Expand a snippet body by replacing `$cursor` with empty string
 * (the caller is responsible for cursor positioning in the editor).
 */
export function expandSnippet(body: string): string {
  return body.replace(/\$cursor/g, "");
}
