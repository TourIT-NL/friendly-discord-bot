import React from "react";
import { motion } from "framer-motion";
import {
  Download,
  FileText,
  Files,
  Archive,
  CheckCircle2,
  AlertTriangle,
} from "lucide-react";
import { IconButton } from "../../common/M3Components";
import { Guild, Channel } from "../../../types/discord";

interface ExportModeProps {
  guilds: Guild[] | null;
  selectedGuilds: Set<string>;
  channelsByGuild: Map<string, Channel[]>;
  selectedChannels: Set<string>;
  onToggleChannel: (id: string) => void;
  exportDirection: "sent" | "received" | "both";
  setExportDirection: (dir: "sent" | "received" | "both") => void;
  includeAttachmentsInHtml: boolean;
  setIncludeAttachmentsInHtml: (inc: boolean) => void;
  onStartExport: (format: "html" | "raw") => void;
  onStartGuildArchive: () => void;
  isProcessing: boolean;
}

export const ExportMode = ({
  guilds,
  selectedGuilds,
  channelsByGuild,
  selectedChannels,
  onToggleChannel,
  exportDirection,
  setExportDirection,
  includeAttachmentsInHtml,
  setIncludeAttachmentsInHtml,
  onStartExport,
  onStartGuildArchive,
  isProcessing,
}: ExportModeProps) => {
  return (
    <div className="flex-1 flex flex-col gap-8">
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Attachment Harvester */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="m3-card p-8 flex flex-col gap-6 border-m3-primary/10"
        >
          <div className="flex items-center gap-4 text-m3-primary">
            <Files className="w-6 h-6" />
            <h3 className="text-xl font-black uppercase tracking-tighter italic">
              Attachment Harvester
            </h3>
          </div>
          <p className="text-[10px] text-m3-onSurfaceVariant uppercase tracking-widest font-bold leading-relaxed">
            Extract every file, image, and document from selected channels.
          </p>

          <div className="space-y-4 py-4 border-y border-white/5">
            <label className="text-[10px] font-black uppercase tracking-widest text-m3-primary/60">
              Extraction Direction
            </label>
            <div className="flex bg-m3-surfaceVariant rounded-lg p-1">
              {(["sent", "received", "both"] as const).map((dir) => (
                <button
                  key={dir}
                  onClick={() => setExportDirection(dir)}
                  className={`flex-1 py-2 text-[10px] font-black uppercase tracking-widest rounded-md transition-all ${
                    exportDirection === dir
                      ? "bg-m3-primary text-m3-onPrimary shadow-lg"
                      : "text-m3-onSurfaceVariant hover:bg-white/5"
                  }`}
                >
                  {dir}
                </button>
              ))}
            </div>
          </div>

          <button
            onClick={() => onStartExport("raw")}
            disabled={isProcessing || selectedChannels.size === 0}
            className="m3-button-primary w-full !py-4 flex items-center justify-center gap-3"
          >
            <Download className="w-4 h-4" />
            <span className="text-[10px] font-black uppercase tracking-[0.2em]">
              Initiate Harvest
            </span>
          </button>
        </motion.div>

        {/* HTML Chronicler */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="m3-card p-8 flex flex-col gap-6 border-m3-secondary/10"
        >
          <div className="flex items-center gap-4 text-m3-secondary">
            <FileText className="w-6 h-6" />
            <h3 className="text-xl font-black uppercase tracking-tighter italic">
              HTML Chronicler
            </h3>
          </div>
          <p className="text-[10px] text-m3-onSurfaceVariant uppercase tracking-widest font-bold leading-relaxed">
            Generate easy-readable HTML logs of your chat history.
          </p>

          <div className="space-y-4 py-4 border-y border-white/5">
            <div
              className="flex items-center justify-between cursor-pointer group"
              onClick={() =>
                setIncludeAttachmentsInHtml(!includeAttachmentsInHtml)
              }
            >
              <span className="text-[10px] font-black uppercase tracking-widest text-m3-onSurfaceVariant group-hover:text-m3-secondary transition-colors">
                Include Attachment Links
              </span>
              <div
                className={`w-10 h-5 rounded-full relative transition-colors ${
                  includeAttachmentsInHtml ? "bg-m3-secondary" : "bg-white/10"
                }`}
              >
                <div
                  className={`absolute top-1 w-3 h-3 rounded-full bg-white transition-all ${
                    includeAttachmentsInHtml ? "left-6" : "left-1"
                  }`}
                />
              </div>
            </div>
          </div>

          <button
            onClick={() => onStartExport("html")}
            disabled={isProcessing || selectedChannels.size === 0}
            className="m3-button-secondary w-full !py-4 flex items-center justify-center gap-3"
          >
            <FileText className="w-4 h-4" />
            <span className="text-[10px] font-black uppercase tracking-[0.2em]">
              Generate Chronicles
            </span>
          </button>
        </motion.div>

        {/* Guild Archivist */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="m3-card p-8 flex flex-col gap-6 border-m3-tertiary/10"
        >
          <div className="flex items-center gap-4 text-m3-tertiary">
            <Archive className="w-6 h-6" />
            <h3 className="text-xl font-black uppercase tracking-tighter italic">
              Server Archivist
            </h3>
          </div>
          <p className="text-[10px] text-m3-onSurfaceVariant uppercase tracking-widest font-bold leading-relaxed">
            Full export of every message & attachment you sent in a specific
            guild.
          </p>

          <div className="mt-auto">
            <button
              onClick={onStartGuildArchive}
              disabled={isProcessing || selectedGuilds.size !== 1}
              className="m3-button-tertiary w-full !py-4 flex items-center justify-center gap-3"
            >
              <Archive className="w-4 h-4" />
              <span className="text-[10px] font-black uppercase tracking-[0.2em]">
                Package Server Data
              </span>
            </button>
            {selectedGuilds.size !== 1 && (
              <p className="text-[8px] text-m3-error font-black uppercase tracking-widest text-center mt-4 animate-pulse">
                Select exactly one server to archive
              </p>
            )}
          </div>
        </motion.div>
      </div>

      {/* Target Buffer Selection */}
      <div className="flex-1 m3-card p-10 overflow-hidden flex flex-col gap-8">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h3 className="text-2xl font-black uppercase italic tracking-tighter text-white">
              Target Buffers
            </h3>
            <div className="px-3 py-1 rounded-full bg-m3-primary/10 border border-m3-primary/20">
              <span className="text-[10px] font-black text-m3-primary uppercase tracking-widest">
                {selectedChannels.size} Selected
              </span>
            </div>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto pr-4 custom-scrollbar">
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
            {Array.from(channelsByGuild.entries()).map(([guildId, channels]) => (
              <div key={guildId} className="space-y-4">
                <div className="sticky top-0 bg-[#1a1a1a]/80 backdrop-blur-md py-2 z-10 flex items-center gap-3">
                  <div className="w-1.5 h-1.5 rounded-full bg-m3-primary" />
                  <h4 className="text-[10px] font-black uppercase tracking-[0.3em] text-m3-onSurfaceVariant">
                    {guilds?.find((g) => g.id === guildId)?.name ||
                      "Direct Messages"}
                  </h4>
                </div>
                <div className="grid gap-2">
                  {channels.map((channel) => (
                    <button
                      key={channel.id}
                      onClick={() => onToggleChannel(channel.id)}
                      className={`flex items-center justify-between p-4 rounded-m3-lg transition-all border ${
                        selectedChannels.has(channel.id)
                          ? "bg-m3-primary/10 border-m3-primary/40"
                          : "bg-white/5 border-transparent hover:bg-white/10"
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className={`p-2 rounded-md ${selectedChannels.has(channel.id) ? "bg-m3-primary text-m3-onPrimary" : "bg-white/5 text-m3-onSurfaceVariant"}`}
                        >
                          <FileText className="w-3 h-3" />
                        </div>
                        <span className="text-[10px] font-bold uppercase tracking-wider text-white truncate max-w-[150px]">
                          {channel.name}
                        </span>
                      </div>
                      {selectedChannels.has(channel.id) && (
                        <CheckCircle2 className="w-4 h-4 text-m3-primary" />
                      )}
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
