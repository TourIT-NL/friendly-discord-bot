import React, { useEffect } from "react";
import {
  CreditCard,
  CheckCircle2,
  ShieldX,
  Package,
  Trash2,
  ArrowUpRight,
  UserX,
  Zap,
} from "lucide-react";
import { SectionLabel } from "../../common/M3Components";

interface AccountModeProps {
  info: any;
  fetchAudit: () => void;
  onOpenDiscordUrl: (type: string) => void;
  onSetHypesquad: (id: number) => void;
  onGhostProfile: () => void;
}

export const AccountMode = ({
  info,
  fetchAudit,
  onOpenDiscordUrl,
  onSetHypesquad,
  onGhostProfile,
}: AccountModeProps) => {
  useEffect(() => {
    fetchAudit();
  }, [fetchAudit]);

  const subscriptions = info?.subscriptions || [];
  const paymentSources = info?.paymentSources || [];

  return (
    <div className="flex-1 flex flex-col gap-8 overflow-y-auto custom-scrollbar pr-2">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <h3 className="text-2xl font-black italic uppercase text-white tracking-tighter">
            Account Integrity & Footprint
          </h3>
          <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest opacity-60">
            Manage profile metadata and financial data
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={onGhostProfile}
            className="m3-button-primary !bg-m3-error/10 !text-m3-error border border-m3-error/20 hover:!bg-m3-error/20 !px-6"
          >
            <UserX className="w-4 h-4" /> Ghost Profile
          </button>
          <button
            onClick={() => onOpenDiscordUrl("account_deletion")}
            className="m3-button-primary !bg-m3-error/10 !text-m3-error border border-m3-error/20 hover:!bg-m3-error/20 !px-6"
          >
            <Trash2 className="w-4 h-4" /> Purge Account
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-8">
        <div className="space-y-4">
          <SectionLabel>Hypesquad Affiliation</SectionLabel>
          <div className="flex gap-4">
            {[
              { id: 1, name: "Bravery", color: "text-purple-400" },
              { id: 2, name: "Brilliance", color: "text-red-400" },
              { id: 3, name: "Balance", color: "text-teal-400" },
              { id: 0, name: "Leave", color: "text-gray-400" },
            ].map((house) => (
              <button
                key={house.id}
                onClick={() => onSetHypesquad(house.id)}
                className={`flex-1 p-4 rounded-m3-lg bg-white/5 border border-white/10 hover:bg-white/10 transition-colors flex flex-col items-center gap-2 ${house.color}`}
              >
                <Zap className="w-5 h-5" />
                <span className="text-[10px] font-bold uppercase tracking-tighter text-white">
                  {house.name}
                </span>
              </button>
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <SectionLabel>Active Subscriptions</SectionLabel>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {subscriptions.length > 0 ? (
              subscriptions.map((sub: any) => (
                <div
                  key={sub.id}
                  className="m3-card !bg-m3-primaryContainer/10 border-m3-primary/20 p-6 flex items-center gap-6"
                >
                  <div className="w-12 h-12 rounded-full bg-m3-primary/20 flex items-center justify-center">
                    <Package className="w-6 h-6 text-m3-primary" />
                  </div>
                  <div>
                    <h4 className="font-black text-white uppercase italic">
                      Nitro {sub.type === 1 ? "Classic" : "Premium"}
                    </h4>
                    <p className="text-[9px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest">
                      Status: {sub.status === 1 ? "Active" : "Canceled"}
                    </p>
                  </div>
                </div>
              ))
            ) : (
              <p className="text-[10px] text-m3-onSurfaceVariant/40 uppercase font-black italic p-4">
                No active subscriptions detected.
              </p>
            )}
          </div>
        </div>

        <div className="space-y-4">
          <SectionLabel>Stored Payment Sources</SectionLabel>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {paymentSources.map((source: any) => (
              <div
                key={source.id}
                className="m3-card !bg-black/20 border-m3-outlineVariant/20 p-6 flex flex-col gap-4"
              >
                <div className="flex items-center gap-3">
                  <CreditCard className="w-5 h-5 text-m3-tertiary" />
                  <span className="font-black text-white text-sm uppercase tracking-tighter">
                    •••• {source.last_4}
                  </span>
                </div>
                <div className="flex justify-between items-center mt-2">
                  <span className="text-[8px] font-black text-m3-onSurfaceVariant uppercase">
                    {source.brand}
                  </span>
                  {source.invalid && (
                    <ShieldX className="w-4 h-4 text-m3-error" />
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
