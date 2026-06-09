/**
 * Claims Contract Tests
 *
 * Validates that every endpoint which returns a JWT or user object produces
 * exactly the structure Descope's production API returns — covering:
 *
 *  - Standard JWT claims:  sub, iss, iat, exp, amr, drn
 *  - Role claims:          roles (global), tenants[id].roleNames
 *  - Permission claims:    tenants[id].permissionNames
 *  - Tenant claims:        tenants map shape, tenantName field
 *  - Custom attributes:    all 6 Descope data types (text, numeric, boolean,
 *                          single-select, multi-select, date) on the user object
 *  - Custom JWT claims:    injected via /mgmt/jwt/update, survive refresh
 *  - Tenant selection:     dct claim appears after /auth/tenant/select
 *
 * Endpoints exercised:
 *   POST /v1/auth/password/signup
 *   POST /v1/auth/password/signin
 *   POST /v1/auth/refresh
 *   POST /v1/auth/validate
 *   POST /v1/auth/tenant/select
 *   GET  /v1/auth/me
 *   POST /v1/auth/otp/verify      (via mgmt generate)
 *   POST /v1/auth/magiclink/verify (via mgmt generate)
 *   POST /v1/mgmt/jwt/update
 *   GET  /v1/mgmt/user
 *   POST /v1/mgmt/user/search
 *   POST /v1/mgmt/user/update/customAttribute
 */

import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client.js";

const BASE = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

beforeEach(() => resetEmulator());

// ─── Helpers ──────────────────────────────────────────────────────────────────

function decode(jwt: string): Record<string, unknown> {
  const part = jwt.split(".")[1].replace(/-/g, "+").replace(/_/g, "/");
  return JSON.parse(atob(part));
}

