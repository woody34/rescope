/**
 * Management API via @descope/node-sdk
 *
 * Tests: Tenant CRUD, User CRUD, test-user utils (OTP/magic-link/embedded-link),
 * batch operations, JWT update.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin } from "../helpers/sdk.js";

beforeEach(() => resetEmulator());

// ─── Tenant CRUD ──────────────────────────────────────────────────────────────

describe("management.tenant", () => {
  it("createWithId + load returns tenant", async () => {
    const sdk = createClient();
    const id = `t-${Date.now()}`;
    const create = await sdk.management.tenant.createWithId(id, "Acme Corp", []);
    expect(create.ok).toBe(true);

    const load = await sdk.management.tenant.load(id);
    expect(load.ok).toBe(true);
    // Emulator wraps in {tenant:{...}}; SDK passes through as-is so data = {tenant:{...}}
    const tenant = (load.data as Record<string, unknown>)?.tenant as Record<string, unknown>;
    expect(tenant?.name ?? load.data?.name).toBe("Acme Corp");
  });

  it("create (auto-id) + loadAll includes it", async () => {
    const sdk = createClient();
    const create = await sdk.management.tenant.create("Auto Corp", []);
    expect(create.ok).toBe(true);
    // data may be wrapped in {tenant:{...}} — acceptable either way
    const id = (create.data as Record<string, unknown>)?.id
      ?? ((create.data as Record<string, unknown>)?.tenant as Record<string, unknown>)?.id;
    expect(id).toBeTruthy();

    const all = await sdk.management.tenant.loadAll();
    expect(all.ok).toBe(true);
    expect(all.data?.some((t) => t.name === "Auto Corp")).toBe(true);
  });

  it("update changes tenant name", async () => {
    const sdk = createClient();
    const id = `t-upd-${Date.now()}`;
    await sdk.management.tenant.createWithId(id, "Old Name", []);
    await sdk.management.tenant.update(id, "New Name", []);

    const load = await sdk.management.tenant.load(id);
    // Emulator wraps in {tenant:{...}}; SDK passes through as-is
    const name = (load.data as Record<string, unknown>)?.name
      ?? ((load.data as Record<string, unknown>)?.tenant as Record<string, unknown>)?.name;
    expect(name).toBe("New Name");
  });

  it("delete removes tenant — subsequent load fails", async () => {
    const sdk = createClient();
    const id = `t-del-${Date.now()}`;
    await sdk.management.tenant.createWithId(id, "Gone", []);

    const del = await sdk.management.tenant.delete(id, false);
    expect(del.ok).toBe(true);

    const load = await sdk.management.tenant.load(id);
    expect(load.ok).toBe(false);
  });

  it("searchAll filters by tenant ID", async () => {
    const sdk = createClient();
    const id = `t-srch-${Date.now()}`;
    await sdk.management.tenant.createWithId(id, "Search Me", []);

    const res = await sdk.management.tenant.searchAll([id]);
    expect(res.ok).toBe(true);
    expect(res.data?.some((t) => t.id === id)).toBe(true);
  });
});

// ─── User CRUD ────────────────────────────────────────────────────────────────

describe("management.user.create + load + delete", () => {
  it("create + load round-trip", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-u");
    const create = await sdk.management.user.create(login, {
      email: login,
      displayName: "Test User",
    });
    expect(create.ok).toBe(true);

    const load = await sdk.management.user.load(login);
    expect(load.ok).toBe(true);
    expect(load.data?.loginIds).toContain(login);
  });

  it("load by userId works", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-u");
    await sdk.management.user.create(login, { email: login });

    const load = await sdk.management.user.load(login);
    const userId = load.data?.userId as string;
    expect(userId).toBeTruthy();

    const byId = await sdk.management.user.loadByUserId(userId);
    expect(byId.ok).toBe(true);
    expect(byId.data?.loginIds).toContain(login);
  });

  it("delete removes user", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-u");
    await sdk.management.user.create(login, { email: login });
    const del = await sdk.management.user.delete(login);
    expect(del.ok).toBe(true);

    const load = await sdk.management.user.load(login);
    expect(load.ok).toBe(false);
  });
});

describe("management.user.update + patch", () => {
  it("update changes displayName", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-upd");
    await sdk.management.user.create(login, { email: login, displayName: "Old" });
    await sdk.management.user.update(login, { email: login, displayName: "New" });

    const load = await sdk.management.user.load(login);
    // Emulator returns 'name' field (Descope API calls it displayName)
    const displayName = load.data?.displayName ?? (load.data as Record<string, unknown>)?.name;
    expect(displayName).toBe("New");
  });

  it("patch changes displayName without wiping other fields", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-patch");
    await sdk.management.user.create(login, { email: login, displayName: "Before" });
    await sdk.management.user.patch(login, { displayName: "After" });

    const load = await sdk.management.user.load(login);
    // Emulator returns 'name' field (Descope API calls it displayName)
    const displayName = load.data?.displayName ?? (load.data as Record<string, unknown>)?.name;
    expect(displayName).toBe("After");
    expect(load.data?.email).toBe(login); // email untouched
  });
});

describe("management.user.updateLoginId", () => {
  it("renames loginId; new loginId works, old does not", async () => {
    const sdk = createClient();
    const oldLogin = uniqueLogin("mgmt-lid-old");
    const newLogin = uniqueLogin("mgmt-lid-new");
    await sdk.management.user.create(oldLogin, { email: oldLogin });

    // updateLoginId(loginId, newLoginId?) — newLoginId is optional
    const res = await sdk.management.user.updateLoginId(oldLogin, newLogin);
    expect(res.ok).toBe(true);

    const good = await sdk.management.user.load(newLogin);
    expect(good.ok).toBe(true);

    const bad = await sdk.management.user.load(oldLogin);
    expect(bad.ok).toBe(false);
  });
});

describe("management.user.search", () => {
  it("returns created users by email filter", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-srch");
    await sdk.management.user.create(login, { email: login });

    // SDK search: { emails?: string[] } — response data is UserResponse[] (via users field)
    const res = await sdk.management.user.search({ emails: [login] });
    expect(res.ok).toBe(true);
    // The SDK transforms the emulator's { users } to data as array via transformResponse
    const users = (res.data as unknown as { users: Array<{ loginIds?: string[] }> })?.users ?? res.data;
    const found = Array.isArray(users)
      ? users
      : (res.data as unknown as Array<{ loginIds?: string[] }>);
    expect(found.some((u: { loginIds?: string[] }) => u.loginIds?.includes(login))).toBe(true);
  });
});

describe("management.user.createBatch + deleteBatch", () => {
  it("createBatch creates all users; deleteBatch removes them", async () => {
    const sdk = createClient();
    const a = uniqueLogin("batch-a");
    const b = uniqueLogin("batch-b");

    const batchCreate = await sdk.management.user.createBatch([
      { loginId: a, email: a },
      { loginId: b, email: b },
    ]);
    expect(batchCreate.ok).toBe(true);
    // createBatch returns { createdUsers, failedUsers }
    expect(batchCreate.data?.createdUsers.length).toBeGreaterThanOrEqual(2);
    expect(batchCreate.data?.failedUsers.length).toBe(0);

    // Get user IDs from createdUsers for deleteBatch
    const uidA = batchCreate.data?.createdUsers.find(
      (u) => u.loginIds?.includes(a)
    )?.userId as string;
    const uidB = batchCreate.data?.createdUsers.find(
      (u) => u.loginIds?.includes(b)
    )?.userId as string;

    const batchDel = await sdk.management.user.deleteBatch([uidA, uidB]);
    expect(batchDel.ok).toBe(true);

    expect((await sdk.management.user.load(a)).ok).toBe(false);
    expect((await sdk.management.user.load(b)).ok).toBe(false);
  });
});

// ─── Test-user utilities ──────────────────────────────────────────────────────

describe("management.user.createTestUser + generateOTPForTestUser", () => {
  it("generates OTP for test user; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("test-otp");
    await sdk.management.user.createTestUser(login, { email: login });

    const otpRes = await sdk.management.user.generateOTPForTestUser("email", login);
    expect(otpRes.ok).toBe(true);
    const code = otpRes.data?.code as string;
    expect(code).toMatch(/^\d{6}$/);

    const verify = await sdk.otp.verify.email(login, code);
    expect(verify.ok).toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();
  });
});

describe("management.user.createTestUser + generateMagicLinkForTestUser", () => {
  it("generates magic link for test user; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("test-ml");
    await sdk.management.user.createTestUser(login, { email: login });

    const linkRes = await sdk.management.user.generateMagicLinkForTestUser(
      "email",
      login,
      "http://localhost/verify"
    );
    expect(linkRes.ok).toBe(true);
    const token = linkRes.data?.token as string;
    expect(token).toBeTruthy();

    const verify = await sdk.magicLink.verify(token);
    expect(verify.ok).toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();
  });
});

describe("management.user.generateEmbeddedLink", () => {
  it("generates embedded link token; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("test-emb");
    // generateEmbeddedLink works on regular users (not just test users)
    await sdk.management.user.create(login, { email: login });

    // SDK: generateEmbeddedLink(loginId, customClaims?, timeout?)
    const linkRes = await sdk.management.user.generateEmbeddedLink(login);
    expect(linkRes.ok).toBe(true);
    const token = linkRes.data?.token as string;
    expect(token).toBeTruthy();

    const verify = await sdk.magicLink.verify(token);
    expect(verify.ok).toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();
  });
});

describe("management.user.deleteAllTestUsers", () => {
  it("removes all test users, leaves regular users", async () => {
    const sdk = createClient();
    const regular = uniqueLogin("regular");
    const testU = uniqueLogin("tu");

    // Create both in same reset cycle
    await sdk.management.user.create(regular, { email: regular });
    await sdk.management.user.createTestUser(testU, { email: testU });

    const delAll = await sdk.management.user.deleteAllTestUsers();
    expect(delAll.ok).toBe(true);

    expect((await sdk.management.user.load(regular)).ok).toBe(true);
    expect((await sdk.management.user.load(testU)).ok).toBe(false);
  });
});

// ─── JWT update ───────────────────────────────────────────────────────────────

describe("management.jwt.update", () => {
  it("adds custom claims to a valid session JWT", async () => {
    const sdk = createClient();
    const login = uniqueLogin("jwt-upd");
    const signupRes = await sdk.password.signUp(login, "Pass1!", { email: login });
    const sessionJwt = signupRes.data?.sessionJwt as string;

    const updatedRes = await sdk.management.jwt.update(sessionJwt, { appRole: "admin" });
    expect(updatedRes.ok).toBe(true);
    const newJwt = updatedRes.data?.jwt as string;
    expect(newJwt).toBeTruthy();

    // Decode JWT payload (no verification needed for claim inspection)
    const payload = JSON.parse(
      Buffer.from(newJwt.split(".")[1], "base64url").toString()
    );
    expect(payload.appRole).toBe("admin");
  });
});
