import { motion } from "framer-motion";
import { Clock, Filter } from "lucide-react";
import { SectionLabel } from "../../../common/M3Components";

interface MessageFiltersProps {
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
  closeEmptyDms: boolean;
  setCloseEmptyDms: (close: boolean) => void;
}

export const MessageFilters = ({
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
  closeEmptyDms,
  setCloseEmptyDms,
}: MessageFiltersProps) => (
  <div className="flex flex-col gap-8">
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
      <button
        onClick={() => setCloseEmptyDms(!closeEmptyDms)}
        className={`w-full flex items-center justify-between p-4 rounded-m3-xl border-2 transition-all ${closeEmptyDms ? "bg-m3-primary/10 border-m3-primary text-white" : "bg-transparent border-m3-outlineVariant/30 text-m3-onSurfaceVariant"}`}
      >
        <span className="text-xs font-bold uppercase italic text-left">
          Close DM/Group after purge (if empty)
        </span>
        <div
          className={`w-10 h-6 rounded-full p-1 transition-colors ${closeEmptyDms ? "bg-m3-primary" : "bg-m3-outline"}`}
        >
          <motion.div
            animate={{ x: closeEmptyDms ? 16 : 0 }}
            className="w-4 h-4 bg-white rounded-full shadow-sm"
          />
        </div>
      </button>
    </div>
  </div>
);
