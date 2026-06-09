/**
 * Scenario: Role-Based Access Control (RBAC)
 *
 * RBAC is the primary way Descope-powered apps gate features. Roles
 * are assigned to users (globally or per-tenant). The roles appear in
 * the JWT claims so the app backend can authorize requests without an
 * extra database lookup.
 *
 * This scenario models:
 * - Platform sets up roles via management SDK (done at deploy time)
 * - Users are created with roles
 * - JWT from sign-in includes the correct roles
 * - Roles can be changed at runtime and are reflected in new tokens
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt, signUpPassword } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: RBAC — Role Setup and Enforcement", () => {
  it("user with an assigned role sees it in their JWT claims", async () => {
    const client = sdk();
    const loginId = email("rbac-user");

    // Platform creates a role at bootstrap — this is a management
    // operation done by the platform itself, not by the end user.
    const roleCreate = await client.management.role.create("editor", "Can edit content", []);
    expect(roleCreate.ok).toBe(true);

    // Admin creates a user and assigns them the editor role.
    // Uses the positional overload: (loginId, email, phone, displayName, roles)
    await client.management.user.create(
      loginId, loginId, undefined, undefined, ["editor"],
    );
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    // End user signs in. The returned JWT must contain the role —
    // this is what the app backend reads to authorize "editor" actions.
    const signIn = await client.password.signIn(loginId, "Pass1Word!");
    expect(signIn.ok).toBe(true);

    const claims = decodeJwt(signIn.data!.sessionJwt!);
    // Descope embeds roles in the 'roles' claim as a string array.
    const roles = (claims.roles ?? claims.r ?? []) as string[];
    expect(roles).toContain("editor");
  });

  it("role removal is reflected in subsequent sign-in tokens", async () => {
    const client = sdk();
    const loginId = email("rbac-demote");

    await client.management.role.create("moderator", "Can moderate", []);
    await client.management.user.create(
      loginId, loginId, undefined, undefined, ["moderator"],
    );
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    const first = await client.password.signIn(loginId, "Pass1Word!");
    const firstClaims = decodeJwt(first.data!.sessionJwt!);
    const firstRoles = (firstClaims.roles ?? firstClaims.r ?? []) as string[];
    expect(firstRoles).toContain("moderator");

    // Admin removes the role — e.g. the user's subscription lapsed.
    await client.management.user.removeRoles(loginId, ["moderator"]);

    const second = await client.password.signIn(loginId, "Pass1Word!");
    const secondClaims = decodeJwt(second.data!.sessionJwt!);
    const secondRoles = (secondClaims.roles ?? secondClaims.r ?? []) as string[];
    expect(secondRoles).not.toContain("moderator");
  });

  it("permissions are associated with roles and can be listed", async () => {
    const client = sdk();

    const pCreate = await client.management.permission.create("articles:write", "Write articles");
    expect(pCreate.ok).toBe(true);

    const rCreate = await client.management.role.create(
      "content-admin", "Manages content", ["articles:write"],
    );
    expect(rCreate.ok).toBe(true);

    const all = await client.management.role.loadAll();
    expect(all.ok).toBe(true);
    const created = all.data?.find((r) => r.name === "content-admin");
    expect(created).toBeTruthy();
    expect(created?.permissionNames).toContain("articles:write");
  });
});
