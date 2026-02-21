import React from 'react';
import { motion } from 'framer-motion';
import { Trash2, Play, Pause, Square, ShieldCheck } from 'lucide-react';
import { Progress, OperationStatus } from '../../types/discord';

interface OperationOverlayProps {
  isLoading: boolean;
  operationStatus: OperationStatus;
  progress: Progress | null;
  mode: 'messages' | 'servers' | 'identity';
  onPause: () => void;
  onResume: () => void;
  onAbort: () => void;
}

export const OperationOverlay = ({
  isLoading,
  operationStatus,
  progress,
  mode,
  onPause,
  onResume,
  onAbort
}: OperationOverlayProps) => {
  if (!isLoading && !operationStatus.is_running) return null;

  return (
    <motion.div 
      initial={{ opacity: 0 }} 
      animate={{ opacity: 1 }} 
      exit={{ opacity: 0 }} 
      className="fixed inset-0 bg-black/95 backdrop-blur-[80px] z-[500] flex flex-col items-center justify-center p-10 text-center"
    >
      {operationStatus.is_running ? (
        <div className="w-full max-w-2xl flex flex-col items-center gap-16 px-10">
          <motion.div 
            animate={{ scale: [1, 1.15, 1], rotate: [0, 8, -8, 0] }} 
            transition={{ repeat: Infinity, duration: 3, ease: "easeInOut" }} 
            className="p-10 rounded-[4rem] bg-m3-errorContainer/10 border-2 border-m3-error/30 shadow-[0_0_50px_rgba(242,184,181,0.1)]"
          >
            <Trash2 className="w-20 h-20 text-m3-error shadow-[0_0_30px_rgba(242,184,181,0.5)]" />
          </motion.div>
          
          <div className="space-y-6 w-full px-10">
            <div className="space-y-2">
              <h2 className="text-6xl font-black italic text-white uppercase tracking-tighter leading-none">
                {mode === 'messages' ? 'Purging Nodes' : mode === 'servers' ? 'Severing Nodes' : 'Nullifying Identity'}
              </h2>
              <p className="text-[10px] text-m3-primary font-black uppercase tracking-[0.6em] animate-pulse">
                Execution Loop: Active
              </p>
            </div>

            <div className="w-full space-y-12 pt-10">
              <div className="space-y-5">
                <div className="flex justify-between text-[11px] font-black text-m3-onSurfaceVariant uppercase tracking-[0.2em] px-6 leading-none">
                  <span>Saturation Level</span>
                  <span className="text-m3-primary italic">{progress?.current} / {progress?.total}</span>
                </div>
                <div className="w-full h-4 bg-m3-surfaceVariant/50 rounded-full overflow-hidden border border-m3-outlineVariant/30 p-1 shadow-2xl">
                  <motion.div 
                    animate={{ width: `${((progress?.current || 0) / (progress?.total || 1)) * 100}%` }} 
                    className="h-full bg-gradient-to-r from-m3-primary via-m3-tertiary to-m3-error rounded-full" 
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-8 w-full px-4">
                <div className="m3-card !bg-black/40 border-m3-outlineVariant/30 flex flex-col gap-3 items-start !p-8 shadow-xl">
                  <span className="text-[10px] font-black text-m3-onSurfaceVariant uppercase tracking-widest leading-none italic">Loop Phase</span>
                  <p className="text-3xl font-black text-m3-primary italic uppercase tracking-tighter">{progress?.status}</p>
                </div>
                <div className="m3-card !bg-black/40 border-m3-outlineVariant/30 flex flex-col gap-3 items-start !p-8 shadow-xl">
                  <span className="text-[10px] font-black text-m3-onSurfaceVariant uppercase tracking-widest leading-none italic">
                    {mode === 'messages' ? 'Items Nullified' : 'Nodes Severed'}
                  </span>
                  <p className="text-3xl font-black text-m3-error italic uppercase tracking-tighter leading-none">
                    {mode === 'messages' ? progress?.deleted_count : progress?.current || 0}
                  </p>
                </div>
              </div>

              <div className="w-full flex justify-center gap-4 mt-8">
                {operationStatus.is_paused ? (
                  <button onClick={onResume} className="m3-button-primary !bg-m3-secondary !text-m3-onSecondary">
                    <Play className="w-5 h-5" /> Resume
                  </button>
                ) : (
                  <button onClick={onPause} className="m3-button-primary">
                    <Pause className="w-5 h-5" /> Pause
                  </button>
                )}
                <button onClick={onAbort} className="m3-button-outlined !border-m3-error !text-m3-error">
                  <Square className="w-5 h-5" /> Abort
                </button>
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center gap-12">
          <div className="relative">
            <div className="w-32 h-32 border-4 border-m3-primary/10 border-t-m3-primary rounded-full animate-spin" />
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="w-12 h-12 bg-m3-primary/10 rounded-full flex items-center justify-center animate-pulse border border-m3-primary/30 shadow-inner">
                <ShieldCheck className="w-6 h-6 text-m3-primary shadow-[0_0_15px_rgba(208,188,255,0.8)]" />
              </div>
            </div>
          </div>
          <div className="text-center space-y-4">
            <p className="text-xl font-black uppercase tracking-[0.8em] text-m3-primary animate-pulse italic leading-none">Synchronizing Protocol</p>
            <div className="flex items-center justify-center gap-3 opacity-40">
              <div className="w-12 h-px bg-m3-onSurface" />
              <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest">Handshake in progress</p>
              <div className="w-12 h-px bg-m3-onSurface" />
            </div>
          </div>
        </div>
      )}
    </motion.div>
  );
};
