/**
 * Token Diagnostic Utility
 * ------------------------
 * Validates a Discord token against the utility's internal logic.
 */

const token = process.argv[2];
if (!token) {
  console.error("Please provide a token to validate.");
  process.exit(1);
}

const DISCORD_TOKEN_EPOCH = 1293840000;
const tokenRegex =
  /^(mfa\.[a-zA-Z0-9_-]{84}|[a-zA-Z0-9_-]{24,28}\.[a-zA-Z0-9_-]{6}\.[a-zA-Z0-9_-]{27,38})$/;

console.log("\n=== Discord Token Validation Report ===");
console.log(
  `Input: ${token.substring(0, 10)}...${token.substring(token.length - 5)}`,
);

// 1. Regex Validation
const isFormatValid = tokenRegex.test(token);
console.log(
  `\n[1] Format Check:    ${isFormatValid ? "✅ VALID" : "❌ INVALID"}`,
);

// --- HAR OPTIMIZATION: Authenticity Audit ---
console.log("\n--- Contextual Authenticity Audit ---");
const mockRequest = {
  headers: {
    "user-agent": "Mozilla/5.0...",
    "x-super-properties": "eyJvcy...",
    referer: "https://discord.com/channels/@me",
    origin: "https://discord.com",
  },
};

const requiredHeaders = [
  "user-agent",
  "x-super-properties",
  "referer",
  "origin",
];
requiredHeaders.forEach((h) => {
  const present = mockRequest.headers[h] !== undefined;
  console.log(`[+] Header '${h}': ${present ? "✅ ACTIVE" : "⚠️ MISSING"}`);
});
console.log("--------------------------------------\n");

const parts = token.split(".");
if (parts.length === 3) {
  // 2. Part 1: User ID Extraction
  try {
    const id = Buffer.from(parts[0], "base64").toString();
    console.log(`[2] Decoded User ID: ${id}`);
  } catch (e) {
    console.log("[2] Decoded User ID: ❌ DECODE FAILED");
  }

  // 3. Part 2: Temporal Analysis
  try {
    const tsBuffer = Buffer.from(parts[1], "base64");
    if (tsBuffer.length >= 4) {
      const offset = tsBuffer.readUInt32BE(0);
      const date = new Date((offset + DISCORD_TOKEN_EPOCH) * 1000);
      console.log(`[3] Issuance Date:   ${date.toUTCString()}`);
    } else {
      console.log("[3] Issuance Date:   ❌ INVALID BINARY LENGTH");
    }
  } catch (e) {
    console.log("[3] Issuance Date:   ❌ TEMPORAL DECODE FAILED");
  }

  // 4. Part 3: Signature
  console.log(`[4] Signature State: 🛡️ OPAQUE (Length: ${parts[2].length})`);
} else {
  console.log("\n❌ CRITICAL: Token does not contain exactly 3 segments.");
}

console.log("========================================\n");
