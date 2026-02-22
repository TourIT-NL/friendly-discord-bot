import React from "react";
import { motion } from "framer-motion";
import { ArrowLeft, RefreshCw, CheckCircle2 } from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import { IconButton } from "../common/M3Components";

interface QRViewProps {
  qrUrl: string | null;
  qrScanned: boolean;
  onBack: () => void;
}

export const QRView = ({ qrUrl, qrScanned, onBack }: QRViewProps) => (
  <motion.div
    initial={{ opacity: 0, scale: 0.95 }}
    animate={{ opacity: 1, scale: 1 }}
    className="w-full max-w-md m3-card flex flex-col items-center p-12 text-center border-m3-primary/20"
  >
    <div className="w-full flex justify-start mb-8">
      <IconButton icon={ArrowLeft} onClick={onBack} />
    </div>
    <div className="bg-white p-8 rounded-m3-xl shadow-[0_0_60px_rgba(255,255,255,0.1)] mb-10 relative">
      {qrUrl ? (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
          <QRCodeSVG value={qrUrl} size={220} level="H" includeMargin />
        </motion.div>
      ) : (
        <div className="w-[220px] h-[220px] flex flex-col items-center justify-center gap-4 bg-gray-100 rounded-2xl">
          <RefreshCw className="w-10 h-10 text-m3-primary animate-spin" />
          <p className="text-[10px] font-black text-m3-primary uppercase tracking-widest animate-pulse">
            Syncing Gateway
          </p>
        </div>
      )}
    </div>
    <h4 className="text-2xl font-black italic uppercase tracking-tight mb-2">
      Scan Signature
    </h4>
    <p className="text-xs text-m3-onSurfaceVariant px-8 leading-relaxed font-bold uppercase tracking-wide">
      {qrScanned ? (
        <span className="text-m3-primary flex items-center justify-center gap-3 animate-pulse">
          <CheckCircle2 className="w-5 h-5" /> Signal Detected. Confirm on
          device.
        </span>
      ) : (
        "Use the Discord Mobile app scanner (Settings > Scan QR Code)."
      )}
    </p>
  </motion.div>
);
