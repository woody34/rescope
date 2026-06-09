/**
 * Helper utilities for the @descope/node-sdk integration test suite.
 *
 * The Node SDK is different from core-js-sdk:
 *   - Auth methods return { data, ok, error } — session tokens live in data.sessionJwt / data.refreshJwt
 *   - Management is available via client.management.* with typed methods
 *   - It requires managementKey for management API access
 */
import DescopeClient from "@descope/node-sdk";

export const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";
const MGMT_KEY = process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";

export type NodeClient = ReturnType<typeof DescopeClient>;

/**
 * Create a DescopeClient pointed at the emulator.
 * Pass managementKey so management.* methods are available.
 */
export function createClient(): NodeClient {
  return DescopeClient({
    projectId: PROJECT_ID,
    managementKey: MGMT_KEY,
    baseUrl: BASE_URL,
  });
}

/** Hard-reset the emulator state before each test. */
export async function resetEmulator(): Promise<void> {
  await fetch(`${BASE_URL}/emulator/reset`, { method: "POST" });
}

let _counter = 0;
export const uniqueLogin = (prefix = "node") =>
  `${prefix}-${Date.now()}-${++_counter}@sdk.example`;

/** Get the OTP code from a signUpOrIn response body directly (emulator convenience). */
export function extractCode(res: { data?: unknown }): string {
  const data = res.data as Record<string, unknown>;
  return data?.code as string;
}

/** Get the token from a magic link response body (emulator convenience). */
export function extractToken(res: { data?: unknown }): string {
  const data = res.data as Record<string, unknown>;
  return (data?.token ?? data?.pendingRef) as string;
}
