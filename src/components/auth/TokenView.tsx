import React from "react";
import { motion } from "framer-motion";
import { ArrowLeft, Info } from "lucide-react";
import { IconButton } from "../common/M3Components";

interface TokenViewProps {
  manualToken: string;
  setManualToken: (token: string) => void;
  isLoading: boolean;
  onBack: () => void;
  onSubmit: (e: React.FormEvent) => void;
}

export const TokenView = ({
  manualToken,
  setManualToken,
  isLoading,
  onBack,
  onSubmit,
}: TokenViewProps) => (
  <motion.div
    initial={{ opacity: 0, scale: 0.95 }}
    animate={{ opacity: 1, scale: 1 }}
    className="w-full max-w-md m3-card relative p-12 border-m3-error/20"
  >
    <IconButton
      icon={ArrowLeft}
      onClick={onBack}
      className="absolute top-6 left-6"
    />
    <div className="text-center mb-10">
      <h3 className="text-3xl font-black italic tracking-tighter uppercase text-m3-error">
        Token Inject
      </h3>
      <p className="text-[10px] text-m3-error mt-2 uppercase tracking-[0.4em] font-black opacity-60">
        High-Level Bypass
      </p>
    </div>
    <form onSubmit={onSubmit} className="flex flex-col gap-10">
      <div className="flex flex-col gap-2">
        <label className="text-[10px] font-black text-m3-error uppercase tracking-widest ml-4">
          Auth Signature
        </label>
        <input
          type="password"
          required
          value={manualToken}
          onChange={(e) => setManualToken(e.target.value)}
          className="m3-input-filled !border-m3-error/30 !text-m3-error !bg-m3-errorContainer/5 shadow-inner"
          placeholder="NJAY..."
        />
      </div>
      <button
        type="submit"
        disabled={isLoading}
        className="m3-button-primary !bg-m3-error !text-m3-onError !py-5 shadow-2xl shadow-m3-error/20"
      >
        Establish Secure Link
      </button>
      <div className="p-5 bg-m3-errorContainer/10 rounded-m3-xl border border-m3-errorContainer/20">
        <div className="flex items-center gap-3 mb-4 text-m3-error">
          <Info className="w-4 h-4" />
          <span className="text-[10px] font-black uppercase tracking-widest">
            Extraction Protocol
          </span>
        </div>
        <ol className="text-[10px] font-bold text-m3-onSurfaceVariant space-y-2 uppercase tracking-wide leading-relaxed">
          <li>1. Open Discord in Chrome/Firefox browser.</li>
          <li>2. Press F12 or Right Click &gt; Inspect.</li>
          <li>3. Go to the 'Network' tab.</li>
          <li>4. Type '/api' in the filter box.</li>
          <li>5. Click any request (e.g., 'library' or 'science').</li>
          <li>6. Find 'Authorization' under 'Request Headers'.</li>
          <li>7. Copy the long code and paste it above.</li>
        </ol>
      </div>
    </form>
  </motion.div>
);