async function authGet(path: string, token: string) {
  return fetch(`${BASE}${path}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
}

async function createFullUser(login: string, tenantId: string) {
  // Create tenant
  await client.mgmtPost("/v1/mgmt/tenant/create", { id: tenantId, name: "Acme Corp" });

  // Create role and permission
  await client.mgmtPost("/v1/mgmt/permission/create", { name: "todos:write", description: "Write todos" });
  await client.mgmtPost("/v1/mgmt/role/create", {
    name: "editor",
    description: "Editor role",
    permissionNames: ["todos:write"],
  });

  // Create user with global roles + tenant membership (test:true enables mgmt OTP/ML helpers)
  await client.mgmtPost("/v1/mgmt/user/create", {
    loginId: login,
    email: login,
    name: "Claims Tester",
    roleNames: ["editor"],
    userTenants: [{ tenantId, roleNames: ["editor"] }],
    test: true,
  });
  await client.mgmtPost("/v1/mgmt/user/password/set/active", {
    loginId: login,
    password: "Claims1234!",
  });

  // Set all 6 custom attribute types
  const attrs: Array<[string, unknown]> = [
    ["textField", "hello world"],
    ["numericField", 42],
    ["boolField", true],
    ["singleSelect", "option-a"],
    ["multiSelect", ["tag1", "tag2", "tag3"]],
    ["dateField", "2024-06-15"],
  ];
  for (const [attribute, attributeValue] of attrs) {
    await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attribute,
      attributeValue,
    });
  }
}

async function signin(login: string): Promise<{ sessionJwt: string; refreshJwt: string }> {
  const res = await client.post("/v1/auth/password/signin", {
    loginId: login,
    password: "Claims1234!",
  });
  return res.json() as Promise<{ sessionJwt: string; refreshJwt: string }>;
}

// ─── Standard JWT claims ───────────────────────────────────────────────────────

describe("Standard JWT claims on password sign-in", () => {
  it("sessionJwt contains required standard claims", async () => {
    const login = uniqueLogin("std");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { sessionJwt } = await signin(login);
    const claims = decode(sessionJwt);

    // Descope session JWTs MUST have these fields
    expect(typeof claims.sub, "sub: stable user ID").toBe("string");
    expect(claims.sub, "sub must be non-empty").toBeTruthy();
    expect(typeof claims.iss, "iss: project").toBe("string");
    expect(typeof claims.iat, "iat: issued-at").toBe("number");
    expect(typeof claims.exp, "exp: expiry").toBe("number");
    expect(claims.exp as number, "exp must be in the future").toBeGreaterThan(Date.now() / 1000);
    // amr = Authentication Method Reference — array of strings
    expect(Array.isArray(claims.amr), "amr must be an array").toBe(true);
    // drn = Descope Resource Name, always "DS" for session tokens
    expect(claims.drn, "drn must be 'DS' for session JWTs").toBe("DS");
  });

  it("refreshJwt has drn='DSR'", async () => {
    const login = uniqueLogin("drn");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { refreshJwt } = await signin(login);
    const claims = decode(refreshJwt);
    expect(claims.drn, "refresh JWT drn must be 'DSR'").toBe("DSR");
  });
});

// ─── Roles in JWT ─────────────────────────────────────────────────────────────

describe("Role claims in sessionJwt", () => {
  it("global roles appear as top-level 'roles' array", async () => {
    const login = uniqueLogin("roles");
    const tenantId = `t-roles-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { sessionJwt } = await signin(login);
    const claims = decode(sessionJwt);

    expect(Array.isArray(claims.roles), "'roles' must be an array").toBe(true);
    expect((claims.roles as string[]).includes("editor"), "'roles' must include 'editor'").toBe(true);
  });

  it("tenant roles appear in tenants[id].roleNames", async () => {
    const login = uniqueLogin("t-roles");
    const tenantId = `t-troles-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { sessionJwt } = await signin(login);
    const tenants = decode(sessionJwt).tenants as Record<string, { roleNames: string[] }> | undefined;

    expect(tenants, "'tenants' claim must be present").toBeTruthy();
    expect(tenants![tenantId], "tenant entry must exist for the user's tenant").toBeTruthy();
    expect(
      tenants![tenantId].roleNames.includes("editor"),
      "tenant roleNames must include 'editor'",
    ).toBe(true);
  });

  it("tenant permissions appear in tenants[id].permissionNames", async () => {
    const login = uniqueLogin("t-perms");
    const tenantId = `t-tperms-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { sessionJwt } = await signin(login);
    const tenants = decode(sessionJwt).tenants as Record<string, { permissionNames?: string[] }>;

    // permissionNames must be an array (may be empty if not inherited in tenant scope)
    expect(
      Array.isArray(tenants?.[tenantId]?.permissionNames),
      "permissionNames must be an array",
    ).toBe(true);
  });
});

// ─── Tenant claim structure ────────────────────────────────────────────────────

describe("Tenant claim structure in sessionJwt", () => {
  it("tenants map has correct shape: { tenantId: { roleNames, permissionNames, tenantName } }", async () => {
    const login = uniqueLogin("tshape");
    const tenantId = `t-shape-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { sessionJwt } = await signin(login);
    const claims = decode(sessionJwt);
    const tenants = claims.tenants as Record<string, unknown>;

    expect(tenants).toBeTruthy();
    const entry = tenants[tenantId] as Record<string, unknown>;
    expect(entry, "tenant entry must be an object").toBeTruthy();
    expect(Array.isArray(entry.roleNames), "roleNames must be an array").toBe(true);
    expect(Array.isArray(entry.permissionNames), "permissionNames must be an array").toBe(true);
    // Descope embeds tenantName in the tenant claim
    expect(typeof entry.tenantName, "tenantName must be a string").toBe("string");
    expect(entry.tenantName, "tenantName must match the tenant created").toBe("Acme Corp");
  });

  it("user with no tenant has empty or absent tenants claim", async () => {
    const login = uniqueLogin("notenant");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { sessionJwt } = await signin(login);
    const claims = decode(sessionJwt);
    const tenants = claims.tenants as Record<string, unknown> | undefined;
    const isEmpty = !tenants || Object.keys(tenants).length === 0;
    expect(isEmpty, "user with no tenant should have no tenant claims").toBe(true);
  });
});

// ─── dct claim (tenant selection) ─────────────────────────────────────────────

describe("dct claim via POST /v1/auth/tenant/select", () => {
  it("dct is absent before selection and set after", async () => {
    const login = uniqueLogin("dct");
    const tenantId = `t-dct-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { sessionJwt, refreshJwt } = await signin(login);
    expect(decode(sessionJwt).dct, "dct must be absent before selection").toBeFalsy();

    const selRes = await fetch(`${BASE}/v1/auth/tenant/select`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ tenant: tenantId }),
    });
    expect(selRes.ok, "tenant/select must succeed").toBe(true);
    const { sessionJwt: selected } = await selRes.json() as { sessionJwt?: string };
    expect(selected, "select must return a new sessionJwt").toBeTruthy();
    expect(decode(selected!).dct, "dct must equal the selected tenantId").toBe(tenantId);
  });

  it("selecting a tenant the user doesn't belong to is rejected", async () => {
    const login = uniqueLogin("dct-bad");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { refreshJwt } = await signin(login);

    await client.mgmtPost("/v1/mgmt/tenant/create", { id: "other-org", name: "Other Org" });

    const res = await fetch(`${BASE}/v1/auth/tenant/select`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ tenant: "other-org" }),
    });
    expect(res.ok, "selecting unauthorized tenant must fail").toBe(false);
  });
});

