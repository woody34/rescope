import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

/**
 * Cross-flow and SDK-level integration tests.
 *
 * These tests verify end-to-end flows that cross multiple API surfaces and
 * mimic real consumer usage patterns.
 */

beforeEach(() => resetEmulator());

// ─── Flow 1: Password signup → set password via mgmt → signin ─────────────────

describe("Full backend password lifecycle", () => {
  it("create user → mgmt setPassword → signin → validate", async () => {
    const login = uniqueLogin("x-flow-pwd");

    // Step 1: mgmt create (no password yet)
    const createRes = await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });
    expect(createRes.status).toBe(200);

    // Step 2: set password via mgmt
    const setRes = await client.mgmtPost("/v1/mgmt/user/password/set/active", {
      loginId: login,
      password: "XFlow1234!",
    });
    expect(setRes.status).toBe(200);

    // Step 3: signin
    const signinRes = await client.post("/v1/auth/password/signin", {
      loginId: login,
      password: "XFlow1234!",
    });
    expect(signinRes.status).toBe(200);
    const { sessionJwt, refreshJwt } = await signinRes.json();

    // Step 4: validate session
    const validateRes = await client.post("/v1/auth/validate", { sessionJwt });
    expect(validateRes.status).toBe(200);
    const { token } = await validateRes.json();
    expect(token.sub).toBeTruthy();

    // Step 5: logout
    const logoutRes = await client.post("/v1/auth/logout", { refreshJwt });
    expect(logoutRes.status).toBe(200);

    // Step 6: refresh after logout must fail
    const refreshRes = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(refreshRes.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Flow 2: createTestUser → generateMagicLink → verify → session valid ──────

describe("Test user magic link flow", () => {
  it("createTestUser → generateMagicLink → verify → validate session", async () => {
    const login = uniqueLogin("x-flow-ml");

    // Step 1: create test user
    const createRes = await client.mgmtPost("/v1/mgmt/user/create/test", {
      loginId: login,
      email: login,
      name: "Test Person",
    });
    expect(createRes.status).toBe(200);

    // Step 2: generate magic link via management API
    const mlRes = await client.mgmtPost("/v1/mgmt/tests/generate/magiclink", {
      loginId: login,
      URI: "http://localhost:3000/verify",
    });
    expect(mlRes.status).toBe(200);
    const { token } = await mlRes.json();

    // Step 3: verify magic link token
    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token });
    expect(verifyRes.status).toBe(200);
    const { sessionJwt } = await verifyRes.json();

    // Step 4: validate session JWT
    const validateRes = await client.post("/v1/auth/validate", { sessionJwt });
    expect(validateRes.status).toBe(200);
    const { token: claims } = await validateRes.json();
    expect(claims.sub).toBeTruthy();

    // Step 5: cleanup — delete all test users
    const cleanRes = await client.mgmtDelete("/v1/mgmt/user/test/delete/all");
    expect(cleanRes.status).toBe(200);

    // User should be gone
    const loadRes = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(loadRes.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Flow 3: signup → update customAttributes → search → verify attributes ────

describe("Custom attribute lifecycle", () => {
  it("signup → patch customAttributes → search → verify", async () => {
    const login = uniqueLogin("x-flow-attr");

    // Step 1: signup
    await client.post("/v1/auth/password/signup", {
      loginId: login,
      password: "Attr1234!",
      user: { email: login },
    });

    // Step 2: patch custom attributes
    const patchRes = await client.mgmtPatch("/v1/mgmt/user/patch", {
      loginId: login,
      customAttributes: { tier: "enterprise", region: "us-east" },
    });
    expect(patchRes.status).toBe(200);
    expect(patchRes.json()).resolves.toMatchObject({
      user: { customAttributes: { tier: "enterprise" } },
    });

    // Step 3: search by custom attribute
    const searchRes = await client.mgmtPost("/v1/mgmt/user/search", {
      customAttributes: { tier: "enterprise" },
    });
    expect(searchRes.status).toBe(200);
    const { users } = await searchRes.json();
    expect(users.some((u: any) => u.loginIds.includes(login))).toBe(true);
  });
});

// ─── Flow 4: signup → refresh → new session passes validate ──────────────────

describe("Refresh token flow", () => {
  it("signin → refresh → new session JWT is valid", async () => {
    const login = uniqueLogin("x-flow-refresh");
    await client.post("/v1/auth/password/signup", {
      loginId: login,
      password: "Refresh1!",
      user: { email: login },
    });

    const signinRes = await client.post("/v1/auth/password/signin", {
      loginId: login,
      password: "Refresh1!",
    });
    const { refreshJwt } = await signinRes.json();

    // Refresh to get a new session
    const refreshRes = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(refreshRes.status).toBe(200);
    const { sessionJwt: newSession } = await refreshRes.json();

    // Validate the new session
    const validateRes = await client.post("/v1/auth/validate", { sessionJwt: newSession });
    expect(validateRes.status).toBe(200);
  });
});

// ─── Flow 5: POST /emulator/reset clears all state ───────────────────────────

describe("Reset lifecycle", () => {
  it("reset clears all users and tokens", async () => {
    const login = uniqueLogin("x-flow-reset");
    await client.post("/v1/auth/password/signup", {
      loginId: login,
      password: "Reset1!",
      user: { email: login },
    });

    const signinRes = await client.post("/v1/auth/password/signin", {
      loginId: login,
      password: "Reset1!",
    });
    const { sessionJwt } = await signinRes.json();

    // Reset
    await client.post("/emulator/reset", {});

    // User gone
    const loadRes = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(loadRes.status).toBeGreaterThanOrEqual(400);

    // Tokens from before reset should fail (JWT is still valid cryptographically,
    // but user is gone so /me should fail)
    const meRes = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/me`, {
      headers: { Authorization: `Bearer ${sessionJwt}` },
    });
    // session JWT is still RSA-valid but user doesn't exist → user not found
    expect(meRes.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Flow 6: Magic link signin → verify → me returns correct user ────────────

describe("Magic link then /me", () => {
  it("signin via magic link → verify → GET /me returns correct user", async () => {
    const login = uniqueLogin("x-flow-me");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login, name: "Me Test" });

    const signinRes = await client.post("/v1/auth/magiclink/signin/email", { loginId: login });
    const { token } = await signinRes.json();

    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token });
    const { refreshJwt } = await verifyRes.json();

    const meRes = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/me`, {
      headers: { Authorization: `Bearer ${refreshJwt}` },
    });
    expect(meRes.status).toBe(200);
    const user = await meRes.json();
    expect(user.loginIds).toContain(login);
    expect(user.name).toBe("Me Test");
  });
});
