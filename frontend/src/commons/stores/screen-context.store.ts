import { create } from "zustand";

interface ScreenContextState {
  currentPgId: string | null;
  currentFormId: string | null;
  breadcrumbs: string[];

  setPageContext: (pgId: string, formId?: string) => void;
  clearContext: () => void;
}

export const useScreenContextStore = create<ScreenContextState>()((set) => ({
  currentPgId: null,
  currentFormId: null,
  breadcrumbs: [],

  setPageContext: (pgId, formId) =>
    set((state) => ({
      currentPgId: pgId,
      currentFormId: formId ?? null,
      breadcrumbs: [...state.breadcrumbs, pgId].slice(-10),
    })),

  clearContext: () =>
    set({
      currentPgId: null,
      currentFormId: null,
    }),
}));
