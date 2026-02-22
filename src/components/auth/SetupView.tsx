import React from "react";
import { motion } from "framer-motion";
import { ArrowLeft, RefreshCw } from "lucide-react";
import { IconButton } from "../common/M3Components";

interface SetupViewProps {
  clientId: string;
  setClientId: (id: string) => void;
  clientSecret: string;
  setClientSecret: (secret: string) => void;
  isLoading: boolean;
  onBack: () => void;
  onSubmit: (e: React.FormEvent) => void;
}

export const SetupView = ({
  clientId,
  setClientId,
  clientSecret,
  setClientSecret,
  isLoading,
  onBack,
  onSubmit,
}: SetupViewProps) => (
  <motion.div
    initial={{ opacity: 0, scale: 0.95 }}
    animate={{ opacity: 1, scale: 1 }}
    className="w-full max-w-md m3-card-elevated relative p-10 border-m3-primary/20"
  >
    <IconButton
      icon={ArrowLeft}
      onClick={onBack}
      className="absolute top-6 left-6"
    />
    <div className="text-center mb-10">
      <h3 className="text-3xl font-black italic tracking-tighter uppercase text-m3-primary">
        Engine Setup
      </h3>
      <p className="text-[10px] text-m3-onSurfaceVariant mt-2 uppercase tracking-[0.4em] font-black">
        Persistence Protocol
      </p>
    </div>
    <form onSubmit={onSubmit} className="flex flex-col gap-8">
      <div className="flex flex-col gap-2">
        <label className="text-[10px] font-black text-m3-primary uppercase tracking-widest ml-4">
          Application ID
        </label>
        <input
          type="text"
          required
          value={clientId}
          onChange={(e) => setClientId(e.target.value)}
          className="m3-input-filled shadow-inner"
          placeholder="123456789..."
        />
      </div>
      <div className="flex flex-col gap-2">
        <label className="text-[10px] font-black text-m3-primary uppercase tracking-widest ml-4">
          Client Secret
        </label>
        <input
          type="password"
          required
          value={clientSecret}
          onChange={(e) => setClientSecret(e.target.value)}
          className="m3-input-filled shadow-inner"
          placeholder="••••••••"
        />
      </div>
      <div className="p-4 bg-m3-surfaceVariant/30 rounded-m3-lg border border-m3-outlineVariant">
        <p className="text-[9px] font-bold text-m3-onSurfaceVariant leading-relaxed italic uppercase tracking-wider">
          Note: In Discord Dev Portal, you MUST add "http://127.0.0.1:58123" to
          Redirect URIs.
        </p>
      </div>
      <button
        type="submit"
        disabled={isLoading}
        className="m3-button-primary !py-5 shadow-2xl shadow-m3-primary/20"
      >
        <RefreshCw className={`w-4 h-4 ${isLoading ? "animate-spin" : ""}`} />
        Save & Connect
      </button>
    </form>
  </motion.div>
);
