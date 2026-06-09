/**
 * Server-side session management via @descope/node-sdk
 *
 * validateSession / refreshSession / validateAndRefreshSession / logout / logoutAll
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin } from "../helpers/sdk.js";

beforeEach(() => resetEmulator());

/** Sign up a user and return { sessionJwt, refreshJwt } */
async function signedIn(login: string) {
  const sdk = createClient();
  const res = await sdk.password.signUp(login, "Pass1!", { email: login });
  return {
    sdk,
    sessionJwt: res.data!.sessionJwt as string,
    refreshJwt: res.data!.refreshJwt as string,
  };
}

describe("validateSession", () => {
  it("validates a fresh session JWT", async () => {
    const login = uniqueLogin("sess");
    const { sdk, sessionJwt } = await signedIn(login);
    const res = await sdk.validateSession(sessionJwt);
    expect(res).toBeTruthy();
    expect(res.token.sub).toBeTruthy();
  });

  it("throws for an invalid JWT", async () => {
    const sdk = createClient();
    await expect(sdk.validateSession("not.a.jwt")).rejects.toThrow();
  });
});

describe("refreshSession", () => {
  it("returns a new sessionJwt from a valid refreshJwt", async () => {
    const login = uniqueLogin("sess");
    const { sdk, refreshJwt } = await signedIn(login);
    const res = await sdk.refreshSession(refreshJwt);
    expect(res.token.sub).toBeTruthy();
  });

  it("throws for an invalid refreshJwt", async () => {
    const sdk = createClient();
    await expect(sdk.refreshSession("not.a.jwt")).rejects.toThrow();
  });
});

describe("validateAndRefreshSession", () => {
  it("validates when session is fresh", async () => {
    const login = uniqueLogin("sess");
    const { sdk, sessionJwt, refreshJwt } = await signedIn(login);
    const res = await sdk.validateAndRefreshSession(sessionJwt, refreshJwt);
    expect(res.token.sub).toBeTruthy();
  });
});

describe("logout", () => {
  it("refresh token is rejected after logout", async () => {
    const login = uniqueLogin("sess");
    const { sdk, refreshJwt } = await signedIn(login);

    await sdk.logout(refreshJwt);

    // Wait a moment to ensure revocation takes effect
    await new Promise((r) => setTimeout(r, 100));

    await expect(sdk.refreshSession(refreshJwt)).rejects.toThrow();
  });
});

describe("management.user.logoutUser (force logout)", () => {
  it("force-logout via management API revokes all sessions", async () => {
    const login = uniqueLogin("sess");
    const { sdk, refreshJwt } = await signedIn(login);

    const logoutRes = await sdk.management.user.logoutUser(login);
    expect(logoutRes.ok).toBe(true);

    await new Promise((r) => setTimeout(r, 100));

    await expect(sdk.refreshSession(refreshJwt)).rejects.toThrow();
  });
});
