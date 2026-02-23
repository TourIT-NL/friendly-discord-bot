// src-tauri/src/core/op_manager.rs

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

/// Manages the runtime state of bulk operations (purges, departures, etc.).
/// It provides thread-safe primitives for pausing, resuming, and aborting
/// long-running asynchronous tasks across the Tauri command boundaries.
pub struct OperationManager {
    pub state: Arc<OperationState>,
}

pub struct OperationState {
    /// True if an operation is actively executing its loop.
    pub is_running: AtomicBool,
    /// True if the user has requested a temporary pause.
    pub is_paused: AtomicBool,
    /// True if the user has requested an immediate termination of all remaining tasks.
    pub should_abort: AtomicBool,
    /// Notification handle used to wake up the worker thread when an operation is resumed.
    pub pause_notifier: Notify,
}

impl OperationManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(OperationState {
                is_running: AtomicBool::new(false),
                is_paused: AtomicBool::new(false),
                should_abort: AtomicBool::new(false),
                pause_notifier: Notify::new(),
            }),
        }
    }
}

impl OperationState {
    /// Blocks the current task if the `is_paused` flag is true.
    /// Used inside the bulk loops in `api/discord.rs`.
    pub async fn wait_if_paused(&self) {
        while self.is_paused.load(Ordering::SeqCst) {
            self.pause_notifier.notified().await;
        }
    }

    /// Resets all flags to their default state after an operation finishes or is aborted.
    pub fn reset(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        self.should_abort.store(false, Ordering::SeqCst);
    }

    /// Prepares the state for a new operation by clearing pause/abort flags.
    pub fn prepare(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
        self.should_abort.store(false, Ordering::SeqCst);
        // We do not set is_running to true here; the caller does that to allow for setup time.
    }
}
