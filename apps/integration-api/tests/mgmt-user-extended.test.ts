/**
 * Extended user management API integration tests.
 * Covers: update/name, update/phone, update/loginid, update/role/set|remove,
 *         create/batch, delete/batch, force logout, password/expire,
 *         password/set/temporary, generate/enchantedlink.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function createUser(login: string) {
  await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });
}

async function createTestUserHelper(login: string) {
  await client.mgmtPost("/v1/mgmt/user/create/test", { loginId: login, email: login });
}

async function loadUser(login: string) {
  const res = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
  const body = await res.json() as { user: Record<string, unknown> };
  return body.user;
}

async function signedInTokens(login: string, password = "Test1234!") {
  await client.post("/v1/auth/password/signup", { loginId: login, password });
  const res = await client.post("/v1/auth/password/signin", { loginId: login, password });
  return await res.json() as { sessionJwt: string; refreshJwt: string };
}

// ─── Update name ──────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/name", () => {
  it("updates the user's display name", async () => {
    const login = uniqueLogin("upd-name");
    await createUser(login);
    const res = await client.mgmtPost("/v1/mgmt/user/update/name", { loginId: login, name: "New Name" });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    expect(body.user.name).toBe("New Name");
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/update/name", { loginId: "ghost@test.com", name: "Ghost" });
    expect(res.status).toBe(400);
  });
});

// ─── Update phone ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/phone", () => {
  it("updates the user's phone field", async () => {
    const login = uniqueLogin("upd-phone");
    await createUser(login);
    const res = await client.mgmtPost("/v1/mgmt/user/update/phone", { loginId: login, phone: "+15550001234" });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    expect(body.user.phone).toBe("+15550001234");
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/update/phone", { loginId: "ghost@test.com", phone: "+1555" });
    expect(res.status).toBe(400);
  });
});

// ─── Update loginId ───────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/loginid", () => {
  it("renames a login ID — old id no longer loadable, new id works", async () => {
    const oldId = uniqueLogin("old-loginid");
    const newId = uniqueLogin("new-loginid");
    // Email must differ from the login id: GET /v1/mgmt/user falls back to an
    // email lookup, so reusing oldId as the email would keep it resolvable
    // after the loginId is renamed.
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: oldId, email: uniqueLogin("rename-mail") });

    const res = await client.mgmtPost("/v1/mgmt/user/update/loginid", { loginId: oldId, newLoginId: newId });
    expect(res.status).toBe(200);

    const { status: oldStatus } = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(oldId)}`);
    expect(oldStatus).toBe(400);

    const newUser = await loadUser(newId);
    expect((newUser.loginIds as string[]).includes(newId)).toBe(true);
  });

  it("returns 400 if new loginId already taken", async () => {
    const a = uniqueLogin("loginid-a");
    const b = uniqueLogin("loginid-b");
    await createUser(a);
    await createUser(b);
    const res = await client.mgmtPost("/v1/mgmt/user/update/loginid", { loginId: a, newLoginId: b });
    expect(res.status).toBe(400);
  });
});

// ─── Set / Remove global roles ────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/role/set", () => {
  it("sets global roles, replacing any existing ones", async () => {
    const login = uniqueLogin("role-set");
    await createUser(login);

    const res = await client.mgmtPost("/v1/mgmt/user/update/role/set", { loginId: login, roleNames: ["admin", "viewer"] });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    const roles = body.user.roleNames as string[];
    expect(roles).toContain("admin");
    expect(roles).toContain("viewer");
  });
});

describe("POST /v1/mgmt/user/update/role/remove", () => {
  it("removes specified roles and leaves others", async () => {
    const login = uniqueLogin("role-remove");
    await createUser(login);
    await client.mgmtPost("/v1/mgmt/user/update/role/set", { loginId: login, roleNames: ["admin", "viewer"] });

    const res = await client.mgmtPost("/v1/mgmt/user/update/role/remove", { loginId: login, roleNames: ["admin"] });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    const roles = body.user.roleNames as string[];
    expect(roles).not.toContain("admin");
    expect(roles).toContain("viewer");
  });
});

// ─── Batch create ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/create/batch", () => {
  it("creates multiple users in one request", async () => {
    const a = uniqueLogin("batch-create-a");
    const b = uniqueLogin("batch-create-b");
    const res = await client.mgmtPost("/v1/mgmt/user/create/batch", {
      users: [{ loginId: a, email: a }, { loginId: b, email: b }],
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { createdUsers: unknown[] };
    expect(body.createdUsers.length).toBe(2);

    // Both users should be loadable
    expect((await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(a)}`)).status).toBe(200);
    expect((await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(b)}`)).status).toBe(200);
  });
});

// ─── Batch delete ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/delete/batch", () => {
  it("deletes multiple users by loginId", async () => {
    const a = uniqueLogin("batch-del-a");
    const b = uniqueLogin("batch-del-b");
    await createUser(a);
    await createUser(b);

    const res = await client.mgmtPost("/v1/mgmt/user/delete/batch", { loginIds: [a, b] });
    expect(res.status).toBe(200);

    expect((await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(a)}`)).status).toBe(400);
    expect((await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(b)}`)).status).toBe(400);
  });

  it("is idempotent — unknown loginIds in batch are silently skipped", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/delete/batch", { loginIds: ["ghost@test.com"] });
    expect(res.status).toBe(200);
  });
});

// ─── Force logout ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/logout (force logout)", () => {
  it("invalidates active tokens — subsequent refresh returns 401", async () => {
    const login = uniqueLogin("force-logout");
    const { refreshJwt } = await signedInTokens(login);

    // Verify token works before force-logout
    const before = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(before.status).toBe(200);

    // Force logout via management API
    const logoutRes = await client.mgmtPost("/v1/mgmt/user/logout", { loginId: login });
    expect(logoutRes.status).toBe(200);

    // Wait 1s so new tokens get a greater iat (JWT iat is 1-second granular)
    await new Promise((r) => setTimeout(r, 1100));

    // Existing token should now be rejected
    const after = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(after.status).toBe(401);
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/logout", { loginId: "ghost@test.com" });
    expect(res.status).toBe(400);
  });
});

// ─── Password expire (stub) ───────────────────────────────────────────────────

describe("POST /v1/mgmt/user/password/expire", () => {
  it("returns 200 for existing user", async () => {
    const login = uniqueLogin("pwd-expire");
    await createUser(login);
    const res = await client.mgmtPost("/v1/mgmt/user/password/expire", { loginId: login });
    expect(res.status).toBe(200);
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/password/expire", { loginId: "ghost@test.com" });
    expect(res.status).toBe(400);
  });
});

// ─── Set temporary password ───────────────────────────────────────────────────

describe("POST /v1/mgmt/user/password/set/temporary", () => {
  it("sets a password so the user can sign in", async () => {
    const login = uniqueLogin("tmp-pwd");
    await createUser(login);

    const setRes = await client.mgmtPost("/v1/mgmt/user/password/set/temporary", {
      loginId: login,
      password: "TempPass1!",
    });
    expect(setRes.status).toBe(200);

    // User should be able to sign in with the new password
    const signinRes = await client.post("/v1/auth/password/signin", { loginId: login, password: "TempPass1!" });
    expect(signinRes.status).toBe(200);
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/password/set/temporary", {
      loginId: "ghost@test.com",
      password: "whatever",
    });
    expect(res.status).toBe(400);
  });
});

// ─── Generate enchanted link for test user ────────────────────────────────────

describe("POST /v1/mgmt/tests/generate/enchantedlink", () => {
  it("returns a usable token for a test user", async () => {
    const login = uniqueLogin("enchanted-test");
    await createTestUserHelper(login);

    const res = await client.mgmtPost("/v1/mgmt/tests/generate/enchantedlink", {
      loginId: login,
      URI: "http://localhost/verify",
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { token: string; link: string };
    expect(typeof body.token).toBe("string");
    expect(body.link).toContain("?t=");

    // Token should be consumable via /v1/auth/magiclink/verify
    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token: body.token });
    expect(verifyRes.status).toBe(200);
    const verifyBody = await verifyRes.json() as { sessionJwt: string };
    expect(typeof verifyBody.sessionJwt).toBe("string");
  });

  it("allows a non-test user — emulator relaxes the test-user check", async () => {
    const login = uniqueLogin("enchanted-regular");
    await createUser(login);

    const res = await client.mgmtPost("/v1/mgmt/tests/generate/enchantedlink", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.token).toBe("string");
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/tests/generate/enchantedlink", { loginId: "ghost@test.com" });
    expect(res.status).toBe(400);
  });
});
