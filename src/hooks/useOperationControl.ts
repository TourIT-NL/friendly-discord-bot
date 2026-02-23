import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { OperationStatus, Progress } from "../types/discord";

export const useOperationControl = () => {
  const [isProcessing, setIsProcessing] = useState(false);
  const [isComplete, setIsComplete] = useState(false);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [operationStatus, setOperationStatus] = useState<OperationStatus>({
    is_running: false,
    is_paused: false,
    should_abort: false,
  });

  const getOperationStatus = useCallback(async () => {
    try {
      const status = await invoke<OperationStatus>("get_operation_status");
      setOperationStatus(status);

      // Auto-reset isComplete if a new operation starts
      if (status.is_running && isComplete) {
        setIsComplete(false);
      }
    } catch (err) {
      console.error("Failed to get op status:", err);
    }
  }, [isComplete]);

  const handlePause = async () => {
    await invoke("pause_operation");
    getOperationStatus();
  };
  const handleResume = async () => {
    await invoke("resume_operation");
    getOperationStatus();
  };
  const handleAbort = async () => {
    await invoke("abort_operation");
    getOperationStatus();
    setIsProcessing(false);
    setIsComplete(false);
    setProgress(null);
  };

  const resetProcessing = useCallback(() => {
    setIsProcessing(false);
    setIsComplete(false);
    setProgress(null);
  }, []);

  return {
    isProcessing,
    setIsProcessing,
    isComplete,
    setIsComplete,
    progress,
    setProgress,
    operationStatus,
    setOperationStatus,
    getOperationStatus,
    handlePause,
    handleResume,
    handleAbort,
    resetProcessing,
  };
};
