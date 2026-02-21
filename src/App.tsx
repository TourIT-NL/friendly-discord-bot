import React, { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAuthStore } from "./store/authStore";
import { motion, AnimatePresence } from "framer-motion";
import { ShieldAlert, XCircle, Server, HelpCircle } from "lucide-react";

import { DiscordUser, Progress } from "./types/discord";
import { IconButton } from "./components/common/M3Components";
import { UserManual } from "./components/UserManual";
import { LoginSelection } from "./components/auth/LoginSelection";
import { SetupView } from "./components/auth/SetupView";
import { QRView } from "./components/auth/QRView";
import { TokenView } from "./components/auth/TokenView";
import { Sidebar } from "./components/dashboard/Sidebar";
import { MessagesMode } from "./components/dashboard/modes/MessagesMode";
import { ServersMode } from "./components/dashboard/modes/ServersMode";
import { IdentityMode } from "./components/dashboard/modes/IdentityMode";
import { OperationOverlay } from "./components/dashboard/OperationOverlay";
import { DeveloperLog } from "./components/dashboard/DeveloperLog";

import { useDiscordAuth } from "./hooks/useDiscordAuth";
import { useDiscordOperations } from "./hooks/useDiscordOperations";

function App() {
  const {
    isAuthenticated,
    user,
    guilds,
    isLoading,
    error,
    setAuthenticated,
    setError,
    addLog,
    showDevLog,
    toggleDevLog,
  } = useAuthStore();

  const {
    view,
    setView,
    identities,
    discordStatus,
    qrUrl,
    setQrUrl,
    qrScanned,
    clientId,
    setClientId,
    clientSecret,
    setClientSecret,
    manualToken,
    setManualToken,
    checkStatus,
    fetchIdentities,
    handleLogout,
    handleLoginOAuth,
    handleLoginQR,
    handleCancelQR,
    handleLoginRPC,
    handleLoginToken,
    handleSaveConfig,
    handleSwitchIdentity,
    handleApiError,
  } = useDiscordAuth();

  const {
    mode,
    setAppMode,
    selectedGuilds,
    channelsByGuild,
    relationships,
    previews,
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
    operationStatus,
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
  } = useDiscordOperations(handleApiError);

  // --- Global Listeners ---
  useEffect(() => {
    const restoreSession = async () => {
      try {
        await invoke("get_current_user");
      } catch (err) {
        console.log("No active session to restore.");
      }
    };
    restoreSession();
    checkStatus();
    fetchIdentities();
    const interval = setInterval(checkStatus, 5000);
    const opInterval = setInterval(getOperationStatus, 1000);
    return () => {
      clearInterval(interval);
      clearInterval(opInterval);
    };
  }, [checkStatus, fetchIdentities, getOperationStatus]);

  useEffect(() => {
    const unlisteners: any[] = [];
    const setup = async () => {
      unlisteners.push(
        await listen("auth_success", (event) => {
          setAuthenticated(event.payload as DiscordUser);
          setView("dashboard");
          fetchGuilds();
          fetchIdentities();
        }),
      );
      unlisteners.push(
        await listen<string>("qr_code_ready", (event) => {
          setQrUrl(event.payload);
          useAuthStore.getState().setLoading(false);
        }),
      );
      unlisteners.push(
        await listen<{ level: any; message: string; metadata: any }>(
          "log_event",
          (event) => {
            addLog(
              event.payload.level,
              event.payload.message,
              event.payload.metadata,
            );
          },
        ),
      );
      unlisteners.push(
        await listen("deletion_progress", (event) =>
          setProgress(event.payload as Progress),
        ),
      );
      unlisteners.push(
        await listen("deletion_complete", () => {
          setIsProcessing(false);
          setProgress(null);
          fetchGuilds();
          getOperationStatus();
        }),
      );
      unlisteners.push(
        await listen("leave_progress", (event) =>
          setProgress(event.payload as Progress),
        ),
      );
      unlisteners.push(
        await listen("leave_complete", () => {
          setIsProcessing(false);
          setProgress(null);
          fetchGuilds();
          getOperationStatus();
        }),
      );
      unlisteners.push(
        await listen("relationship_progress", (event) =>
          setProgress(event.payload as Progress),
        ),
      );
      unlisteners.push(
        await listen("relationship_complete", () => {
          setIsProcessing(false);
          setProgress(null);
          fetchRelationships();
          getOperationStatus();
        }),
      );
      unlisteners.push(
        await listen("audit_log_progress", (event) =>
          setProgress(event.payload as Progress),
        ),
      );
      unlisteners.push(
        await listen("audit_log_complete", () => {
          setIsProcessing(false);
          setProgress(null);
          getOperationStatus();
          setError("Audit Log burial complete.");
        }),
      );
      unlisteners.push(
        await listen("webhook_progress", (event) =>
          setProgress(event.payload as Progress),
        ),
      );
      unlisteners.push(
        await listen("webhook_complete", () => {
          setIsProcessing(false);
          setProgress(null);
          getOperationStatus();
          setError("Webhook Ghosting complete.");
        }),
      );
    };
    setup();
    return () => unlisteners.forEach((u) => u && u());
  }, [
    setAuthenticated,
    fetchGuilds,
    fetchRelationships,
    fetchIdentities,
    getOperationStatus,
    setView,
    setIsProcessing,
    setProgress,
    setError,
    addLog,
    setQrUrl,
  ]);

  useEffect(() => {
    if (mode === "identity") fetchRelationships();
  }, [mode, fetchRelationships]);

  return (
    <div className="w-full h-full">
      <AnimatePresence mode="wait">
        {view !== "dashboard" || !isAuthenticated ? (
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
                  <div className="w-12 h-px bg-m3-primary/40" />
                  Privacy Enforcement Unit v5.1
                  <div className="w-12 h-px bg-m3-primary/40" />
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
        ) : (
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
                        onClick={() => setAppMode("messages")}
                        className={`px-8 py-2.5 rounded-m3-full text-[10px] font-black uppercase tracking-widest transition-all ${mode === "messages" ? "bg-m3-primary text-m3-onPrimary" : "text-m3-onSurfaceVariant"}`}
                      >
                        Messages
                      </button>
                      {selectedGuilds.size > 0 && (
                        <button
                          onClick={() => setAppMode("servers")}
                          className={`px-8 py-2.5 rounded-m3-full text-[10px] font-black uppercase tracking-widest transition-all ${mode === "servers" ? "bg-m3-primary text-m3-onPrimary" : "text-m3-onSurfaceVariant"}`}
                        >
                          Servers
                        </button>
                      )}
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
                  )}
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
                        setSelectedGuildsToLeave(
                          new Set(guilds?.map((g) => g.id)),
                        )
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
        )}
      </AnimatePresence>
      <OperationOverlay
        isLoading={isLoading}
        operationStatus={operationStatus}
        progress={progress}
        mode={mode}
        onPause={handlePause}
        onResume={handleResume}
        onAbort={handleAbort}
      />
      <DeveloperLog />
      <AnimatePresence>
        {error && (
          <motion.div
            initial={{ opacity: 0, y: 100 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 50 }}
            className="fixed bottom-12 left-1/2 -translate-x-1/2 z-[600] w-full max-w-xl px-10"
          >
            <div className="m3-card-elevated !bg-m3-errorContainer !text-m3-onErrorContainer !border-none flex items-center gap-6 shadow-[0_30px_100px_rgba(0,0,0,0.8)] p-8 rounded-[2.5rem] relative overflow-hidden group">
              <div className="absolute top-0 left-0 w-full h-1 bg-m3-onErrorContainer/20 group-hover:h-2 transition-all" />
              <div className="p-3 bg-m3-onErrorContainer/10 rounded-full shadow-inner">
                <ShieldAlert className="w-8 h-8 text-m3-onErrorContainer drop-shadow-[0_0_10px_rgba(96,20,16,0.4)]" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-[10px] font-black uppercase tracking-[0.4em] leading-none mb-2 italic">
                  System Alert
                </p>
                <p className="text-xs font-bold opacity-90 leading-relaxed uppercase tracking-tight">
                  {error}
                </p>
                <button
                  onClick={() => {
                    if (!showDevLog) toggleDevLog();
                  }}
                  className="mt-3 text-[9px] font-black uppercase tracking-widest bg-m3-onErrorContainer/10 hover:bg-m3-onErrorContainer/20 px-3 py-1.5 rounded-full transition-colors border border-m3-onErrorContainer/20"
                >
                  View Protocol Log
                </button>
              </div>
              <IconButton
                icon={XCircle}
                onClick={() => setError(null)}
                className="!text-m3-onErrorContainer hover:!bg-m3-onErrorContainer/10 !p-4 !rounded-[1.5rem]"
              />
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
