/**
 * Scenario: OTP (Email Code) Authentication
 *
 * OTP is the most common passwordless flow. The user enters their
 * email, receives a 6-digit code, and submits it. In production
 * Descope emails the code; in the emulator the code is available
 * via the emulator API so tests can complete the flow without a
 * real mail server.
 *
 * This scenario validates:
 * - Sign-in starts a pending OTP challenge for the email
 * - If the user doesn't exist, Descope auto-provisions them (sign-up-in)
 * - The emulator stores the code; it's a valid 6-digit value
 * - Verifying the correct code returns a usable session JWT
 * - Verifying a wrong code is rejected (no session leak)
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt, BASE_URL } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: OTP Email Authentication", () => {
  it("full flow: sign-in → retrieve code → verify → get session", async () => {
    const client = sdk();
    const loginId = email("otp");

    // Step 1: App calls signIn to start the OTP challenge.
    // Descope sends an email in production; here the emulator intercepts
    // and stores the code. The user doesn't need to pre-register.
    const start = await client.otp.signIn.email(loginId);
    expect(start.ok, "OTP sign-in should start successfully").toBe(true);

    // Step 2: Read the code the emulator generated via the escape-hatch API.
    // In real apps this would come from the email inbox.
    const otpRes = await fetch(
      `${BASE_URL}/emulator/otp/${encodeURIComponent(loginId)}`,
    );
    const json = await otpRes.json() as { code?: string };
    const code = json.code ?? "";
    expect(code, "OTP must be exactly 6 digits").toMatch(/^\d{6}$/);

    // Step 3: User submits the code — same auth surface as password sign-in.
    const verify = await client.otp.verify.email(loginId, code);
    expect(verify.ok, "correct OTP must verify successfully").toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();

    const claims = decodeJwt(verify.data!.sessionJwt!);
    expect(claims.sub, "verified JWT must have a user identity").toBeTruthy();
  });

  it("wrong code is rejected — no session issued", async () => {
    const client = sdk();
    const loginId = email("otp-bad");
    await client.otp.signIn.email(loginId);

    const bad = await client.otp.verify.email(loginId, "000000");
    expect(bad.ok).toBe(false);
    expect(bad.data?.sessionJwt).toBeFalsy();
  });

  it("OTP sign-in auto-provisions the user (sign-up-in behavior)", async () => {
    const client = sdk();
    const loginId = email("otp-auto");

    // Real Descope: OTP signIn creates the user immediately if they don't exist.
    // This is "sign-up-in" — users never see a separate sign-up step.
    await client.otp.signIn.email(loginId);

    const load = await client.management.user.load(loginId);
    expect(load.ok, "user should be auto-provisioned by OTP signIn").toBe(true);
  });
});