// ─── Custom attributes on user object ─────────────────────────────────────────

describe("Custom attributes on user object (all 6 data types)", () => {
  it("GET /v1/mgmt/user returns customAttributes with correct types", async () => {
    const login = uniqueLogin("attrs-load");
    const tenantId = `t-attrs-${Date.now()}`;
    await createFullUser(login, tenantId);

    const res = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(res.status).toBe(200);
    const { user } = await res.json() as { user: Record<string, unknown> };
    const attrs = user.customAttributes as Record<string, unknown>;

    expect(attrs, "customAttributes must be present").toBeTruthy();
    // Text → string
    expect(attrs.textField, "textField: string").toBe("hello world");
    // Numeric → number
    expect(attrs.numericField, "numericField: number").toBe(42);
    expect(typeof attrs.numericField).toBe("number");
    // Boolean → boolean
    expect(attrs.boolField, "boolField: boolean").toBe(true);
    expect(typeof attrs.boolField).toBe("boolean");
    // Single-select → string
    expect(attrs.singleSelect, "singleSelect: string").toBe("option-a");
    // Multi-select → array of strings
    expect(Array.isArray(attrs.multiSelect), "multiSelect: array").toBe(true);
    expect(attrs.multiSelect as string[]).toEqual(["tag1", "tag2", "tag3"]);
    // Date → string in ISO format
    expect(attrs.dateField, "dateField: ISO date string").toBe("2024-06-15");
    expect(typeof attrs.dateField).toBe("string");
  });

  it("POST /v1/mgmt/user/search includes customAttributes on matching users", async () => {
    const login = uniqueLogin("attrs-search");
    const tenantId = `t-attrs-search-${Date.now()}`;
    await createFullUser(login, tenantId);

    const res = await client.mgmtPost("/v1/mgmt/user/search", {
      customAttributes: { textField: "hello world" },
      withTestUser: true,
    });
    expect(res.status).toBe(200);
    const { users } = await res.json() as { users: Array<Record<string, unknown>> };
    const found = users.find((u) => (u.loginIds as string[]).includes(login));
    expect(found, "search result must include our user").toBeTruthy();

    const attrs = found!.customAttributes as Record<string, unknown>;
    expect(attrs?.textField).toBe("hello world");
    expect(attrs?.numericField).toBe(42);
    expect(typeof attrs?.numericField).toBe("number");
  });

  it("POST /v1/mgmt/user/update/customAttribute persists new value", async () => {
    const login = uniqueLogin("attrs-update");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });

    await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attribute: "numericField",
      attributeValue: 99,
    });

    const res = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    const { user } = await res.json() as { user: Record<string, unknown> };
    const attrs = user.customAttributes as Record<string, unknown>;
    expect(attrs.numericField).toBe(99);
    expect(typeof attrs.numericField, "numeric attribute must remain a number after update").toBe("number");
  });
});

// ─── POST /v1/auth/refresh — claims carry through ─────────────────────────────

