/**
 * Scenario: New User Onboarding via Password
 *
 * Represents the complete lifecycle of a new user discovering an app,
 * creating an account with email+password, then coming back later and
 * signing in. Tests that the emulator correctly persists user state
 * between sign-up and sign-in — the #1 thing every app relies on.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt, signUpPassword } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: User Onboarding (Password)", () => {
  it("new user signs up, receives a valid session, then signs back in", async () => {
    const client = sdk();
    const loginId = email("onboard");

    // Step 1: User discovers the app and creates an account.
    // The SDK should return a session JWT immediately on sign-up —
    // Descope does this so users are not forced to separately sign in
    // after registration (frictionless onboarding).
    const signUp = await client.password.signUp(loginId, "Welcome1!", {
      email: loginId,
      displayName: "New User",
    });
    expect(signUp.ok, "sign-up should succeed").toBe(true);
    expect(signUp.data?.sessionJwt, "session JWT must be present on sign-up").toBeTruthy();

    // Step 2: Verify the JWT contains the expected claims.
    // The 'sub' claim is the user's stable Descope user ID — apps use
    // this as the primary key when creating their own user records.
    const claims = decodeJwt(signUp.data!.sessionJwt!);
    expect(claims.sub, "JWT must contain a user ID as 'sub'").toBeTruthy();
    expect(typeof claims.exp, "JWT must have an expiry").toBe("number");

    // Step 3: User closes the browser and returns later.
    // Sign-in should succeed with the same credentials and return
    // a new session JWT for the same user (same 'sub').
    const signIn = await client.password.signIn(loginId, "Welcome1!");
    expect(signIn.ok, "sign-in with correct password should succeed").toBe(true);
    expect(signIn.data?.sessionJwt).toBeTruthy();

    const signInClaims = decodeJwt(signIn.data!.sessionJwt!);
    expect(signInClaims.sub).toBe(claims.sub); // same user identity
  });

  it("sign-in with wrong password fails gracefully", async () => {
    const client = sdk();
    const loginId = email("wrong-pw");
    await signUpPassword(client, loginId);

    // Descope SDKs never throw — they return { ok: false, error: ... }.
    // Apps should check ok before accessing data. Verify this contract
    // is honored by the emulator.
    const bad = await client.password.signIn(loginId, "WrongPassword!");
    expect(bad.ok).toBe(false);
    expect(bad.data).toBeFalsy();
  });

  it("sign-in for unknown user fails gracefully", async () => {
    const client = sdk();
    const bad = await client.password.signIn("nobody@example.com", "Pass1!");
    expect(bad.ok).toBe(false);
  });

  it("admin can retrieve the user created via sign-up", async () => {
    const client = sdk();
    const loginId = email("admin-load");
    await signUpPassword(client, loginId);

    // Management APIs use the same identity store as auth APIs.
    // This is critical — if an admin creates/looks up a user, the
    // sign-in flow must find the same record and vice versa.
    const load = await client.management.user.load(loginId);
    expect(load.ok).toBe(true);
    expect(load.data?.email).toBe(loginId);
  });
});
