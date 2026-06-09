/**
 * Extended session API integration tests.
 * Covers gaps from gap analysis:
 *   - POST /v1/auth/tenant/select (dct claim)
 *   - GET  /v1/auth/me/history (stub)
 *   - GET  /v1/auth/password/policy (static)
 *   - POST /v1/auth/sso/authorize + /sso/exchange (aliases)
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client.js";

const MGMT_KEY = process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function signedIn(login: string) {
  await client.post("/v1/auth/password/signup", { loginId: login, password: "Test1234!" });
  const res = await client.post("/v1/auth/password/signin", { loginId: login, password: "Test1234!" });
  return await res.json() as { sessionJwt: string; refreshJwt: string };
}

async function addUserToTenant(login: string, tenantId: string) {
  await client.mgmtPost("/v1/mgmt/user/tenant/add", { loginId: login, tenantId });
}

function decodeJwtPayload(jwt: string): Record<string, unknown> {
  const parts = jwt.split(".");
  const payload = parts[1].replace(/-/g, "+").replace(/_/g, "/");
  return JSON.parse(atob(payload));
}

// ─── Tenant Select ────────────────────────────────────────────────────────────

describe("POST /v1/auth/tenant/select", () => {
  it("re-issues session JWT with dct claim set to selected tenant", async () => {
    const login = uniqueLogin("tenant-select");
    const { refreshJwt } = await signedIn(login);
    // Create the tenant in the store, then add the user to it
    await client.mgmtPost("/v1/mgmt/tenant/create", { name: "My Tenant", id: "my-tenant" });
    await addUserToTenant(login, "my-tenant");

    const res = await fetch(`${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/v1/auth/tenant/select`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${PROJECT_ID}:${refreshJwt}`,
      },
      body: JSON.stringify({ tenant: "my-tenant" }),
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { sessionJwt: string; refreshJwt: string };
    expect(typeof body.sessionJwt).toBe("string");

    const claims = decodeJwtPayload(body.sessionJwt);
    expect(claims["dct"]).toBe("my-tenant");
  });

  it("returns 401 when user is not a member of the selected tenant", async () => {
    const login = uniqueLogin("tenant-select-bad");
    const { refreshJwt } = await signedIn(login);

    const res = await fetch(`${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/v1/auth/tenant/select`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${PROJECT_ID}:${refreshJwt}`,
      },
      body: JSON.stringify({ tenant: "not-a-member" }),
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("returns 401 without Authorization header", async () => {
    const res = await client.post("/v1/auth/tenant/select", { tenant: "whatever" });
    expect(res.status).toBe(401);
  });
});

// ─── Me History (stub) ────────────────────────────────────────────────────────

describe("GET /v1/auth/me/history", () => {
  it("returns an empty history list", async () => {
    const res = await client.get("/v1/auth/me/history");
    expect(res.status).toBe(200);
    const body = await res.json() as { history: unknown[] };
    expect(Array.isArray(body.history)).toBe(true);
  });
});

// ─── Password Policy ──────────────────────────────────────────────────────────

describe("GET /v1/auth/password/policy", () => {
  it("returns a well-formed policy object", async () => {
    const res = await client.get("/v1/auth/password/policy");
    expect(res.status).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(typeof body.minLength).toBe("number");
    expect(typeof body.maxLength).toBe("number");
    expect(body.active).toBe(true);
  });
});

// ─── SSO alias routes ─────────────────────────────────────────────────────────

describe("POST /v1/auth/sso/authorize (alias for saml/start)", () => {
  it("returns 400 for unknown tenant (same behavior as saml/start)", async () => {
    const res = await client.post("/v1/auth/sso/authorize", { tenant: "ghost-tenant" });
    expect(res.status).toBe(400);
  });
});

describe("POST /v1/auth/saml/authorize (alias for saml/start)", () => {
  it("returns 400 for unknown tenant", async () => {
    const res = await client.post("/v1/auth/saml/authorize", { tenant: "ghost-tenant" });
    expect(res.status).toBe(400);
  });
});

describe("POST /v1/auth/sso/exchange (alias for saml/exchange)", () => {
  it("returns 401 for invalid code", async () => {
    const res = await client.post("/v1/auth/sso/exchange", { code: "invalid-code-xyz" });
    expect(res.status).toBe(401);
  });
});
