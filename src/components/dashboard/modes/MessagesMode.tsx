import { M3Card } from "../../common/M3Components";
import { Channel, Guild } from "../../../types/discord";
import { MessageFilters } from "./messages/MessageFilters";
import { TargetBuffers } from "./messages/TargetBuffers";
import { ExecutionProtocol } from "./messages/ExecutionProtocol";

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
  closeEmptyDms: boolean;
  setCloseEmptyDms: (close: boolean) => void;
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
  closeEmptyDms,
  setCloseEmptyDms,
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
    <MessageFilters
      timeRange={timeRange}
      setTimeRange={setTimeRange}
      simulation={simulation}
      setSimulation={setSimulation}
      searchQuery={searchQuery}
      setSearchQuery={setSearchQuery}
      purgeReactions={purgeReactions}
      setPurgeReactions={setPurgeReactions}
      onlyAttachments={onlyAttachments}
      setOnlyAttachments={setOnlyAttachments}
      closeEmptyDms={closeEmptyDms}
      setCloseEmptyDms={setCloseEmptyDms}
    />

    <div className="flex flex-col gap-8">
      <TargetBuffers
        guilds={guilds}
        channelsByGuild={channelsByGuild}
        selectedChannels={selectedChannels}
        onToggleChannel={onToggleChannel}
        onMapAll={onMapAll}
        previews={previews}
      />

      <ExecutionProtocol
        simulation={simulation}
        selectedChannels={selectedChannels}
        channelsByGuild={channelsByGuild}
        confirmText={confirmText}
        setConfirmText={setConfirmText}
        isProcessing={isProcessing}
        onStartAction={onStartAction}
      />
    </div>
  </M3Card>
);
