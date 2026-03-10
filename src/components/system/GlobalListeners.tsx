import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAuthStore } from "../../store/authStore";
import { useDiscordAuth } from "../../hooks/useDiscordAuth";
import { useDiscordOperations } from "../../hooks/useDiscordOperations";
import { DiscordUser, Progress } from "../../types/discord";

export function GlobalListeners() {
  const { setAuthenticated, setError, addLog, setView } = useAuthStore();

  const { handleLogout, setQrUrl, handleApiError } = useDiscordAuth();
  const {
    fetchGuilds,
    fetchRelationships,
    getOperationStatus,
    setIsComplete,
    setProgress,
  } = useDiscordOperations(handleApiError);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    const setup = async () => {
      unlisteners.push(
        await listen("auth_success", (event) => {
          setAuthenticated(event.payload as DiscordUser);
          fetchGuilds(true);
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
        await listen("force_logout", () => {
          handleLogout();
        }),
      );

      const progressEvents = [
        "deletion_progress",
        "leave_progress",
        "relationship_progress",
        "audit_log_progress",
        "webhook_progress",
      ];
      for (const eventName of progressEvents) {
        unlisteners.push(
          await listen(eventName, (event) =>
            setProgress(event.payload as Progress),
          ),
        );
      }

      const completionHandlers: { [key: string]: () => void } = {
        deletion_complete: () => fetchGuilds(true),
        leave_complete: () => fetchGuilds(true),
        relationship_complete: () => fetchRelationships(),
        audit_log_complete: () => setError("Audit Log burial complete."),
        webhook_complete: () => setError("Webhook Ghosting complete."),
      };

      for (const eventName in completionHandlers) {
        unlisteners.push(
          await listen(eventName, () => {
            setIsComplete(true);
            completionHandlers[eventName]();
            getOperationStatus();
          }),
        );
      }

      unlisteners.push(
        await getCurrentWindow().onCloseRequested(async (event) => {
          console.log("Window close requested. Performing cleanup...");
          // If we had critical async cleanup, we'd do it here and use event.preventDefault()
          // But since we just want to close, we'll let it proceed.
        }),
      );
    };
    setup();
    return () => unlisteners.forEach((u) => u && u());
  }, [
    addLog,
    fetchGuilds,
    fetchRelationships,
    getOperationStatus,
    handleLogout,
    setAuthenticated,
    setIsComplete,
    setError,
    setProgress,
    setQrUrl,
    setView,
  ]);

  return null;
}
