import { create } from "zustand";

interface QuickOpenState {
  isOpen: boolean;
  query: string;
  selectedIndex: number;

  open: () => void;
  close: () => void;
  setQuery: (query: string) => void;
  setSelectedIndex: (index: number) => void;
}

export const useQuickOpenStore = create<QuickOpenState>()((set) => ({
  isOpen: false,
  query: "",
  selectedIndex: 0,

  open: () => set({ isOpen: true }),
  close: () => set({ isOpen: false, query: "", selectedIndex: 0 }),
  setQuery: (query) => set({ query, selectedIndex: 0 }),
  setSelectedIndex: (index) => set({ selectedIndex: index }),
}));
