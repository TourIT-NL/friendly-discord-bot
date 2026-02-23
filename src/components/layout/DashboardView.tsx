import { motion, AnimatePresence } from "framer-motion";
import { Server, HelpCircle, Hash } from "lucide-react";
import { Sidebar } from "../dashboard/Sidebar";
import { MessagesMode } from "../dashboard/modes/MessagesMode";
import { ServersMode } from "../dashboard/modes/ServersMode";
import { IdentityMode } from "../dashboard/modes/IdentityMode";
import { IconButton } from "../common/M3Components";
import { Guild, Channel, Relationship } from "../../types/discord";

interface DashboardViewProps {
  user: any;
  identities: any[];
  guilds: Guild[] | null;
  selectedGuilds: Set<string>;
  handleSwitchIdentity: (id: string) => void;
  setView: (
    view: "manual" | "auth" | "setup" | "qr" | "token" | "dashboard",
  ) => void;
  handleToggleGuildSelection: (guild: Guild | null) => void;
  handleStealthWipe: () => void;
  handleNitroWipe: () => void;
  handleLogout: () => void;
  mode: "messages" | "servers" | "identity";
  setMode: (mode: "messages" | "servers" | "identity") => void;
  timeRange: "24h" | "7d" | "all";
  setTimeRange: (range: "24h" | "7d" | "all") => void;
  simulation: boolean;
  setSimulation: (sim: boolean) => void;
  closeEmptyDms: boolean;
  setCloseEmptyDms: (close: boolean) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  purgeReactions: boolean;
  setPurgeReactions: (purge: boolean) => void;
  onlyAttachments: boolean;
  setOnlyAttachments: (only: boolean) => void;
  channelsByGuild: Map<string, Channel[]>;
  selectedChannels: Set<string>;
  handleToggleChannel: (id: string) => void;
  setSelectedChannels: (ids: Set<string>) => void;
  previews: any[];
  confirmText: string;
  setConfirmText: (text: string) => void;
  isProcessing: boolean;
  startAction: () => void;
  selectedGuildsToLeave: Set<string>;
  setSelectedGuildsToLeave: (ids: Set<string>) => void;
  handleBuryAuditLog: () => void;
  handleWebhookGhosting: () => void;
  handleOpenDonateLink: () => void;
  isLoading: boolean;
  relationships: Relationship[] | null;
  selectedRelationships: Set<string>;
  setSelectedRelationships: (ids: Set<string>) => void;
}

