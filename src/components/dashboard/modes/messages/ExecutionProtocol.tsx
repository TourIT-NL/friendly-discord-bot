import { Settings, ShieldAlert, Play, Trash2 } from "lucide-react";
import { SectionLabel } from "../../../common/M3Components";

interface ExecutionProtocolProps {
  simulation: boolean;
  selectedChannels: Set<string>;
  channelsByGuild: Map<string, any[]>;
  confirmText: string;
  setConfirmText: (text: string) => void;
  isProcessing: boolean;
  onStartAction: () => void;
}

export const ExecutionProtocol = ({
  simulation,
  selectedChannels,
  channelsByGuild,
  confirmText,
  setConfirmText,
  isProcessing,
  onStartAction,
}: ExecutionProtocolProps) => (
  <div className="flex flex-col gap-10">
    <SectionLabel>
      <Settings className="w-3.5 h-3.5" /> Execution Protocol
    </SectionLabel>
    <div className="bg-m3-errorContainer/5 border border-m3-errorContainer/20 rounded-m3-xl p-8 flex-1 flex flex-col items-center justify-center text-center gap-8 shadow-inner">
      <div className="p-6 rounded-full bg-m3-errorContainer/10 border border-m3-error/20">
        <ShieldAlert className="w-12 h-12 text-m3-error drop-shadow-[0_0_15px_rgba(242,184,181,0.4)]" />
      </div>
      <div>
        <h4 className="text-2xl font-black italic uppercase text-m3-error tracking-tight">
          {simulation ? "Simulation Run" : "Security Required"}
        </h4>
        <p className="text-[10px] text-m3-onSurfaceVariant font-bold uppercase tracking-widest mt-2 px-10 leading-relaxed">
          Authorized for{" "}
          <span className="text-white underline decoration-m3-error decoration-2 underline-offset-4">
            {selectedChannels.size} buffers
          </span>{" "}
          across{" "}
          <span className="text-white">{channelsByGuild.size} sources</span>.{" "}
          {simulation
            ? "No data will be destroyed."
            : "Permanent purge protocol."}
        </p>
      </div>
      <div className="w-full space-y-4">
        <p className="text-[9px] font-black text-m3-error uppercase tracking-[0.4em] italic">
          Auth Signature: "DELETE"
        </p>
        <input
          type="text"
          value={confirmText}
          onChange={(e) => setConfirmText(e.target.value.toUpperCase())}
          className="w-full bg-black/60 border-2 border-m3-error/30 focus:border-m3-error rounded-m3-xl p-6 text-center text-m3-error font-mono text-3xl font-black tracking-[0.8em] outline-none transition-all shadow-inner uppercase"
          placeholder="••••"
        />
      </div>
    </div>
    <button
      disabled={
        selectedChannels.size === 0 || confirmText !== "DELETE" || isProcessing
      }
      onClick={onStartAction}
      className={`m3-button-primary !py-8 !text-base shadow-2xl active:scale-[0.98] !rounded-m3-xl ${simulation ? "!bg-m3-secondary !text-m3-onSecondary" : "!bg-m3-error !text-m3-onError"}`}
    >
      {simulation ? (
        <Play className="w-6 h-6" />
      ) : (
        <Trash2 className="w-6 h-6" />
      )}
      {simulation ? "Start Safety Simulation" : "Execute Destructive Purge"}
    </button>
  </div>
);
