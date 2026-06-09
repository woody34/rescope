/**
 * Management API integration tests — covers CRUD, search, delete,
 * update/patch, and tenant operations via raw fetch (core-js-sdk
 * has no mgmt module).
 */
import { describe, it, expect, beforeEach } from "vitest";
import { resetEmulator, uniqueLogin, mgmtAuth } from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

const H = { "Content-Type": "application/json", Authorization: mgmtAuth };

async function mgmtCreateUser(loginId: string, opts: Record<string, unknown> = {}) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/create`, {
    method: "POST", headers: H,
    body: JSON.stringify({ loginId, email: loginId, ...opts }),
  });
  return res;
}

async function mgmtLoadUser(loginId: string) {
  const res = await fetch(
    `${BASE_URL}/v1/mgmt/user?loginid=${encodeURIComponent(loginId)}`,
    { headers: H }
  );
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtLoadByUserId(userId: string) {
  const res = await fetch(
    `${BASE_URL}/v1/mgmt/user/userid?userid=${encodeURIComponent(userId)}`,
    { headers: H }
  );
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtSearch(body: Record<string, unknown> = {}) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/search`, {
    method: "POST", headers: H,
    body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtUpdate(loginId: string, fields: Record<string, unknown>) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/update`, {
    method: "POST", headers: H,
    body: JSON.stringify({ loginId, ...fields }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtPatch(loginId: string, fields: Record<string, unknown>) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/patch`, {
    method: "PATCH", headers: H,
    body: JSON.stringify({ loginId, ...fields }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtDeleteUser(loginId: string) {
  const res = await fetch(
    `${BASE_URL}/v1/mgmt/user?loginid=${encodeURIComponent(loginId)}`,
    { method: "DELETE", headers: H }
  );
  return res.status;
}

async function mgmtDeleteByUserId(userId: string) {
  const res = await fetch(
    `${BASE_URL}/v1/mgmt/user/userid?userid=${encodeURIComponent(userId)}`,
    { method: "DELETE", headers: H }
  );
  return res.status;
}

async function mgmtUpdateEmail(loginId: string, email: string) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/update/email`, {
    method: "POST", headers: H,
    body: JSON.stringify({ loginId, email }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtSetPassword(loginId: string, password: string) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/password/set/active`, {
    method: "POST", headers: H,
    body: JSON.stringify({ loginId, password }),
  });
  return res.status;
}

async function mgmtDeleteAllTestUsers() {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/test/delete/all`, {
    method: "DELETE", headers: H,
  });
  return res.status;
}

async function mgmtEmbeddedLink(loginId: string) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/embeddedlink`, {
    method: "POST", headers: H,
    body: JSON.stringify({ loginId }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtTenantAll() {
  const res = await fetch(`${BASE_URL}/v1/mgmt/tenant/all`, { headers: H });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

// ─── GET /v1/mgmt/user ────────────────────────────────────────────────────────

describe("GET /v1/mgmt/user (load by loginId)", () => {
  it("returns user for existing loginId", async () => {
    const login = uniqueLogin("mgmt-load");
    await mgmtCreateUser(login);
    const { status, body } = await mgmtLoadUser(login);
    expect(status).toBe(200);
    const user = (body.user ?? body) as Record<string, unknown>;
    expect((user.loginIds as string[]).includes(login)).toBe(true);
  });

  it("returns 400 for unknown loginId", async () => {
    const { status } = await mgmtLoadUser("nobody@test.com");
    expect(status).toBe(400);
  });
});

// ─── GET /v1/mgmt/user/userid ─────────────────────────────────────────────────

describe("GET /v1/mgmt/user/userid (load by userId)", () => {
  it("returns user for valid userId", async () => {
    const login = uniqueLogin("mgmt-userid");
    await mgmtCreateUser(login);
    const { body: loadBody } = await mgmtLoadUser(login);
    const userId = ((loadBody.user ?? loadBody) as Record<string, unknown>).userId as string;

    const { status, body } = await mgmtLoadByUserId(userId);
    expect(status).toBe(200);
    const user = (body.user ?? body) as Record<string, unknown>;
    expect(user.userId).toBe(userId);
  });

  it("returns 400 for unknown userId", async () => {
    const { status } = await mgmtLoadByUserId("uid-does-not-exist");
    expect(status).toBe(400);
  });
});

// ─── POST /v1/mgmt/user/search ───────────────────────────────────────────────

describe("POST /v1/mgmt/user/search", () => {
  it("returns all users with empty query", async () => {
    const login1 = uniqueLogin("mgmt-search1");
    const login2 = uniqueLogin("mgmt-search2");
    await mgmtCreateUser(login1);
    await mgmtCreateUser(login2);
    const { status, body } = await mgmtSearch({});
    expect(status).toBe(200);
    const users = body.users as unknown[];
    expect(users.length).toBeGreaterThanOrEqual(2);
  });

  it("filters by email", async () => {
    const login = uniqueLogin("mgmt-search-email");
    await mgmtCreateUser(login);
    const { body } = await mgmtSearch({ emails: [login] });
    const users = body.users as unknown[];
    expect(users.length).toBe(1);
  });

  it("withTestUser=false excludes test users", async () => {
    const testLogin = uniqueLogin("mgmt-search-test");
    await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
      method: "POST", headers: H,
      body: JSON.stringify({ loginId: testLogin, email: testLogin }),
    });
    const regularLogin = uniqueLogin("mgmt-search-regular");
    await mgmtCreateUser(regularLogin);

    const { body } = await mgmtSearch({ withTestUser: false });
    const users = body.users as Array<Record<string, unknown>>;
    const hasTestUser = users.some((u) => (u.loginIds as string[]).includes(testLogin));
    expect(hasTestUser).toBe(false);
  });

  it("withTestUser=true includes test users", async () => {
    const testLogin = uniqueLogin("mgmt-search-incl");
    await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
      method: "POST", headers: H,
      body: JSON.stringify({ loginId: testLogin, email: testLogin }),
    });
    const { body } = await mgmtSearch({ withTestUser: true });
    const users = body.users as Array<Record<string, unknown>>;
    const hasTestUser = users.some((u) => (u.loginIds as string[]).includes(testLogin));
    expect(hasTestUser).toBe(true);
  });
});

// ─── POST /v1/mgmt/user/update ───────────────────────────────────────────────

describe("POST /v1/mgmt/user/update", () => {
  it("replaces name and verifiedEmail", async () => {
    const login = uniqueLogin("mgmt-update");
    await mgmtCreateUser(login);
    const { status, body } = await mgmtUpdate(login, { name: "Updated Name", verifiedEmail: true });
    expect(status).toBe(200);
    const user = (body.user ?? body) as Record<string, unknown>;
    expect(user.name).toBe("Updated Name");
    expect(user.verifiedEmail).toBe(true);
  });

  it("returns 400 for unknown loginId", async () => {
    const { status } = await mgmtUpdate("nobody@test.com", { name: "Ghost" });
    expect(status).toBe(400);
  });
});

// ─── PATCH /v1/mgmt/user/patch ───────────────────────────────────────────────

describe("PATCH /v1/mgmt/user/patch", () => {
  it("patches only provided fields (non-provided fields unchanged)", async () => {
    const login = uniqueLogin("mgmt-patch");
    await mgmtCreateUser(login, { name: "Original Name", phone: "+15550001111" });
    const { status, body } = await mgmtPatch(login, { name: "Patched Name" });
    expect(status).toBe(200);
    const user = (body.user ?? body) as Record<string, unknown>;
    expect(user.name).toBe("Patched Name");
    // phone should be unchanged
    expect(user.phone).toBe("+15550001111");
  });

  it("returns 400 for unknown loginId", async () => {
    const { status } = await mgmtPatch("nobody@test.com", { name: "Ghost" });
    expect(status).toBe(400);
  });
});

// ─── DELETE /v1/mgmt/user (by loginId) ───────────────────────────────────────

describe("DELETE /v1/mgmt/user (by loginId)", () => {
  it("deletes existing user — subsequent load returns 400", async () => {
    const login = uniqueLogin("mgmt-delete");
    await mgmtCreateUser(login);
    const deleted = await mgmtDeleteUser(login);
    expect(deleted).toBe(200);
    const { status } = await mgmtLoadUser(login);
    expect(status).toBe(400);
  });
});

// ─── DELETE /v1/mgmt/user/userid (by userId) ─────────────────────────────────

describe("DELETE /v1/mgmt/user/userid (by userId)", () => {
  it("deletes existing user by userId — subsequent load returns 400", async () => {
    const login = uniqueLogin("mgmt-delete-uid");
    await mgmtCreateUser(login);
    const { body: loadBody } = await mgmtLoadUser(login);
    const userId = ((loadBody.user ?? loadBody) as Record<string, unknown>).userId as string;

    const deleted = await mgmtDeleteByUserId(userId);
    expect(deleted).toBe(200);

    const { status } = await mgmtLoadByUserId(userId);
    expect(status).toBe(400);
  });
});

// ─── POST /v1/mgmt/user/update/email ─────────────────────────────────────────

describe("POST /v1/mgmt/user/update/email", () => {
  it("updates user email field", async () => {
    const login = uniqueLogin("mgmt-email-upd");
    await mgmtCreateUser(login);
    const newEmail = uniqueLogin("new-email");
    const { status, body } = await mgmtUpdateEmail(login, newEmail);
    expect(status).toBe(200);
    const user = (body.user ?? body) as Record<string, unknown>;
    expect(user.email).toBe(newEmail);
  });
});

// ─── POST /v1/mgmt/user/password/set/active ──────────────────────────────────

describe("POST /v1/mgmt/user/password/set/active", () => {
  it("sets password so user can sign in via password flow", async () => {
    const login = uniqueLogin("mgmt-setpwd");
    await mgmtCreateUser(login);
    const status = await mgmtSetPassword(login, "AdminSet1!");
    expect(status).toBe(200);

    // Verify new password works
    const signinRes = await fetch(`${BASE_URL}/v1/auth/password/signin`, {
      method: "POST",
      headers: { "Content-Type": "application/json", "x-descope-project-id": PROJECT_ID },
      body: JSON.stringify({ loginId: login, password: "AdminSet1!" }),
    });
    expect(signinRes.ok).toBe(true);
  });
});

// ─── DELETE /v1/mgmt/user/test/delete/all ────────────────────────────────────

describe("DELETE /v1/mgmt/user/test/delete/all", () => {
  it("removes test users but leaves regular users", async () => {
    const testLogin = uniqueLogin("mgmt-del-test");
    const regularLogin = uniqueLogin("mgmt-del-regular");
    await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
      method: "POST", headers: H,
      body: JSON.stringify({ loginId: testLogin, email: testLogin }),
    });
    await mgmtCreateUser(regularLogin);

    const delStatus = await mgmtDeleteAllTestUsers();
    expect(delStatus).toBe(200);

    // Test user should be gone
    const { status: testStatus } = await mgmtLoadUser(testLogin);
    expect(testStatus).toBe(400);

    // Regular user should still exist
    const { status: regularStatus } = await mgmtLoadUser(regularLogin);
    expect(regularStatus).toBe(200);
  });
});