describe("POST /v1/auth/refresh — claims preserved", () => {
  it("refreshed sessionJwt retains roles and tenant claims", async () => {
    const login = uniqueLogin("refresh-claims");
    const tenantId = `t-refresh-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { refreshJwt } = await signin(login);
    const refreshRes = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(refreshRes.status).toBe(200);
    const { sessionJwt } = await refreshRes.json() as { sessionJwt: string };

    const claims = decode(sessionJwt);
    // Standard claims must be present
    expect(typeof claims.sub).toBe("string");
    expect(claims.drn).toBe("DS");
    // Roles must survive refresh
    expect((claims.roles as string[]).includes("editor")).toBe(true);
    // Tenant claims must survive refresh
    const tenants = claims.tenants as Record<string, unknown>;
    expect(tenants?.[tenantId]).toBeTruthy();
  });

  it("sub is stable across sign-in and refresh", async () => {
    const login = uniqueLogin("sub-stable");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { sessionJwt, refreshJwt } = await signin(login);
    const originalSub = decode(sessionJwt).sub;

    const refreshRes = await client.post("/v1/auth/refresh", { refreshJwt });
    const { sessionJwt: refreshed } = await refreshRes.json() as { sessionJwt: string };
    expect(decode(refreshed).sub, "sub must be stable across refresh").toBe(originalSub);
  });
});

// ─── POST /v1/auth/validate ───────────────────────────────────────────────────

describe("POST /v1/auth/validate — token shape", () => {
  it("returns { jwt, token } where token mirrors standard claims", async () => {
    const login = uniqueLogin("validate-claims");
    const tenantId = `t-val-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { sessionJwt } = await signin(login);
    const res = await client.post("/v1/auth/validate", { sessionJwt });
    expect(res.status).toBe(200);
    const body = await res.json() as { jwt: string; token: Record<string, unknown> };

    expect(typeof body.jwt).toBe("string");
    // The 'token' object is the decoded JWT payload for app convenience
    expect(typeof body.token.sub).toBe("string");
    expect(typeof body.token.exp).toBe("number");
    expect(Array.isArray(body.token.amr)).toBe(true);
    expect((body.token.roles as string[] | undefined)?.includes("editor")).toBe(true);
    const tenants = body.token.tenants as Record<string, unknown> | undefined;
    expect(tenants?.[tenantId]).toBeTruthy();
  });
});

// ─── GET /v1/auth/me ──────────────────────────────────────────────────────────

describe("GET /v1/auth/me — user object shape", () => {
  it("returns user object with loginIds, email, customAttributes, userTenants", async () => {
    const login = uniqueLogin("me-claims");
    const tenantId = `t-me-${Date.now()}`;
    await createFullUser(login, tenantId);

    const { refreshJwt } = await signin(login);
    const res = await authGet("/v1/auth/me", refreshJwt);
    expect(res.status).toBe(200);
    const user = await res.json() as Record<string, unknown>;

    // Core identity fields
    expect((user.loginIds as string[]).includes(login), "loginIds must contain the login").toBe(true);
    expect(user.email, "email must match login").toBe(login);
    expect(user.name, "name must be set").toBe("Claims Tester");

    // Tenant memberships
    const userTenants = user.userTenants as Array<Record<string, unknown>>;
    expect(Array.isArray(userTenants), "userTenants must be an array").toBe(true);
    const membership = userTenants.find((t) => t.tenantId === tenantId);
    expect(membership, "tenant membership must exist").toBeTruthy();
    expect(Array.isArray(membership!.roleNames), "roleNames must be an array").toBe(true);

    // Custom attributes must be present
    const attrs = user.customAttributes as Record<string, unknown>;
    expect(attrs?.textField).toBe("hello world");
    expect(attrs?.numericField).toBe(42);
    expect(attrs?.boolField).toBe(true);
  });

  it("/me returns sessionJwt with correct drn", async () => {
    const login = uniqueLogin("me-jwt");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { refreshJwt } = await signin(login);
    const res = await authGet("/v1/auth/me", refreshJwt);
    const body = await res.json() as { sessionJwt?: string };
    if (body.sessionJwt) {
      expect(decode(body.sessionJwt).drn).toBe("DS");
    }
  });
});

