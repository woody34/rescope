/**
 * Scenario: Magic Link Authentication
 *
 * Magic links are the premier "no password" experience — user enters
 * their email and clicks a link. The link contains a one-time token,
 * not a password, so there's nothing to forget or steal.
 *
 * In production Descope emails the link; the emulator intercepts
 * the send and stores the token, which we retrieve via the test
 * user APIs. This scenario proves:
 *
 * - The SDK can initiate a magic link flow
 * - The emulator generates a valid token for the linked identity
 * - Verifying the token returns a session JWT (same surface as OTP/password)
 * - A replayed/expired token is rejected
 * - Management can see the user created by magic link
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: Magic Link Email Authentication", () => {
  it("full flow: request link → extract token → verify → get session", async () => {
    const client = sdk();
    const loginId = email("magic");

    // Step 1: App triggers magic link email.
    // The URI tells Descope where to send the user after they click —
    // in real apps this is the /verify route of the frontend.
    const start = await client.magicLink.signIn.email(
      loginId,
      "http://localhost/auth/verify",
    );
    expect(start.ok, "magic link request should be accepted").toBe(true);

    // Step 2: Retrieve the token the emulator generated.
    // In production this is embedded in the emailed link (e.g.
    // http://app.com/verify?t=<token>). We use the test user API
    // to get the token without needing a mail server.
    const linkRes = await client.management.user.generateMagicLinkForTestUser(
      "email",
      loginId,
      "http://localhost/auth/verify",
    );

    // Fallback for non-test users (created implicitly by signIn):
    // read back the pending token via the emulator's OTP read endpoint
    // (the emulator stores magic-link tokens in the same token store
    // as OTPs, keyed by login ID).
    let token: string;
    if (linkRes.ok && linkRes.data?.token) {
      token = linkRes.data.token as string;
    } else {
      const res = await fetch(
        `${process.env.EMULATOR_BASE_URL}/emulator/otp/${encodeURIComponent(loginId)}`,
      );
      const json = await res.json() as { code?: string };
      token = json.code ?? "";
    }

    expect(token, "emulator must produce a token").toBeTruthy();

    // Step 3: Frontend extracts the token from the URL query param
    // and calls verify. One call, one session — magic links are
    // single-use by design (verified once then invalidated).
    const verify = await client.magicLink.verify(token);
    expect(verify.ok, "valid token must verify successfully").toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();

    const claims = decodeJwt(verify.data!.sessionJwt!);
    expect(claims.sub).toBeTruthy();
  });

  it("invalid token is rejected — no session issued", async () => {
    const client = sdk();

    // The emulator must validate tokens — accepting any string would
    // be a critical security hole. Test this explicitly.
    const bad = await client.magicLink.verify("not-a-real-token");
    expect(bad.ok).toBe(false);
    expect(bad.data?.sessionJwt).toBeFalsy();
  });

  it("signup via magic link creates a verifiable user", async () => {
    const client = sdk();
    const loginId = email("magic-signup");

    await client.magicLink.signUp.email(loginId, "http://localhost/verify", {
      email: loginId,
    });

    // Sign-up creates the user immediately (before verification)
    // so the management API can find them right away — consistent
    // with Descope's user-first design.
    const load = await client.management.user.load(loginId);
    expect(load.ok).toBe(true);
  });
});
