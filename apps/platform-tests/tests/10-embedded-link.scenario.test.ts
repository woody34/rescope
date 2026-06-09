/**
 * Scenario: CLI Auth / Embedded Sign-In Link
 *
 * Embedded sign-in links are Descope's answer to "how do I authenticate
 * a CLI tool, desktop app, or headless device that can't run a browser?"
 *
 * The flow: the platform creates a special one-time URL for the user.
 * The user opens the URL in their browser (out-of-band) and the CLI polls
 * for the session. This is the same pattern used by GitHub CLI and AWS SSO.
 *
 * The emulator exposes the embedded link at POST /v1/mgmt/user/embeddedlink
 * and its Node SDK alias /v1/mgmt/user/signin/embeddedlink.
 *
 * This scenario tests:
 * - Sign-in embedded link for an existing user → valid session on verify
 * - A token cannot be used twice (replay prevention)
 * - An invalid token is rejected with no session returned
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt, BASE_URL, MGMT_AUTH_HEADER } from "../helpers/platform.js";

beforeEach(reset);

const H = { "Content-Type": "application/json", Authorization: MGMT_AUTH_HEADER };

/** Generate a sign-in embedded link token for an existing user. */
async function generateSignInLink(loginId: string): Promise<string> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/user/signin/embeddedlink`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId }),
  });
  if (!res.ok) throw new Error(`generateSignInLink failed: ${res.status}`);
  const body = (await res.json()) as { token?: string };
  if (!body.token) throw new Error("No token in embedded link response");
  return body.token;
}

/** Verify a magic-link token — both embedded link variants use this path. */
async function verifyToken(token: string): Promise<Response> {
  return fetch(`${BASE_URL}/v1/auth/magiclink/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ token }),
  });
}

describe("Scenario: CLI Auth / Embedded Sign-In Link", () => {
  it("sign-in embedded link for existing user produces a valid session", async () => {
    const client = sdk();
    const loginId = email("cli-user");

    // Platform pre-creates the user (e.g. provisioned via SCIM or admin UI).
    const created = await client.management.user.create(loginId, loginId);
    expect(created.ok, "user creation should succeed").toBe(true);

    // Platform generates the one-time sign-in link.
    const token = await generateSignInLink(loginId);
    expect(token).toBeTruthy();

    // Verify the token — simulates the browser completing auth.
    const verifyRes = await verifyToken(token);
    expect(verifyRes.ok, "token verification must succeed").toBe(true);

    const session = (await verifyRes.json()) as { sessionJwt?: string };
    expect(session.sessionJwt, "verify must return a session JWT").toBeTruthy();

    // Confirm the JWT identifies a real user.
    const claims = decodeJwt(session.sessionJwt!);
    expect(typeof claims.sub).toBe("string");
    expect(claims.sub).toBeTruthy();
  });

  it("sign-in token cannot be used twice (replay prevention)", async () => {
    const client = sdk();
    const loginId = email("cli-replay");
    await client.management.user.create(loginId, loginId);

    const token = await generateSignInLink(loginId);

    // First use — valid.
    const first = await verifyToken(token);
    expect(first.ok, "first use of token should succeed").toBe(true);

    // Second use — the token is spent; emulator must reject it.
    const second = await verifyToken(token);
    expect(second.ok, "second use of the same token should be rejected").toBe(false);
  });

  it("invalid token returns an error without a session", async () => {
    const res = await verifyToken("not-a-real-token-xyz");

    expect(res.ok, "invalid token should fail").toBe(false);
    // Response body must not contain a session token.
    const body = (await res.json()) as { sessionJwt?: string };
    expect(body.sessionJwt).toBeFalsy();
  });

  it("embedded link for unknown user fails gracefully", async () => {
    const res = await fetch(`${BASE_URL}/v1/mgmt/user/signin/embeddedlink`, {
      method: "POST",
      headers: H,
      body: JSON.stringify({ loginId: "doesnotexist@nobody.com" }),
    });
    // Emulator should reject requests for users who don't exist.
    expect(res.ok, "embedded link for non-existent user should fail").toBe(false);
  });
});
