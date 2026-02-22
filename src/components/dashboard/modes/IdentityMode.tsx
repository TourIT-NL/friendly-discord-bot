import React from "react";
import { motion } from "framer-motion";
import { Users, CheckCircle2, AlertCircle, UserMinus } from "lucide-react";
import { M3Card } from "../../common/M3Components";
import { Relationship } from "../../../types/discord";

interface IdentityModeProps {
  relationships: Relationship[] | null;
  selectedRelationships: Set<string>;
  onToggleRelationship: (id: string) => void;
  onMapAllLinks: () => void;
  confirmText: string;
  setConfirmText: (text: string) => void;
  isProcessing: boolean;
  onStartAction: () => void;
}

export const IdentityMode = ({
  relationships,
  selectedRelationships,
  onToggleRelationship,
  onMapAllLinks,
  confirmText,
  setConfirmText,
  isProcessing,
  onStartAction,
}: IdentityModeProps) => (
  <M3Card className="flex flex-col gap-10 flex-1 border-m3-tertiary/10 shadow-2xl p-10">
    <div className="flex items-center justify-between border-b border-m3-outlineVariant/30 pb-8">
      <div className="flex items-center gap-4">
        <div className="p-4 rounded-m3-lg bg-m3-tertiaryContainer text-m3-onTertiaryContainer shadow-lg">
          <Users className="w-6 h-6" />
        </div>
        <div>
          <h3 className="text-3xl font-black italic uppercase tracking-tighter text-white leading-none">
            Identity Purge
          </h3>
          <p className="text-[10px] text-m3-tertiary font-black uppercase tracking-[0.4em] mt-3">
            Bulk Relationship Severance Protocol
          </p>
        </div>
      </div>
      <div className="flex gap-4">
        <button
          onClick={onMapAllLinks}
          className="m3-button-outlined !border-m3-primary/30 !text-m3-primary !px-8 !py-3 hover:!bg-m3-primary/10"
        >
          Map All Links
        </button>
      </div>
    </div>

    <div className="m3-card !p-4 !bg-black/30 border-m3-outlineVariant/20 flex-1 overflow-y-auto custom-scrollbar min-h-[300px] shadow-inner">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {relationships?.map((r) => (
          <button
            key={r.id}
            onClick={() => onToggleRelationship(r.id)}
            className={`flex flex-col gap-4 p-5 rounded-m3-xl border-2 transition-all relative overflow-hidden items-center text-center ${selectedRelationships.has(r.id) ? "bg-m3-tertiaryContainer/20 border-m3-tertiary text-white shadow-md" : "bg-transparent border-m3-outlineVariant/20 text-m3-onSurfaceVariant hover:border-m3-outline hover:bg-m3-onSurface/5"}`}
          >
            <div className="relative">
              {r.user.avatar ? (
                <img
                  src={`https://cdn.discordapp.com/avatars/${r.user.id}/${r.user.avatar}.png`}
                  className="w-16 h-16 rounded-full border-2 border-white/10"
                />
              ) : (
                <div className="w-16 h-16 rounded-full bg-m3-secondaryContainer text-m3-onSecondaryContainer flex items-center justify-center font-black text-xl uppercase">
                  {r.user.username[0]}
                </div>
              )}
              <div
                className={`absolute -bottom-1 -right-1 w-5 h-5 rounded-full border-2 border-black ${r.rel_type === 1 ? "bg-green-500" : r.rel_type === 2 ? "bg-red-500" : "bg-yellow-500"}`}
                title={
                  r.rel_type === 1
                    ? "Friend"
                    : r.rel_type === 2
                      ? "Blocked"
                      : "Pending"
                }
              />
            </div>
            <div className="min-w-0">
              <span className="text-xs font-black truncate block uppercase italic tracking-tight">
                {r.nickname || r.user.username}
              </span>
              <p className="text-[8px] opacity-40 font-bold uppercase tracking-widest mt-1">
                ID: {r.user.id}
              </p>
            </div>
            <div
              className={`w-5 h-5 rounded-m3-xs border-2 flex items-center justify-center transition-all ${selectedRelationships.has(r.id) ? "bg-m3-tertiary border-m3-tertiary scale-110" : "border-m3-outlineVariant"}`}
            >
              {selectedRelationships.has(r.id) && (
                <CheckCircle2 className="w-3.5 h-3.5 text-m3-onTertiary" />
              )}
            </div>
          </button>
        ))}
      </div>
    </div>

    <div className="mt-auto flex flex-col lg:flex-row gap-8 items-center pt-8 border-t border-m3-outlineVariant/30 px-4">
      <div className="flex-1 flex items-center gap-6 px-6 py-5 bg-m3-tertiaryContainer/10 rounded-m3-xl border border-m3-tertiaryContainer/20 w-full lg:w-auto">
        <AlertCircle className="w-6 h-6 text-m3-tertiary" />
        <div className="flex-1 min-w-0">
          <p className="text-[10px] font-black text-m3-tertiary uppercase tracking-widest leading-none">
            Authorization Signature
          </p>
          <p className="text-[9px] text-m3-onSurfaceVariant uppercase font-bold mt-1.5 italic">
            Type "REMOVE" to finalize link termination
          </p>
        </div>
        <input
          type="text"
          value={confirmText}
          onChange={(e) => setConfirmText(e.target.value.toUpperCase())}
          className="bg-black/60 border border-m3-tertiary/30 rounded-m3-lg px-5 py-2.5 text-m3-tertiary font-mono text-xl font-black tracking-widest w-40 outline-none focus:border-m3-tertiary shadow-inner text-center uppercase"
          placeholder="••••"
        />
      </div>
      <button
        disabled={
          selectedRelationships.size === 0 ||
          confirmText !== "REMOVE" ||
          isProcessing
        }
        onClick={onStartAction}
        className="m3-button-primary !py-8 !px-12 !text-base !bg-m3-tertiary !text-m3-onTertiary shadow-2xl shadow-m3-tertiary/30 active:scale-[0.98] w-full lg:w-auto !rounded-m3-xl"
      >
        <UserMinus className="w-6 h-6" />
        Nullify {selectedRelationships.size} Identities
      </button>
    </div>
  </M3Card>
);