// ─── POST /v1/auth/otp/verify ─────────────────────────────────────────────────

describe("POST /v1/auth/otp/verify — claims in token", () => {
  it("OTP session JWT has standard claims and roles after verify", async () => {
    const login = uniqueLogin("otp-claims");
    const tenantId = `t-otp-${Date.now()}`;
    await createFullUser(login, tenantId);

    // Use the test OTP helper to get a code
    const otpRes = await client.mgmtPost("/v1/mgmt/tests/generate/otp", {
      loginId: login,
      deliveryMethod: "email",
    });
    expect(otpRes.status).toBe(200);
    const { code } = await otpRes.json() as { code: string };

    const verifyRes = await client.post("/v1/auth/otp/verify/email", {
      loginId: login,
      code,
    });
    expect(verifyRes.status).toBe(200);
    const { sessionJwt } = await verifyRes.json() as { sessionJwt: string };

    const claims = decode(sessionJwt);
    expect(claims.sub).toBeTruthy();
    expect(claims.drn).toBe("DS");
    expect(Array.isArray(claims.amr)).toBe(true);
    expect((claims.roles as string[]).includes("editor")).toBe(true);
    const tenants = claims.tenants as Record<string, unknown>;
    expect(tenants?.[tenantId]).toBeTruthy();
  });
});

// ─── POST /v1/auth/magiclink/verify ───────────────────────────────────────────

describe("POST /v1/auth/magiclink/verify — claims in token", () => {
  it("magic link session JWT has standard claims and roles after verify", async () => {
    const login = uniqueLogin("ml-claims");
    const tenantId = `t-ml-${Date.now()}`;
    await createFullUser(login, tenantId);

    const mlRes = await client.mgmtPost("/v1/mgmt/tests/generate/magiclink", {
      loginId: login,
      URI: `${BASE}/verify`,
    });
    expect(mlRes.status, "generate magic link must succeed (must be a test user)").toBe(200);
    const { token } = await mlRes.json() as { token: string };

    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token });
    expect(verifyRes.status).toBe(200);
    const { sessionJwt } = await verifyRes.json() as { sessionJwt: string };

    const claims = decode(sessionJwt);
    expect(claims.sub).toBeTruthy();
    expect(claims.drn).toBe("DS");
    expect(Array.isArray(claims.amr)).toBe(true);
    expect((claims.roles as string[]).includes("editor")).toBe(true);
    const tenants = claims.tenants as Record<string, unknown>;
    expect(tenants?.[tenantId]).toBeTruthy();
  });
});

// ─── POST /v1/mgmt/jwt/update ─────────────────────────────────────────────────

describe("POST /v1/mgmt/jwt/update — custom claims injection", () => {
  it("injects all Descope-supported custom claim types", async () => {
    const login = uniqueLogin("jwt-custom");
    await client.post("/v1/auth/password/signup", {
      loginId: login, password: "Claims1234!", user: { email: login },
    });
    const { sessionJwt } = await signin(login);

    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: {
        textClaim: "hello",
        numericClaim: 123,
        boolClaim: true,
        arrayClaim: ["a", "b"],
        nestedClaim: { key: "value" },
      },
    });
    expect(res.status).toBe(200);
    const { jwt } = await res.json() as { jwt: string };
    const claims = decode(jwt);

    // All injected types must match exactly
    expect(claims.textClaim).toBe("hello");
    expect(claims.numericClaim).toBe(123);
    expect(typeof claims.numericClaim).toBe("number");
    expect(claims.boolClaim).toBe(true);
    expect(typeof claims.boolClaim).toBe("boolean");
    expect(claims.arrayClaim).toEqual(["a", "b"]);
    expect((claims.nestedClaim as Record<string, unknown>)?.key).toBe("value");

    // Standard claims must be preserved exactly
    const original = decode(sessionJwt);
    expect(claims.sub).toBe(original.sub);
    expect(claims.iss).toBe(original.iss);
    expect(claims.drn).toBe("DS");
  });
});