// ─── POST /v1/mgmt/user/embeddedlink ─────────────────────────────────────────

describe("POST /v1/mgmt/user/embeddedlink", () => {
  it("returns a token for existing user", async () => {
    const login = uniqueLogin("mgmt-embedded");
    await mgmtCreateUser(login);
    const { status, body } = await mgmtEmbeddedLink(login);
    expect(status).toBe(200);
    expect(typeof body.token).toBe("string");
  });

  it("returns 400 for unknown user", async () => {
    const { status } = await mgmtEmbeddedLink("ghost@test.com");
    expect(status).toBe(400);
  });
});

// ─── GET /v1/mgmt/tenant/all ─────────────────────────────────────────────────

describe("GET /v1/mgmt/tenant/all", () => {
  it("returns tenant list (may be empty in clean state)", async () => {
    const { status, body } = await mgmtTenantAll();
    expect(status).toBe(200);
    expect(Array.isArray(body.tenants)).toBe(true);
  });
});

// ─── GET /.well-known/jwks.json ───────────────────────────────────────────────

describe("GET /.well-known/jwks.json", () => {
  it("returns JWKS with at least one RSA key", async () => {
    const res = await fetch(`${BASE_URL}/.well-known/jwks.json`);
    expect(res.ok).toBe(true);
    const body = await res.json() as { keys: Array<Record<string, unknown>> };
    expect(Array.isArray(body.keys)).toBe(true);
    expect(body.keys.length).toBeGreaterThan(0);
    expect(body.keys[0].kty).toBe("RSA");
  });
});

