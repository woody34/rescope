/**
 * Scenario: Multi-Tenant SaaS
 *
 * Multi-tenancy is one of Descope's most distinctive features. A single
 * Descope project serves multiple organizations (tenants), each with
 * their own user memberships and roles. Users can belong to multiple
 * tenants with different roles in each.
 *
 * This scenario models a typical B2B SaaS setup:
 * - Platform admin creates organizations (tenants) at signup time
 * - Platform admin provisions user → organization memberships
 * - User signs in and receives a JWT with tenant claims embedded
 * - Tenant-specific roles appear in the JWT under the tenant's section
 * - A user with no org membership signs in and has no tenant claims
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: Multi-Tenant Organization Management", () => {
  it("user in a tenant sees tenant ID in their JWT", async () => {
    const client = sdk();
    const tenantId = `org-${Date.now()}`;
    const loginId = email("tenant-user");

    // Platform provisions a new customer organization.
    const tCreate = await client.management.tenant.createWithId(tenantId, "Acme Corp", []);
    expect(tCreate.ok, "tenant creation should succeed").toBe(true);

    // Platform creates the user and immediately adds them to the tenant.
    // Uses positional overload: (loginId, email, phone, displayName, roles, userTenants)
    await client.management.user.create(
      loginId, loginId, undefined, undefined, [],
      [{ tenantId, roleNames: ["owner"] }],
    );
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    const signIn = await client.password.signIn(loginId, "Pass1Word!");
    expect(signIn.ok).toBe(true);

    // The JWT must contain a 'tenants' claim as a map { tenantId -> {roleNames} }.
    // This is how the app backend authorizes tenant-scoped API calls without a DB lookup.
    const claims = decodeJwt(signIn.data!.sessionJwt!);
    const tenants = claims.tenants as Record<string, unknown> | undefined;
    expect(tenants, "JWT must include tenants claim").toBeTruthy();
    expect(Object.keys(tenants!)).toContain(tenantId);
  });

  it("user in two tenants has both in their JWT", async () => {
    const client = sdk();
    const orgA = `orgA-${Date.now()}`;
    const orgB = `orgB-${Date.now()}`;
    const loginId = email("dual-tenant");

    await client.management.tenant.createWithId(orgA, "Alpha Corp", []);
    await client.management.tenant.createWithId(orgB, "Beta Inc", []);

    await client.management.user.create(
      loginId, loginId, undefined, undefined, [],
      [
        { tenantId: orgA, roleNames: ["admin"] },
        { tenantId: orgB, roleNames: ["viewer"] },
      ],
    );
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    const signIn = await client.password.signIn(loginId, "Pass1Word!");
    expect(signIn.ok).toBe(true);

    const { tenants } = decodeJwt(signIn.data!.sessionJwt!) as {
      tenants?: Record<string, { roleNames?: string[] }>;
    };
    expect(tenants).toBeTruthy();
    expect(Object.keys(tenants ?? {})).toContain(orgA);
    expect(Object.keys(tenants ?? {})).toContain(orgB);
  });

  it("user without tenant membership has no tenants claim in JWT", async () => {
    const client = sdk();
    const loginId = email("no-org");
    const signUp = await client.password.signUp(loginId, "Pass1Word!", { email: loginId });
    expect(signUp.ok).toBe(true);

    const claims = decodeJwt(signUp.data!.sessionJwt!);

    // A user who hasn't been added to any organization should have no tenant claims.
    const tenants = claims.tenants as Record<string, unknown> | undefined;
    const isEmpty = !tenants || Object.keys(tenants).length === 0;
    expect(isEmpty).toBe(true);
  });

  it("admin can list all tenants and search by ID", async () => {
    const client = sdk();
    const id1 = `search-${Date.now()}-1`;
    const id2 = `search-${Date.now()}-2`;

    await client.management.tenant.createWithId(id1, "Org One", []);
    await client.management.tenant.createWithId(id2, "Org Two", []);

    const all = await client.management.tenant.loadAll();
    expect(all.ok).toBe(true);
    expect(all.data?.map((t) => t.id)).toContain(id1);

    const search = await client.management.tenant.searchAll([id1]);
    expect(search.ok).toBe(true);
    expect(search.data?.some((t) => t.id === id1)).toBe(true);
    expect(search.data?.some((t) => t.id === id2)).toBe(false);
  });
});
