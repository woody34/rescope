/**
 * Shared helpers for platform scenario tests.
 *
 * These scenarios test real-world flows end-to-end using only the
 * @descope/node-sdk — no raw HTTP calls, no internal knowledge
 * of the emulator's data model. If the SDK can't do it, then
 * real Descope apps can't do it either.
 */
import DescopeClient from "@descope/node-sdk";

export const BASE_URL =
  process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const PROJECT_ID =
  process.env.EMULATOR_PROJECT_ID ?? "emulator-project";
const MGMT_KEY =
  process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";

/** Authorization header value for direct management API calls.
 *  The emulator expects: Bearer <projectId>:<managementKey> */
export const MGMT_AUTH_HEADER = `Bearer ${PROJECT_ID}:${MGMT_KEY}`;

export type Sdk = ReturnType<typeof DescopeClient>;

/** Create a fully-configured SDK client for the emulator. */
export function sdk(): Sdk {
  return DescopeClient({
    projectId: PROJECT_ID,
    managementKey: MGMT_KEY,
    baseUrl: BASE_URL,
  });
}

/**
 * Reset the emulator to a clean slate.
 * Called in beforeEach so each scenario starts from zero — this
 * mirrors production where each customer project starts empty.
 */
export async function reset(): Promise<void> {
  await fetch(`${BASE_URL}/emulator/reset`, { method: "POST" });
}

let _seq = 0;
/** Generate a collision-free email address for a test identity. */
export const email = (prefix = "user") =>
  `${prefix}-${Date.now()}-${++_seq}@platform.test`;

/**
 * Decode the payload of a JWT without verifying the signature.
 * Used in scenarios to assert token claims produced by the emulator.
 */
export function decodeJwt(token: string): Record<string, unknown> {
  const payload = token.split(".")[1];
  return JSON.parse(Buffer.from(payload, "base64url").toString());
}

/**
 * Helper: sign up via password and return the session JWT.
 * A common operation in scenarios that need an authenticated user.
 */
export async function signUpPassword(
  client: Sdk,
  loginId: string,
  password = "SecurePass1!",
): Promise<string> {
  const res = await client.password.signUp(loginId, password, {
    email: loginId,
  });
  if (!res.ok || !res.data?.sessionJwt) {
    throw new Error(`Password sign-up failed for ${loginId}: ${JSON.stringify(res.error)}`);
  }
  return res.data.sessionJwt;
}
