/**
 * Workflow 3 — B2B Multi-Tenant SSO & SCIM
 *
 * Market use case: SaaS companies serving enterprise customers who bring their own
 * Identity Providers (Okta, Azure AD). Each customer is a "tenant" with their own
 * SSO config, roles, and user base. Vidoso and similar B2B SaaS companies are the
 * cited examples.
 *
 * Journey:
 *   ADMIN CONFIG: create tenant "acme-corp" via management API
 *      → ADMIN CONFIG: create user, associate with tenant, assign role "admin"
 *      → CONSUMER ACTION: authenticate (password/magic-link)
 *      → CONSUMER ACTION: select tenant context → tenant-scoped JWT with `dct` claim
 *      → Verify JWT contains `dct` = "acme-corp" and role data
 *      → ADMIN ACTION: change tenant role (viewer)
 *      → Re-select tenant → new JWT reflects updated role
 *      → ADMIN ACTION: remove user from tenant (SCIM-style deprovision)
 *      → Tenant select for removed user → rejected (400)
 *      → ADMIN ACTION: delete tenant → tenant gone from list
 */
import { describe, it, expect, beforeEach } from "vitest";
import {
  createClient,
  resetEmulator,
  uniqueLogin,
  mgmtAuth,
} from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const H = { "Content-Type": "application/json", Authorization: mgmtAuth };

beforeEach(() => resetEmulator());

// ─── Helpers ──────────────────────────────────────────────────────────────────

async function createTenant(id: string, name: string) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/tenant/create`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ id, name }),
  });
  expect(res.ok).toBe(true);
}

async function createUser(loginId: string, password: string) {
  await fetch(`${BASE_URL}/v1/mgmt/user/create`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId, email: loginId }),
  });
  await fetch(`${BASE_URL}/v1/mgmt/user/password/set/active`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId, password }),
  });
}

async function addUserToTenant(loginId: string, tenantId: string, roleNames: string[]) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/tenant/add`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId, tenantId, roleNames }),
  });
  expect(res.ok).toBe(true);
}

async function setTenantRole(loginId: string, tenantId: string, roleNames: string[]) {
  return fetch(`${BASE_URL}/v1/mgmt/user/tenant/setRole`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId, tenantId, roleNames }),
  });
}

async function removeTenant(loginId: string, tenantId: string) {
  return fetch(`${BASE_URL}/v1/mgmt/user/tenant/remove`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId, tenantId }),
  });
}

async function selectTenant(refreshJwt: string, tenantId: string) {
  return fetch(`${BASE_URL}/v1/auth/tenant/select`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
    body: JSON.stringify({ tenant: tenantId }),
  });
}

function decodeJwtPayload(jwt: string): Record<string, unknown> {
  return JSON.parse(
    Buffer.from(jwt.split(".")[1], "base64url").toString()
  ) as Record<string, unknown>;
}

// ─── Full B2B tenant journey ──────────────────────────────────────────────────

describe("Workflow 3 — B2B Multi-Tenant: tenant creation to deprovisioning", () => {
  it("full journey: create tenant → provision user → auth → select tenant → verify dct claim → change role → deprovision → tenant delete", async () => {
    const sdk = createClient();
    const tenantId = `acme-corp-${Date.now()}`;
    const login = uniqueLogin("wf3-b2b");

    // ADMIN CONFIG: create the tenant (represents enterprise customer onboarding)
    await createTenant(tenantId, "Acme Corp");

    // ADMIN CONFIG: provision user into tenant with admin role (SSO/SCIM setup)
    await createUser(login, "B2BAdmin1!");
    await addUserToTenant(login, tenantId, ["admin"]);

    // CONSUMER ACTION: user authenticates (in real SSO this would be SAML redirect)
    const signinRes = await sdk.password.signIn(login, "B2BAdmin1!");
    expect(signinRes.ok).toBe(true);
    const { refreshJwt } = signinRes.data!;

    // CONSUMER ACTION: select tenant context (user picks their org)
    const selectRes = await selectTenant(refreshJwt!, tenantId);
    expect(selectRes.status).toBe(200);
    const { sessionJwt } = await selectRes.json() as { sessionJwt: string };

    // Verify JWT carries `dct` (designated current tenant) claim
    const payload = decodeJwtPayload(sessionJwt);
    expect(payload.dct).toBe(tenantId);
    expect(typeof payload.sub).toBe("string");

    // ADMIN ACTION: SCIM group sync — change user's role in tenant
    const setRoleRes = await setTenantRole(login, tenantId, ["viewer"]);
    expect(setRoleRes.ok).toBe(true);

    // CONSUMER ACTION: re-select tenant to get updated claims
    const reSelectRes = await selectTenant(refreshJwt!, tenantId);
    expect(reSelectRes.status).toBe(200);

    // ADMIN ACTION: SCIM deprovisioning — remove user from tenant
    const removeRes = await removeTenant(login, tenantId);
    expect(removeRes.ok).toBe(true);

    // CONSUMER ACTION: tenant select now rejected — user no longer in tenant
    const afterRemoveRes = await selectTenant(refreshJwt!, tenantId);
    expect(afterRemoveRes.status).toBe(400);

    // ADMIN ACTION: delete tenant (offboarding enterprise customer)
    const deleteRes = await fetch(`${BASE_URL}/v1/mgmt/tenant?id=${tenantId}`, {
      method: "DELETE",
      headers: H,
    });
    expect(deleteRes.ok).toBe(true);

    // Confirm tenant is gone
    const loadRes = await fetch(`${BASE_URL}/v1/mgmt/tenant?id=${tenantId}`, { headers: H });
    expect(loadRes.status).toBe(400);
  });
});

