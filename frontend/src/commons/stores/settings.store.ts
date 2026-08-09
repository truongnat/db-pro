import { create } from "zustand";
import { persist } from "zustand/middleware";

import i18n from "@/commons/locales/i18n";

export type DataEditMode = "inline" | "row-dialog";

interface SettingsState {
  language: "en" | "ja";
  defaultConnectionId: string | null;
  pageSize: 25 | 50 | 100 | 200;
  autoCommit: boolean;
  dataEditMode: DataEditMode;

  setLanguage: (language: "en" | "ja") => void;
  setDefaultConnection: (id: string | null) => void;
  setPageSize: (size: 25 | 50 | 100 | 200) => void;
  setAutoCommit: (value: boolean) => void;
  setDataEditMode: (mode: DataEditMode) => void;
  reset: () => void;
}

const initialState = {
  language: "en" as const,
  defaultConnectionId: null,
  pageSize: 50 as const,
  autoCommit: true,
  dataEditMode: "inline" as DataEditMode,
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      ...initialState,

      setLanguage: (language) => {
        i18n.changeLanguage(language);
        set({ language });
      },

      setDefaultConnection: (id) => set({ defaultConnectionId: id }),

      setPageSize: (size) => set({ pageSize: size }),

      setAutoCommit: (value) => set({ autoCommit: value }),

      setDataEditMode: (mode) => set({ dataEditMode: mode }),

      reset: () => set(initialState),
    }),
    {
      name: "db-pro-settings",
      onRehydrateStorage: () => (state) => {
        if (state?.language) {
          i18n.changeLanguage(state.language);
        }
      },
    },
  ),
);
