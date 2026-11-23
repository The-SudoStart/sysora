import { create } from "zustand";
import type { TabId, UpdateInfo } from "@/types";

interface AppStore {
  activeTab: TabId;
  setActiveTab: (tab: TabId) => void;
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  updateInfo: UpdateInfo | null;
  setUpdateInfo: (info: UpdateInfo | null) => void;
  isUpdateModalOpen: boolean;
  setIsUpdateModalOpen: (open: boolean) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  activeTab: "memory",
  setActiveTab: (tab) => set({ activeTab: tab, searchQuery: "" }),
  searchQuery: "",
  setSearchQuery: (q) => set({ searchQuery: q }),
  updateInfo: null,
  setUpdateInfo: (info) => set({ updateInfo: info }),
  isUpdateModalOpen: false,
  setIsUpdateModalOpen: (open) => set({ isUpdateModalOpen: open }),
}));
