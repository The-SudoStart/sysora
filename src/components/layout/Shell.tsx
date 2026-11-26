import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { MemoryTab }    from "@/components/tabs/MemoryTab";
import { ProcessesTab } from "@/components/tabs/ProcessesTab";
import { NetworkTab }   from "@/components/tabs/NetworkTab";
import { AppsTab }      from "@/components/tabs/AppsTab";
import { DiskTab }      from "@/components/tabs/DiskTab";
import { SystemInfoTab } from "@/components/tabs/SystemInfoTab";
import { HistoryTab } from "@/components/tabs/HistoryTab";
import { SettingsTab }  from "@/components/tabs/SettingsTab";
import { UpdateModal }  from "@/components/modals/UpdateModal";
import { useAppStore }  from "@/store/app";
import { api } from "@/lib/api";
import type { UpdateInfo } from "@/types";

export function Shell() {
  const activeTab = useAppStore((s) => s.activeTab);
  const { data: settings } = useQuery({ queryKey: ["settings"], queryFn: api.getSettings });

  useEffect(() => {
    if (!settings) return;

    const root = window.document.documentElement;
    const applyTheme = (t: string) => {
      if (t === "dark" || (t === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)) {
        root.classList.add("dark");
      } else {
        root.classList.remove("dark");
      }
    };

    applyTheme(settings.theme);

    if (settings.theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => applyTheme("system");
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [settings?.theme]);

  // Check for updates on startup & listen for background update events
  const setUpdateInfo = useAppStore((s) => s.setUpdateInfo);
  useEffect(() => {
    // Initial check
    api.checkForUpdates()
      .then((info) => {
        setUpdateInfo(info);
      })
      .catch((err) => {
        console.warn("[updater] Could not check for updates:", err);
      });

    // Background push event from Tauri backend
    let unlisten: (() => void) | undefined;
    listen<UpdateInfo>("update-available", (event) => {
      setUpdateInfo(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [setUpdateInfo]);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-surface-50 text-surface-900 dark:bg-surface-900 dark:text-white font-sans transition-colors duration-300">
      <Sidebar />
      <div className="flex flex-col flex-1 overflow-hidden">
        <TopBar />
        <main className="flex-1 overflow-y-auto p-5 animate-fade-in">
          {activeTab === "memory"      && <MemoryTab />}
          {activeTab === "processes"   && <ProcessesTab />}
          {activeTab === "network"     && <NetworkTab />}
          {activeTab === "apps"        && <AppsTab />}
          {activeTab === "disk"        && <DiskTab />}
          {activeTab === "history"     && <HistoryTab />}
          {activeTab === "system-info" && <SystemInfoTab />}
          {activeTab === "settings"    && <SettingsTab />}
        </main>
      </div>
      <UpdateModal />
    </div>
  );
}
