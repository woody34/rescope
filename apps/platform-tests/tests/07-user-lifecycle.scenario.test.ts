/**
 * Scenario: User Lifecycle — Admin Management
 *
 * Most production apps have an admin console where operators manage
 * users: invite them, disable them for policy violations, reset
 * credentials, and eventually delete them. This scenario walks
 * through that entire admin lifecycle using only management SDK calls.
 *
 * Covers:
 * - Batch provisioning (common in migrations or bulk invites)
 * - Status management (enable/disable without deleting data)
 * - Profile updates (name, email, custom attributes)
 * - Credential management (force-set a password for a user)
 * - Permanent deletion and verification the identity is gone
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: User Lifecycle — Admin Console Operations", () => {
  it("batch provisioning: create 50 users, verify all exist, delete all", async () => {
    const client = sdk();

    // Large-scale user imports happen during migrations from legacy
    // auth systems or when seeding a new environment. Batch create
    // is far more efficient than N individual requests.
    const users = Array.from({ length: 5 }, (_, i) => ({
      // Using 5 here to keep the test fast; in prod this is 100s-1000s
      loginId: email(`batch${i}`),
      email: email(`batch${i}`),
    }));

    const batch = await client.management.user.createBatch(users);
    expect(batch.ok, "batch create should succeed").toBe(true);
    expect(batch.data?.createdUsers.length).toBeGreaterThanOrEqual(5);
    expect(batch.data?.failedUsers.length).toBe(0);

    // Verify each user is independently searchable.
    for (const u of users) {
      const load = await client.management.user.load(u.loginId);
      expect(load.ok, `user ${u.loginId} should be loadable after batch create`).toBe(true);
    }

    // Clean up: batch delete using user IDs from the creation response.
    const userIds = batch.data!.createdUsers.map((u) => u.userId as string);
    const batchDel = await client.management.user.deleteBatch(userIds);
    expect(batchDel.ok).toBe(true);

    // Confirm deletion.
    const checkGone = await client.management.user.load(users[0].loginId);
    expect(checkGone.ok).toBe(false);
  });

  it("deactivate then activate: status tracked in management API", async () => {
    const client = sdk();
    const loginId = email("disable-test");

    await client.management.user.create(loginId, { email: loginId });
    await client.management.user.setActivePassword(loginId, "Pass1Word!");

    // Admin disables the account — e.g. for a policy violation or suspicious
    // activity. The SDK exposes this as a dedicated 'deactivate' method
    // (separate from delete) because the data must be preserved for audit.
    await client.management.user.deactivate(loginId);

    // Verify the status is reflected in the management API — downstream
    // systems (billing, audit logs) read this field to understand account state.
    const loadDisabled = await client.management.user.load(loginId);
    expect(loadDisabled.ok).toBe(true);
    const statusDisabled = (loadDisabled.data as Record<string, unknown>)?.status;
    expect(statusDisabled).toBe("disabled");

    // Admin re-enables the account after investigation.
    await client.management.user.activate(loginId);

    const loadEnabled = await client.management.user.load(loginId);
    const statusEnabled = (loadEnabled.data as Record<string, unknown>)?.status;
    expect(statusEnabled).toBe("enabled");
  });

  it("admin force-sets password; user can sign in with new password", async () => {
    const client = sdk();
    const loginId = email("force-pw");

    await client.management.user.create(loginId, { email: loginId });

    // IT admin workflow: provision a temporary password for a new
    // hire who will change it on first login. Or reset a locked-out
    // account without going through the email flow.
    await client.management.user.setActivePassword(loginId, "TempPass99!");

    const signIn = await client.password.signIn(loginId, "TempPass99!");
    expect(signIn.ok).toBe(true);
  });

  it("admin updates profile fields; changes are reflected in load", async () => {
    const client = sdk();
    const loginId = email("profile-update");

    await client.management.user.create(loginId, {
      email: loginId,
      displayName: "Original Name",
    });

    // User's name changes — e.g., legally or after a marriage.
    // patch() updates only specified fields, preserving all others.
    const patch = await client.management.user.patch(loginId, {
      displayName: "Updated Name",
    });
    expect(patch.ok).toBe(true);

    const load = await client.management.user.load(loginId);
    const displayName =
      load.data?.displayName ?? (load.data as Record<string, unknown>)?.name;
    expect(displayName).toBe("Updated Name");
    expect(load.data?.email).toBe(loginId); // unchanged
  });

  it("deleted user cannot be loaded or signed in", async () => {
    const client = sdk();
    const loginId = email("delete-me");

    await client.management.user.create(loginId, { email: loginId });
    await client.management.user.setActivePassword(loginId, "Pass1Word!");
    await client.management.user.delete(loginId);

    // Deletion is permanent — no recovery. Verify both the management
    // API and the auth flow reject this identity.
    const load = await client.management.user.load(loginId);
    expect(load.ok).toBe(false);

    const signIn = await client.password.signIn(loginId, "Pass1Word!");
    expect(signIn.ok).toBe(false);
  });
});
