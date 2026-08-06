import { create } from "zustand";
import { persist } from "zustand/middleware";

import i18n from "@/commons/locales/i18n";

interface SettingsState {
  language: "en" | "ja";
  defaultConnectionId: string | null;
  pageSize: 25 | 50 | 100 | 200;
  autoCommit: boolean;

  setLanguage: (language: "en" | "ja") => void;
  setDefaultConnection: (id: string | null) => void;
  setPageSize: (size: 25 | 50 | 100 | 200) => void;
  setAutoCommit: (value: boolean) => void;
  reset: () => void;
}

const initialState = {
  language: "en" as const,
  defaultConnectionId: null,
  pageSize: 50 as const,
  autoCommit: true,
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

      reset: () => set(initialState),
    }),
    {
      name: "db-pro-settings",
    },
  ),
);
