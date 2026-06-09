/**
 * Workflow 1 — Passwordless (Consumer App)
 *
 * Market use case: Consumer-facing apps reducing login friction via magic links
 * and OTP instead of passwords. No password storage, no password-related support.
 *
 * Journey (magic link variant):
 *   ADMIN CONFIG: create test user via mgmt API
 *      → CONSUMER ACTION: receive magic link token, verify it via SDK
 *      → Validate JWT signature + claims
 *      → Refresh session
 *      → Fetch own profile (me)
 *      → Logout (revoke session)
 *      → Confirm subsequent refresh is rejected (401)
 *
 * Journey (OTP variant):
 *   ADMIN CONFIG: create test user via mgmt API, generate OTP
 *      → CONSUMER ACTION: verify OTP code via SDK
 *      → Validate JWT + lifecycle
 */
import { describe, it, expect, beforeEach } from "vitest";
import {
  createClient,
  resetEmulator,
  uniqueLogin,
  mgmtCreateTestUser,
  mgmtGenerateMagicLink,
  mgmtAuth,
} from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";

beforeEach(() => resetEmulator());

// ─── Magic Link: full journey ─────────────────────────────────────────────────

describe("Workflow 1 — Passwordless via Magic Link", () => {
  it("full journey: create test user → generate link → verify → validate JWT → refresh → me → logout → revoked", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf1-ml");

    // ADMIN CONFIG: create a test user (no password needed)
    await mgmtCreateTestUser(login);

    // ADMIN CONFIG: generate magic link token (emulator returns token directly)
    const token = await mgmtGenerateMagicLink(login);
    expect(typeof token).toBe("string");
    expect(token.length).toBeGreaterThan(10);

    // CONSUMER ACTION: user clicks magic link → SDK verifies token
    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    const { sessionJwt, refreshJwt } = verifyRes.data!;
    expect(sessionJwt.split(".").length).toBe(3);
    expect(refreshJwt!.split(".").length).toBe(3);

    // CONSUMER ACTION: validate JWT signature and standard claims
    const validateRes = await fetch(`${BASE_URL}/v1/auth/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionJwt }),
    });
    expect(validateRes.ok).toBe(true);
    const validateBody = await validateRes.json() as { jwt: string; token: Record<string, unknown> };
    expect(validateBody.jwt).toBe(sessionJwt);
    expect(typeof validateBody.token.sub).toBe("string");
    expect(typeof validateBody.token.iss).toBe("string");

    // CONSUMER ACTION: refresh session (silent re-auth)
    const refreshRes = await sdk.refresh(refreshJwt!);
    expect(refreshRes.ok).toBe(true);
    expect(refreshRes.data!.sessionJwt.split(".").length).toBe(3);

    // CONSUMER ACTION: fetch own profile
    const meRes = await sdk.me(refreshJwt!);
    expect(meRes.ok).toBe(true);
    expect(meRes.data?.user?.loginIds).toContain(login);

    // CONSUMER ACTION: logout (revoke session)
    const logoutRes = await sdk.logout(refreshJwt!);
    expect(logoutRes.ok).toBe(true);

    // Post-logout: refresh must be rejected (session revoked)
    const postLogout = await sdk.refresh(refreshJwt!);
    expect(postLogout.ok).toBe(false);
    expect(postLogout.code).toBe(401);
  });

  it("magic link token is single-use — second verify is rejected", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf1-ml-single");

    await mgmtCreateTestUser(login);
    const token = await mgmtGenerateMagicLink(login);

    // First use succeeds
    const first = await sdk.magicLink.verify(token);
    expect(first.ok).toBe(true);

    // Second use fails
    const second = await sdk.magicLink.verify(token);
    expect(second.ok).toBe(false);
    expect(second.code).toBe(401);
  });
});

// ─── OTP: full journey ────────────────────────────────────────────────────────

describe("Workflow 1 — Passwordless via OTP (Email)", () => {
  it("full journey: create test user → generate OTP → verify code → validate JWT → refresh → logout", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf1-otp");

    // ADMIN CONFIG: create test user
    await mgmtCreateTestUser(login);

    // ADMIN CONFIG: generate OTP for test user via mgmt API
    const otpRes = await fetch(`${BASE_URL}/v1/mgmt/tests/generate/otp`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
      body: JSON.stringify({ loginId: login, deliveryMethod: "email" }),
    });
    expect(otpRes.ok).toBe(true);
    const otpBody = await otpRes.json() as { code: string };
    expect(otpBody.code).toMatch(/^\d{6}$/);

    // CONSUMER ACTION: user enters OTP code → SDK verifies
    const verifyRes = await sdk.otp.verify.email(login, otpBody.code);
    expect(verifyRes.ok).toBe(true);
    const { sessionJwt, refreshJwt } = verifyRes.data!;
    expect(sessionJwt.split(".").length).toBe(3);

    // CONSUMER ACTION: validate JWT claims
    const validateRes = await fetch(`${BASE_URL}/v1/auth/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionJwt }),
    });
    expect(validateRes.ok).toBe(true);

    // CONSUMER ACTION: refresh, then logout
    const refreshRes = await sdk.refresh(refreshJwt!);
    expect(refreshRes.ok).toBe(true);

    await sdk.logout(refreshJwt!);
    const post = await sdk.refresh(refreshJwt!);
    expect(post.ok).toBe(false);
  });

  it("OTP: signUpOrIn flow creates account on first use; code retrievable via emulator inspector", async () => {
    const sdk = createClient();
    const newLogin = uniqueLogin("wf1-otp-new");

    // CONSUMER ACTION: user triggers OTP send (signUpOrIn creates account + sends code)
    const signUpRes = await sdk.otp.signUpOrIn.email(newLogin);
    expect(signUpRes.ok).toBe(true);

    // EMULATOR CONVENIENCE: read the pending OTP code from the emulator inspector
    // (in real apps this arrives via email — the emulator exposes it for testing)
    const otpInspect = await fetch(`${BASE_URL}/emulator/otp/${encodeURIComponent(newLogin)}`);
    expect(otpInspect.ok).toBe(true);
    const { code } = await otpInspect.json() as { code: string };
    expect(code).toMatch(/^\d{6}$/);

    // CONSUMER ACTION: user enters OTP → account created + authenticated
    const verifyRes = await sdk.otp.verify.email(newLogin, code);
    expect(verifyRes.ok).toBe(true);
    const { sessionJwt, refreshJwt } = verifyRes.data!;
    expect(sessionJwt.split(".").length).toBe(3);

    // Can refresh and logout with newly created account
    expect((await sdk.refresh(refreshJwt!)).ok).toBe(true);
    await sdk.logout(refreshJwt!);
    expect((await sdk.refresh(refreshJwt!)).ok).toBe(false);
  });
});

