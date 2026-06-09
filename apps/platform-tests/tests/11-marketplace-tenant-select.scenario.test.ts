/**
 * Scenario: Marketplace / Tenant Selection (dct claim)
 *
 * In marketplace-style B2B SaaS products, a user may belong to multiple
 * organizations. After signing in, the user picks which org they want to
 * work in for this session. Descope models this with the "selected tenant"
 * concept: after picking an org, the JWT gains a 'dct' (Descope Current
 * Tenant) claim that identifies the active org for all downstream authZ.
 *
 * This means the backend can authorize requests based on the CURRENT
 * tenant from the JWT alone — no session state needed.
 *
 * This scenario tests:
 * - User in multiple tenants signs in → JWT has 'tenants' map, no 'dct'
 * - User selects a tenant → new JWT has 'dct' set to that tenant
 * - Selecting a tenant the user doesn't belong to fails
 * - Each tenant user's data is isolated (todos per tenant)
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt, signUpPassword, BASE_URL } from "../helpers/platform.js";

beforeEach(reset);

/** Select a tenant by exchanging a refresh JWT for one scoped to a tenant.
 *  Descope exposes this as POST /v1/auth/tenant/select. */
async function selectTenant(refreshJwt: string, tenantId: string): Promise<Response> {
  return fetch(`${BASE_URL}/v1/auth/tenant/select`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ tenant: tenantId }),
    // Refresh JWT carries the user identity.
    // Descope expects it in the Authorization header for tenant select.
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${refreshJwt}`,
    } as Record<string, string>,
  });
}

describe("Scenario: Marketplace — Tenant Selection", () => {
  it("initial sign-in JWT has 'tenants' map but no 'dct' (tenant not selected yet)", async () => {
    const client = sdk();
    const tenantId = `mkt-${Date.now()}`;
    const loginId = email("mkt-user");

    await client.management.tenant.createWithId(tenantId, "Marketplace Co", []);
    await client.management.user.create(
      loginId, loginId, undefined, undefined, [],
      [{ tenantId, roleNames: ["admin"] }],
    );
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    const signIn = await client.password.signIn(loginId, "Pass1Word!");
    expect(signIn.ok).toBe(true);

    const claims = decodeJwt(signIn.data!.sessionJwt!);
    // The 'tenants' map must be present but no 'dct' yet — user hasn't picked a tenant.
    const tenants = claims.tenants as Record<string, unknown> | undefined;
    expect(tenants && Object.keys(tenants).length > 0, "'tenants' must be non-empty").toBe(true);
    expect(claims.dct, "'dct' must not be set before tenant selection").toBeFalsy();
  });

  it("after selecting a tenant the JWT contains the 'dct' claim for that tenant", async () => {
    const client = sdk();
    const tenantId = `mkt-sel-${Date.now()}`;
    const loginId = email("mkt-sel-user");

    await client.management.tenant.createWithId(tenantId, "Selected Corp", []);
    await client.management.user.create(
      loginId, loginId, undefined, undefined, [],
      [{ tenantId, roleNames: ["member"] }],
    );
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    const signIn = await client.password.signIn(loginId, "Pass1Word!");
    expect(signIn.ok).toBe(true);
    const { refreshJwt } = signIn.data!;

    // User explicitly picks which org they're working in this session.
    const selectRes = await selectTenant(refreshJwt!, tenantId);
    expect(selectRes.ok, "tenant selection should succeed").toBe(true);

    const body = (await selectRes.json()) as { sessionJwt?: string };
    expect(body.sessionJwt, "select must return a new session JWT").toBeTruthy();

    const claims = decodeJwt(body.sessionJwt!);
    expect(claims.dct, "'dct' claim must equal the selected tenant").toBe(tenantId);
  });

  it("selecting a tenant the user doesn't belong to is rejected", async () => {
    const client = sdk();
    const loginId = email("mkt-no-access");
    const unauthorizedTenantId = `other-org-${Date.now()}`;

    // Create the tenant but don't add the user to it.
    await client.management.tenant.createWithId(unauthorizedTenantId, "Other Org", []);
    const signUp = await client.password.signUp(loginId, "Pass1Word!", { email: loginId });
    expect(signUp.ok).toBe(true);
    const { refreshJwt } = signUp.data!;

    // The emulator/Descope backend must enforce tenant membership.
    const selectRes = await selectTenant(refreshJwt!, unauthorizedTenantId);
    expect(selectRes.ok, "selecting an unauthorized tenant must fail").toBe(false);
  });

  it("two users in different tenants have isolated JWT tenant scopes", async () => {
    const client = sdk();
    const tenantA = `iso-a-${Date.now()}`;
    const tenantB = `iso-b-${Date.now()}`;
    const userA = email("iso-a");
    const userB = email("iso-b");

    await client.management.tenant.createWithId(tenantA, "Tenant A", []);
    await client.management.tenant.createWithId(tenantB, "Tenant B", []);

    await client.management.user.create(
      userA, userA, undefined, undefined, [], [{ tenantId: tenantA, roleNames: ["admin"] }],
    );
    await client.management.user.setActivePassword(userA, "Pass1Word!");

    await client.management.user.create(
      userB, userB, undefined, undefined, [], [{ tenantId: tenantB, roleNames: ["admin"] }],
    );
    await client.management.user.setActivePassword(userB, "Pass1Word!");

    const signInA = await client.password.signIn(userA, "Pass1Word!");
    const signInB = await client.password.signIn(userB, "Pass1Word!");

    const claimsA = decodeJwt(signInA.data!.sessionJwt!);
    const claimsB = decodeJwt(signInB.data!.sessionJwt!);

    const tenantsA = Object.keys((claimsA.tenants ?? {}) as object);
    const tenantsB = Object.keys((claimsB.tenants ?? {}) as object);

    expect(tenantsA).toContain(tenantA);
    expect(tenantsA).not.toContain(tenantB); // cross-tenant isolation

    expect(tenantsB).toContain(tenantB);
    expect(tenantsB).not.toContain(tenantA);
  });
});
