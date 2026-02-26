// src-tauri/src/api/discord/types.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relationship {
    pub id: String,
    pub nickname: Option<String>,
    pub user: serde_json::Value,
    #[serde(rename = "type")]
    pub rel_type: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationStatus {
    pub is_running: bool,
    pub is_paused: bool,
    pub should_abort: bool,
}
