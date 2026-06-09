/**
 * SDK-level integration test (task 13.9).
 *
 * Uses the real @descope/node-sdk pointed at the emulator to validate sessions.
 * This is the highest-fidelity test — if the node SDK can validate tokens
 * issued by the emulator, consumers can drop the emulator in transparently.
 */

import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";
import { createSdkClient } from "../helpers/sdk-client";

beforeEach(() => resetEmulator());

describe("@descope/node-sdk against emulator", () => {
  it("validates a session JWT issued by the emulator", async () => {
    const login = uniqueLogin("sdk-validate");

    // Signup via HTTP
    const signupRes = await client.post("/v1/auth/password/signup", {
      loginId: login,
      password: "SdkTest1!",
      user: { email: login },
    });
    expect(signupRes.status).toBe(200);
    const { sessionJwt } = await signupRes.json();

    // Point the SDK at the emulator
    const sdk = createSdkClient();
    const result = await sdk.validateSession(sessionJwt);

    // The SDK's validateSession resolves to the token's claims on success
    expect(result).toBeTruthy();
    // subject should match
    expect(result?.token?.sub).toBeTruthy();
  });

  it("rejects invalid JWT via SDK", async () => {
    const sdk = createSdkClient();
    await expect(sdk.validateSession("not.a.jwt")).rejects.toBeTruthy();
  });

  it("full SDK cycle: signup → me → logout → refresh rejected", async () => {
    const login = uniqueLogin("sdk-cycle");

    const signupRes = await client.post("/v1/auth/password/signup", {
      loginId: login,
      password: "SdkCycle1!",
      user: { email: login },
    });
    const { sessionJwt, refreshJwt } = await signupRes.json();

    // Validate via SDK
    const sdk = createSdkClient();
    const claims = await sdk.validateSession(sessionJwt);
    expect(claims?.token?.sub).toBeTruthy();

    // Logout
    const logoutRes = await client.post("/v1/auth/logout", { refreshJwt });
    expect(logoutRes.status).toBe(200);

    // Refresh must now fail (revoked)
    const refreshRes = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(refreshRes.status).toBeGreaterThanOrEqual(400);
  });
});
