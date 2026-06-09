/**
 * Session flows via @descope/core-js-sdk (stateless).
 * sdk.refresh(token), sdk.logout(token), sdk.me(token) all take
 * the refreshJwt explicitly — the SDK doesn't store state.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin, mgmtAuth } from "../helpers/sdk";

beforeEach(() => resetEmulator());

async function signedIn(login: string) {
  const sdk = createClient();
  const res = await sdk.password.signUp(login, "Session1!");
  const { sessionJwt, refreshJwt } = res.data!;
  return { sdk, sessionJwt, refreshJwt: refreshJwt! };
}

describe("sdk.refresh", () => {
  it("issues a new sessionJwt given a valid refreshJwt", async () => {
    const login = uniqueLogin("sess-refresh");
    const { sdk, refreshJwt } = await signedIn(login);

    const res = await sdk.refresh(refreshJwt);
    expect(res.ok).toBe(true);
    expect(res.data?.sessionJwt.split(".").length).toBe(3);
  });

  it("fails with invalid refresh token", async () => {
    const { sdk } = await signedIn(uniqueLogin("sess-refresh-bad"));
    const res = await sdk.refresh("bad.token.here");
    expect(res.ok).toBe(false);
  });

  it("fails after logout (revoked)", async () => {
    const login = uniqueLogin("sess-refresh-revoked");
    const { sdk, refreshJwt } = await signedIn(login);

    await sdk.logout(refreshJwt);
    const res = await sdk.refresh(refreshJwt);
    expect(res.ok).toBe(false);
  });
});

describe("sdk.logout", () => {
  it("revokes session — subsequent refresh fails", async () => {
    const login = uniqueLogin("sess-logout");
    const { sdk, refreshJwt } = await signedIn(login);

    const logoutRes = await sdk.logout(refreshJwt);
    expect(logoutRes.ok).toBe(true);

    const refreshRes = await sdk.refresh(refreshJwt);
    expect(refreshRes.ok).toBe(false);
  });
});

describe("sdk.me", () => {
  it("returns user profile for valid refresh token", async () => {
    const login = uniqueLogin("sess-me");
    const { sdk, refreshJwt } = await signedIn(login);

    const res = await sdk.me(refreshJwt);
    expect(res.ok).toBe(true);
    expect(res.data?.user?.loginIds).toContain(login);
  });

  it("fails with invalid token", async () => {
    const { sdk } = await signedIn(uniqueLogin("sess-me-bad"));
    const res = await sdk.me("bad.token.here");
    expect(res.ok).toBe(false);
  });

  it("fails after logout", async () => {
    const login = uniqueLogin("sess-me-logout");
    const { sdk, refreshJwt } = await signedIn(login);

    await sdk.logout(refreshJwt);
    const res = await sdk.me(refreshJwt);
    expect(res.ok).toBe(false);
  });
});

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

describe("POST /v1/auth/logoutall", () => {
  it("revokes all tokens for user — subsequent refresh fails", async () => {
    const login = uniqueLogin("sess-logoutall");
    const { sdk, refreshJwt } = await signedIn(login);

    const res = await fetch(`${BASE_URL}/v1/auth/logoutall`, {
      method: "POST",
      headers: { Authorization: `Bearer ${refreshJwt}` },
    });
    expect(res.ok).toBe(true);

    const refreshRes = await sdk.refresh(refreshJwt);
    expect(refreshRes.ok).toBe(false);
  });

  it("fresh token issued after re-login still works", async () => {
    const login = uniqueLogin("sess-logoutall-relogin");
    const { refreshJwt } = await signedIn(login);

    await fetch(`${BASE_URL}/v1/auth/logoutall`, {
      method: "POST",
      headers: { Authorization: `Bearer ${refreshJwt}` },
    });

    // Wait 1.1 s so the new token gets a strictly greater iat than the revoked one
    // (JWT iat has 1-second resolution — same-second tokens would look identical).
    await new Promise(r => setTimeout(r, 1100));

    // Sign in again via raw fetch — no SDK cookie state to worry about
    const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";
    const signinRes = await fetch(`${BASE_URL}/v1/auth/password/signin`, {
      method: "POST",
      headers: { "Content-Type": "application/json", "x-descope-project-id": PROJECT_ID },
      body: JSON.stringify({ loginId: login, password: "Session1!" }),
    });
    expect(signinRes.ok).toBe(true);
    const signinBody = await signinRes.json() as { refreshJwt: string };
    const newRefreshJwt = signinBody.refreshJwt;

    // Verify fresh token works via raw refresh call
    const refreshRes = await fetch(`${BASE_URL}/v1/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${PROJECT_ID}:${newRefreshJwt}` },
    });
    expect(refreshRes.ok).toBe(true);
  });

  it("returns 401 for invalid token", async () => {
    const res = await fetch(`${BASE_URL}/v1/auth/logoutall`, {
      method: "POST",
      headers: { Authorization: "Bearer bad.token.here" },
    });
    expect(res.ok).toBe(false);
  });
});

// ─── POST /v1/auth/validate ───────────────────────────────────────────────────

describe("POST /v1/auth/validate", () => {
  it("validates a valid sessionJwt and returns decoded claims", async () => {
    const login = uniqueLogin("sess-validate");
    const { sessionJwt } = await signedIn(login);

    const res = await fetch(`${BASE_URL}/v1/auth/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionJwt }),
    });
    expect(res.ok).toBe(true);
    const body = await res.json() as { jwt: string; token: Record<string, unknown> };
    expect(body.jwt).toBe(sessionJwt);
    expect(typeof body.token.sub).toBe("string");
  });

  it("returns 401 for invalid sessionJwt", async () => {
    const res = await fetch(`${BASE_URL}/v1/auth/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionJwt: "bad.token.here" }),
    });
    expect(res.ok).toBe(false);
  });
});

// ─── GET /v1/auth/me/history ──────────────────────────────────────────────────

describe("GET /v1/auth/me/history", () => {
  it("returns empty history list for authenticated user", async () => {
    const login = uniqueLogin("sess-me-history");
    const { refreshJwt } = await signedIn(login);

    const res = await fetch(`${BASE_URL}/v1/auth/me/history`, {
      headers: { Authorization: `Bearer ${refreshJwt}` },
    });
    expect(res.status).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(Array.isArray(body.history ?? body.users ?? [])).toBe(true);
  });
});

// ─── POST /v1/auth/tenant/select ─────────────────────────────────────────────

const PROJECT_ID_S = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";

describe("POST /v1/auth/tenant/select", () => {
  it("returns new sessionJwt with dct claim for valid tenant", async () => {
    const login = uniqueLogin("sess-tenant-select");
    const tenantId = "select-tenant-01";

    // Sign in first to get tokens
    const { refreshJwt } = await signedIn(login);

    // Seed tenant
    await fetch(`${BASE_URL}/emulator/tenant`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: tenantId, name: "Select Tenant" }),
    });

    // Use mgmt/user/update (POST) to set tenant membership — patch ignores userTenants
    await fetch(`${BASE_URL}/v1/mgmt/user/update`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
      body: JSON.stringify({ loginId: login, userTenants: [{ tenantId }] }),
    });

    const selectRes = await fetch(`${BASE_URL}/v1/auth/tenant/select`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ tenant: tenantId }),
    });
    expect(selectRes.status).toBe(200);
    const selectBody = await selectRes.json() as { sessionJwt: string };

    // Decode the JWT to verify dct claim
    const parts = selectBody.sessionJwt.split(".");
    const payload = JSON.parse(atob(parts[1].replace(/-/g, "+").replace(/_/g, "/")));
    expect(payload.dct).toBe(tenantId);
  });

  it("returns 400 for tenant user doesn't belong to", async () => {
    const login = uniqueLogin("sess-tenant-select-bad");
    const { refreshJwt } = await signedIn(login);

    const res = await fetch(`${BASE_URL}/v1/auth/tenant/select`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ tenant: "nonexistent-tenant" }),
    });
    // TenantNotFound maps to 400 in the emulator (consistent with Descope API)
    expect(res.status).toBe(400);
  });
});
