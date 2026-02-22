import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAuthStore } from "./store/authStore";
import { AnimatePresence } from "framer-motion";

import { DiscordUser, Progress } from "./types/discord";
import { AuthView } from "./components/layout/AuthView";
import { DashboardView } from "./components/layout/DashboardView";
import { ErrorOverlay } from "./components/layout/ErrorOverlay";
import { OperationOverlay } from "./components/dashboard/OperationOverlay";
import { DeveloperLog } from "./components/dashboard/DeveloperLog";

import { useDiscordAuth } from "./hooks/useDiscordAuth";
import { useDiscordOperations } from "./hooks/useDiscordOperations";
import { invoke } from "@tauri-apps/api/core";

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
    setSelectedGuilds,
    channelsByGuild,
    setChannelsByGuild,
    relationships,
    setRelationships,
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
    closeEmptyDms,
    setCloseEmptyDms,
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
        await listen<{
          user_message: string;
          error_code: string;
          technical_details?: string;
        }>("tauri://error", (event) => {
          const { error_code, user_message } = event.payload;
          if (error_code === "vault_credentials_missing") {
            setError(user_message);
            setView("setup");
          } else if (error_code === "no_active_session") {
            setError(user_message);
            setView("auth");
          } else {
            setError(user_message);
          }
        }),
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
          <AuthView
            view={view}
            setView={setView}
            isAuthenticated={isAuthenticated}
            discordStatus={discordStatus}
            isLoading={isLoading}
            qrUrl={qrUrl}
            qrScanned={qrScanned}
            clientId={clientId}
            setClientId={setClientId}
            clientSecret={clientSecret}
            setClientSecret={setClientSecret}
            manualToken={manualToken}
            setManualToken={setManualToken}
            handleLoginRPC={handleLoginRPC}
            handleLoginQR={handleLoginQR}
            handleLoginOAuth={handleLoginOAuth}
            handleCancelQR={handleCancelQR}
            handleLoginToken={handleLoginToken}
            handleSaveConfig={handleSaveConfig}
          />
        ) : (
          <DashboardView
            user={user}
            identities={identities}
            guilds={guilds}
            selectedGuilds={selectedGuilds}
            handleSwitchIdentity={handleSwitchIdentity}
            setView={setView}
            handleToggleGuildSelection={handleToggleGuildSelection}
            handleStealthWipe={handleStealthWipe}
            handleNitroWipe={handleNitroWipe}
            handleLogout={handleLogout}
            mode={mode}
            setAppMode={setAppMode}
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
            channelsByGuild={channelsByGuild}
            selectedChannels={selectedChannels}
            handleToggleChannel={handleToggleChannel}
            setSelectedChannels={setSelectedChannels}
            previews={previews}
            confirmText={confirmText}
            setConfirmText={setConfirmText}
            isProcessing={isProcessing}
            startAction={startAction}
            selectedGuildsToLeave={selectedGuildsToLeave}
            setSelectedGuildsToLeave={setSelectedGuildsToLeave}
            handleBuryAuditLog={handleBuryAuditLog}
            handleWebhookGhosting={handleWebhookGhosting}
            isLoading={isLoading}
            relationships={relationships}
            selectedRelationships={selectedRelationships}
            setSelectedRelationships={setSelectedRelationships}
          />
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
      <ErrorOverlay
        error={error}
        setError={setError}
        showDevLog={showDevLog}
        toggleDevLog={toggleDevLog}
      />
    </div>
  );
}

export default App;
