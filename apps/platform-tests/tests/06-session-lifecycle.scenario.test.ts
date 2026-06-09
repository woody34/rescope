/**
 * Scenario: Session Lifecycle
 *
 * Sessions are what Descope issues after any successful auth.
 * Understanding the session lifecycle — refresh, /me, and logout —
 * is critical for building secure apps.
 *
 * Covers:
 * - Refresh returns a new session JWT from a refresh JWT
 * - /me returns current user info (accepts both session and refresh JWT)
 * - Logout invalidates the session (refresh no longer works after logout)
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, signUpPassword } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: Session Lifecycle", () => {
  it("refresh issues a new session JWT from a valid refresh JWT", async () => {
    const client = sdk();
    const loginId = email("session-refresh");

    const signUp = await client.password.signUp(loginId, "Pass1Word!", { email: loginId });
    expect(signUp.ok).toBe(true);
    const { refreshJwt } = signUp.data!;
    expect(refreshJwt).toBeTruthy();

    // Refresh tokens are long-lived; session tokens are short-lived.
    // The client periodically uses the refresh token to get a new
    // session token — transparent to the app user.
    const refresh = await client.refresh(refreshJwt!);
    expect(refresh.ok, "refresh should succeed with valid refresh JWT").toBe(true);
    expect(refresh.data?.sessionJwt).toBeTruthy();
  });

  it("logout invalidates the refresh token — subsequent refresh fails", async () => {
    const client = sdk();
    const loginId = email("session-logout");

    const signUp = await client.password.signUp(loginId, "Pass1Word!", { email: loginId });
    const { refreshJwt } = signUp.data!;

    const logout = await client.logout(refreshJwt!);
    expect(logout.ok).toBe(true);

    // After logout the refresh token must be revoked — a key security guarantee.
    const tryRefresh = await client.refresh(refreshJwt!);
    expect(tryRefresh.ok, "refresh after logout should fail").toBe(false);
  });

  it("/me returns correct user details using the session JWT", async () => {
    const client = sdk();
    const loginId = email("session-me");
    await signUpPassword(client, loginId);

    const signIn = await client.password.signIn(loginId, "SecurePass1!");
    const { sessionJwt, refreshJwt } = signIn.data!;

    // /me accepts both session and refresh JWTs — use session JWT here.
    const me = await client.me(sessionJwt!);
    expect(me.ok).toBe(true);
    const userData = me.data as Record<string, unknown>;
    const loginIds = (userData?.loginIds ?? (userData?.user as Record<string, unknown>)?.loginIds) as string[] | undefined;
    expect(loginIds?.includes(loginId) ?? userData?.email === loginId).toBe(true);
    void refreshJwt;
  });
});
