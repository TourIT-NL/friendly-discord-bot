import { useEffect } from "react";
import { useAuthStore } from "./store/authStore";
import { AnimatePresence } from "framer-motion";

import { AuthView } from "./components/layout/AuthView";
import { DashboardView } from "./components/layout/DashboardView";
import { ErrorOverlay } from "./components/layout/ErrorOverlay";
import { OperationOverlay } from "./components/dashboard/OperationOverlay";
import { DeveloperLog } from "./components/dashboard/DeveloperLog";
import { GlobalListeners } from "./components/system/GlobalListeners";

import { useDiscordAuth } from "./hooks/useDiscordAuth";
import { useDiscordOperations } from "./hooks/useDiscordOperations";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const {
    isAuthenticated,
    view,
    isLoading,
    error,
    setError,
    showDevLog,
    toggleDevLog,
    setAuthenticated,
    setUnauthenticated,
    setView,
    setLoading,
  } = useAuthStore();

  const { checkStatus, fetchIdentities, handleApiError } = useDiscordAuth();
  const {
    isProcessing,
    isComplete,
    resetProcessing,
    progress,
    operationStatus,
    getOperationStatus,
    handlePause,
    handleResume,
    handleAbort,
    mode,
  } = useDiscordOperations(handleApiError);

  // --- Global Setup Effects ---
  useEffect(() => {
    const restoreSession = async () => {
      setLoading(true);
      try {
        const user = await invoke("get_current_user");
        // @ts-expect-error: The backend returns a DiscordUser object, but the type is not explicitly defined in the invoke call
        setAuthenticated(user);
      } catch (err) {
        console.log("No active session to restore.");
        setUnauthenticated();
        // Force back to manual or auth view if session restoration failed
        if (view === "dashboard") {
          setView("auth");
        }
      } finally {
        setLoading(false);
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
  }, [
    checkStatus,
    fetchIdentities,
    getOperationStatus,
    setAuthenticated,
    setUnauthenticated,
    setView,
    setLoading,
  ]);

  if (isLoading && !isAuthenticated) {
    return (
      <div className="w-screen h-screen bg-[#0a0a0a] flex items-center justify-center">
        <div className="text-white text-lg font-bold animate-pulse">
          Initializing Engine...
        </div>
      </div>
    );
  }

  return (
    <div className="w-full h-full">
      <GlobalListeners />
      <AnimatePresence mode="wait">
        {view !== "dashboard" || !isAuthenticated ? (
          <AuthView />
        ) : (
          <DashboardView />
        )}
      </AnimatePresence>
      <OperationOverlay
        isLoading={isProcessing} // Use isProcessing from operations hook
        operationStatus={operationStatus}
        progress={progress}
        isComplete={isComplete}
        onReset={resetProcessing}
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
