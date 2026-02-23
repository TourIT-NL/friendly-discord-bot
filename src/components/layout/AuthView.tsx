import { motion, AnimatePresence } from "framer-motion";
import { UserManual } from "../UserManual";
import { LoginSelection } from "../auth/LoginSelection";
import { SetupView } from "../auth/SetupView";
import { QRView } from "../auth/QRView";
import { TokenView } from "../auth/TokenView";

interface AuthViewProps {
  view: "manual" | "auth" | "setup" | "qr" | "token" | "dashboard";
  setView: (
    view: "manual" | "auth" | "setup" | "qr" | "token" | "dashboard",
  ) => void;
  isAuthenticated: boolean;
  discordStatus: any;
  isLoading: boolean;
  qrUrl: string | null;
  qrScanned: boolean;
  clientId: string;
  setClientId: (id: string) => void;
  clientSecret: string;
  setClientSecret: (secret: string) => void;
  manualToken: string;
  setManualToken: (token: string) => void;
  handleLoginRPC: () => void;
  handleLoginQR: () => void;
  handleLoginOAuth: () => void;
  handleCancelQR: () => void;
  handleLoginToken: (e?: React.FormEvent) => Promise<void>;
  handleSaveConfig: (e?: React.FormEvent) => Promise<void>;
}

export const AuthView = ({
  view,
  setView,
  isAuthenticated,
  discordStatus,
  isLoading,
  qrUrl,
  qrScanned,
  clientId,
  setClientId,
  clientSecret,
  setClientSecret,
  manualToken,
  setManualToken,
  handleLoginRPC,
  handleLoginQR,
  handleLoginOAuth,
  handleCancelQR,
  handleLoginToken,
  handleSaveConfig,
}: AuthViewProps) => (
  <div
    key="auth-wrapper"
    className="min-h-screen flex flex-col items-center justify-center p-10 bg-[#0a0a0a] relative overflow-hidden"
  >
    <div className="absolute inset-0 bg-m3-primary/5 pointer-events-none blur-[150px] rounded-full scale-150" />
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      className="flex flex-col items-center gap-16 w-full max-w-5xl relative z-10"
    >
      <div className="text-center space-y-4">
        <motion.h1
          layoutId="title"
          className="text-7xl font-black tracking-tighter text-white uppercase italic leading-none shadow-[0_0_30px_rgba(255,255,255,0.1)]"
        >
          Discord Purge
        </motion.h1>
        <p className="text-xs text-m3-primary font-bold uppercase tracking-[0.8em] flex items-center justify-center gap-4 opacity-60">
          <span className="w-12 h-px bg-m3-primary/40" />
          Privacy Enforcement Unit v1.2.0
          <span className="w-12 h-px bg-m3-primary/40" />
        </p>
      </div>
      <AnimatePresence mode="wait">
        {view === "manual" && (
          <UserManual
            key="manual"
            onComplete={() =>
              isAuthenticated ? setView("dashboard") : setView("auth")
            }
          />
        )}
        {view === "auth" && (
          <LoginSelection
            key="auth"
            discordStatus={discordStatus}
            isLoading={isLoading}
            onLoginRPC={handleLoginRPC}
            onLoginQR={handleLoginQR}
            onLoginOAuth={handleLoginOAuth}
            onSwitchToSetup={() => setView("setup")}
            onSwitchToToken={() => setView("token")}
          />
        )}
        {view === "setup" && (
          <SetupView
            key="setup"
            clientId={clientId}
            setClientId={setClientId}
            clientSecret={clientSecret}
            setClientSecret={setClientSecret}
            isLoading={isLoading}
            onBack={() => setView("auth")}
            onSubmit={handleSaveConfig}
          />
        )}
        {view === "qr" && (
          <QRView
            key="qr"
            qrUrl={qrUrl}
            qrScanned={qrScanned}
            onBack={handleCancelQR}
          />
        )}
        {view === "token" && (
          <TokenView
            key="token"
            manualToken={manualToken}
            setManualToken={setManualToken}
            isLoading={isLoading}
            onBack={() => setView("auth")}
            onSubmit={handleLoginToken}
          />
        )}
      </AnimatePresence>
    </motion.div>
  </div>
);
