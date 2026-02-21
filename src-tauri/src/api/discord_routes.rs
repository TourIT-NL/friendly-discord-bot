// src-tauri/src/api/discord_routes.rs

/// Represents a standardized Discord API route for rate limiting purposes.
/// This enum simplifies route identification and ensures consistent bucketing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiscordApiRoute {
    /// Generic route for unknown or unhandled endpoints.
    Default,
    /// /users/@me
    CurrentUser,
    /// /users/@me/guilds
    CurrentUserGuilds,
    /// /users/@me/channels
    CurrentUserChannels,
    /// /users/@me/relationships
    CurrentUserRelationships,
    /// /channels/{channel.id}
    Channel(String),
    /// /channels/{channel.id}/messages
    ChannelMessages(String),
    /// /channels/{channel.id}/messages/{message.id}
    ChannelMessage(String, String),
    /// /guilds/{guild.id}
    Guild(String),
    // Add more specific routes as needed
    // Example: GuildWebhooks(String),
}

impl ToString for DiscordApiRoute {
    fn to_string(&self) -> String {
        match self {
            DiscordApiRoute::Default => "default".to_string(),
            DiscordApiRoute::CurrentUser => "users/@me".to_string(),
            DiscordApiRoute::CurrentUserGuilds => "users/@me/guilds".to_string(),
            DiscordApiRoute::CurrentUserChannels => "users/@me/channels".to_string(),
            DiscordApiRoute::CurrentUserRelationships => "users/@me/relationships".to_string(),
            DiscordApiRoute::Channel(id) => format!("channels/{}", id),
            DiscordApiRoute::ChannelMessages(id) => format!("channels/{}/messages", id),
            DiscordApiRoute::ChannelMessage(channel_id, message_id) => {
                format!("channels/{}/messages/{}", channel_id, message_id)
            }
            DiscordApiRoute::Guild(id) => format!("guilds/{}", id),
        }
    }
}

/// Parses a Discord API URL and returns a standardized route for rate limiting.
///
/// This function attempts to identify specific Discord API endpoints and their major parameters
/// to ensure accurate rate limit bucketing.
pub fn get_discord_route(url_str: &str) -> DiscordApiRoute {
    let parsed_url = match url::Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return DiscordApiRoute::Default,
    };
    let path = parsed_url.path();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Check for /users/@me routes
    if let Some(pos) = segments.iter().position(|&s| s == "@me") {
        if segments.get(pos.saturating_add(1)) == Some(&"guilds") {
            return DiscordApiRoute::CurrentUserGuilds;
        }
        if segments.get(pos.saturating_add(1)) == Some(&"channels") {
            return DiscordApiRoute::CurrentUserChannels;
        }
        if segments.get(pos.saturating_add(1)) == Some(&"relationships") {
            return DiscordApiRoute::CurrentUserRelationships;
        }
        return DiscordApiRoute::CurrentUser;
    }

    // Check for /channels/{id} routes
    if let Some(pos) = segments.iter().position(|&s| s == "channels") {
        if let Some(channel_id) = segments.get(pos.saturating_add(1)) {
            if segments.get(pos.saturating_add(2)) == Some(&"messages") {
                if let Some(message_id) = segments.get(pos.saturating_add(3)) {
                    return DiscordApiRoute::ChannelMessage(
                        channel_id.to_string(),
                        message_id.to_string(),
                    );
                }
                return DiscordApiRoute::ChannelMessages(channel_id.to_string());
            }
            return DiscordApiRoute::Channel(channel_id.to_string());
        }
    }

    // Check for /guilds/{id} routes
    if let Some(pos) = segments.iter().position(|&s| s == "guilds") {
        if let Some(guild_id) = segments.get(pos.saturating_add(1)) {
            // Further sub-routes could be added here (e.g., /guilds/{id}/webhooks)
            return DiscordApiRoute::Guild(guild_id.to_string());
        }
    }

    DiscordApiRoute::Default
}
