import { useState, useEffect, useRef } from "react";
import { 
  ArrowUpCircle, 
  RotateCw, 
  CheckCircle2, 
  AlertCircle, 
  X, 
  Terminal, 
  Download,
  Sparkles,
  FileText
} from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { api } from "@/lib/api";
import { useAppStore } from "@/store/app";
import type { UpdateLogLine } from "@/types";

type UpdateState = "ready" | "updating" | "completed" | "error";

export function UpdateModal() {
  const { isUpdateModalOpen, setIsUpdateModalOpen, updateInfo, setUpdateInfo } = useAppStore();
  const [status, setStatus] = useState<UpdateState>("ready");
  const [logs, setLogs] = useState<UpdateLogLine[]>([]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [countdown, setCountdown] = useState<number>(3);
  const terminalEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (terminalEndRef.current) {
      terminalEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs]);

  // Handle ESC key (only if not actively updating)
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === "Escape" && status !== "updating") {
        setIsUpdateModalOpen(false);
      }
    };
    window.addEventListener("keydown", handleEsc);
    return () => window.removeEventListener("keydown", handleEsc);
  }, [status, setIsUpdateModalOpen]);

  // Listen to update logs from Tauri backend
  useEffect(() => {
    if (!isUpdateModalOpen) return;

    let unlisten: (() => void) | undefined;
    listen<UpdateLogLine>("update-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [isUpdateModalOpen]);

  // Countdown timer on completed
  useEffect(() => {
    if (status !== "completed") return;

    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(timer);
          api.restartApp().catch(console.error);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [status]);

  if (!isUpdateModalOpen) return null;

  const handleStartUpdate = async () => {
    setStatus("updating");
    setErrorMessage(null);
    setLogs([
      { line: `Starting Sysora update to v${updateInfo?.latest_version ?? "latest"}...`, level: "info" }
    ]);

    try {
      await api.runAppUpdate(updateInfo?.download_url);
      setStatus("completed");
      setCountdown(3);
      // Re-fetch update info to refresh state
      api.checkForUpdates().then(setUpdateInfo).catch(() => {});
    } catch (err: unknown) {
      console.error("Update failed:", err);
      const msg = typeof err === "string" ? err : err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
      setStatus("error");
      setLogs((prev) => [
        ...prev,
        { line: `❌ ERROR: ${msg}`, level: "error" }
      ]);
    }
  };

  const handleRestartNow = () => {
    api.restartApp().catch(console.error);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200"
      onClick={() => status !== "updating" && setIsUpdateModalOpen(false)}
    >
      <div
        className="w-full max-w-xl bg-white dark:bg-surface-800 border border-surface-200 dark:border-white/10 rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 flex flex-col max-h-[85vh]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-5 border-b border-surface-200 dark:border-white/5 bg-surface-50/50 dark:bg-surface-900/50">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-brand-600/20 flex items-center justify-center text-brand-600 dark:text-brand-400 border border-brand-400/20">
              <ArrowUpCircle size={22} />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-surface-900 dark:text-white flex items-center gap-2">
                Sysora Software Update
                {updateInfo?.latest_version && (
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-brand-600/10 text-brand-600 dark:text-brand-400 border border-brand-600/20">
                    v{updateInfo.latest_version}
                  </span>
                )}
              </h2>
              <p className="text-xs text-surface-400 dark:text-white/40">
                {status === "ready" && "A new release is available on GitHub"}
                {status === "updating" && "Downloading and installing update..."}
                {status === "completed" && "Update installed! Restarting application..."}
                {status === "error" && "An error occurred during update"}
              </p>
            </div>
          </div>
          {status !== "updating" && (
            <button
              onClick={() => setIsUpdateModalOpen(false)}
              className="p-1.5 rounded-lg hover:bg-surface-100 dark:hover:bg-white/5 text-surface-400 hover:text-surface-900 dark:hover:text-white transition-colors"
            >
              <X size={18} />
            </button>
          )}
        </div>

        {/* Body */}
        <div className="p-6 space-y-5 overflow-y-auto flex-1">
          {/* Version Comparison Card */}
          <div className="grid grid-cols-2 gap-3">
            <div className="p-3.5 rounded-xl bg-surface-50 dark:bg-white/[0.02] border border-surface-200 dark:border-white/5">
              <span className="text-[10px] uppercase font-bold text-surface-400 dark:text-white/30 tracking-wider">
                Current Version
              </span>
              <p className="text-sm font-mono font-semibold text-surface-800 dark:text-white/80 mt-1">
                v{updateInfo?.current_version ?? "0.3.0"}
              </p>
            </div>
            <div className="p-3.5 rounded-xl bg-brand-600/5 dark:bg-brand-500/10 border border-brand-600/20">
              <span className="text-[10px] uppercase font-bold text-brand-600 dark:text-brand-400 tracking-wider flex items-center gap-1">
                <Sparkles size={11} /> New Version
              </span>
              <p className="text-sm font-mono font-bold text-brand-600 dark:text-brand-400 mt-1">
                v{updateInfo?.latest_version ?? "Latest"}
              </p>
            </div>
          </div>

          {/* Release Notes */}
          {status === "ready" && updateInfo?.release_notes && (
            <div className="space-y-2">
              <span className="text-[10px] uppercase font-bold text-surface-400 dark:text-white/40 tracking-wider flex items-center gap-1.5">
                <FileText size={12} /> Release Highlights
              </span>
              <div
                className="p-3.5 rounded-xl bg-surface-50 dark:bg-white/[0.02] border border-surface-200 dark:border-white/5 text-xs text-surface-700 dark:text-surface-300 whitespace-pre-wrap max-h-40 overflow-y-auto leading-relaxed font-sans"
                style={{ scrollbarWidth: "thin" }}
              >
                {updateInfo.release_notes}
              </div>
            </div>
          )}

          {/* Status Banners */}
          {status === "completed" && (
            <div className="flex items-center gap-3 p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-emerald-600 dark:text-emerald-400 text-xs">
              <CheckCircle2 size={18} className="shrink-0" />
              <div>
                <p className="font-semibold">Update successfully installed!</p>
                <p className="text-[11px] opacity-80 mt-0.5">
                  Sysora will automatically restart in <span className="font-bold">{countdown}s</span>...
                </p>
              </div>
            </div>
          )}

          {status === "error" && errorMessage && (
            <div className="flex items-start gap-3 p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-500 dark:text-red-400 text-xs">
              <AlertCircle size={18} className="shrink-0 mt-0.5" />
              <div>
                <p className="font-semibold">Update encountered an issue</p>
                <p className="text-[11px] font-mono mt-1 break-all opacity-80">{errorMessage}</p>
              </div>
            </div>
          )}

          {/* Terminal / Live Log Area */}
          {(status === "updating" || logs.length > 0) && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-[11px] text-surface-400 dark:text-white/40">
                <span className="flex items-center gap-1.5 font-medium">
                  <Terminal size={12} /> Installation Log
                </span>
                {status === "updating" && (
                  <span className="flex items-center gap-1.5 text-brand-600 dark:text-brand-400 font-mono text-[10px]">
                    <RotateCw size={11} className="animate-spin" /> In progress...
                  </span>
                )}
              </div>
              <div
                className="bg-surface-950 dark:bg-black/80 text-surface-100 p-3.5 rounded-xl border border-surface-800 dark:border-white/10 font-mono text-[11px] leading-relaxed max-h-48 overflow-y-auto space-y-1"
                style={{ scrollbarWidth: "thin" }}
              >
                {logs.map((log, index) => {
                  let colorClass = "text-surface-300 dark:text-white/70";
                  if (log.level === "error") colorClass = "text-red-400 font-semibold";
                  if (log.level === "success") colorClass = "text-emerald-400 font-semibold";
                  if (log.level === "warn") colorClass = "text-amber-400";

                  return (
                    <div key={index} className={`break-words ${colorClass}`}>
                      {log.line}
                    </div>
                  );
                })}
                <div ref={terminalEndRef} />
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-surface-200 dark:border-white/5 bg-surface-50/50 dark:bg-surface-900/50 flex items-center justify-end gap-3">
          {status === "ready" && (
            <>
              <button
                onClick={() => setIsUpdateModalOpen(false)}
                className="px-4 py-2 text-xs font-semibold text-surface-600 dark:text-white/60 hover:text-surface-900 dark:hover:text-white transition-colors"
              >
                Later
              </button>
              <button
                onClick={handleStartUpdate}
                className="flex items-center gap-2 px-5 py-2 text-xs font-semibold bg-brand-600 hover:bg-brand-500 text-white rounded-xl shadow-lg transition-all"
              >
                <Download size={14} />
                Download & Install Update
              </button>
            </>
          )}

          {status === "updating" && (
            <div className="flex items-center gap-2 text-xs text-brand-600 dark:text-brand-400 font-medium py-1">
              <RotateCw size={14} className="animate-spin" />
              <span>Applying update package...</span>
            </div>
          )}

          {status === "completed" && (
            <button
              onClick={handleRestartNow}
              className="flex items-center gap-2 px-5 py-2 text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl shadow-lg transition-all"
            >
              <RotateCw size={14} />
              Restart Now ({countdown}s)
            </button>
          )}

          {status === "error" && (
            <>
              <button
                onClick={() => setIsUpdateModalOpen(false)}
                className="px-4 py-2 text-xs font-semibold text-surface-600 dark:text-white/60 hover:text-surface-900 dark:hover:text-white transition-colors"
              >
                Close
              </button>
              <button
                onClick={handleStartUpdate}
                className="flex items-center gap-2 px-5 py-2 text-xs font-semibold bg-brand-600 hover:bg-brand-500 text-white rounded-xl shadow-lg transition-all"
              >
                <RotateCw size={14} />
                Retry Update
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
