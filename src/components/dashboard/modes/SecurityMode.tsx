import React, { useEffect } from "react";
import { Shield, Trash2, ExternalLink } from "lucide-react";
import { SectionLabel } from "../../common/M3Components";

interface SecurityModeProps {
  apps: any[];
  fetchAudit: () => void;
  onRevoke: (id: string) => void;
  onOpenDiscordUrl: (type: string) => void;
}

export const SecurityMode = ({
  apps,
  fetchAudit,
  onRevoke,
  onOpenDiscordUrl,
}: SecurityModeProps) => {
  useEffect(() => {
    fetchAudit();
  }, [fetchAudit]);

  return (
    <div className="flex-1 flex flex-col gap-8 overflow-y-auto custom-scrollbar pr-2">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <h3 className="text-2xl font-black italic uppercase text-white tracking-tighter">
            Third-Party Breach Audit
          </h3>
          <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest opacity-60">
            Identify and shred external application access tokens
          </p>
        </div>
        <button
          onClick={() => onOpenDiscordUrl("support_portal")}
          className="m3-button-outlined !py-2 !text-[9px] !px-4"
        >
          <ExternalLink className="w-3 h-3" /> Node Support Portal
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {apps.map((app) => (
          <div
            key={app.id}
            className="m3-card !bg-black/20 border-m3-outlineVariant/20 flex flex-col gap-4 p-6 group"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-m3-lg bg-m3-secondaryContainer flex items-center justify-center">
                  <Shield className="w-6 h-6 text-m3-onSecondaryContainer" />
                </div>
                <div>
                  <h4 className="font-black text-white uppercase italic">
                    {app.application.name}
                  </h4>
                  <p className="text-[9px] text-m3-primary font-bold uppercase tracking-widest mt-1">
                    Authorized:{" "}
                    {new Date(
                      app.id / 4194304 + 1420070400000,
                    ).toLocaleDateString()}
                  </p>
                </div>
              </div>
              <button
                onClick={() => onRevoke(app.id)}
                className="p-2 rounded-m3-full hover:bg-m3-error/10 text-m3-error transition-all opacity-0 group-hover:opacity-100"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <SectionLabel>Scopes Granted</SectionLabel>
              <div className="flex flex-wrap gap-1.5">
                {app.scopes.map((scope: string) => (
                  <span
                    key={scope}
                    className="px-2 py-0.5 rounded-full bg-m3-surfaceVariant text-m3-onSurfaceVariant text-[8px] font-black uppercase tracking-tighter"
                  >
                    {scope}
                  </span>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
