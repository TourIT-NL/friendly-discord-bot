import { Hash, Eye } from "lucide-react";
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
}: TargetBuffersProps) => (
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
      {Array.from(channelsByGuild.entries()).map(([guildId, guildChannels]) => (
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
            <div key={c.id} className="flex flex-col gap-1 mb-2 last:mb-0">
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
      ))}
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
);
