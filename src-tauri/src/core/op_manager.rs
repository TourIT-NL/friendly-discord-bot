// src-tauri/src/core/op_manager.rs

use crate::core::error::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::sync::Notify;

/// Standardized trait for all long-running Discord operations.
#[async_trait::async_trait]
pub trait Operation: Send + Sync {
    fn id(&self) -> &str;
    #[allow(dead_code)]
    fn name(&self) -> &str;
    #[allow(dead_code)]
    async fn run(&self, app: AppHandle, state: Arc<OperationState>) -> Result<(), AppError>;
}

/// Manages the runtime state and registry of bulk operations.
pub struct OperationManager {
    pub state: Arc<OperationState>,
    pub registry: Mutex<HashMap<String, Arc<dyn Operation>>>,
}

pub struct OperationState {
    pub is_running: AtomicBool,
    pub is_paused: AtomicBool,
    pub should_abort: AtomicBool,
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
            registry: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register(&self, op: Arc<dyn Operation>) {
        let mut reg = self.registry.lock().await;
        reg.insert(op.id().to_string(), op);
    }

    #[allow(dead_code)]
    pub async fn get_operation(&self, id: &str) -> Option<Arc<dyn Operation>> {
        let reg = self.registry.lock().await;
        reg.get(id).cloned()
    }
}

impl OperationState {
    pub async fn wait_if_paused(&self) {
        while self.is_paused.load(Ordering::SeqCst) {
            self.pause_notifier.notified().await;
        }
    }

    pub fn reset(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        self.should_abort.store(false, Ordering::SeqCst);
    }

    pub fn prepare(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
        self.should_abort.store(false, Ordering::SeqCst);
    }
}
