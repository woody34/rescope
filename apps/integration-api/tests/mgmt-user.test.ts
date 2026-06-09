import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── Create user ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/create", () => {
  it("creates a user and returns user profile", async () => {
    const login = uniqueLogin("mgmt-create");
    const res = await client.mgmtPost("/v1/mgmt/user/create", {
      loginId: login,
      email: login,
      name: "Alice Test",
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.user.loginIds).toContain(login);
    expect(body.user.name).toBe("Alice Test");
  });

  it("rejects duplicate loginId", async () => {
    const login = uniqueLogin("mgmt-dup");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login });
    const res2 = await client.mgmtPost("/v1/mgmt/user/create", { loginId: login });
    expect(res2.status).toBe(400);
  });

  it("rejects request with wrong management key", async () => {
    const login = uniqueLogin("mgmt-auth");
    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/mgmt/user/create`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer wrong:key" },
      body: JSON.stringify({ loginId: login }),
    });
    expect(res.status).toBe(401);
  });
});

// ─── Create test user ─────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/create/test", () => {
  it("creates a test user flagged for test cleanup", async () => {
    const login = uniqueLogin("test-user");
    const res = await client.mgmtPost("/v1/mgmt/user/create/test", { loginId: login, email: login });
    expect(res.status).toBe(200);
  });
});

// ─── Load user ───────────────────────────────────────────────────────────────

describe("GET /v1/mgmt/user", () => {
  it("loads user by loginId", async () => {
    const login = uniqueLogin("mgmt-load");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.user.loginIds).toContain(login);
  });

  it("returns 4xx for unknown loginId", async () => {
    const res = await client.mgmtGet("/v1/mgmt/user?loginid=ghost%40nowhere.com");
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

describe("GET /v1/mgmt/user/userid", () => {
  it("loads user by userId", async () => {
    const login = uniqueLogin("mgmt-load-uid");
    const createRes = await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });
    const { user } = await createRes.json();

    const res = await client.mgmtGet(`/v1/mgmt/user/userid?userid=${user.userId}`);
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.user.userId).toBe(user.userId);
  });
});

// ─── Search ──────────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/search", () => {
  it("returns all non-test users by default", async () => {
    const login = uniqueLogin("search-reg");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.mgmtPost("/v1/mgmt/user/search", {});
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.users.length).toBeGreaterThan(0);
    expect(body.users.every((u: any) => !u._isTestUser)).toBe(true);
  });

  it("filters by email", async () => {
    const login = uniqueLogin("search-email");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.mgmtPost("/v1/mgmt/user/search", { emails: [login] });
    const body = await res.json();
    expect(body.users.length).toBe(1);
    expect(body.users[0].email).toBe(login);
  });

  it("includes test users when withTestUser=true", async () => {
    const login = uniqueLogin("search-test");
    await client.mgmtPost("/v1/mgmt/user/create/test", { loginId: login });

    const notIncluded = await client.mgmtPost("/v1/mgmt/user/search", {});
    const notBody = await notIncluded.json();
    expect(notBody.users.every((u: any) => u.loginIds[0] !== login)).toBe(true);

    const included = await client.mgmtPost("/v1/mgmt/user/search", { withTestUser: true });
    const incBody = await included.json();
    expect(incBody.users.some((u: any) => u.loginIds.includes(login))).toBe(true);
  });

  it("filters by customAttributes", async () => {
    const login = uniqueLogin("search-attr");
    await client.mgmtPost("/v1/mgmt/user/create", {
      loginId: login,
      customAttributes: { department: "engineering" },
    });

    const res = await client.mgmtPost("/v1/mgmt/user/search", {
      customAttributes: { department: "engineering" },
    });
    const body = await res.json();
    expect(body.users.some((u: any) => u.loginIds.includes(login))).toBe(true);
  });
});

// ─── Update ──────────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update", () => {
  it("replaces user fields", async () => {
    const login = uniqueLogin("mgmt-update");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login, name: "Old Name" });

    const res = await client.mgmtPost("/v1/mgmt/user/update", {
      loginId: login,
      name: "New Name",
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.user.name).toBe("New Name");
    // phone should be null (cleared by full replace)
    expect(body.user.phone).toBeFalsy();
  });
});

describe("PATCH /v1/mgmt/user/patch", () => {
  it("preserves unspecified fields", async () => {
    const login = uniqueLogin("mgmt-patch");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login, name: "Keep Me" });

    const res = await client.mgmtPatch("/v1/mgmt/user/patch", { loginId: login, givenName: "Patched" });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.user.name).toBe("Keep Me"); // preserved
    expect(body.user.givenName).toBe("Patched"); // updated
  });
});

// ─── Update email ─────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/email", () => {
  it("updates email and returns updated user", async () => {
    const login = uniqueLogin("mgmt-email");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.mgmtPost("/v1/mgmt/user/update/email", {
      loginId: login,
      email: "updated@example.com",
      verified: true,
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.user.email).toBe("updated@example.com");
    expect(body.user.verifiedEmail).toBe(true);
  });
});

// ─── Set active password ──────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/password/set/active", () => {
  it("enables password signin for user", async () => {
    const login = uniqueLogin("mgmt-setpwd");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    await client.mgmtPost("/v1/mgmt/user/password/set/active", { loginId: login, password: "Admin1234!" });

    const signinRes = await client.post("/v1/auth/password/signin", {
      loginId: login,
      password: "Admin1234!",
    });
    expect(signinRes.status).toBe(200);
  });
});

// ─── Delete ──────────────────────────────────────────────────────────────────

describe("DELETE /v1/mgmt/user", () => {
  it("deletes user by loginId and is idempotent", async () => {
    const login = uniqueLogin("mgmt-del");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login });

    const res1 = await client.mgmtDelete(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(res1.status).toBe(200);

    // Idempotent
    const res2 = await client.mgmtDelete(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(res2.status).toBe(200);

    const loadRes = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(loadRes.status).toBeGreaterThanOrEqual(400);
  });
});

describe("DELETE /v1/mgmt/user/userid", () => {
  it("deletes user by userId", async () => {
    const login = uniqueLogin("mgmt-del-uid");
    const createRes = await client.mgmtPost("/v1/mgmt/user/create", { loginId: login });
    const { user } = await createRes.json();

    const res = await client.mgmtDelete(`/v1/mgmt/user/userid?userid=${user.userId}`);
    expect(res.status).toBe(200);
  });
});

describe("DELETE /v1/mgmt/user/test/delete/all", () => {
  it("removes test users but keeps regular users", async () => {
    const reg = uniqueLogin("mgmt-del-reg");
    const test = uniqueLogin("mgmt-del-test");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: reg });
    await client.mgmtPost("/v1/mgmt/user/create/test", { loginId: test });

    const res = await client.mgmtDelete("/v1/mgmt/user/test/delete/all");
    expect(res.status).toBe(200);

    const regRes = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(reg)}`);
    expect(regRes.status).toBe(200);

    const testRes = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(test)}`);
    expect(testRes.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Add tenant ──────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/tenant/add", () => {
  it("adds tenant to user and is idempotent", async () => {
    const login = uniqueLogin("mgmt-tenant");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login });

    await client.mgmtPost("/v1/mgmt/user/tenant/add", { loginId: login, tenantId: "tenant-abc" });
    await client.mgmtPost("/v1/mgmt/user/tenant/add", { loginId: login, tenantId: "tenant-abc" }); // idempotent

    const userRes = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    const { user } = await userRes.json();
    const tenants = user.userTenants.filter((t: any) => t.tenantId === "tenant-abc");
    expect(tenants.length).toBe(1);
  });
});

// ─── Generate magic link for test user ───────────────────────────────────────

describe("POST /v1/mgmt/tests/generate/magiclink", () => {
  it("returns token and link for test user", async () => {
    const login = uniqueLogin("mgmt-ml");
    await client.mgmtPost("/v1/mgmt/user/create/test", { loginId: login, email: login });

    const res = await client.mgmtPost("/v1/mgmt/tests/generate/magiclink", {
      loginId: login,
      URI: "http://localhost:3000/verify",
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.token).toBe("string");
    expect(body.link).toContain("?t=");

    // Token can be used to authenticate
    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token: body.token });
    expect(verifyRes.status).toBe(200);
  });

  it("allows a regular (non-test) user — emulator relaxes the test-user check", async () => {
    const login = uniqueLogin("mgmt-ml-reg");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.mgmtPost("/v1/mgmt/tests/generate/magiclink", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.token).toBe("string");
  });
});

// ─── Generate embedded link ───────────────────────────────────────────────────

describe("POST /v1/mgmt/user/embeddedlink", () => {
  it("returns a 64-char token", async () => {
    const login = uniqueLogin("mgmt-embedded");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login });

    const res = await client.mgmtPost("/v1/mgmt/user/embeddedlink", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.token.length).toBe(64);
  });
});

// ─── Tenant list ─────────────────────────────────────────────────────────────

describe("GET /v1/mgmt/tenant/all", () => {
  it("returns empty list by default", async () => {
    const res = await client.mgmtGet("/v1/mgmt/tenant/all");
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(Array.isArray(body.tenants)).toBe(true);
  });
});
