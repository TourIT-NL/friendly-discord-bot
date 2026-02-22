import React from "react";
import { motion } from "framer-motion";
import {
  Shield,
  CheckCircle2,
  AlertCircle,
  LogOut,
  Hammer,
  Cloud,
  Hash,
} from "lucide-react";
import { M3Card, SectionLabel } from "../../common/M3Components";
import { Guild, Channel } from "../../../types/discord";

interface ServersModeProps {
  guilds: Guild[] | null;
  selectedGuildsToLeave: Set<string>;
  onToggleGuildToLeave: (id: string) => void;
  onSelectAllNodes: () => void;
  confirmText: string;
  setConfirmText: (text: string) => void;
  isProcessing: boolean;
  onStartAction: () => void;
  selectedGuilds: Set<string>;
  channelsByGuild: Map<string, Channel[]>;
  selectedChannels: Set<string>;
  onToggleChannelForAudit: (id: string) => void;
  onBuryAuditLog: () => void;
  onWebhookGhosting: () => void;
  isLoading: boolean;
}

export const ServersMode = ({
  guilds,
  selectedGuildsToLeave,
  onToggleGuildToLeave,
  onSelectAllNodes,
  confirmText,
  setConfirmText,
  isProcessing,
  onStartAction,
  selectedGuilds,
  channelsByGuild,
  selectedChannels,
  onToggleChannelForAudit,
  onBuryAuditLog,
  onWebhookGhosting,
  isLoading,
}: ServersModeProps) => {
  const firstSelectedId = Array.from(selectedGuilds)[0];
  const firstSelectedGuild =
    guilds?.find((g) => g.id === firstSelectedId) || null;
  const currentChannels = firstSelectedId
    ? channelsByGuild.get(firstSelectedId) || []
    : [];

  return (
    <M3Card className="flex flex-col gap-10 flex-1 border-m3-error/10 shadow-2xl p-10">
      <div className="flex items-center justify-between border-b border-m3-outlineVariant/30 pb-8">
        <div className="flex items-center gap-4">
          <div className="p-4 rounded-m3-lg bg-m3-errorContainer text-m3-onErrorContainer shadow-lg">
            <Shield className="w-6 h-6" />
          </div>
          <div>
            <h3 className="text-3xl font-black italic uppercase tracking-tighter text-white leading-none">
              Connection Severance
            </h3>
            <p className="text-[10px] text-m3-error font-black uppercase tracking-[0.4em] mt-3">
              Bulk Server Departure Protocol
            </p>
          </div>
        </div>
        <button
          onClick={onSelectAllNodes}
          className="m3-button-outlined !border-m3-primary/30 !text-m3-primary !px-8 !py-3 hover:!bg-m3-primary/10"
        >
          Select All Nodes
        </button>
      </div>

      <div className="m3-card !p-4 !bg-black/30 border-m3-outlineVariant/20 flex-1 overflow-y-auto custom-scrollbar min-h-[300px] shadow-inner">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {guilds?.map((g) => (
            <button
              key={g.id}
              onClick={() => onToggleGuildToLeave(g.id)}
              className={`flex items-center gap-4 p-5 rounded-m3-xl border-2 transition-all relative overflow-hidden ${selectedGuildsToLeave.has(g.id) ? "bg-m3-errorContainer/20 border-m3-error text-white shadow-md" : "bg-transparent border-m3-outlineVariant/20 text-m3-onSurfaceVariant hover:border-m3-outline hover:bg-m3-onSurface/5"}`}
            >
              <div className="relative">
                {g.icon ? (
                  <img
                    src={`https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png`}
                    className="w-10 h-10 rounded-m3-md border border-white/5"
                  />
                ) : (
                  <div className="w-10 h-10 rounded-m3-md bg-m3-secondaryContainer text-m3-onSecondaryContainer flex items-center justify-center font-black text-sm border border-white/5 uppercase">
                    {g.name[0]}
                  </div>
                )}
                {selectedGuildsToLeave.has(g.id) && (
                  <motion.div
                    layoutId="pulse-active"
                    className="absolute -inset-1 rounded-m3-lg border border-m3-primary animate-pulse"
                  />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <span className="text-[13px] font-black truncate block uppercase italic tracking-tight">
                  {g.name}
                </span>
                <p className="text-[9px] opacity-50 font-bold uppercase tracking-widest mt-1">
                  Authorized Node
                </p>
              </div>
              <div
                className={`w-5 h-5 rounded-m3-xs border-2 flex items-center justify-center transition-all ${selectedGuildsToLeave.has(g.id) ? "bg-m3-error border-m3-error scale-110" : "border-m3-outlineVariant"}`}
              >
                {selectedGuildsToLeave.has(g.id) && (
                  <CheckCircle2 className="w-3.5 h-3.5 text-m3-onError" />
                )}
              </div>
            </button>
          ))}
        </div>
      </div>

      <div className="mt-auto flex flex-col lg:flex-row gap-8 items-center pt-8 border-t border-m3-outlineVariant/30 px-4">
        <div className="flex-1 flex items-center gap-6 px-6 py-5 bg-m3-errorContainer/10 rounded-m3-xl border border-m3-errorContainer/20 w-full lg:w-auto">
          <AlertCircle className="w-6 h-6 text-m3-error" />
          <div className="flex-1 min-w-0">
            <p className="text-[10px] font-black text-m3-error uppercase tracking-widest leading-none">
              Authorization Signature
            </p>
            <p className="text-[9px] text-m3-onSurfaceVariant uppercase font-bold mt-1.5 italic">
              Type "LEAVE" to finalize link termination
            </p>
          </div>
          <input
            type="text"
            value={confirmText}
            onChange={(e) => setConfirmText(e.target.value.toUpperCase())}
            className="bg-black/60 border border-m3-error/30 rounded-m3-lg px-5 py-2.5 text-m3-error font-mono text-xl font-black tracking-widest w-36 outline-none focus:border-m3-error shadow-inner text-center uppercase"
            placeholder="••••"
          />
        </div>
        <button
          disabled={
            selectedGuildsToLeave.size === 0 ||
            confirmText !== "LEAVE" ||
            isProcessing
          }
          onClick={onStartAction}
          className="m3-button-primary !py-8 !px-12 !text-base !bg-m3-error !text-m3-onError shadow-2xl shadow-m3-error/30 active:scale-[0.98] w-full lg:w-auto !rounded-m3-xl"
        >
          <LogOut className="w-6 h-6" />
          Sever {selectedGuildsToLeave.size} Connections
        </button>
      </div>

      {firstSelectedGuild &&
        firstSelectedId !== "dms" &&
        currentChannels.length > 0 && (
          <div className="flex flex-col gap-8 pt-8 border-t border-m3-outlineVariant/30 px-4">
            <div className="space-y-2">
              <SectionLabel>
                <Hammer className="w-3.5 h-3.5" /> Audit Log Burial:{" "}
                {firstSelectedGuild.name}
              </SectionLabel>
              <p className="text-[10px] text-m3-primary/60 font-bold uppercase tracking-tight ml-2 italic">
                Mechanism: Triggers rapid cyclic renames to flood the server
                audit log, pushing sensitive entries out of immediate view.
              </p>
            </div>
            <div className="space-y-4">
              <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest leading-relaxed bg-white/5 p-4 rounded-m3-md border border-white/5">
                <span className="text-m3-error mr-2">REQUIRED:</span> 'Manage
                Channels' permission. Select a target node below to begin the
                burial protocol.
              </p>
              {/* ... (channels list) ... */}
              <div className="m3-card !p-2 !bg-black/30 border-m3-outlineVariant/20 overflow-y-auto custom-scrollbar min-h-[100px] max-h-[200px] shadow-inner">
                <div className="grid grid-cols-1 gap-2 p-1">
                  {currentChannels
                    .filter((c) => c.channel_type === 0)
                    .map((c) => (
                      <button
                        key={c.id}
                        onClick={() => onToggleChannelForAudit(c.id)}
                        className={`flex items-center justify-between p-4 rounded-m3-lg border-2 transition-all group ${selectedChannels.has(c.id) ? "bg-m3-primaryContainer/20 border-m3-primary text-white shadow-sm" : "bg-transparent border-transparent text-m3-onSurfaceVariant hover:bg-m3-onSurface/5"}`}
                      >
                        <div className="flex items-center gap-3">
                          <Hash
                            className={`w-3.5 h-3.5 ${selectedChannels.has(c.id) ? "text-m3-primary" : "text-m3-outline"}`}
                          />
                          <span className="text-xs font-bold uppercase italic">
                            {c.name}
                          </span>
                        </div>
                        <div
                          className={`w-5 h-5 rounded-m3-xs border-2 flex items-center justify-center transition-all ${selectedChannels.has(c.id) ? "bg-m3-primary border-m3-primary scale-110" : "border-m3-outlineVariant"}`}
                        >
                          {selectedChannels.has(c.id) && (
                            <CheckCircle2 className="w-3.5 h-3.5 text-m3-onPrimary" />
                          )}
                        </div>
                      </button>
                    ))}
                </div>
              </div>
              <button
                onClick={onBuryAuditLog}
                disabled={
                  selectedChannels.size !== 1 || isLoading || isProcessing
                }
                className="m3-button-primary !py-5 !bg-m3-secondary !text-m3-onSecondary w-full"
              >
                <Hammer className="w-4 h-4" /> Initialize Burial Protocol
              </button>
            </div>
          </div>
        )}

      {firstSelectedGuild && firstSelectedId !== "dms" && (
        <div className="flex flex-col gap-8 pt-8 border-t border-m3-outlineVariant/30 px-4">
          <div className="space-y-2">
            <SectionLabel>
              <Cloud className="w-3.5 h-3.5" /> Webhook Ghosting:{" "}
              {firstSelectedGuild.name}
            </SectionLabel>
            <p className="text-[10px] text-m3-primary/60 font-bold uppercase tracking-tight ml-2 italic">
              Mechanism: Scans all node integrations and permanently nullifies
              any webhooks linked to your current identity.
            </p>
          </div>
          <div className="space-y-4">
            <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest leading-relaxed bg-white/5 p-4 rounded-m3-md border border-white/5">
              <span className="text-m3-error mr-2">REQUIRED:</span> 'Manage
              Webhooks' permission. This action is deep and irreversible.
            </p>
            <button
              onClick={onWebhookGhosting}
              disabled={isLoading || isProcessing}
              className="m3-button-primary !py-5 !bg-m3-tertiary !text-m3-onTertiary w-full"
            >
              <Cloud className="w-4 h-4" /> Purge Identity Webhooks
            </button>
          </div>
        </div>
      )}
    </M3Card>
  );
};