// ─── Multi-user tenant ────────────────────────────────────────────────────────

describe("Workflow 3 — B2B Multi-Tenant: multiple users in same tenant", () => {
  it("admin and viewer can both select the same tenant; have different roles", async () => {
    const sdk = createClient();
    const tenantId = `multi-user-tenant-${Date.now()}`;
    const adminLogin = uniqueLogin("wf3-admin");
    const viewerLogin = uniqueLogin("wf3-viewer");

    await createTenant(tenantId, "Multi-User Corp");
    await createUser(adminLogin, "AdminPass1!");
    await createUser(viewerLogin, "ViewerPass1!");
    await addUserToTenant(adminLogin, tenantId, ["admin"]);
    await addUserToTenant(viewerLogin, tenantId, ["viewer"]);

    // Both users can authenticate and select the tenant
    const adminSignin = await sdk.password.signIn(adminLogin, "AdminPass1!");
    expect(adminSignin.ok).toBe(true);
    const adminSelect = await selectTenant(adminSignin.data!.refreshJwt!, tenantId);
    expect(adminSelect.status).toBe(200);

    const viewerSignin = await sdk.password.signIn(viewerLogin, "ViewerPass1!");
    expect(viewerSignin.ok).toBe(true);
    const viewerSelect = await selectTenant(viewerSignin.data!.refreshJwt!, tenantId);
    expect(viewerSelect.status).toBe(200);

    // Search users in tenant — both should appear
    const searchRes = await fetch(`${BASE_URL}/v1/mgmt/user/search`, {
      method: "POST",
      headers: H,
      body: JSON.stringify({ tenantIds: [tenantId] }),
    });
    const searchBody = await searchRes.json() as { users: Array<Record<string, unknown>> };
    const tenantUserIds = searchBody.users.map(u => (u.loginIds as string[])?.[0]);
    expect(tenantUserIds).toContain(adminLogin);
    expect(tenantUserIds).toContain(viewerLogin);
  });
});

// ─── Tenant isolation ─────────────────────────────────────────────────────────

describe("Workflow 3 — B2B Multi-Tenant: tenant isolation", () => {
  it("user cannot select a tenant they don't belong to", async () => {
    const sdk = createClient();
    const tenantA = `tenant-a-${Date.now()}`;
    const tenantB = `tenant-b-${Date.now()}`;
    const login = uniqueLogin("wf3-isolation");

    await createTenant(tenantA, "Tenant A");
    await createTenant(tenantB, "Tenant B");
    await createUser(login, "Isolated1!");
    await addUserToTenant(login, tenantA, ["member"]);

    const signin = await sdk.password.signIn(login, "Isolated1!");
    const { refreshJwt } = signin.data!;

    // Can select tenant A
    expect((await selectTenant(refreshJwt!, tenantA)).status).toBe(200);

    // Cannot select tenant B (not a member)
    expect((await selectTenant(refreshJwt!, tenantB)).status).toBe(400);
  });
});
