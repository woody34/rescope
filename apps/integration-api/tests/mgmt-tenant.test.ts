/**
 * Tenant CRUD management API integration tests.
 * Covers: create, update, load, delete, search.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function createTenant(name: string, id?: string, domains?: string[]) {
  const res = await client.mgmtPost("/v1/mgmt/tenant/create", {
    ...(id ? { id } : {}),
    name,
    selfProvisioningDomains: domains ?? [],
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function updateTenant(id: string, name?: string, domains?: string[]) {
  const res = await client.mgmtPost("/v1/mgmt/tenant/update", {
    id,
    ...(name !== undefined ? { name } : {}),
    ...(domains !== undefined ? { selfProvisioningDomains: domains } : {}),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function loadTenant(id: string) {
  const res = await client.mgmtGet(`/v1/mgmt/tenant?id=${encodeURIComponent(id)}`);
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function deleteTenant(id: string) {
  const res = await client.mgmtDelete(`/v1/mgmt/tenant?id=${encodeURIComponent(id)}`);
  return res.status;
}

async function searchTenants(body: Record<string, unknown> = {}) {
  const res = await client.mgmtPost("/v1/mgmt/tenant/search", body);
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

// ─── Create ──────────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/tenant/create", () => {
  it("creates tenant and returns it with generated id", async () => {
    const { status, body } = await createTenant("Acme Corp");
    expect(status).toBe(200);
    const tenant = body.tenant as Record<string, unknown>;
    expect(typeof tenant.id).toBe("string");
    expect(tenant.name).toBe("Acme Corp");
  });

  it("creates tenant with an explicit id", async () => {
    const { status, body } = await createTenant("Explicit Corp", "my-explicit-id");
    expect(status).toBe(200);
    const tenant = body.tenant as Record<string, unknown>;
    expect(tenant.id).toBe("my-explicit-id");
  });

  it("stores self-provisioning domains", async () => {
    const { body } = await createTenant("Domain Corp", undefined, ["corp.example"]);
    const tenant = body.tenant as Record<string, unknown>;
    expect((tenant.selfProvisioningDomains as string[]).includes("corp.example")).toBe(true);
  });

  it("returns 400 when duplicate id is provided", async () => {
    await createTenant("First", "dup-tenant-id");
    const { status } = await createTenant("Second", "dup-tenant-id");
    expect(status).toBe(400);
  });
});

// ─── Update ──────────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/tenant/update", () => {
  it("updates tenant name", async () => {
    await createTenant("Before", "update-tenant-1");
    const { status, body } = await updateTenant("update-tenant-1", "After");
    expect(status).toBe(200);
    expect((body.tenant as Record<string, unknown>).name).toBe("After");
  });

  it("updates self-provisioning domains", async () => {
    await createTenant("Domains Inc", "update-tenant-2");
    const { status, body } = await updateTenant("update-tenant-2", undefined, ["new.example"]);
    expect(status).toBe(200);
    const t = body.tenant as Record<string, unknown>;
    expect((t.selfProvisioningDomains as string[]).includes("new.example")).toBe(true);
  });

  it("returns 400 for unknown tenant id", async () => {
    const { status } = await updateTenant("no-such-tenant-xyz", "Whatever");
    expect(status).toBe(400);
  });
});

// ─── Load ─────────────────────────────────────────────────────────────────────

describe("GET /v1/mgmt/tenant", () => {
  it("loads tenant by id", async () => {
    await createTenant("Loadable Corp", "load-tenant-1");
    const { status, body } = await loadTenant("load-tenant-1");
    expect(status).toBe(200);
    expect((body.tenant as Record<string, unknown>).id).toBe("load-tenant-1");
  });

  it("returns 400 for unknown id", async () => {
    const { status } = await loadTenant("ghost-tenant-xyz");
    expect(status).toBe(400);
  });
});

// ─── Delete ──────────────────────────────────────────────────────────────────

describe("DELETE /v1/mgmt/tenant", () => {
  it("deletes tenant — subsequent load returns 400", async () => {
    await createTenant("To Delete", "delete-tenant-1");
    const delStatus = await deleteTenant("delete-tenant-1");
    expect(delStatus).toBe(200);
    const { status } = await loadTenant("delete-tenant-1");
    expect(status).toBe(400);
  });

  it("is idempotent — deleting unknown tenant returns 200", async () => {
    const status = await deleteTenant("does-not-exist-xyz");
    expect(status).toBe(200);
  });
});

// ─── Search ──────────────────────────────────────────────────────────────────

describe("POST /v1/mgmt/tenant/search", () => {
  it("returns all tenants when no filter is provided", async () => {
    await createTenant("Search Corp A", "search-t-1");
    await createTenant("Search Corp B", "search-t-2");
    const { status, body } = await searchTenants({});
    expect(status).toBe(200);
    const tenants = body.tenants as unknown[];
    expect(tenants.length).toBeGreaterThanOrEqual(2);
  });

  it("filters by tenantIds", async () => {
    await createTenant("Filter By Id", "filter-by-id-tenant");
    const { body } = await searchTenants({ tenantIds: ["filter-by-id-tenant"] });
    const tenants = body.tenants as Array<Record<string, unknown>>;
    expect(tenants.length).toBe(1);
    expect(tenants[0].id).toBe("filter-by-id-tenant");
  });

  it("filters by tenantNames", async () => {
    await createTenant("Unique Name Corp", "unique-name-tenant");
    const { body } = await searchTenants({ tenantNames: ["Unique Name Corp"] });
    const tenants = body.tenants as Array<Record<string, unknown>>;
    expect(tenants.some((t) => t.name === "Unique Name Corp")).toBe(true);
  });

  it("returns empty list when no match", async () => {
    const { body } = await searchTenants({ tenantIds: ["tenant-does-not-exist-xyz"] });
    expect((body.tenants as unknown[]).length).toBe(0);
  });
});

// ─── GET /v1/mgmt/tenant/all (regression) ────────────────────────────────────

describe("GET /v1/mgmt/tenant/all after create", () => {
  it("lists newly created tenant", async () => {
    await createTenant("Listed Corp", "listed-tenant");
    const res = await client.mgmtGet("/v1/mgmt/tenant/all");
    const body = await res.json() as { tenants: Array<Record<string, unknown>> };
    expect(body.tenants.some((t) => t.id === "listed-tenant")).toBe(true);
  });
});
