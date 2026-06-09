import DescopeClient from "@descope/node-sdk";

/**
 * Create a Descope SDK client configured to point at the local emulator.
 *
 * Usage:
 *   const sdk = createSdkClient();
 *   const result = await sdk.validateSession(sessionJwt);
 */
export function createSdkClient() {
  const baseUrl = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
  const projectId = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";

  return DescopeClient({
    projectId,
    baseUrl,
  });
}
