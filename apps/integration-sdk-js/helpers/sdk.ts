/**
 * @descope/core-js-sdk helper for the sdk-js test suite.
 *
 * core-js-sdk is STATELESS — it does not store or manage tokens.
 * Tests must extract sessionJwt/refreshJwt from responses and pass them
 * explicitly to subsequent calls (refresh, logout, me).
 */
import createSdk from "@descope/core-js-sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";
const MGMT_KEY = process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";

export const mgmtAuth = `Bearer ${PROJECT_ID}:${MGMT_KEY}`;

/**
 * Create a configured @descope/core-js-sdk client.
 * The SDK is stateless — tokens come back in res.data.sessionJwt / refreshJwt.
 */
export function createClient() {
  return createSdk({
    projectId: PROJECT_ID,
    baseUrl: BASE_URL,
  });
}

// ─── Management helpers (core-js-sdk has no mgmt API — use raw fetch) ─────────

export async function resetEmulator(): Promise<void> {
  await fetch(`${BASE_URL}/emulator/reset`, { method: "POST" });
}

export async function mgmtCreateUser(loginId: string, opts: Record<string, unknown> = {}): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/create`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, email: loginId, ...opts }),
  });
}

export async function mgmtCreateTestUser(loginId: string): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, email: loginId }),
  });
}

export async function mgmtGenerateMagicLink(loginId: string): Promise<string> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/tests/generate/magiclink`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, URI: "http://localhost:3000/verify" }),
  });
  const body = await res.json() as { token: string };
  return body.token;
}

/** Get the reset/magic-link token from the emulator directly (test convenience) */
export async function getEmulatorToken(path: string, body: Record<string, unknown>): Promise<string> {
  const res = await fetch(`${BASE_URL}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = await res.json() as { token: string };
  return data.token;
}

let _counter = 0;
export const uniqueLogin = (prefix = "sdk") =>
  `${prefix}-${Date.now()}-${++_counter}@sdk.example`;
