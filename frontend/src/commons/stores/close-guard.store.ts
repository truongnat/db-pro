import { create } from "zustand";

interface CloseGuardState {
  open: boolean;
  tabIds: string[];
  dirtyCount: number;
  openDialog: (tabIds: string[], dirtyCount: number) => void;
  closeDialog: () => void;
}

export const useCloseGuardStore = create<CloseGuardState>()((set) => ({
  open: false,
  tabIds: [],
  dirtyCount: 0,
  openDialog: (tabIds, dirtyCount) => set({ open: true, tabIds, dirtyCount }),
  closeDialog: () => set({ open: false, tabIds: [], dirtyCount: 0 }),
}));
