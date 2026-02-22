import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { OperationStatus, Progress } from "../types/discord";

export const useOperationControl = () => {
  const [isProcessing, setIsProcessing] = useState(false);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [operationStatus, setOperationStatus] = useState<OperationStatus>({
    is_running: false,
    is_paused: false,
    should_abort: false,
  });

  const getOperationStatus = useCallback(async () => {
    try {
      setOperationStatus(await invoke("get_operation_status"));
    } catch (err) {
      console.error("Failed to get op status:", err);
    }
  }, []);

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
    setProgress(null);
  };

  return {
    isProcessing,
    setIsProcessing,
    progress,
    setProgress,
    operationStatus,
    setOperationStatus,
    getOperationStatus,
    handlePause,
    handleResume,
    handleAbort,
  };
};
