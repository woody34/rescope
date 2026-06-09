/**
 * Integration tests for user-api-completeness: invited status, addRole,
 * updatePicture, updateCustomAttribute endpoints.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client.js";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function createUser(login: string, extra?: Record<string, unknown>) {
  await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login, ...extra });
}

async function loadUser(login: string) {
  const res = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
  return (await res.json() as { user: Record<string, unknown> }).user;
}

// ─── Invited status ───────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/status (invited)", () => {
  it("sets user status to invited", async () => {
    const login = uniqueLogin("invited");
    await createUser(login);

    const res = await client.mgmtPost("/v1/mgmt/user/update/status", {
      loginId: login,
      status: "invited",
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    expect(body.user.status).toBe("invited");
  });

  it("invited users appear in status filter search", async () => {
    const login = uniqueLogin("invited-search");
    await createUser(login);
    await client.mgmtPost("/v1/mgmt/user/update/status", {
      loginId: login,
      status: "invited",
    });

    const searchRes = await client.mgmtPost("/v2/mgmt/user/search", {
      statuses: ["invited"],
    });
    expect(searchRes.status).toBe(200);
    const body = await searchRes.json() as { users: Array<Record<string, unknown>> };
    expect(body.users.length).toBe(1);
    expect((body.users[0].loginIds as string[])[0]).toBe(login);
  });
});

// ─── Add role ─────────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/role/add", () => {
  it("appends roles without replacing existing ones", async () => {
    const login = uniqueLogin("add-role");
    await createUser(login);
    await client.mgmtPost("/v1/mgmt/user/update/role/set", {
      loginId: login,
      roleNames: ["admin"],
    });

    const res = await client.mgmtPost("/v1/mgmt/user/update/role/add", {
      loginId: login,
      roleNames: ["editor"],
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    const roles = body.user.roleNames as string[];
    expect(roles).toContain("admin");
    expect(roles).toContain("editor");
  });

  it("does not duplicate existing roles", async () => {
    const login = uniqueLogin("add-role-dedup");
    await createUser(login);
    await client.mgmtPost("/v1/mgmt/user/update/role/set", {
      loginId: login,
      roleNames: ["admin"],
    });

    const res = await client.mgmtPost("/v1/mgmt/user/update/role/add", {
      loginId: login,
      roleNames: ["admin", "viewer"],
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    const roles = body.user.roleNames as string[];
    expect(roles.length).toBe(2);
    expect(roles).toContain("admin");
    expect(roles).toContain("viewer");
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/update/role/add", {
      loginId: "ghost@test.com",
      roleNames: ["admin"],
    });
    expect(res.status).toBe(400);
  });
});

// ─── Update picture ───────────────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/picture", () => {
  it("sets the user's profile picture URL", async () => {
    const login = uniqueLogin("picture");
    await createUser(login);

    const res = await client.mgmtPost("/v1/mgmt/user/update/picture", {
      loginId: login,
      picture: "https://example.com/avatar.png",
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    expect(body.user.picture).toBe("https://example.com/avatar.png");

    // Verify via load
    const user = await loadUser(login);
    expect(user.picture).toBe("https://example.com/avatar.png");
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/update/picture", {
      loginId: "ghost@test.com",
      picture: "https://example.com/avatar.png",
    });
    expect(res.status).toBe(400);
  });
});

// ─── Update custom attribute ──────────────────────────────────────────────────

describe("POST /v1/mgmt/user/update/customAttribute", () => {
  it("sets a single custom attribute on a user", async () => {
    const login = uniqueLogin("custom-attr");
    await createUser(login);

    const res = await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attributeKey: "tier",
      attributeValue: "gold",
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    expect((body.user.customAttributes as Record<string, unknown>).tier).toBe("gold");
  });

  it("overwrites an existing attribute value", async () => {
    const login = uniqueLogin("custom-attr-overwrite");
    await createUser(login);
    await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attributeKey: "tier",
      attributeValue: "gold",
    });

    const res = await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attributeKey: "tier",
      attributeValue: "platinum",
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { user: Record<string, unknown> };
    expect((body.user.customAttributes as Record<string, unknown>).tier).toBe("platinum");
  });

  it("preserves other attributes when updating one", async () => {
    const login = uniqueLogin("custom-attr-preserve");
    await createUser(login);
    await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attributeKey: "plan",
      attributeValue: "enterprise",
    });
    await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: login,
      attributeKey: "tier",
      attributeValue: "gold",
    });

    const user = await loadUser(login);
    const attrs = user.customAttributes as Record<string, unknown>;
    expect(attrs.plan).toBe("enterprise");
    expect(attrs.tier).toBe("gold");
  });

  it("returns 400 for unknown user", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId: "ghost@test.com",
      attributeKey: "foo",
      attributeValue: "bar",
    });
    expect(res.status).toBe(400);
  });
});
