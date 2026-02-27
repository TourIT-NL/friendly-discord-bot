// src-tauri/src/api/discord/ops/ghost.rs

use crate::api::discord::privacy::ghost_profile;
use crate::core::error::AppError;
use crate::core::op_manager::{Operation, OperationState};
use std::sync::Arc;
use tauri::AppHandle;

pub struct GhostProfileOperation;

#[async_trait::async_trait]
impl Operation for GhostProfileOperation {
    fn id(&self) -> &str {
        "ghost_profile"
    }
    fn name(&self) -> &str {
        "Profile Anonymization"
    }

    async fn run(&self, app: AppHandle, state: Arc<OperationState>) -> Result<(), AppError> {
        state.prepare();
        state
            .is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let result = ghost_profile(app).await;

        state.reset();
        result
    }
}
