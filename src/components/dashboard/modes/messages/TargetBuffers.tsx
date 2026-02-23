import { motion } from "framer-motion";
import {
  Hash,
  Eye,
  Globe,
  MessageCircle,
  XCircle,
  CheckSquare,
} from "lucide-react";
import { SectionLabel } from "../../../common/M3Components";
import { Channel, Guild } from "../../../../types/discord";

interface TargetBuffersProps {
  guilds: Guild[] | null;
  channelsByGuild: Map<string, Channel[]>;
  selectedChannels: Set<string>;
  onToggleChannel: (id: string) => void;
  onMapAll: () => void;
  previews: any[];
}

export const TargetBuffers = ({
  guilds,
  channelsByGuild,
  selectedChannels,
  onToggleChannel,
  onMapAll,
  previews,
}: TargetBuffersProps) => {
  const totalChannelsCount = Array.from(channelsByGuild.values()).flat().length;

  return (
    <div className="flex flex-col gap-6 flex-1 min-h-0">
      <div className="flex items-center justify-between px-2">
        <SectionLabel>
          <Hash className="w-3.5 h-3.5" /> Target Buffers (
          {selectedChannels.size}/{totalChannelsCount})
        </SectionLabel>
        <div className="flex gap-3 mb-4">
          <button
            onClick={onMapAll}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-m3-primary/10 text-[9px] font-black text-m3-primary uppercase hover:bg-m3-primary/20 transition-all border border-m3-primary/20"
          >
            <CheckSquare className="w-3 h-3" /> Select All
          </button>
          <button
            onClick={() =>
              Array.from(channelsByGuild.values())
                .flat()
                .forEach(
                  (c) => selectedChannels.has(c.id) && onToggleChannel(c.id),
                )
            }
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-white/5 text-[9px] font-black text-m3-outline uppercase hover:bg-white/10 transition-all border border-white/10"
          >
            <XCircle className="w-3 h-3" /> Deselect
          </button>
        </div>
      </div>
      <div className="m3-card !p-4 !bg-black/30 border-m3-outlineVariant/20 flex-1 overflow-y-auto custom-scrollbar min-h-[300px] shadow-inner">
        {Array.from(channelsByGuild.entries()).map(
          ([guildId, guildChannels]) => (
            <div key={guildId} className="mb-8 last:mb-0">
              <div className="px-4 py-3 bg-m3-surfaceVariant/20 rounded-m3-lg mb-3 flex items-center justify-between border border-white/5">
                <div className="flex items-center gap-3">
                  {guildId === "dms" ? (
                    <MessageCircle className="w-4 h-4 text-m3-primary" />
                  ) : (
                    <Globe className="w-4 h-4 text-m3-primary" />
                  )}
                  <span className="text-[11px] font-black uppercase tracking-[0.2em] text-white italic">
                    {guildId === "dms"
                      ? "Direct Messages"
                      : guilds?.find((g) => g.id === guildId)?.name ||
                        "Unknown Server"}
                  </span>
                </div>
                <span className="text-[9px] font-mono text-m3-onSurfaceVariant opacity-50">
                  {guildChannels.length} Node
                  {guildChannels.length !== 1 ? "s" : ""}
                </span>
              </div>
              <div className="grid grid-cols-1 gap-2 pl-2">
                {guildChannels.map((c) => (
                  <div key={c.id} className="flex flex-col gap-1">
                    <button
                      onClick={() => onToggleChannel(c.id)}
                      className={`flex items-center justify-between p-4 rounded-m3-lg border-2 transition-all relative overflow-hidden group ${selectedChannels.has(c.id) ? "bg-m3-primaryContainer/10 border-m3-primary text-white shadow-md" : "bg-transparent border-transparent text-m3-onSurfaceVariant hover:bg-white/5 hover:border-white/10"}`}
                    >
                      <div className="flex items-center gap-3 relative z-10">
                        <Hash
                          className={`w-4 h-4 ${selectedChannels.has(c.id) ? "text-m3-primary" : "opacity-40"}`}
                        />
                        <span
                          className={`text-[11px] font-bold uppercase italic tracking-tight ${selectedChannels.has(c.id) ? "text-white" : "opacity-70"}`}
                        >
                          {c.name}
                        </span>
                      </div>
                      {selectedChannels.has(c.id) && (
                        <div className="flex items-center gap-2 relative z-10">
                          <span className="text-[8px] font-black text-m3-primary uppercase tracking-widest animate-pulse">
                            Linked
                          </span>
                          <Eye className="w-4 h-4 text-m3-primary" />
                        </div>
                      )}
                      {selectedChannels.has(c.id) && (
                        <motion.div
                          layoutId={`active-bg-${c.id}`}
                          className="absolute inset-0 bg-m3-primary/5 pointer-events-none"
                        />
                      )}
                    </button>
                    {selectedChannels.has(c.id) && previews.length > 0 && (
                      <motion.div
                        initial={{ opacity: 0, y: -5 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="mx-4 mt-1 p-4 bg-black/60 rounded-m3-xl border border-m3-primary/20 space-y-2 shadow-2xl"
                      >
                        <div className="flex items-center gap-2 mb-2 border-b border-white/5 pb-2">
                          <div className="w-1 h-3 bg-m3-primary rounded-full" />
                          <span className="text-[8px] font-black text-m3-primary uppercase tracking-[0.2em]">
                            Live Data Preview
                          </span>
                        </div>
                        {previews.map((p, i) => (
                          <div
                            key={p.id || i}
                            className="text-[10px] font-mono text-m3-onSurfaceVariant border-b border-white/5 pb-2 last:border-none last:pb-0"
                          >
                            <span className="text-m3-primary font-black uppercase italic mr-2">
                              {p.author.username}
                            </span>{" "}
                            <span className="opacity-80 leading-relaxed">
                              {p.content || (
                                <span className="italic opacity-40">
                                  [Encrypted/Binary Node]
                                </span>
                              )}
                            </span>
                          </div>
                        ))}
                      </motion.div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ),
        )}
        {channelsByGuild.size === 0 && (
          <div className="flex flex-col items-center justify-center h-full gap-6 opacity-20 py-20">
            <XCircle className="w-16 h-16" />
            <p className="text-sm font-black uppercase tracking-[0.4em] italic">
              No Data Streams Linked
            </p>
          </div>
        )}
      </div>
    </div>
  );
};
