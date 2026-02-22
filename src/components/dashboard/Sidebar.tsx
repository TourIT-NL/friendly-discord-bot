import React from "react";
import { motion } from "framer-motion";
import {
  Users,
  Server,
  MessageSquare,
  Plus,
  Ghost,
  LogOut,
  Terminal,
} from "lucide-react";
import { SectionLabel } from "../common/M3Components";
import { DiscordIdentity, Guild, DiscordUser } from "../../types/discord";
import { useAuthStore } from "../../store/authStore";

interface SidebarProps {
  user: DiscordUser | null;
  identities: DiscordIdentity[];
  guilds: Guild[] | null;
  selectedGuilds: Set<string>;
  onSwitchIdentity: (id: string) => void;
  onNewIdentity: () => void;
  onToggleGuildSelection: (guild: Guild | null) => void;
  onStealthWipe: () => void;
  onNitroWipe: () => void;
  onLogout: () => void;
}

export const Sidebar = ({
  user,
  identities,
  guilds,
  selectedGuilds,
  onSwitchIdentity,
  onNewIdentity,
  onToggleGuildSelection,
  onStealthWipe,
  onNitroWipe,
  onLogout,
}: SidebarProps) => {
  const { showDevLog, toggleDevLog } = useAuthStore();

  return (
    <aside className="w-80 flex flex-col gap-8">
      <div className="flex flex-col gap-4 flex-1">
        <SectionLabel>
          <Users className="w-3.5 h-3.5" /> Identities
        </SectionLabel>
        <div className="flex flex-col gap-2 p-2 bg-black/20 rounded-m3-xl border border-m3-outlineVariant/20">
          {identities.map((id) => (
            <button
              key={id.id}
              onClick={() => onSwitchIdentity(id.id)}
              className={`flex items-center gap-3 p-3 rounded-m3-lg transition-all text-left ${user?.id === id.id ? "bg-m3-primaryContainer text-m3-onPrimaryContainer" : "hover:bg-m3-surfaceVariant/40 text-m3-onSurfaceVariant"}`}
            >
              <div className="w-8 h-8 rounded-full bg-m3-secondaryContainer flex items-center justify-center font-black text-xs uppercase">
                {id.username[0]}
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-[11px] font-black truncate uppercase italic">
                  {id.username}
                </p>
                <p className="text-[8px] opacity-50 uppercase tracking-widest">
                  {id.is_oauth ? "OFFICIAL" : "BYPASS"}
                </p>
              </div>
              {user?.id === id.id && (
                <div className="w-1.5 h-1.5 rounded-full bg-m3-primary animate-pulse" />
              )}
            </button>
          ))}
          <button
            onClick={onNewIdentity}
            className="flex items-center gap-3 p-3 rounded-m3-lg hover:bg-m3-surfaceVariant/40 text-m3-onSurfaceVariant border border-dashed border-m3-outlineVariant/40"
          >
            <Plus className="w-4 h-4" />
            <span className="text-[10px] font-black uppercase tracking-widest">
              New Protocol
            </span>
          </button>
        </div>

        <div className="flex items-center justify-between px-2">
          <SectionLabel>
            <Server className="w-3.5 h-3.5" /> Source Handshakes
          </SectionLabel>
          <div className="flex gap-2 mb-4">
            <button
              onClick={() =>
                guilds?.forEach(
                  (g) => !selectedGuilds.has(g.id) && onToggleGuildSelection(g),
                )
              }
              className="text-[9px] font-black text-m3-primary uppercase hover:underline"
            >
              All
            </button>
            <span className="text-white/10 text-[9px]">|</span>
            <button
              onClick={() => {
                selectedGuilds.forEach((id) => {
                  if (id === "dms") onToggleGuildSelection(null);
                  else {
                    const g = guilds?.find((guild) => guild.id === id);
                    if (g) onToggleGuildSelection(g);
                  }
                });
              }}
              className="text-[9px] font-black text-m3-outline uppercase hover:underline"
            >
              Clear
            </button>
          </div>
        </div>
        <div className="m3-card !p-2 max-h-[calc(100vh-520px)] overflow-y-auto custom-scrollbar flex flex-col gap-1.5 shadow-inner bg-black/20 border-m3-outlineVariant/20">
          {" "}
          <button
            onClick={() => onToggleGuildSelection(null)}
            className={`flex items-center gap-4 p-4 rounded-m3-xl transition-all text-left relative group ${selectedGuilds.has("dms") ? "bg-m3-primaryContainer text-m3-onPrimaryContainer shadow-lg" : "hover:bg-m3-surfaceVariant/40 text-m3-onSurfaceVariant"}`}
          >
            <div className="relative">
              <div className="w-10 h-10 rounded-m3-md bg-m3-tertiaryContainer text-m3-onTertiaryContainer flex items-center justify-center font-black text-sm border border-white/5 shadow-md">
                <MessageSquare className="w-5 h-5" />
              </div>
              {selectedGuilds.has("dms") && (
                <motion.div
                  layoutId="pulse-active"
                  className="absolute -inset-1 rounded-m3-lg border border-m3-primary animate-pulse"
                />
              )}
            </div>
            <div className="flex-1 min-w-0">
              <span className="text-[13px] font-black truncate block uppercase italic tracking-tight">
                Direct Messages
              </span>
              <p className="text-[9px] opacity-50 font-bold uppercase tracking-widest mt-0.5">
                Private Buffers
              </p>
            </div>
          </button>
          <div className="h-px bg-white/5 my-2 mx-4" />
          {guilds?.map((g) => (
            <button
              key={g.id}
              onClick={() => onToggleGuildSelection(g)}
              className={`flex items-center gap-4 p-4 rounded-m3-xl transition-all text-left relative group ${selectedGuilds.has(g.id) ? "bg-m3-primaryContainer text-m3-onPrimaryContainer shadow-lg" : "hover:bg-m3-surfaceVariant/40 text-m3-onSurfaceVariant"}`}
            >
              <div className="relative">
                {g.icon ? (
                  <img
                    src={`https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png`}
                    className="w-10 h-10 rounded-m3-md shadow-md border border-white/5"
                  />
                ) : (
                  <div className="w-10 h-10 rounded-m3-md bg-m3-secondaryContainer text-m3-onSecondaryContainer flex items-center justify-center font-black text-sm border border-white/5 uppercase">
                    {g.name[0]}
                  </div>
                )}
                {selectedGuilds.has(g.id) && (
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
                <p className="text-[9px] opacity-50 font-bold uppercase tracking-widest mt-0.5">
                  Stream Ready
                </p>
              </div>
            </button>
          ))}
        </div>
      </div>

      <div className="mt-auto space-y-4">
        <button
          onClick={toggleDevLog}
          className={`w-full flex items-center justify-center gap-3 p-4 rounded-m3-xl transition-all border font-black uppercase tracking-widest text-[10px] italic ${showDevLog ? "bg-m3-primary/20 text-m3-primary border-m3-primary/40" : "bg-white/5 text-m3-onSurfaceVariant border-white/10 hover:bg-white/10"}`}
        >
          <Terminal className="w-4 h-4" /> System Protocol Log
        </button>
        <button
          onClick={onStealthWipe}
          className="w-full flex items-center justify-center gap-3 p-4 rounded-m3-xl bg-m3-secondaryContainer/10 text-m3-secondary hover:bg-m3-secondaryContainer/20 transition-all border border-m3-secondary/20 font-black uppercase tracking-widest text-[10px] italic"
        >
          <Ghost className="w-4 h-4" /> Stealth Profile Wipe
        </button>
        <button
          onClick={onNitroWipe}
          className="w-full flex items-center justify-center gap-3 p-4 rounded-m3-xl bg-m3-secondaryContainer/10 text-m3-secondary hover:bg-m3-secondaryContainer/20 transition-all border border-m3-secondary/20 font-black uppercase tracking-widest text-[10px] italic"
        >
          <Ghost className="w-4 h-4" /> Nitro Stealth Wipe
        </button>
        <button
          onClick={onLogout}
          className="w-full flex items-center justify-center gap-3 p-4 rounded-m3-xl bg-m3-errorContainer/10 text-m3-error hover:bg-m3-errorContainer/20 transition-all border border-m3-error/20 font-black uppercase tracking-widest text-[10px] italic"
        >
          <LogOut className="w-4 h-4" /> Terminate Session
        </button>
      </div>
    </aside>
  );
};
