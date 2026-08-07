import { create } from "zustand";
import { persist } from "zustand/middleware";

/**
 * Frontend-only metadata for saved queries.
 * Tags and favorites are stored locally so we don't need
 * backend schema migrations for these lightweight features.
 */
interface SavedQueryMeta {
  tags: string[];
  favorite: boolean;
}

interface SavedQueryMetaState {
  meta: Record<string, SavedQueryMeta>;

  /** Toggle favorite status for a saved query. */
  toggleFavorite: (id: string) => void;
  /** Set tags for a saved query. */
  setTags: (id: string, tags: string[]) => void;
  /** Add a tag to a saved query. */
  addTag: (id: string, tag: string) => void;
  /** Remove a tag from a saved query. */
  removeTag: (id: string, tag: string) => void;
  /** Get meta for a saved query (returns defaults if not set). */
  getMeta: (id: string) => SavedQueryMeta;
  /** Check if a saved query is a favorite. */
  isFavorite: (id: string) => boolean;
  /** Get all unique tags across all saved queries. */
  getAllTags: () => string[];
}

function defaultMeta(): SavedQueryMeta {
  return { tags: [], favorite: false };
}

export const useSavedQueryMetaStore = create<SavedQueryMetaState>()(
  persist(
    (set, get) => ({
      meta: {},

      toggleFavorite: (id) =>
        set((state) => {
          const current = state.meta[id] ?? defaultMeta();
          return {
            meta: { ...state.meta, [id]: { ...current, favorite: !current.favorite } },
          };
        }),

      setTags: (id, tags) =>
        set((state) => {
          const current = state.meta[id] ?? defaultMeta();
          return {
            meta: { ...state.meta, [id]: { ...current, tags } },
          };
        }),

      addTag: (id, tag) =>
        set((state) => {
          const current = state.meta[id] ?? defaultMeta();
          if (current.tags.includes(tag)) return state;
          return {
            meta: { ...state.meta, [id]: { ...current, tags: [...current.tags, tag] } },
          };
        }),

      removeTag: (id, tag) =>
        set((state) => {
          const current = state.meta[id] ?? defaultMeta();
          return {
            meta: {
              ...state.meta,
              [id]: { ...current, tags: current.tags.filter((t) => t !== tag) },
            },
          };
        }),

      getMeta: (id) => {
        return get().meta[id] ?? defaultMeta();
      },

      isFavorite: (id) => {
        return get().meta[id]?.favorite ?? false;
      },

      getAllTags: () => {
        const tagSet = new Set<string>();
        for (const m of Object.values(get().meta)) {
          for (const tag of m.tags) {
            tagSet.add(tag);
          }
        }
        return Array.from(tagSet).sort();
      },
    }),
    {
      name: "db-pro-saved-query-meta",
    },
  ),
);
