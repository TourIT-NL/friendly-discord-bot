# Discord Purge Development: Comprehensive Discord HAR File Analysis & Internal API Reference

This document provides an exhaustive, reverse-engineered list of **Discord API endpoints**, methods, and data structures. This information is extrapolated from the `discord.com.har` file and serves as a critical reference for **Discord Purge utility developers** to understand and safely interact with Discord's internal API for features like **bulk message deletion**, **privacy management**, and **account cleanup**. Understanding these internal workings is key to developing robust **Discord client automation**.

## 1. Core Authentication & Identity Management for Discord Purge

This section focuses on the **Discord API endpoints** related to user authentication, profile information, and authorized applications, crucial for secure **Discord login** and identity management within **Discord Purge**.

### Get Current OAuth2 Tokens

- **Endpoint:** `GET /api/v9/oauth2/tokens`
- **Purpose:** Lists active **OAuth2 tokens** associated with the user's Discord account.
- **Utility:** Essential for **Discord Purge** to audit third-party application access and ensure secure token management.

### User Profile (Detailed)

- **Endpoint:** `GET /api/v9/users/{user_id}/profile`
- **Params:** `type=sidebar`, `with_mutual_guilds=true`, `with_mutual_friends=true`
- **Purpose:** Fetches comprehensive user profile data, including bio, banner, and badges.
- **JSON Structure:**
  - `user`: `{ id, username, global_name, avatar, bio, banner, accent_color }`
  - `user_profile`: `{ bio, pronouns, theme_colors, profile_effect }`
  - `badges`: List of `{ id, description, icon, link }`
  - `premium_type`: User's Discord Nitro subscription level.

### User Notes (Private)

- **Endpoint:** `GET /api/v9/users/@me/notes/{user_id}`
- **Purpose:** Fetches private notes the authenticated user has written about another user, a component of personal **Discord data management**.

### Application Identities (Authorized Bots/Apps)

- **Endpoint:** `GET /api/v9/users/{user_id}/application-identities`
- **Purpose:** Lists bots or applications the user has authorized or is associated with. Relevant for **Discord account cleanup** and revoking third-party access.

---

## 2. Discord Privacy, GDPR & Data Management Endpoints

This section covers **Discord API endpoints** pertinent to user privacy settings, GDPR compliance, and data export functionalities, which are vital for **Discord Purge's privacy features**.

### Data Harvest (GDPR Export)

- **Endpoint:** `GET /api/v9/users/@me/harvest`
- **Purpose:** Retrieves the status or content of Discord's collected user data.
- **POST Structure:** Used to trigger specific data exports:
  ```json
  {
    "backends": [
      "Account",
      "Analytics",
      "Activities",
      "Ads",
      "Messages",
      "Servers",
      "Zendesk"
    ]
  }
  ```
- **Utility:** Directly useful for **Discord Purge** in guiding users through GDPR compliance features and triggering full data exports for **Discord data management**.

### Account Consent Settings

- **Endpoint:** `GET /api/v9/users/@me/consent`
- **Purpose:** Fetches user settings for data usage, personalization, and tracking. Essential for **Discord privacy utility** development.

### Internal Settings (Protobuf Encoded)

- **Endpoint:** `PATCH /api/v9/users/@me/settings-proto/1`
- **Purpose:** Updates highly specific client settings related to Privacy, Appearance, and Notifications.
- **Format:** Utilizes binary Protobuf, often seen as a Base64 encoded string in HAR files.
  - Example: `{"settings": "mgECCgA="}`

### Presence Heartbeat

- **Endpoint:** `POST /api/v9/users/@me/meaningfully-online`
- **Purpose:** Internal endpoint indicating active client engagement, relevant for understanding **Discord client behavior**.

---

## 3. Discord Message & Channel Interaction API

These **Discord API endpoints** are crucial for fetching and manipulating messages within channels and DMs, forming the backbone of **Discord Purge's bulk message deletion** capabilities.

### Fetch Messages

- **Endpoint:** `GET /api/v9/channels/{channel_id}/messages`
- **Params:** `limit=30`, `before={msg_id}`, `after={msg_id}`
- **Purpose:** Retrieves a batch of messages from a specified Discord channel or DM.
- **Response:** Returns an array of Message objects, directly usable for **Discord message cleanup**.

### Invite Inspection

- **Endpoint:** `GET /api/v9/invites/{invite_code}`
- **Params:** `with_counts=true`, `with_expiration=true`
- **Purpose:** Validates a Discord invite code and displays member counts and guild information before joining, useful for **Discord server management**.

