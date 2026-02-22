import React from "react";
import { motion } from "framer-motion";
import {
  Clock,
  Filter,
  Hash,
  Eye,
  Settings,
  ShieldAlert,
  Trash2,
  Play,
} from "lucide-react";
import { M3Card, SectionLabel } from "../../common/M3Components";
import { Channel, Guild } from "../../../types/discord";

interface MessagesModeProps {
  timeRange: "24h" | "7d" | "all";
  setTimeRange: (range: "24h" | "7d" | "all") => void;
  simulation: boolean;
  setSimulation: (sim: boolean) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  purgeReactions: boolean;
  setPurgeReactions: (purge: boolean) => void;
  onlyAttachments: boolean;
  setOnlyAttachments: (only: boolean) => void;
  guilds: Guild[] | null;
  channelsByGuild: Map<string, Channel[]>;
  selectedChannels: Set<string>;
  onToggleChannel: (id: string) => void;
  onMapAll: () => void;
  previews: any[];
  confirmText: string;
  setConfirmText: (text: string) => void;
  isProcessing: boolean;
  onStartAction: () => void;
}

export const MessagesMode = ({
  timeRange,
  setTimeRange,
  simulation,
  setSimulation,
  searchQuery,
  setSearchQuery,
  purgeReactions,
  setPurgeReactions,
  onlyAttachments,
  setOnlyAttachments,
  guilds,
  channelsByGuild,
  selectedChannels,
  onToggleChannel,
  onMapAll,
  previews,
  confirmText,
  setConfirmText,
  isProcessing,
  onStartAction,
}: MessagesModeProps) => (
  <M3Card className="grid grid-cols-1 lg:grid-cols-2 gap-10 flex-1 border-m3-primary/10 shadow-2xl p-10">
    <div className="flex flex-col gap-8">
      {/* ... (Range & Simulations and Content Filters sections remain same) ... */}
      <div className="space-y-4">
        <SectionLabel>
          <Clock className="w-3.5 h-3.5" /> Range & Simulations
        </SectionLabel>
        <div className="grid grid-cols-3 gap-3 p-2 bg-black/40 rounded-m3-xl border border-m3-outlineVariant/30 shadow-inner">
          {(["24h", "7d", "all"] as const).map((r) => (
            <button
              key={r}
              onClick={() => setTimeRange(r)}
              className={`py-4 rounded-m3-lg text-[10px] font-black uppercase tracking-widest transition-all ${timeRange === r ? "bg-m3-secondaryContainer text-m3-onSecondaryContainer" : "text-m3-onSurfaceVariant"}`}
            >
              {r}
            </button>
          ))}
        </div>
        <button
          onClick={() => setSimulation(!simulation)}
          className={`w-full flex items-center justify-between p-4 rounded-m3-xl border-2 transition-all ${simulation ? "bg-m3-secondary/10 border-m3-secondary text-m3-secondary" : "bg-transparent border-m3-outlineVariant/30 text-m3-onSurfaceVariant"}`}
        >
          <span className="text-[10px] font-black uppercase tracking-widest">
            Simulation Mode (Safe Run)
          </span>
          <div
            className={`w-10 h-6 rounded-full p-1 transition-colors ${simulation ? "bg-m3-secondary" : "bg-m3-outline"}`}
          >
            <motion.div
              animate={{ x: simulation ? 16 : 0 }}
              className="w-4 h-4 bg-white rounded-full"
            />
          </div>
        </button>
      </div>

      <div className="space-y-4">
        <SectionLabel>
          <Filter className="w-3.5 h-3.5" /> Content Filters
        </SectionLabel>
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Filter by keyword..."
          className="w-full bg-black/40 border-2 border-m3-outlineVariant/30 focus:border-m3-primary rounded-m3-xl px-6 py-4 text-xs font-bold text-white outline-none transition-all shadow-inner"
        />
        <button
          onClick={() => setPurgeReactions(!purgeReactions)}
          className={`w-full flex items-center justify-between p-4 rounded-m3-xl border-2 transition-all ${purgeReactions ? "bg-m3-primary/10 border-m3-primary text-white" : "bg-transparent border-m3-outlineVariant/30 text-m3-onSurfaceVariant"}`}
        >
          <span className="text-xs font-bold uppercase italic">
            Purge My Reactions
          </span>
          <div
            className={`w-10 h-6 rounded-full p-1 transition-colors ${purgeReactions ? "bg-m3-primary" : "bg-m3-outline"}`}
          >
            <motion.div
              animate={{ x: purgeReactions ? 16 : 0 }}
              className="w-4 h-4 bg-white rounded-full shadow-sm"
            />
          </div>
        </button>
        <button
          onClick={() => setOnlyAttachments(!onlyAttachments)}
          className={`w-full flex items-center justify-between p-4 rounded-m3-xl border-2 transition-all ${onlyAttachments ? "bg-m3-primary/10 border-m3-primary text-white" : "bg-transparent border-m3-outlineVariant/30 text-m3-onSurfaceVariant"}`}
        >
          <span className="text-xs font-bold uppercase italic">
            Only Attachments
          </span>
          <div
            className={`w-10 h-6 rounded-full p-1 transition-colors ${onlyAttachments ? "bg-m3-primary" : "bg-m3-outline"}`}
          >
            <motion.div
              animate={{ x: onlyAttachments ? 16 : 0 }}
              className="w-4 h-4 bg-white rounded-full shadow-sm"
            />
          </div>
        </button>
      </div>
    </div>

    <div className="flex flex-col gap-8">
      <div className="flex flex-col gap-4 flex-1">
        <div className="flex items-center justify-between px-2">
          <SectionLabel>
            <Hash className="w-3.5 h-3.5" /> Target Buffers
          </SectionLabel>
          <div className="flex gap-2 mb-4">
            <button
              onClick={onMapAll}
              className="text-[9px] font-black text-m3-primary uppercase hover:underline"
            >
              All
            </button>
            <span className="text-white/10 text-[9px]">|</span>
            <button
              onClick={() =>
                Array.from(channelsByGuild.values())
                  .flat()
                  .forEach(
                    (c) => selectedChannels.has(c.id) && onToggleChannel(c.id),
                  )
              }
              className="text-[9px] font-black text-m3-outline uppercase hover:underline"
            >
              Clear
            </button>
          </div>
        </div>
        <div className="m3-card !p-2 !bg-black/30 border-m3-outlineVariant/20 flex-1 overflow-y-auto custom-scrollbar min-h-[200px]">
          {/* ... (channels rendering) ... */}
          {Array.from(channelsByGuild.entries()).map(
            ([guildId, guildChannels]) => (
              <div key={guildId} className="mb-6 last:mb-0">
                {channelsByGuild.size > 1 && (
                  <div className="px-4 py-2 bg-m3-surfaceVariant/30 rounded-m3-md mb-2 flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-m3-primary" />
                    <span className="text-[10px] font-black uppercase tracking-widest text-m3-onSurfaceVariant">
                      {guildId === "dms"
                        ? "Direct Messages"
                        : guilds?.find((g) => g.id === guildId)?.name ||
                          "Unknown Server"}
                    </span>
                  </div>
                )}
                {guildChannels.map((c) => (
                  <div
                    key={c.id}
                    className="flex flex-col gap-1 mb-2 last:mb-0"
                  >
                    <button
                      onClick={() => onToggleChannel(c.id)}
                      className={`flex items-center justify-between p-4 rounded-m3-lg border-2 transition-all ${selectedChannels.has(c.id) ? "bg-m3-primaryContainer/20 border-m3-primary text-white" : "bg-transparent border-transparent text-m3-onSurfaceVariant"}`}
                    >
                      <div className="flex items-center gap-3">
                        <Hash className="w-3.5 h-3.5" />
                        <span className="text-xs font-bold uppercase italic">
                          {c.name}
                        </span>
                      </div>
                      {selectedChannels.has(c.id) && (
                        <Eye className="w-3.5 h-3.5 animate-pulse text-m3-primary" />
                      )}
                    </button>
                    {selectedChannels.has(c.id) && previews.length > 0 && (
                      <div className="mx-4 p-3 bg-black/40 rounded-m3-lg border border-m3-outlineVariant/20 space-y-2">
                        {previews.map((p, i) => (
                          <div
                            key={i}
                            className="text-[9px] font-mono text-m3-onSurfaceVariant border-b border-white/5 pb-1 last:border-none truncate"
                          >
                            <span className="text-m3-primary font-bold">
                              {p.author.username}:
                            </span>{" "}
                            {p.content || "[Embed/File]"}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            ),
          )}
          {channelsByGuild.size === 0 && (
            <div className="flex flex-col items-center justify-center h-full gap-4 opacity-40">
              <Hash className="w-12 h-12" />
              <p className="text-[10px] font-black uppercase tracking-widest">
                No Sources Linked
              </p>
            </div>
          )}
        </div>
      </div>

      <div className="flex flex-col gap-10">
        <SectionLabel>
          <Settings className="w-3.5 h-3.5" /> Execution Protocol
        </SectionLabel>
        <div className="bg-m3-errorContainer/5 border border-m3-errorContainer/20 rounded-m3-xl p-8 flex-1 flex flex-col items-center justify-center text-center gap-8 shadow-inner">
          <div className="p-6 rounded-full bg-m3-errorContainer/10 border border-m3-error/20">
            <ShieldAlert className="w-12 h-12 text-m3-error drop-shadow-[0_0_15px_rgba(242,184,181,0.4)]" />
          </div>
          <div>
            <h4 className="text-2xl font-black italic uppercase text-m3-error tracking-tight">
              {simulation ? "Simulation Run" : "Security Required"}
            </h4>
            <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest mt-2 px-10 leading-relaxed">
              Authorized for{" "}
              <span className="text-white underline decoration-m3-error decoration-2 underline-offset-4">
                {selectedChannels.size} buffers
              </span>{" "}
              across{" "}
              <span className="text-white">{channelsByGuild.size} sources</span>
              .{" "}
              {simulation
                ? "No data will be destroyed."
                : "Permanent purge protocol."}
            </p>
          </div>
          <div className="w-full space-y-4">
            <p className="text-[9px] font-black text-m3-error uppercase tracking-[0.4em] italic">
              Auth Signature: "DELETE"
            </p>
            <input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value.toUpperCase())}
              className="w-full bg-black/60 border-2 border-m3-error/30 focus:border-m3-error rounded-m3-xl p-6 text-center text-m3-error font-mono text-3xl font-black tracking-[0.8em] outline-none transition-all shadow-inner uppercase"
              placeholder="••••"
            />
          </div>
        </div>
        <button
          disabled={
            selectedChannels.size === 0 ||
            confirmText !== "DELETE" ||
            isProcessing
          }
          onClick={onStartAction}
          className={`m3-button-primary !py-8 !text-base shadow-2xl active:scale-[0.98] !rounded-m3-xl ${simulation ? "!bg-m3-secondary !text-m3-onSecondary" : "!bg-m3-error !text-m3-onError"}`}
        >
          {simulation ? (
            <Play className="w-6 h-6" />
          ) : (
            <Trash2 className="w-6 h-6" />
          )}
          {simulation ? "Start Safety Simulation" : "Execute Destructive Purge"}
        </button>
      </div>
    </div>
  </M3Card>
);
