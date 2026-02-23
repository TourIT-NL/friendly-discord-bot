import {
  Fingerprint,
  Database,
  ShieldAlert,
  CheckCircle2,
  Clock,
  ExternalLink,
} from "lucide-react";
import { SectionLabel } from "../../common/M3Components";

interface PrivacyModeProps {
  status: any;
  fetchAudit: () => void;
  onTriggerHarvest: () => void;
  onSanitize: () => void;
  onOpenDiscordUrl: (type: string) => void;
}

export const PrivacyMode = ({
  status,
  fetchAudit,
  onTriggerHarvest,
  onSanitize,
  onOpenDiscordUrl,
}: PrivacyModeProps) => {
  useEffect(() => {
    fetchAudit();
  }, [fetchAudit]);

  return (
    <div className="flex-1 flex flex-col gap-8 overflow-y-auto custom-scrollbar pr-2">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <h3 className="text-2xl font-black italic uppercase text-white tracking-tighter">
            Privacy Guard & Data Sovereignty
          </h3>
          <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest opacity-60">
            Enforce maximum privacy protocols and trigger GDPR data requests
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => onOpenDiscordUrl("data_privacy")}
            className="m3-button-outlined !py-2 !text-[9px] !px-4"
          >
            <ExternalLink className="w-3 h-3" /> Node Privacy Settings
          </button>
          <button
            onClick={() => onOpenDiscordUrl("gdpr_request")}
            className="m3-button-outlined !py-2 !text-[9px] !px-4"
          >
            <Database className="w-3 h-3" /> GDPR Protocol Docs
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="m3-card !bg-black/20 border-m3-outlineVariant/20 p-8 flex flex-col gap-6">
          <div className="flex items-center gap-4">
            <Database className="w-8 h-8 text-m3-primary" />
            <h4 className="text-lg font-black text-white uppercase italic">
              GDPR Data Harvest
            </h4>
          </div>
          <p className="text-xs text-m3-onSurfaceVariant leading-relaxed">
            Trigger a complete forensic collection of every byte Discord has
            stored about your identity, messages, and activity. This protocol
            bypasses the manual settings menu.
          </p>
          <div className="mt-auto pt-4 flex flex-col gap-4">
            {status ? (
              <div className="flex items-center gap-3 p-4 rounded-m3-lg bg-m3-primary/10 border border-m3-primary/20">
                <Clock className="w-5 h-5 text-m3-primary animate-pulse" />
                <div>
                  <p className="text-[10px] font-black text-white uppercase">
                    Harvest In Progress
                  </p>
                  <p className="text-[8px] text-m3-primary font-bold uppercase tracking-widest">
                    Last requested: {new Date().toLocaleDateString()}
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
            cross-server tracking, contact syncing, and member-direct-messaging
            in a single pulse.
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
