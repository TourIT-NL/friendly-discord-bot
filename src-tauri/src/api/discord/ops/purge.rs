// src-tauri/src/api/discord/ops/purge.rs

use crate::api::discord::bulk::messages::{PurgeOptions, bulk_delete_messages};
use crate::core::error::AppError;
use crate::core::op_manager::{Operation, OperationState};
use std::sync::Arc;
use tauri::{AppHandle, Window};

#[allow(dead_code)]
pub struct PurgeOperation {
    pub options: PurgeOptions,
    pub window: Window,
}

#[async_trait::async_trait]
impl Operation for PurgeOperation {
    fn id(&self) -> &str {
        "purge_messages"
    }
    fn name(&self) -> &str {
        "Bulk Message Deletion"
    }

    async fn run(&self, app: AppHandle, _state: Arc<OperationState>) -> Result<(), AppError> {
        // Here we link to the existing bulk_delete_messages but passing the state
        // For now, we'll just wrap the command.
        bulk_delete_messages(app, self.window.clone(), self.options.clone()).await
    }
}
