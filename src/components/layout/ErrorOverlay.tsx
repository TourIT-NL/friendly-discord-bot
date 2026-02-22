import { motion, AnimatePresence } from "framer-motion";
import { ShieldAlert, XCircle } from "lucide-react";
import { IconButton } from "../common/M3Components";

interface ErrorOverlayProps {
  error: string | null;
  setError: (error: string | null) => void;
  showDevLog: boolean;
  toggleDevLog: () => void;
}

export const ErrorOverlay = ({
  error,
  setError,
  showDevLog,
  toggleDevLog,
}: ErrorOverlayProps) => (
  <AnimatePresence>
    {error && (
      <motion.div
        initial={{ opacity: 0, y: 100 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 50 }}
        className="fixed bottom-12 left-1/2 -translate-x-1/2 z-[600] w-full max-w-xl px-10"
      >
        <div className="m3-card-elevated !bg-m3-errorContainer !text-m3-onErrorContainer !border-none flex items-center gap-6 shadow-[0_30px_100px_rgba(0,0,0,0.8)] p-8 rounded-[2.5rem] relative overflow-hidden group">
          <div className="absolute top-0 left-0 w-full h-1 bg-m3-onErrorContainer/20 group-hover:h-2 transition-all" />
          <div className="p-3 bg-m3-onErrorContainer/10 rounded-full shadow-inner">
            <ShieldAlert className="w-8 h-8 text-m3-onErrorContainer drop-shadow-[0_0_10px_rgba(96,20,16,0.4)]" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-[10px] font-black uppercase tracking-[0.4em] leading-none mb-2 italic">
              System Alert
            </p>
            <p className="text-xs font-bold opacity-90 leading-relaxed uppercase tracking-tight">
              {error}
            </p>
            <button
              onClick={() => {
                if (!showDevLog) toggleDevLog();
              }}
              className="mt-3 text-[9px] font-black uppercase tracking-widest bg-m3-onErrorContainer/10 hover:bg-m3-onErrorContainer/20 px-3 py-1.5 rounded-full transition-colors border border-m3-onErrorContainer/20"
            >
              View Protocol Log
            </button>
          </div>
          <IconButton
            icon={XCircle}
            onClick={() => setError(null)}
            className="!text-m3-onErrorContainer hover:!bg-m3-onErrorContainer/10 !p-4 !rounded-[1.5rem]"
          />
        </div>
      </motion.div>
    )}
  </AnimatePresence>
);