// ─── GET /health ──────────────────────────────────────────────────────────────

describe("GET /health", () => {
  it("returns 200 ok", async () => {
    const res = await fetch(`${BASE_URL}/health`);
    expect(res.ok).toBe(true);
  });
});

// ─── Tenant CRUD ──────────────────────────────────────────────────────────────

async function tenantPost(path: string, body: Record<string, unknown>) {
  const res = await fetch(`${BASE_URL}${path}`, {
    method: "POST", headers: H, body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function tenantGet(path: string) {
  const res = await fetch(`${BASE_URL}${path}`, { headers: H });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

describe("Tenant CRUD — create/load/update/search/delete", () => {
  it("full round-trip: create → load → update → search → delete", async () => {
    const id = `test-tenant-${Date.now()}`;
    // Create
    const { status: createStatus, body: createBody } = await tenantPost("/v1/mgmt/tenant/create", {
      id, name: "Round Trip Corp",
    });
    expect(createStatus).toBe(200);
    expect((createBody.tenant as Record<string, unknown> ?? createBody).id).toBe(id);

    // Load
    const { status: loadStatus, body: loadBody } = await tenantGet(`/v1/mgmt/tenant?id=${id}`);
    expect(loadStatus).toBe(200);
    expect(((loadBody.tenant as Record<string, unknown>) ?? loadBody).name).toBe("Round Trip Corp");

    // Update
    const { status: updateStatus } = await tenantPost("/v1/mgmt/tenant/update", {
      id, name: "Updated Corp",
    });
    expect(updateStatus).toBe(200);

    // Search by name
    const { body: searchBody } = await tenantPost("/v1/mgmt/tenant/search", {
      tenantNames: ["Updated Corp"],
    });
    const tenants = searchBody.tenants as Array<Record<string, unknown>>;
    expect(tenants.some(t => t.id === id)).toBe(true);

    // Delete
    const deleteRes = await fetch(`${BASE_URL}/v1/mgmt/tenant?id=${id}`, {
      method: "DELETE", headers: H,
    });
    expect(deleteRes.status).toBe(200);

    // Confirm gone
    const { status: afterDelete } = await tenantGet(`/v1/mgmt/tenant?id=${id}`);
    expect(afterDelete).toBe(400); // TenantNotFound maps to 400 in emulator
  });
});

// ─── User field updates ───────────────────────────────────────────────────────

async function userPost(path: string, body: Record<string, unknown>) {
  const res = await fetch(`${BASE_URL}${path}`, {
    method: "POST", headers: H, body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

describe("POST /v1/mgmt/user/update/name", () => {
  it("updates user name", async () => {
    const login = uniqueLogin("upd-name");
    await mgmtCreateUser(login);
    const { status, body } = await userPost("/v1/mgmt/user/update/name", { loginId: login, name: "New Name" });
    expect(status).toBe(200);
    expect(((body.user ?? body) as Record<string, unknown>).name).toBe("New Name");
  });

  it("returns 400 for unknown user", async () => {
    const { status } = await userPost("/v1/mgmt/user/update/name", { loginId: "ghost@test.com", name: "X" });
    expect(status).toBe(400);
  });
});

describe("POST /v1/mgmt/user/update/phone", () => {
  it("updates user phone field", async () => {
    const login = uniqueLogin("upd-phone");
    await mgmtCreateUser(login);
    const { status } = await userPost("/v1/mgmt/user/update/phone", { loginId: login, phone: "+15550081001" });
    expect(status).toBe(200);
    const { body } = await mgmtLoadUser(login);
    expect(((body.user ?? body) as Record<string, unknown>).phone).toBe("+15550081001");
  });
});

describe("POST /v1/mgmt/user/update/loginid", () => {
  it("renames login ID", async () => {
    const login = uniqueLogin("upd-loginid-old");
    const newLogin = uniqueLogin("upd-loginid-new");
    await mgmtCreateUser(login);
    const { status } = await userPost("/v1/mgmt/user/update/loginid", { loginId: login, newLoginId: newLogin });
    expect(status).toBe(200);
    // Old login should be gone
    const { status: oldStatus } = await mgmtLoadUser(login);
    expect(oldStatus).toBe(400);
    // New login should exist
    const { status: newStatus } = await mgmtLoadUser(newLogin);
    expect(newStatus).toBe(200);
  });
});

describe("POST /v1/mgmt/user/update/role/set and /remove", () => {
  it("set roles replaces existing, remove subtracts", async () => {
    const login = uniqueLogin("upd-roles");
    await mgmtCreateUser(login);

    // Set roles
    const { status: setStatus } = await userPost("/v1/mgmt/user/update/role/set", {
      loginId: login, roleNames: ["admin", "editor"],
    });
    expect(setStatus).toBe(200);

    let { body } = await mgmtLoadUser(login);
    let user = (body.user ?? body) as Record<string, unknown>;
    expect((user.roleNames as string[]).includes("admin")).toBe(true);

    // Remove one role
    const { status: remStatus } = await userPost("/v1/mgmt/user/update/role/remove", {
      loginId: login, roleNames: ["admin"],
    });
    expect(remStatus).toBe(200);

    ({ body } = await mgmtLoadUser(login));
    user = (body.user ?? body) as Record<string, unknown>;
    expect((user.roleNames as string[]).includes("admin")).toBe(false);
    expect((user.roleNames as string[]).includes("editor")).toBe(true);
  });
});

// ─── Batch operations ─────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/create/batch", () => {
  it("creates multiple users in one request", async () => {
    const l1 = uniqueLogin("batch-create-1");
    const l2 = uniqueLogin("batch-create-2");
    const { status, body } = await userPost("/v1/mgmt/user/create/batch", {
      users: [
        { loginId: l1, email: l1 },
        { loginId: l2, email: l2 },
      ],
    });
    expect(status).toBe(200);
    const created = body.createdUsers as unknown[];
    expect(created.length).toBe(2);

    // Both users exist
    const r1 = await mgmtLoadUser(l1);
    const r2 = await mgmtLoadUser(l2);
    expect(r1.status).toBe(200);
    expect(r2.status).toBe(200);
  });
});

describe("POST /v1/mgmt/user/delete/batch", () => {
  it("deletes multiple users — non-existent IDs silently ignored", async () => {
    const l1 = uniqueLogin("batch-del-1");
    const l2 = uniqueLogin("batch-del-2");
    await mgmtCreateUser(l1);
    await mgmtCreateUser(l2);

    const { status } = await userPost("/v1/mgmt/user/delete/batch", {
      loginIds: [l1, l2, "nonexistent@test.com"],
    });
    expect(status).toBe(200);

    // Both users gone
    expect((await mgmtLoadUser(l1)).status).toBe(400);
    expect((await mgmtLoadUser(l2)).status).toBe(400);
  });
});

// ─── Force logout ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/logout (force logout)", () => {
  it("invalidates user refresh tokens after force logout", async () => {
    const login = uniqueLogin("force-logout");
    // Sign up with password to get a refresh token
    const signupRes = await fetch(`${BASE_URL}/v1/auth/password/signup`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ loginId: login, password: "ForceOut1!" }),
    });
    const { refreshJwt } = await signupRes.json() as { refreshJwt: string };

    // Force logout
    const { status } = await userPost("/v1/mgmt/user/logout", { loginId: login });
    expect(status).toBe(200);

    // Wait a tick then try refresh — should fail
    await new Promise(r => setTimeout(r, 1100));
    const refreshRes = await fetch(`${BASE_URL}/v1/auth/refresh`, {
      method: "POST",
      headers: { Authorization: `Bearer ${refreshJwt}` },
    });
    expect(refreshRes.status).toBe(401);
  });
});

