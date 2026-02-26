import React from "react";
import { motion } from "framer-motion";
import { Lock, Unlock, ShieldAlert } from "lucide-react";
import { M3Card } from "../common/M3Components";

interface UnlockViewProps {
  password: string;
  setPassword: (val: string) => void;
  onUnlock: (e: React.FormEvent) => void;
  isLoading: boolean;
}

export const UnlockView = ({
  password,
  setPassword,
  onUnlock,
  isLoading,
}: UnlockViewProps) => (
  <motion.div
    initial={{ opacity: 0, scale: 0.9 }}
    animate={{ opacity: 1, scale: 1 }}
    exit={{ opacity: 0, scale: 1.1 }}
    className="w-full max-w-md p-4"
  >
    <M3Card className="p-8 flex flex-col gap-6 items-center text-center relative overflow-hidden shadow-2xl">
      <div className="absolute top-0 left-0 w-full h-1 bg-m3-error" />

      <div className="p-4 rounded-full bg-m3-errorContainer text-m3-onErrorContainer shadow-inner">
        <Lock className="w-10 h-10" />
      </div>

      <div>
        <h2 className="text-2xl font-bold tracking-tight">Vault Locked</h2>
        <p className="text-sm text-m3-onSurfaceVariant mt-2">
          Your credentials and sessions are encrypted. Please enter your Master
          Password to proceed.
        </p>
      </div>

      <form onSubmit={onUnlock} className="w-full space-y-4">
        <div className="relative">
          <input
            type="password"
            value={password as string}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Master Password"
            autoFocus
            required
            className="m3-input w-full pr-10"
          />
          <Unlock className="absolute right-3 top-1/2 -translate-y-1/2 w-5 h-5 text-m3-outline" />
        </div>

        <button
          type="submit"
          disabled={isLoading || !password}
          className="m3-button-error w-full py-4 shadow-lg"
        >
          {isLoading ? "Unlocking..." : "Unlock Security Vault"}
        </button>
      </form>

      <div className="flex items-center gap-2 text-[10px] uppercase font-bold tracking-widest text-m3-error/70 bg-m3-errorContainer/30 px-3 py-1 rounded-full">
        <ShieldAlert className="w-3 h-3" />
        Military-Grade Encryption (Argon2id)
      </div>
    </M3Card>
  </motion.div>
);
