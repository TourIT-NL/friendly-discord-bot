/**
 * Discord Mock Token Generator (v4 - Ultimate Fidelity)
 * ----------------------------------------------------
 * Generates tokens that are indistinguishable from real tokens in
 * structure, length, and temporal data.
 *
 * Logic:
 * 1. Part 1: Base64(String(Snowflake))
 * 2. Part 2: Base64(Binary(UInt32(Now - 2011 Epoch)))
 * 3. Part 3: Base64URL(Random(28 bytes))
 */

import crypto from "node:crypto";

// Discord's Snowflake Epoch (January 1, 2015)
const DISCORD_SNOWFLAKE_EPOCH = 1420070400000;
// Discord's Token Epoch (January 1, 2011)
const DISCORD_TOKEN_EPOCH = 1293840000;

/**
 * Generates a realistic Discord Snowflake ID string
 */
function generateSnowflake() {
  const timestamp = Date.now() - DISCORD_SNOWFLAKE_EPOCH;
  // We use BigInt to handle 64-bit integer math in JS
  const snowflake =
    (BigInt(timestamp) << 22n) |
    (1n << 17n) |
    (1n << 12n) |
    BigInt(Math.floor(Math.random() * 4096));
  return snowflake.toString();
}

function generateUltimateFidelityToken(providedId) {
  const id = providedId || generateSnowflake();

  // 1. Part 1: Base64 of the ID String
  const part1 = Buffer.from(id).toString("base64").replace(/=/g, "");

  // 2. Part 2: 4-byte Binary Timestamp (Big-Endian)
  const now = Math.floor(Date.now() / 1000);

  // Inject random jitter within the current hour as requested
  const jitteredDate = new Date();
  jitteredDate.setMinutes(Math.floor(Math.random() * 60));
  jitteredDate.setSeconds(Math.floor(Math.random() * 60));
  const jitteredSeconds = Math.floor(jitteredDate.getTime() / 1000);

  const offset = jitteredSeconds - DISCORD_TOKEN_EPOCH;
  const tsBuffer = Buffer.alloc(4);
  tsBuffer.writeUInt32BE(offset, 0);
  const part2 = tsBuffer.toString("base64").replace(/=/g, "");

  // 3. Part 3: 28-byte High-Entropy Signature (Base64URL)
  const part3 = crypto
    .randomBytes(28)
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=/g, "");

  return `${part1}.${part2}.${part3}`;
}

/**
 * Demonstrates how to use the token with genuine headers from HAR analysis
 */
function displayGenuineEnvelope(token) {
  const userAgent =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36";

  // Sample x-super-properties (high fidelity)
  const superProps = Buffer.from(
    JSON.stringify({
      os: "Windows",
      browser: "Chrome",
      system_locale: "en-US",
      browser_user_agent: userAgent,
      browser_version: "144.0.0.0",
      release_channel: "stable",
      client_build_number: 501798,
      client_launch_id: "9ce26f83-8107-4404-8b9d-1446e092d2e3",
      client_heartbeat_session_id: "5200f980-4101-4268-bdbe-bf523f25ca98",
    }),
  ).toString("base64");

  console.log("--- Sample Genuine Request Envelope (v5) ---");
  console.log(
    JSON.stringify(
      {
        method: "GET",
        url: "https://discord.com/api/v9/users/@me",
        headers: {
          authorization: token,
          "user-agent": userAgent,
          "x-super-properties": superProps,
          referer: "https://discord.com/channels/@me",
          origin: "https://discord.com",
          "accept-language": "en-US,en;q=0.9",
          cookie: "__dcfduid=...; __sdcfduid=...; locale=en-US;",
          "sec-ch-ua":
            '"Not(A:Brand";v="8", "Chromium";v="144", "Chrome";v="144"',
          "sec-ch-ua-mobile": "?0",
          "sec-ch-ua-platform": '"Windows"',
          "sec-fetch-dest": "empty",
          "sec-fetch-mode": "cors",
          "sec-fetch-site": "same-origin",
          "x-discord-locale": "en-US",
          "x-discord-timezone": "UTC",
        },
      },
      null,
      2,
    ),
  );
  console.log("-------------------------------------------\n");
}

const userId = process.argv[2];
const mockToken = generateUltimateFidelityToken(userId);

console.log("\n--- Generated Ultimate-Fidelity Mock Token ---");
console.log(mockToken);
console.log("----------------------------------------------\n");

displayGenuineEnvelope(mockToken);

console.log(
  `User ID Used: ${Buffer.from(mockToken.split(".")[0], "base64").toString()}`,
);
const tsOffset = Buffer.from(mockToken.split(".")[1], "base64").readUInt32BE(0);
console.log(
  `Timestamp:    ${new Date((tsOffset + DISCORD_TOKEN_EPOCH) * 1000).toLocaleString()}`,
);
console.log("----------------------------------------------\n");
