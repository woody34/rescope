/**
 * Integration tests for the user search endpoint's new filter fields.
 * Covers: loginIds, statuses, tenantIds, roleNames, text (case-insensitive),
 *         createdAfter/createdBefore, sort, and combined (AND) filters.
 *
 * These hit the real HTTP API (POST /v2/mgmt/user/search) to verify
 * end-to-end request parsing + filtering + response serialization.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client.js";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function createUser(login: string, extra?: Record<string, unknown>) {
  await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login, ...extra });
}

async function searchUsers(params: Record<string, unknown>) {
  const res = await client.mgmtPost("/v2/mgmt/user/search", params);
  expect(res.status).toBe(200);
  const body = await res.json() as { users: Array<Record<string, unknown>> };
  return body.users;
}

// ─── Filter by loginIds ───────────────────────────────────────────────────────

describe("search filter: loginIds", () => {
  it("returns only users whose loginId is in the list", async () => {
    const a = uniqueLogin("search-lid-a");
    const b = uniqueLogin("search-lid-b");
    const c = uniqueLogin("search-lid-c");
    await createUser(a);
    await createUser(b);
    await createUser(c);

    const users = await searchUsers({ loginIds: [a, c] });
    expect(users.length).toBe(2);
    const loginIds = users.map((u) => (u.loginIds as string[])[0]);
    expect(loginIds).toContain(a);
    expect(loginIds).toContain(c);
    expect(loginIds).not.toContain(b);
  });

  it("returns empty array when no loginIds match", async () => {
    await createUser(uniqueLogin("search-lid-miss"));
    const users = await searchUsers({ loginIds: ["nonexistent@test.com"] });
    expect(users.length).toBe(0);
  });
});

// ─── Filter by statuses ──────────────────────────────────────────────────────

describe("search filter: statuses", () => {
  it("filters by enabled/disabled status", async () => {
    const enabled = uniqueLogin("search-status-on");
    const disabled = uniqueLogin("search-status-off");
    await createUser(enabled);
    await createUser(disabled);
    // Mgmt-created users start as "invited"; explicitly enable one, disable the other
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: enabled, status: "enabled" });
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: disabled, status: "disabled" });

    const enabledOnly = await searchUsers({ statuses: ["enabled"] });
    expect(enabledOnly.length).toBe(1);
    expect((enabledOnly[0].loginIds as string[])[0]).toBe(enabled);

    const disabledOnly = await searchUsers({ statuses: ["disabled"] });
    expect(disabledOnly.length).toBe(1);
    expect((disabledOnly[0].loginIds as string[])[0]).toBe(disabled);
  });

  it("OR within statuses — returns both enabled and disabled", async () => {
    const a = uniqueLogin("search-status-both-a");
    const b = uniqueLogin("search-status-both-b");
    await createUser(a);
    await createUser(b);
    // Mgmt-created users start as "invited"; explicitly set statuses
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: a, status: "enabled" });
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: b, status: "disabled" });

    const users = await searchUsers({ statuses: ["enabled", "disabled"] });
    expect(users.length).toBe(2);
  });
});

// ─── Filter by tenantIds ──────────────────────────────────────────────────────

describe("search filter: tenantIds", () => {
  it("returns only users in the specified tenant", async () => {
    const tenantId = `search-tenant-${Date.now()}`;
    await client.mgmtPost("/v1/mgmt/tenant/create", { name: "SearchCo", id: tenantId });

    const inTenant = uniqueLogin("search-in-tenant");
    const noTenant = uniqueLogin("search-no-tenant");
    await createUser(inTenant);
    await createUser(noTenant);
    await client.mgmtPost("/v1/mgmt/user/tenant/add", { loginId: inTenant, tenantId });

    const users = await searchUsers({ tenantIds: [tenantId] });
    expect(users.length).toBe(1);
    expect((users[0].loginIds as string[])[0]).toBe(inTenant);
  });
});

// ─── Filter by roleNames ─────────────────────────────────────────────────────

describe("search filter: roleNames", () => {
  it("returns users with matching project-level role", async () => {
    const admin = uniqueLogin("search-role-admin");
    const viewer = uniqueLogin("search-role-viewer");
    await createUser(admin);
    await createUser(viewer);
    await client.mgmtPost("/v1/mgmt/user/update/role/set", { loginId: admin, roleNames: ["admin"] });
    await client.mgmtPost("/v1/mgmt/user/update/role/set", { loginId: viewer, roleNames: ["viewer"] });

    const users = await searchUsers({ roleNames: ["admin"] });
    expect(users.length).toBe(1);
    expect((users[0].loginIds as string[])[0]).toBe(admin);
  });

  it("returns users with matching tenant-level role", async () => {
    const tenantId = `search-role-tenant-${Date.now()}`;
    await client.mgmtPost("/v1/mgmt/tenant/create", { name: "RoleCo", id: tenantId });

    const user = uniqueLogin("search-tenant-role");
    await createUser(user);
    await client.mgmtPost("/v1/mgmt/user/tenant/add", {
      loginId: user,
      tenantId,
      roleNames: ["editor"],
    });

    const users = await searchUsers({ roleNames: ["editor"] });
    expect(users.length).toBe(1);
    expect((users[0].loginIds as string[])[0]).toBe(user);
  });
});

// ─── Free-text search ─────────────────────────────────────────────────────────

describe("search filter: text", () => {
  it("matches against display name (case-insensitive)", async () => {
    const login = uniqueLogin("search-text-name");
    await createUser(login, { name: "Alice Wonderland" });
    await createUser(uniqueLogin("search-text-noise"));

    const users = await searchUsers({ text: "wonderland" });
    expect(users.length).toBe(1);
    expect(users[0].name).toBe("Alice Wonderland");
  });

  it("matches against email", async () => {
    const login = uniqueLogin("search-text-email");
    await createUser(login);

    const users = await searchUsers({ text: "search-text-email" });
    expect(users.length).toBe(1);
  });

  it("is case insensitive", async () => {
    const login = uniqueLogin("search-text-case");
    await createUser(login, { name: "Bob Builder" });

    const users = await searchUsers({ text: "BOB BUILDER" });
    expect(users.length).toBe(1);
  });

  it("matches against loginId", async () => {
    const login = uniqueLogin("search-text-lid");
    await createUser(login);

    // Use a substring of the login ID
    const users = await searchUsers({ text: "search-text-lid" });
    expect(users.length).toBe(1);
  });
});

// ─── Sort ─────────────────────────────────────────────────────────────────────

describe("search sort", () => {
  it("sorts by name ascending", async () => {
    const c = uniqueLogin("sort-charlie");
    const a = uniqueLogin("sort-alice");
    const b = uniqueLogin("sort-bob");
    await createUser(c, { name: "Charlie" });
    await createUser(a, { name: "Alice" });
    await createUser(b, { name: "Bob" });

    const users = await searchUsers({ sort: [{ field: "name", desc: false }] });
    const names = users.map((u) => u.name);
    expect(names).toEqual(["Alice", "Bob", "Charlie"]);
  });

  it("sorts by name descending", async () => {
    const c = uniqueLogin("sort-desc-charlie");
    const a = uniqueLogin("sort-desc-alice");
    const b = uniqueLogin("sort-desc-bob");
    await createUser(c, { name: "Charlie" });
    await createUser(a, { name: "Alice" });
    await createUser(b, { name: "Bob" });

    const users = await searchUsers({ sort: [{ field: "name", desc: true }] });
    const names = users.map((u) => u.name);
    expect(names).toEqual(["Charlie", "Bob", "Alice"]);
  });
});

// ─── Combined (AND) filters ──────────────────────────────────────────────────

describe("search combined filters (AND semantics)", () => {
  it("statuses + tenantIds narrows to intersection", async () => {
    const tenantId = `combined-tenant-${Date.now()}`;
    await client.mgmtPost("/v1/mgmt/tenant/create", { name: "CombinedCo", id: tenantId });

    const match = uniqueLogin("combined-match");
    const wrongStatus = uniqueLogin("combined-wrong-status");
    const wrongTenant = uniqueLogin("combined-wrong-tenant");

    await createUser(match);
    await createUser(wrongStatus);
    await createUser(wrongTenant);

    // match: enabled + tenant
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: match, status: "enabled" });
    await client.mgmtPost("/v1/mgmt/user/tenant/add", { loginId: match, tenantId });
    // wrongStatus: disabled + tenant
    await client.mgmtPost("/v1/mgmt/user/tenant/add", { loginId: wrongStatus, tenantId });
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: wrongStatus, status: "disabled" });
    // wrongTenant: enabled + no tenant
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: wrongTenant, status: "enabled" });

    const users = await searchUsers({
      statuses: ["enabled"],
      tenantIds: [tenantId],
    });
    expect(users.length).toBe(1);
    expect((users[0].loginIds as string[])[0]).toBe(match);
  });

  it("text + statuses works together", async () => {
    const a = uniqueLogin("combined-text-a");
    const b = uniqueLogin("combined-text-b");
    await createUser(a, { name: "Alice Active" });
    await createUser(b, { name: "Alice Disabled" });
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: a, status: "enabled" });
    await client.mgmtPost("/v1/mgmt/user/update/status", { loginId: b, status: "disabled" });

    const users = await searchUsers({ text: "alice", statuses: ["enabled"] });
    expect(users.length).toBe(1);
    expect(users[0].name).toBe("Alice Active");
  });
});