export const DashboardView = ({
  user,
  identities,
  guilds,
  selectedGuilds,
  handleSwitchIdentity,
  setView,
  handleToggleGuildSelection,
  handleStealthWipe,
  handleNitroWipe,
  handleLogout,
  mode,
  setMode,
  timeRange,
  setTimeRange,
  simulation,
  setSimulation,
  closeEmptyDms,
  setCloseEmptyDms,
  searchQuery,
  setSearchQuery,
  purgeReactions,
  setPurgeReactions,
  onlyAttachments,
  setOnlyAttachments,
  channelsByGuild,
  selectedChannels,
  handleToggleChannel,
  setSelectedChannels,
  previews,
  confirmText,
  setConfirmText,
  isProcessing,
  startAction,
  selectedGuildsToLeave,
  setSelectedGuildsToLeave,
  handleBuryAuditLog,
  handleWebhookGhosting,
  handleOpenDonateLink,
  isLoading,
  relationships,
  selectedRelationships,
  setSelectedRelationships,
}: DashboardViewProps) => (
  <div className="w-full h-full flex gap-10 p-4">
    <Sidebar
      user={user}
      identities={identities}
      guilds={guilds}
      selectedGuilds={selectedGuilds}
      onSwitchIdentity={handleSwitchIdentity}
      onNewIdentity={() => setView("auth")}
      onToggleGuildSelection={handleToggleGuildSelection}
      onStealthWipe={handleStealthWipe}
      onNitroWipe={handleNitroWipe}
      onLogout={handleLogout}
      onOpenDonateLink={handleOpenDonateLink}
    />
    <main className="flex-1 flex flex-col min-w-0">
      <AnimatePresence mode="wait">
        <motion.div
          key={Array.from(selectedGuilds).join("-") || "empty"}
          initial={{ opacity: 0, scale: 0.98 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.98 }}
          className="flex-1 flex flex-col gap-10"
        >
          <div className="flex items-center justify-between px-2">
            <div className="flex items-center gap-6">
              <div className="p-5 rounded-m3-xl bg-m3-surfaceVariant shadow-lg border border-m3-outlineVariant/30 text-m3-onSurfaceVariant group relative overflow-hidden">
                <div className="absolute inset-0 bg-m3-primary/5 animate-pulse" />
                <Server className="w-8 h-8 relative z-10" />
              </div>
              <div>
                <h2 className="text-5xl font-black italic tracking-tighter uppercase leading-none text-white">
                  {selectedGuilds.size === 0
                    ? "Select Sources"
                    : selectedGuilds.size === 1
                      ? guilds?.find(
                          (g) => g.id === Array.from(selectedGuilds)[0],
                        )?.name || "Direct Messages"
                      : `${selectedGuilds.size} Sources Selected`}
                </h2>
                <div className="flex items-center gap-3 mt-4 bg-m3-primary/10 w-fit px-4 py-1.5 rounded-full border border-m3-primary/20 shadow-inner">
                  <div className="w-2 h-2 bg-m3-primary rounded-full animate-pulse shadow-[0_0_10px_rgba(208,188,255,0.8)]" />
                  <p className="text-[10px] text-m3-primary font-black uppercase tracking-[0.4em] italic leading-none">
                    Node Connection Established
                  </p>
                </div>
              </div>
            </div>
            <div className="flex bg-m3-surfaceVariant rounded-m3-full p-1.5 border border-m3-outlineVariant shadow-inner">
              <button
                onClick={() => setMode("messages")}
                className={`px-8 py-2.5 rounded-m3-full text-[10px] font-black uppercase tracking-widest transition-all ${mode === "messages" ? "bg-m3-primary text-m3-onPrimary" : "text-m3-onSurfaceVariant"}`}
              >
                Messages
              </button>
              {selectedGuilds.size > 0 && (
                <button
                  onClick={() => setMode("servers")}
                  className={`px-8 py-2.5 rounded-m3-full text-[10px] font-black uppercase tracking-widest transition-all ${mode === "servers" ? "bg-m3-primary text-m3-onPrimary" : "text-m3-onSurfaceVariant"}`}
                >
                  Servers
                </button>
              )}
              <button
                onClick={() => setMode("identity")}
                className={`px-8 py-2.5 rounded-m3-full text-[10px] font-black uppercase tracking-widest transition-all ${mode === "identity" ? "bg-m3-primary text-m3-onPrimary" : "text-m3-onSurfaceVariant"}`}
              >
                Friends
              </button>
              <div className="w-px bg-white/10 mx-2" />
              <IconButton
                icon={HelpCircle}
                onClick={() => setView("manual")}
                className="!text-m3-onSurfaceVariant hover:!text-m3-primary transition-colors"
              />
            </div>
          </div>
          {mode === "messages" && (
            <MessagesMode
              timeRange={timeRange}
              setTimeRange={setTimeRange}
              simulation={simulation}
              setSimulation={setSimulation}
              closeEmptyDms={closeEmptyDms}
              setCloseEmptyDms={setCloseEmptyDms}
              searchQuery={searchQuery}
              setSearchQuery={setSearchQuery}
              purgeReactions={purgeReactions}
              setPurgeReactions={setPurgeReactions}
              onlyAttachments={onlyAttachments}
              setOnlyAttachments={setOnlyAttachments}
              guilds={guilds}
              channelsByGuild={channelsByGuild}
              selectedChannels={selectedChannels}
              onToggleChannel={handleToggleChannel}
              onMapAll={() => {
                const all = new Set<string>();
                channelsByGuild.forEach((cs) =>
                  cs.forEach((c) => all.add(c.id)),
                );
                setSelectedChannels(all);
              }}
              previews={previews}
              confirmText={confirmText}
              setConfirmText={setConfirmText}
              isProcessing={isProcessing}
              onStartAction={startAction}
            />
          )}{" "}
          {mode === "servers" && (
            <ServersMode
              guilds={guilds}
              selectedGuildsToLeave={selectedGuildsToLeave}
              onToggleGuildToLeave={(id) => {
                const next = new Set(selectedGuildsToLeave);
                if (next.has(id)) next.delete(id);
                else next.add(id);
                setSelectedGuildsToLeave(next);
              }}
              onSelectAllNodes={() =>
                setSelectedGuildsToLeave(new Set(guilds?.map((g) => g.id)))
              }
              confirmText={confirmText}
              setConfirmText={setConfirmText}
              isProcessing={isProcessing}
              onStartAction={startAction}
              selectedGuilds={selectedGuilds}
              channelsByGuild={channelsByGuild}
              selectedChannels={selectedChannels}
              onToggleChannelForAudit={(id) =>
                setSelectedChannels(new Set([id]))
              }
              onBuryAuditLog={handleBuryAuditLog}
              onWebhookGhosting={handleWebhookGhosting}
              isLoading={isLoading}
            />
          )}
          {mode === "identity" && (
            <IdentityMode
              relationships={relationships}
              selectedRelationships={selectedRelationships}
              onToggleRelationship={(id) => {
                const next = new Set(selectedRelationships);
                if (next.has(id)) next.delete(id);
                else next.add(id);
                setSelectedRelationships(next);
              }}
              onMapAllLinks={() =>
                setSelectedRelationships(
                  new Set(relationships?.map((r) => r.id)),
                )
              }
              confirmText={confirmText}
              setConfirmText={setConfirmText}
              isProcessing={isProcessing}
              onStartAction={startAction}
            />
          )}
        </motion.div>
      </AnimatePresence>
    </main>
  </div>
);
