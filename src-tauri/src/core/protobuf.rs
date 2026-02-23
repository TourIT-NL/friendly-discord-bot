// src-tauri/src/core/protobuf.rs

use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct PredefinedSettings {
    #[prost(message, optional, tag = "1")]
    pub privacy: Option<PrivacySettings>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PrivacySettings {
    #[prost(bool, optional, tag = "1")]
    pub allow_dms_from_guild_members: Option<bool>,
    #[prost(bool, optional, tag = "2")]
    pub contact_sync_enabled: Option<bool>,
}

pub fn encode_max_privacy() -> Vec<u8> {
    let settings = PredefinedSettings {
        privacy: Some(PrivacySettings {
            allow_dms_from_guild_members: Some(false),
            contact_sync_enabled: Some(false),
        }),
    };
    
    let mut buf = Vec::new();
    settings.encode(&mut buf).unwrap_or_default();
    buf
}
