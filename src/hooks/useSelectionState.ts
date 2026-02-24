import { useState } from "react";
import { Channel, Relationship, PreviewMessage } from "../types/discord";

export const useSelectionState = () => {
  const [selectedGuilds, setSelectedGuilds] = useState<Set<string>>(
    () => new Set(),
  );
  const [channelsByGuild, setChannelsByGuild] = useState<
    Map<string, Channel[]>
  >(() => new Map());
  const [selectedChannels, setSelectedChannels] = useState<Set<string>>(
    () => new Set(),
  );
  const [selectedGuildsToLeave, setSelectedGuildsToLeave] = useState<
    Set<string>
  >(() => new Set());
  const [selectedRelationships, setSelectedRelationships] = useState<
    Set<string>
  >(() => new Set());
  const [relationships, setRelationships] = useState<Relationship[] | null>(
    null,
  );
  const [previews, setPreviews] = useState<PreviewMessage[]>([]);

  return {
    selectedGuilds,
    setSelectedGuilds,
    channelsByGuild,
    setChannelsByGuild,
    selectedChannels,
    setSelectedChannels,
    selectedGuildsToLeave,
    setSelectedGuildsToLeave,
    selectedRelationships,
    setSelectedRelationships,
    relationships,
    setRelationships,
    previews,
    setPreviews,
  };
};
