import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Terminal, X, Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { useAuthStore } from "../../store/authStore";

export const DeveloperLog = () => {
  const { logs, clearLogs, showDevLog, toggleDevLog } = useAuthStore();
  const [expandedLog, setExpandedLog] = React.useState<number | null>(null);

  if (!showDevLog) return null;

  return (
    <motion.div
      initial={{ y: 300, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ y: 300, opacity: 0 }}
      className="fixed bottom-0 left-0 right-0 z-[1000] bg-black/90 backdrop-blur-2xl border-t border-m3-outlineVariant/30 h-[300px] flex flex-col font-mono"
    >
      <div className="flex items-center justify-between px-6 py-3 bg-m3-surfaceVariant/20 border-b border-m3-outlineVariant/20">
        <div className="flex items-center gap-3">
          <Terminal className="w-4 h-4 text-m3-primary" />
          <span className="text-[10px] font-black uppercase tracking-widest text-m3-onSurface">
            System Protocol Log
          </span>
          <span className="px-2 py-0.5 rounded-full bg-m3-primary/10 text-m3-primary text-[9px] font-bold">
            {logs.length} Entries
          </span>
        </div>
        <div className="flex items-center gap-4">
          <button
            onClick={clearLogs}
            className="p-2 hover:bg-white/5 rounded-full text-m3-onSurfaceVariant transition-colors"
          >
            <Trash2 className="w-4 h-4" />
          </button>
          <button
            onClick={toggleDevLog}
            className="p-2 hover:bg-white/5 rounded-full text-m3-onSurfaceVariant transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto custom-scrollbar p-4 space-y-1">
        {logs.map((log, i) => {
          return (
            <div
              key={`${log.timestamp}-${i}`} // eslint-disable-line @eslint-react/no-array-index-key
              className="group flex flex-col gap-1 text-[11px] leading-relaxed border-b border-white/5 pb-1 last:border-none"
            >
              <div className="flex items-start gap-4">
                <span className="text-white/30 whitespace-nowrap">
                  [{log.timestamp}]
                </span>
                <span
                  className={`font-black uppercase w-12 text-center rounded-[2px] ${
                    log.level === "error"
                      ? "bg-m3-error text-m3-onError"
                      : log.level === "warn"
                        ? "bg-m3-tertiaryContainer text-m3-onTertiaryContainer"
                        : log.level === "debug"
                          ? "bg-m3-secondaryContainer text-m3-onSecondaryContainer"
                          : "bg-white/10 text-white/60"
                  }`}
                >
                  {log.level}
                </span>
                <span
                  className={`flex-1 ${log.level === "error" ? "text-m3-error" : log.level === "warn" ? "text-m3-tertiary" : "text-white/80"}`}
                >
                  {log.message}
                </span>
                {log.metadata && (
                  <button
                    onClick={() => setExpandedLog(expandedLog === i ? null : i)}
                    className="p-1 opacity-0 group-hover:opacity-100 transition-opacity text-white/40"
                  >
                    {expandedLog === i ? (
                      <ChevronUp className="w-3 h-3" />
                    ) : (
                      <ChevronDown className="w-3 h-3" />
                    )}
                  </button>
                )}
              </div>
              <AnimatePresence>
                {expandedLog === i && log.metadata && (
                  <motion.pre
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    className="overflow-hidden bg-white/5 p-3 rounded-m3-sm text-[10px] text-white/50 overflow-x-auto"
                  >
                    {JSON.stringify(log.metadata, null, 2)}
                  </motion.pre>
                )}
              </AnimatePresence>
            </div>
          );
        })}
        {logs.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center gap-4 opacity-20 py-10">
            <Terminal className="w-12 h-12" />
            <p className="text-[10px] font-black uppercase tracking-widest">
              Listening for System Events...
            </p>
          </div>
        )}
      </div>
    </motion.div>
  );
};