---

## 4. Discord Guild (Server) Deep-Dive API

This section details **Discord API endpoints** that provide granular access to server-specific information, vital for **Discord Purge's server cleanup** and analysis features.

### Guild Integrations (Webhooks, Bots)

- **Endpoint:** `GET /api/v9/guilds/{guild_id}/integrations`
- **Purpose:** Lists webhooks, authorized bots, and Twitch/YouTube links active within a server.
- **Utility:** Identifying potential "identity hooks" or tracking bots within a Discord server, aiding **Discord privacy analysis**.

### Guild Entitlements

- **Endpoint:** `GET /api/v9/guilds/{guild_id}/entitlements`
- **Purpose:** Lists purchased features or subscriptions associated with a specific server.

### Guild Boosts (Powerups Status)

- **Endpoint:** `GET /api/v9/guilds/{guild_id}/powerups`
- **Purpose:** Shows the status of server boosts and unlockable perks, relevant for **Discord server information**.

### Command Scope Migration (Internal)

- **Endpoint:** `POST /api/v9/guilds/{guild_id}/migrate-command-scope`
- **Purpose:** An internal Discord tool for managing slash command permissions.

---

## 5. Discord Billing & Monetization Endpoints

These **Discord API endpoints** provide access to billing information, subscriptions, and payment history, offering insights into account monetization and purchases.

### Payment Sources

- **Endpoint:** `GET /api/v9/users/@me/billing/payment-sources`
- **Purpose:** Lists saved payment methods (e.g., credit cards, PayPal) on the user's Discord account.

### Active Subscriptions (Nitro, Server Boosts)

- **Endpoint:** `GET /api/v9/users/@me/billing/subscriptions`
- **Params:** `sync_level=2`
- **Purpose:** Details Nitro status and active server boost subscriptions.

### Payment History

- **Endpoint:** `GET /api/v9/users/@me/billing/payments`
- **Params:** `limit=30`
- **Purpose:** Provides a transaction log of all Discord-related payments.

### Entitlements & Gifts

- **Endpoint:** `GET /api/v9/users/@me/entitlements/gifts`
- **Purpose:** Lists the inventory of claimable gift codes and entitlements.

### Collectibles Store (Shop Items)

- **Endpoint:** `GET /api/v9/collectibles-products/{id}`
- **Purpose:** Fetches metadata for items available in the Discord Collectibles shop (e.g., Avatar decorations, profile effects).

---

## 6. Discord Social & Family API

This section explores **Discord API endpoints** related to social connections and family center features, useful for comprehensive **Discord account cleanup**.

### Family Center (Parental Oversight)

- **Endpoint:** `GET /api/v9/family-center/@me`
- **Purpose:** Accesses parental oversight settings and linked teen accounts.
- **Metrics Tracked:** `family_center_view`

### Relationships (Friends/Blocked Users)

- **Endpoint:** `GET /api/v9/users/@me/relationships`
- **Purpose:** Retrieves the user's friend list and blocked users.
- **Utility:** Essential for **Discord Purge's mass unfriending** or **mass blocking analysis** features for complete **Discord account cleanup**.

---

## 7. Discord Metrics & Internal Telemetry API

These **Discord API endpoints** are primarily used by the Discord client for internal metrics collection and telemetry.

### Beaker (Telemetry)

- **Endpoint:** `POST /api/v9/beaker`
- **Purpose:** Tracks UI interactions and feature usage within the Discord client.
- **Structure:** Contains `client_telemetry` with details like `rpc_success_count`, `science_request_id`, etc.

### Metrics v2

- **Endpoint:** `POST /api/v9/metrics/v2`
- **Purpose:** General client performance and error reporting.

### Activity Statistics

- **Endpoint:** `GET /api/v9/users/@me/activities/statistics/applications`
- **Purpose:** Reports how long the user has played specific games or used specific applications on Discord.

---

## 8. Technical Headers Observed in Discord API Interactions

This section highlights important HTTP headers commonly observed in **Discord API requests**, providing insights for **Discord Purge development** and **Discord client reverse engineering**.

- `X-Super-Properties`: (Essential for mimicking the official client and critical for **Discord API interaction**).
- `X-Discord-Locale`: (Indicates the user's preferred language).
- `X-Discord-Timezone`: (Indicates the user's local timezone).
- `X-Debug-Options`: `bugReporterEnabled` (Enables internal debugging tools within the Discord client).
- `Referer`: Typically `https://discord.com/channels/@me` or specific channel URLs, indicating the origin of the API request.
