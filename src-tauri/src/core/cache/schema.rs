// src-tauri/src/core/cache/schema.rs

pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS guilds (
    id TEXT,
    identity_id TEXT NOT NULL,
    name TEXT NOT NULL,
    icon TEXT,
    owner BOOLEAN NOT NULL,
    last_synced INTEGER NOT NULL,
    PRIMARY KEY (id, identity_id)
);

CREATE TABLE IF NOT EXISTS channels (
    id TEXT,
    identity_id TEXT NOT NULL,
    guild_id TEXT, -- NULL for DMs
    name TEXT,
    type INTEGER NOT NULL,
    last_synced INTEGER NOT NULL,
    PRIMARY KEY (id, identity_id),
    FOREIGN KEY(guild_id, identity_id) REFERENCES guilds(id, identity_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    has_attachments BOOLEAN NOT NULL,
    is_deleted BOOLEAN DEFAULT 0,
    FOREIGN KEY(channel_id, identity_id) REFERENCES channels(id, identity_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS discovery (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_id, identity_id);
CREATE INDEX IF NOT EXISTS idx_messages_author ON messages(author_id);
CREATE INDEX IF NOT EXISTS idx_channels_guild ON channels(guild_id, identity_id);
";
