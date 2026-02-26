import React from "react";
import { motion } from "framer-motion";
import {
  Monitor,
  Smartphone,
  Globe,
  Shield,
  Key,
  ChevronRight,
  ShieldCheck,
  Settings,
  Lock,
} from "lucide-react";
import { IconButton, M3Card } from "../common/M3Components";
import { DiscordStatus } from "../../types/discord";

interface LoginSelectionProps {
  discordStatus: DiscordStatus | null;
  isLoading: boolean;
  onLoginRPC: () => void;
  onLoginQR: () => void;
  onLoginOAuth: () => void;
  onSwitchToSetup: () => void;
  onSwitchToToken: () => void;
  onSwitchToMaster: () => void;
}

export const LoginSelection = ({
  discordStatus,
  isLoading,
  onLoginRPC,
  onLoginQR,
  onLoginOAuth,
  onSwitchToSetup,
  onSwitchToToken,
  onSwitchToMaster,
}: LoginSelectionProps) => (
  <motion.div
    initial={{ opacity: 0, y: 20 }}
    animate={{ opacity: 1, y: 0 }}
    exit={{ opacity: 0, scale: 0.95 }}
    className="w-full max-w-4xl grid grid-cols-1 md:grid-cols-2 gap-6 p-4"
  >
    <div className="space-y-6">
      <M3Card className="flex flex-col gap-6 p-6">
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-m3-lg bg-m3-primaryContainer text-m3-onPrimaryContainer">
            <Monitor className="w-6 h-6" />
          </div>
          <div>
            <h3 className="font-bold text-lg">Local Handshake</h3>
            <p className="text-xs text-m3-onSurfaceVariant">
              Zero-config link via desktop app
            </p>
          </div>
        </div>
        <div className="flex items-center justify-between p-4 bg-m3-surfaceVariant/50 rounded-m3-lg border border-m3-outlineVariant shadow-inner">
          <div className="flex items-center gap-3">
            <div
              className={`w-2 h-2 rounded-full ${discordStatus?.is_running ? "bg-green-500 shadow-[0_0_8px_green]" : "bg-m3-outline"}`}
            />
            <span className="text-xs font-bold uppercase tracking-wider">
              Discord Process
            </span>
          </div>
          <span className="text-[10px] font-black text-m3-onSurfaceVariant tracking-widest">
            {discordStatus?.is_running ? "DETECTED" : "NOT FOUND"}
          </span>
        </div>
        <button
          onClick={onLoginRPC}
          disabled={!discordStatus?.rpc_available || isLoading}
          className="m3-button-primary w-full shadow-lg"
        >
          <ShieldCheck className="w-4 h-4" /> Instant Link
        </button>
      </M3Card>

      <M3Card
        onClick={onLoginQR}
        className="flex flex-col items-center gap-4 text-center group py-8"
      >
        <div className="p-4 rounded-full bg-m3-secondaryContainer text-m3-onSecondaryContainer group-hover:scale-110 transition-transform shadow-md">
          <Smartphone className="w-8 h-8" />
        </div>
        <div>
          <h3 className="font-bold text-lg leading-none">QR Signature</h3>
          <p className="text-xs text-m3-onSurfaceVariant mt-2 uppercase tracking-widest font-bold">
            Mobile bridge
          </p>
        </div>
      </M3Card>
    </div>

    <div className="space-y-6">
      <div className="m3-card-elevated flex flex-col gap-6 !bg-m3-primaryContainer !text-m3-onPrimaryContainer border-none relative overflow-hidden h-full shadow-2xl p-6">
        <Globe className="absolute -right-8 -bottom-8 w-40 h-40 opacity-10 pointer-events-none" />
        <div className="flex items-center gap-4 relative z-10">
          <div className="p-3 rounded-m3-lg bg-m3-onPrimaryContainer/10 shadow-inner">
            <Shield className="w-6 h-6" />
          </div>
          <div>
            <h3 className="font-bold text-lg">Official Gate</h3>
            <p className="text-xs opacity-70">Secured OAuth2 Authorization</p>
          </div>
        </div>
        <p className="text-sm leading-relaxed opacity-80 relative z-10 flex-1">
          Standard linkage using the official Discord authorization protocol.
          Requires Application ID and Secret (fixed port 58123).
        </p>
        <div className="flex gap-3 relative z-10 mt-auto">
          <IconButton
            icon={Settings}
            onClick={onSwitchToSetup}
            className="!bg-m3-onPrimaryContainer/10 hover:!bg-m3-onPrimaryContainer/20 !p-4 !rounded-2xl"
          />
          <IconButton
            icon={Lock}
            onClick={onSwitchToMaster}
            className="!bg-m3-onPrimaryContainer/10 hover:!bg-m3-onPrimaryContainer/20 !p-4 !rounded-2xl"
          />
          <button
            onClick={onLoginOAuth}
            disabled={isLoading}
            className="m3-button-primary flex-1 !bg-m3-onPrimaryContainer !text-m3-primaryContainer shadow-xl !py-4"
          >
            Start Authorize Loop
          </button>
        </div>
      </div>

      <M3Card
        onClick={onSwitchToToken}
        className="flex items-center justify-between group py-6 px-6"
      >
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-m3-lg bg-m3-errorContainer text-m3-onErrorContainer shadow-sm">
            <Key className="w-6 h-6" />
          </div>
          <div>
            <h3 className="font-bold text-lg">Bypass Mode</h3>
            <p className="text-xs text-m3-onSurfaceVariant mt-1 uppercase tracking-widest font-bold">
              Manual Injection
            </p>
          </div>
        </div>
        <ChevronRight className="w-5 h-5 text-m3-outline group-hover:translate-x-1 transition-transform" />
      </M3Card>
    </div>
  </motion.div>
);
