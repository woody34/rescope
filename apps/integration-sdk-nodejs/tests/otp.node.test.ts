/**
 * OTP authentication via @descope/node-sdk
 *
 * The emulator returns the OTP code in the response body (test convenience).
 * Real Descope sends it via email/SMS.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin } from "../helpers/sdk.js";

beforeEach(() => resetEmulator());

describe("otp.signUpOrIn + verify (email)", () => {
  it("signUpOrIn.email returns code; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("otp-e");
    const signupRes = await sdk.otp.signUpOrIn.email(login);
    expect(signupRes.ok).toBe(true);
    const code = (signupRes.data as Record<string, unknown>)?.code as string;
    expect(code).toMatch(/^\d{6}$/);

    const verifyRes = await sdk.otp.verify.email(login, code);
    expect(verifyRes.ok).toBe(true);
    expect(verifyRes.data?.sessionJwt).toBeTruthy();
    expect(verifyRes.data?.refreshJwt).toBeTruthy();
  });

  it("verify with wrong code fails", async () => {
    const sdk = createClient();
    const login = uniqueLogin("otp-e");
    await sdk.otp.signUpOrIn.email(login);
    const res = await sdk.otp.verify.email(login, "000000");
    expect(res.ok).toBe(false);
  });

  it("code is single-use", async () => {
    const sdk = createClient();
    const login = uniqueLogin("otp-e");
    const signupRes = await sdk.otp.signUpOrIn.email(login);
    const code = (signupRes.data as Record<string, unknown>)?.code as string;

    await sdk.otp.verify.email(login, code);
    // Second use of same code must fail
    const reuse = await sdk.otp.verify.email(login, code);
    expect(reuse.ok).toBe(false);
  });
});

describe("otp.signUpOrIn (sms)", () => {
  it("signUpOrIn.sms creates user and returns code in response body", async () => {
    const sdk = createClient();
    const phone = "+15550019999";
    const signupRes = await sdk.otp.signUpOrIn.sms(phone);
    expect(signupRes.ok).toBe(true);
    const code = (signupRes.data as Record<string, unknown>)?.code as string;
    expect(code).toMatch(/^\d{6}$/);
  });
});

describe("otp.update.phone", () => {
  it("updates phone number on user record", async () => {
    const sdk = createClient();
    const login = uniqueLogin("otp-upd");
    // Sign up and get a refresh token
    const signupRes = await sdk.otp.signUpOrIn.email(login);
    const code = (signupRes.data as Record<string, unknown>)?.code as string;
    const verifyRes = await sdk.otp.verify.email(login, code);
    const refreshJwt = verifyRes.data?.refreshJwt as string;

    const newPhone = "+15550019001";
    // SDK: otp.update.phone.sms(loginId, phone, refreshJwt)
    const updateRes = await sdk.otp.update.phone.sms(login, newPhone, refreshJwt);
    expect(updateRes.ok).toBe(true);
  });
});
