import React from "react";
import { motion } from "framer-motion";
import { ShieldPlus, ShieldCheck, ArrowLeft, Info } from "lucide-react";
import { IconButton, M3Card } from "../common/M3Components";

interface SetMasterPasswordViewProps {
  newPassword: string;
  setNewPassword: (val: string) => void;
  confirmPassword: string;
  setConfirmPassword: (val: string) => void;
  onSave: (e: React.FormEvent) => void;
  onBack: () => void;
  isLoading: boolean;
}

export const SetMasterPasswordView = ({
  newPassword,
  setNewPassword,
  confirmPassword,
  setConfirmPassword,
  onSave,
  onBack,
  isLoading,
}: SetMasterPasswordViewProps) => (
  <motion.div
    initial={{ opacity: 0, x: 20 }}
    animate={{ opacity: 1, x: 0 }}
    exit={{ opacity: 0, x: -20 }}
    className="w-full max-w-md p-4"
  >
    <M3Card className="p-8 flex flex-col gap-6 relative shadow-2xl">
      <div className="flex items-center gap-4">
        <IconButton
          icon={ArrowLeft}
          onClick={onBack}
          className="!bg-m3-surfaceVariant"
        />
        <h2 className="text-xl font-bold">Secure Your Vault</h2>
      </div>

      <div className="flex items-start gap-3 p-4 bg-m3-secondaryContainer/30 rounded-m3-lg text-xs text-m3-onSecondaryContainer leading-relaxed">
        <Info className="w-5 h-5 shrink-0" />
        <p>
          A Master Password provides an additional layer of security. If set,
          your Discord tokens and credentials will be encrypted using this
          password.
          <br />
          <br />
          <strong>Warning:</strong> If you lose this password, your stored data
          will be irrecoverable. Leave empty to disable (not recommended).
        </p>
      </div>

      <form onSubmit={onSave} className="space-y-4">
        <div className="space-y-1">
          <label className="text-[10px] uppercase font-bold tracking-wider text-m3-outline ml-1">
            New Password
          </label>
          <div className="relative">
            <input
              type="password"
              value={newPassword as string}
              onChange={(e) => setNewPassword(e.target.value)}
              placeholder="••••••••"
              className="m3-input w-full pr-10"
            />
            <ShieldPlus className="absolute right-3 top-1/2 -translate-y-1/2 w-5 h-5 text-m3-outline" />
          </div>
        </div>

        <div className="space-y-1">
          <label className="text-[10px] uppercase font-bold tracking-wider text-m3-outline ml-1">
            Confirm Password
          </label>
          <div className="relative">
            <input
              type="password"
              value={confirmPassword as string}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="••••••••"
              className="m3-input w-full pr-10"
            />
            <ShieldCheck className="absolute right-3 top-1/2 -translate-y-1/2 w-5 h-5 text-m3-outline" />
          </div>
        </div>

        <button
          type="submit"
          disabled={isLoading}
          className="m3-button-primary w-full py-4 mt-2 shadow-xl"
        >
          {isLoading
            ? "Securing Vault..."
            : newPassword
              ? "Enable Encryption"
              : "Disable Master Password"}
        </button>
      </form>
    </M3Card>
  </motion.div>
);