// ─── Enchanted link (test user) ───────────────────────────────────────────────

describe("POST /v1/mgmt/tests/generate/enchantedlink", () => {
  it("generates token for test user that is consumable via magiclink/verify", async () => {
    const login = uniqueLogin("enchanted-test");
    // Create test user
    await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
      method: "POST", headers: H,
      body: JSON.stringify({ loginId: login, email: login }),
    });

    const { status, body } = await userPost("/v1/mgmt/tests/generate/enchantedlink", {
      loginId: login, uri: "http://localhost/verify",
    });
    expect(status).toBe(200);
    const token = body.token as string;
    expect(typeof token).toBe("string");

    // Token is consumable
    const verifyRes = await fetch(`${BASE_URL}/v1/auth/magiclink/verify`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token }),
    });
    expect(verifyRes.status).toBe(200);
    const verifyBody = await verifyRes.json() as Record<string, unknown>;
    expect(typeof verifyBody.sessionJwt).toBe("string");
  });

  it("returns 400 for non-test user", async () => {
    const login = uniqueLogin("enchanted-regular");
    await mgmtCreateUser(login);
    const { status } = await userPost("/v1/mgmt/tests/generate/enchantedlink", {
      loginId: login, uri: "http://localhost/verify",
    });
    expect(status).toBe(400);
  });
});

// ─── JWT update ──────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/jwt/update", () => {
  it("issues new sessionJwt with merged custom claims", async () => {
    const login = uniqueLogin("jwt-upd");
    const signupRes = await fetch(`${BASE_URL}/v1/auth/password/signup`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ loginId: login, password: "JwtUpdate1!" }),
    });
    const { sessionJwt } = await signupRes.json() as { sessionJwt: string };

    const { status, body } = await userPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { appRole: "superadmin" },
    });
    expect(status).toBe(200);
    const newJwt = body.jwt as string;
    expect(typeof newJwt).toBe("string");
    expect(newJwt).not.toBe(sessionJwt);

    // Decode to verify claim
    const parts = newJwt.split(".");
    const payload = JSON.parse(atob(parts[1].replace(/-/g, "+").replace(/_/g, "/")));
    expect(payload.appRole).toBe("superadmin");
  });

  it("returns 401 for invalid session JWT", async () => {
    const { status } = await userPost("/v1/mgmt/jwt/update", {
      jwt: "bad.token.here",
      customClaims: { x: 1 },
    });
    expect(status).toBe(401);
  });
});
