import React, { useEffect } from "react";
import {
  Fingerprint,
  Database,
  ShieldAlert,
  CheckCircle2,
  Clock,
  ExternalLink,
  Globe,
  Upload,
} from "lucide-react";
import { SectionLabel } from "../../common/M3Components";

interface PrivacyModeProps {
  status: any;
  fetchAudit: () => void;
  onTriggerHarvest: () => void;
  onSanitize: () => void;
  onOpenDiscordUrl: (type: string) => void;
  onProcessGdprData: () => void;
  onSetProxy: (url: string | null) => void;
}

export const PrivacyMode = ({
  status,
  fetchAudit,
  onTriggerHarvest,
  onSanitize,
  onOpenDiscordUrl,
  onProcessGdprData,
  onSetProxy,
}: PrivacyModeProps) => {
  const [proxyInput, setProxyInput] = React.useState("");

  useEffect(() => {
    fetchAudit();
  }, [fetchAudit]);

  return (
    <div className="flex-1 flex flex-col gap-8 overflow-y-auto custom-scrollbar pr-2">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <h3 className="text-2xl font-black italic uppercase text-white tracking-tighter">
            Privacy Guard & Network Stealth
          </h3>
          <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest opacity-60">
            Enforce maximum privacy protocols and route traffic through secure
            nodes
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => onOpenDiscordUrl("data_privacy")}
            className="m3-button-outlined !py-2 !text-[9px] !px-4"
          >
            <ExternalLink className="w-3 h-3" /> Node Privacy Settings
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="m3-card !bg-black/20 border-m3-outlineVariant/20 p-8 flex flex-col gap-6">
          <div className="flex items-center gap-4">
            <Globe className="w-8 h-8 text-m3-tertiary" />
            <h4 className="text-lg font-black text-white uppercase italic">
              Secure Proxy Tunnel
            </h4>
          </div>
          <p className="text-xs text-m3-onSurfaceVariant leading-relaxed">
            Route all Discord API traffic through a SOCKS5 or Tor proxy. This
            masks your IP address from Discord's telemetry nodes.
          </p>
          <div className="mt-auto space-y-3">
            <input
              type="text"
              placeholder="socks5://127.0.0.1:9050"
              value={proxyInput}
              onChange={(e) => setProxyInput(e.target.value)}
              className="m3-input w-full text-[10px]"
            />
            <div className="flex gap-2">
              <button
                onClick={() => onSetProxy(proxyInput)}
                className="m3-button-primary flex-1 !py-3 text-[9px]"
              >
                Apply Proxy
              </button>
              <button
                onClick={() => {
                  setProxyInput("");
                  onSetProxy(null);
                }}
                className="m3-button-outlined !py-3 text-[9px]"
              >
                Disable
              </button>
            </div>
          </div>
        </div>

        <div className="m3-card !bg-black/20 border-m3-outlineVariant/20 p-8 flex flex-col gap-6">
          <div className="flex items-center gap-4">
            <Upload className="w-8 h-8 text-m3-secondary" />
            <h4 className="text-lg font-black text-white uppercase italic">
              GDPR Data Analysis
            </h4>
          </div>
          <p className="text-xs text-m3-onSurfaceVariant leading-relaxed">
            Upload your Discord <code>data.zip</code> package to discover hidden
            channels, deleted servers, and forgotten threads for deeper purging.
          </p>
          <button
            onClick={onProcessGdprData}
            className="m3-button-secondary w-full mt-auto !py-4 text-[10px]"
          >
            Load Data Package
          </button>
        </div>

        <div className="m3-card !bg-black/20 border-m3-outlineVariant/20 p-8 flex flex-col gap-6 border-l-4 border-l-m3-primary">
          <div className="flex items-center gap-4">
            <Database className="w-8 h-8 text-m3-primary" />
            <h4 className="text-lg font-black text-white uppercase italic">
              GDPR Data Harvest
            </h4>
          </div>
          <p className="text-xs text-m3-onSurfaceVariant leading-relaxed">
            Trigger a complete forensic collection of every byte Discord has
            stored about your identity, messages, and activity.
          </p>
          <div className="mt-auto pt-4 flex flex-col gap-4">
            {status ? (
              <div className="flex items-center gap-3 p-4 rounded-m3-lg bg-m3-primary/10 border border-m3-primary/20">
                <Clock className="w-5 h-5 text-m3-primary animate-pulse" />
                <div>
                  <p className="text-[10px] font-black text-white uppercase">
                    Harvest In Progress
                  </p>
                </div>
              </div>
            ) : (
              <button
                onClick={onTriggerHarvest}
                className="m3-button-primary w-full !py-4 text-[10px]"
              >
                Trigger New Harvest Protocol
              </button>
            )}
          </div>
        </div>

        <div className="m3-card !bg-black/20 border-m3-outlineVariant/20 p-8 flex flex-col gap-6 border-l-4 border-l-m3-error">
          <div className="flex items-center gap-4">
            <ShieldAlert className="w-8 h-8 text-m3-error" />
            <h4 className="text-lg font-black text-white uppercase italic">
              Forensic Sanitizer
            </h4>
          </div>
          <p className="text-xs text-m3-onSurfaceVariant leading-relaxed">
            Apply binary Protobuf hardening to your profile. This disables
            cross-server tracking, contact syncing, and member-direct-messaging.
          </p>
          <button
            onClick={onSanitize}
            className="m3-button-outlined !border-m3-error !text-m3-error w-full mt-auto !py-4 text-[10px]"
          >
            Apply Max-Privacy Hardening
          </button>
        </div>
      </div>
    </div>
  );
};
