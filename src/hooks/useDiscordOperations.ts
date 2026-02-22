import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAuthStore } from "../store/authStore";
import { Guild, Channel } from "../types/discord";
import { useSelectionState } from "./useSelectionState";
import { useOperationControl } from "./useOperationControl";

export const useDiscordOperations = (
  handleApiError: (err: any, fallback: string) => void,
) => {
  const { setGuilds, setError, setLoading } = useAuthStore();

  const [mode, setAppMode] = useState<"messages" | "servers" | "identity">(
    "messages",
  );
  const [confirmText, setConfirmText] = useState("");
  const [timeRange, setTimeRange] = useState<"24h" | "7d" | "all">("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [purgeReactions, setPurgeReactions] = useState(false);
  const [onlyAttachments, setOnlyAttachments] = useState(false);
  const [simulation, setSimulation] = useState(false);
  const [closeEmptyDms, setCloseEmptyDms] = useState(false);

  const {
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
  } = useSelectionState();

  const {
    isProcessing,
    setIsProcessing,
    progress,
    setProgress,
    operationStatus,
    setOperationStatus,
    getOperationStatus,
    handlePause,
    handleResume,
    handleAbort,
  } = useOperationControl();

  const fetchGuilds = useCallback(async () => {
    setLoading(true);
    try {
      setGuilds(await invoke("fetch_guilds"));
    } catch (err: any) {
      handleApiError(err, "Failed to load servers.");
    } finally {
      setLoading(false);
    }
  }, [setLoading, setGuilds, handleApiError]);

  const fetchRelationships = useCallback(async () => {
    setLoading(true);
    try {
      setRelationships(await invoke("fetch_relationships"));
    } catch (err: any) {
      handleApiError(err, "Failed to load identity links.");
    } finally {
      setLoading(false);
    }
  }, [setLoading, setRelationships, handleApiError]);

  const handleNitroWipe = async () => {
    setLoading(true);
    try {
      await invoke("nitro_stealth_wipe");
      setError("Nitro stealth wipe protocol execution complete.");
    } catch (err: any) {
      handleApiError(err, "Nitro stealth wipe failed.");
    } finally {
      setLoading(false);
    }
  };

  const handleStealthWipe = async () => {
    setLoading(true);
    try {
      await invoke("stealth_privacy_wipe");
      setError("Stealth protocol execution complete.");
    } catch (err: any) {
      handleApiError(err, "Stealth wipe failed.");
    } finally {
      setLoading(false);
    }
  };

  const handleToggleGuildSelection = async (guild: Guild | null) => {
    const effectiveId = guild?.id || "dms";

    setSelectedGuilds((prev) => {
      const next = new Set(prev);
      if (next.has(effectiveId)) {
        next.delete(effectiveId);
      } else {
        next.add(effectiveId);
      }
      return next;
    });

    if (!selectedGuilds.has(effectiveId)) {
      setLoading(true);
      try {
        const fetchedChannels: Channel[] = await invoke("fetch_channels", {
          guildId: guild?.id || null,
        });
        setChannelsByGuild((prev) => {
          const next = new Map(prev);
          next.set(effectiveId, fetchedChannels);
          return next;
        });
      } catch (err: any) {
        handleApiError(
          err,
          `Failed to load buffers for ${guild?.name || "Direct Messages"}.`,
        );
        setSelectedGuilds((prev) => {
          const next = new Set(prev);
          next.delete(effectiveId);
          return next;
        });
      } finally {
        setLoading(false);
      }
    } else {
      setChannelsByGuild((prev) => {
        const next = new Map(prev);
        const removedChannels = next.get(effectiveId) || [];
        next.delete(effectiveId);

        setSelectedChannels((prevSelected) => {
          const nextSelected = new Set(prevSelected);
          removedChannels.forEach((c) => nextSelected.delete(c.id));
          return nextSelected;
        });

        return next;
      });
    }
  };

  const fetchPreview = async (channelId: string) => {
    try {
      setPreviews(await invoke("fetch_preview_messages", { channelId }));
    } catch (err) {
      // Ignore preview errors
    }
  };

  const handleToggleChannel = (id: string) => {
    const next = new Set(selectedChannels);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelectedChannels(next);
    if (!next.has(id)) setPreviews([]);
    else fetchPreview(id);
  };

  const handleBuryAuditLog = async () => {
    if (selectedGuilds.size === 0 || selectedChannels.size === 0) {
      setError(
        "Please select at least one guild and one channel for audit log burial.",
      );
      return;
    }
    const guildId = Array.from(selectedGuilds)[0];
    if (guildId === "dms") {
      setError("Audit log burial cannot be performed on DMs.");
      return;
    }

    setLoading(true);
    try {
      const channelId = Array.from(selectedChannels)[0];
      await invoke("bury_audit_log", { guildId, channelId });
      setError(
        "Audit log burial initiated. Check Discord's audit log for details.",
      );
    } catch (err: any) {
      handleApiError(err, "Failed to bury audit log.");
    } finally {
      setLoading(false);
    }
  };

  const handleWebhookGhosting = async () => {
    if (selectedGuilds.size === 0) {
      setError("Please select a guild for webhook ghosting.");
      return;
    }
    const guildId = Array.from(selectedGuilds)[0];
    if (guildId === "dms") {
      setError("Webhook ghosting cannot be performed on DMs.");
      return;
    }

    setLoading(true);
    try {
      await invoke("webhook_ghosting", { guildId });
      setError("Webhook Ghosting initiated.");
    } catch (err: any) {
      handleApiError(err, "Failed to perform webhook ghosting.");
    } finally {
      setLoading(false);
    }
  };

  const startAction = async () => {
    const required =
      mode === "messages" ? "DELETE" : mode === "servers" ? "LEAVE" : "REMOVE";
    if (confirmText !== required) return;
    setIsProcessing(true);
    setConfirmText("");
    try {
      if (mode === "messages") {
        const now = Date.now();
        const start =
          timeRange === "24h"
            ? now - 86400000
            : timeRange === "7d"
              ? now - 604800000
              : undefined;
        await invoke("bulk_delete_messages", {
          options: {
            channelIds: Array.from(selectedChannels),
            startTime: start,
            endTime: undefined,
            searchQuery: searchQuery || undefined,
            purgeReactions,
            simulation,
            onlyAttachments,
            closeEmptyDms,
          },
        });
      } else if (mode === "servers") {
        await invoke("bulk_leave_guilds", {
          guildIds: Array.from(selectedGuildsToLeave),
        });
      } else if (mode === "identity") {
        await invoke("bulk_remove_relationships", {
          userIds: Array.from(selectedRelationships),
        });
      }
    } catch (err: any) {
      handleApiError(err, "Protocol execution error.");
    }
  };

  return {
    mode,
    setAppMode,
    selectedGuilds,
    setSelectedGuilds,
    channelsByGuild,
    setChannelsByGuild,
    relationships,
    setRelationships,
    previews,
    setPreviews,
    selectedChannels,
    setSelectedChannels,
    selectedGuildsToLeave,
    setSelectedGuildsToLeave,
    selectedRelationships,
    setSelectedRelationships,
    isProcessing,
    setIsProcessing,
    progress,
    setProgress,
    confirmText,
    setConfirmText,
    timeRange,
    setTimeRange,
    searchQuery,
    setSearchQuery,
    purgeReactions,
    setPurgeReactions,
    onlyAttachments,
    setOnlyAttachments,
    simulation,
    setSimulation,
    closeEmptyDms,
    setCloseEmptyDms,
    operationStatus,
    setOperationStatus,
    fetchGuilds,
    fetchRelationships,
    getOperationStatus,
    handleNitroWipe,
    handleStealthWipe,
    handleToggleGuildSelection,
    handleToggleChannel,
    handlePause,
    handleResume,
    handleAbort,
    handleBuryAuditLog,
    handleWebhookGhosting,
    startAction,
  };
};
