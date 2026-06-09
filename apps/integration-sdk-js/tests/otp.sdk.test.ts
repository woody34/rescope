/**
 * OTP authentication flows via raw fetch (SDK doesn't have OTP API).
 * Tests: email signup/signin/verify, phone/sms signup/signin/verify,
 * emulator escape hatch (GET /emulator/otp/:loginId),
 * mgmt generate OTP for test user, disabled-user blocking.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin, mgmtAuth } from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function otpSignupEmail(loginId: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/signup/email`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function otpSigninEmail(loginId: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/signin/email`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function otpVerifyEmail(loginId: string, code: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/verify/email`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId, code }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function getEmulatorOtp(loginId: string) {
  const res = await fetch(`${BASE_URL}/emulator/otp/${encodeURIComponent(loginId)}`);
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function mgmtGenerateOtp(loginId: string, deliveryMethod = "email") {
  const res = await fetch(`${BASE_URL}/v1/mgmt/tests/generate/otp`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, deliveryMethod }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function createTestUser(loginId: string) {
  await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, email: loginId }),
  });
}

// ─── OTP email signup ─────────────────────────────────────────────────────────

describe("POST /v1/auth/otp/signup/email", () => {
  it("creates user and returns maskedEmail + code", async () => {
    const login = uniqueLogin("otp-signup");
    const { status, body } = await otpSignupEmail(login);
    expect(status).toBe(200);
    expect(typeof body.maskedEmail).toBe("string");
    expect(typeof body.code).toBe("string");
    expect((body.code as string).length).toBe(6);
  });

  it("returns 400 for duplicate loginId", async () => {
    const login = uniqueLogin("otp-signup-dup");
    await otpSignupEmail(login);
    const { status } = await otpSignupEmail(login);
    expect(status).toBe(400);
  });
});

// ─── OTP email signin ─────────────────────────────────────────────────────────

describe("POST /v1/auth/otp/signin/email", () => {
  it("returns code for existing user", async () => {
    const login = uniqueLogin("otp-signin");
    await otpSignupEmail(login);
    const { status, body } = await otpSigninEmail(login);
    expect(status).toBe(200);
    expect(typeof body.code).toBe("string");
  });

  it("auto-provisions unknown user (sign-in is sign-up-in)", async () => {
    const { status, body } = await otpSigninEmail("ghost-otp@nothing.com");
    expect(status).toBe(200);
    expect(typeof body.code).toBe("string");
  });
});

// ─── OTP verify email ─────────────────────────────────────────────────────────

describe("POST /v1/auth/otp/verify/email", () => {
  it("full round-trip: signup → escape-hatch → verify → tokens", async () => {
    const login = uniqueLogin("otp-verify");
    await otpSignupEmail(login);

    // Get code via emulator escape hatch
    const { body: otpBody } = await getEmulatorOtp(login);
    const code = otpBody.code as string;

    const { status, body } = await otpVerifyEmail(login, code);
    expect(status).toBe(200);
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
  });

  it("code is single-use (second verify → 401)", async () => {
    const login = uniqueLogin("otp-single-use");
    await otpSignupEmail(login);
    const { body: otpBody } = await getEmulatorOtp(login);
    const code = otpBody.code as string;

    await otpVerifyEmail(login, code);
    const { status } = await otpVerifyEmail(login, code);
    expect(status).toBe(401);
  });

  it("wrong code returns 401", async () => {
    const login = uniqueLogin("otp-wrong-code");
    await otpSignupEmail(login);
    const { status } = await otpVerifyEmail(login, "000000");
    expect(status).toBe(401);
  });
});

// ─── Emulator escape hatch: GET /emulator/otp/:loginId ───────────────────────

describe("GET /emulator/otp/:loginId", () => {
  it("returns pending code without consuming it", async () => {
    const login = uniqueLogin("otp-escape");
    await otpSignupEmail(login);

    const { status, body: first } = await getEmulatorOtp(login);
    expect(status).toBe(200);
    const { body: second } = await getEmulatorOtp(login);
    // Same code returned both times — peek doesn't consume
    expect(first.code).toBe(second.code);
  });

  it("returns 400 for unknown loginId", async () => {
    const { status } = await getEmulatorOtp("nobody@test.com");
    expect(status).toBe(400);
  });

  it("returns 401 when no code is pending", async () => {
    // Signup but don't yet trigger code generation
    const login = uniqueLogin("otp-no-pending");
    await createTestUser(login);
    const { status } = await getEmulatorOtp(login);
    expect(status).toBe(401);
  });
});

// ─── mgmt.generateOTPForTestUser ─────────────────────────────────────────────

describe("POST /v1/mgmt/tests/generate/otp", () => {
  it("generates OTP for test user and code is usable", async () => {
    const login = uniqueLogin("otp-mgmt-gen");
    await createTestUser(login);

    const { status, body } = await mgmtGenerateOtp(login);
    expect(status).toBe(200);
    expect((body.code as string).length).toBe(6);
    expect(body.loginId).toBe(login);
  });

  it("returns 400 for non-test user", async () => {
    const login = uniqueLogin("otp-mgmt-regular");
    // Create regular (non-test) user via signup
    await otpSignupEmail(login);
    const { status } = await mgmtGenerateOtp(login);
    expect(status).toBe(400);
  });
});

// ─── OTP phone/sms ────────────────────────────────────────────────────────────

const PHONE = "+15550001234";
const PHONE2 = "+15550005678";

async function otpSignupPhone(phone: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/signup/sms`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId: phone }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function otpSigninPhone(phone: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/signin/sms`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId: phone }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function otpVerifyPhone(phone: string, code: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/verify/sms`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId: phone, code }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

describe("POST /v1/auth/otp/signup/sms", () => {
  it("creates user with phone and returns maskedPhone + code", async () => {
    const { status, body } = await otpSignupPhone(PHONE);
    expect(status).toBe(200);
    expect(typeof body.maskedPhone).toBe("string");
    expect(typeof body.code).toBe("string");
    expect((body.code as string).length).toBe(6);
  });

  it("returns 400 for duplicate phone number", async () => {
    await otpSignupPhone(PHONE2);
    const { status } = await otpSignupPhone(PHONE2);
    expect(status).toBe(400);
  });
});

describe("POST /v1/auth/otp/signin/sms", () => {
  it("returns code for existing phone user", async () => {
    await otpSignupPhone(PHONE);
    const { status, body } = await otpSigninPhone(PHONE);
    expect(status).toBe(200);
    expect(typeof body.code).toBe("string");
  });

  it("returns 400 for unknown phone", async () => {
    const { status } = await otpSigninPhone("+19990000000");
    expect(status).toBe(400);
  });
});

describe("POST /v1/auth/otp/verify/sms", () => {
  it("full round-trip: signup → escape-hatch → verify → tokens", async () => {
    await otpSignupPhone(PHONE);
    const { body: otpBody } = await getEmulatorOtp(PHONE);
    const code = otpBody.code as string;
    const { status, body } = await otpVerifyPhone(PHONE, code);
    expect(status).toBe(200);
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
  });

  it("code is single-use (second verify → 401)", async () => {
    await otpSignupPhone(PHONE);
    const { body: otpBody } = await getEmulatorOtp(PHONE);
    const code = otpBody.code as string;
    await otpVerifyPhone(PHONE, code);
    const { status } = await otpVerifyPhone(PHONE, code);
    expect(status).toBe(401);
  });

  it("wrong code returns 401", async () => {
    await otpSignupPhone(PHONE);
    const { status } = await otpVerifyPhone(PHONE, "000000");
    expect(status).toBe(401);
  });
});

// ─── OTP update phone ─────────────────────────────────────────────────────────

describe("POST /v1/auth/otp/update/phone/sms", () => {
  it("returns ok for a logged-in user updating their phone", async () => {
    // Sign up via email, get tokens, then update phone
    const login = uniqueLogin("otp-update-phone");
    await otpSignupEmail(login);
    const { body: otpBody } = await getEmulatorOtp(login);
    const code = otpBody.code as string;
    const { body: verifyBody } = await otpVerifyEmail(login, code);
    const refreshJwt = verifyBody.refreshJwt as string;

    const res = await fetch(`${BASE_URL}/v1/auth/otp/update/phone/sms`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ loginId: login, phone: "+15550009999" }),
    });
    expect(res.ok).toBe(true);
  });
});

// ─── OTP signup-in (email) ────────────────────────────────────────────────────

async function otpSignupInEmail(loginId: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/signup-in/email`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function otpSignupInPhone(phone: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/otp/signup-in/sms`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ loginId: phone }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

describe("POST /v1/auth/otp/signup-in/email", () => {
  it("creates new user and returns code when user does not exist", async () => {
    const login = uniqueLogin("otp-signup-in-new");
    const { status, body } = await otpSignupInEmail(login);
    expect(status).toBe(200);
    expect(typeof body.maskedEmail).toBe("string");
    const code = body.code as string;
    expect(code.length).toBe(6);
    // Code is verifiable → full round-trip
    const { status: verifyStatus, body: tokens } = await otpVerifyEmail(login, code);
    expect(verifyStatus).toBe(200);
    expect(typeof tokens.sessionJwt).toBe("string");
  });

  it("signs in existing user without creating a duplicate", async () => {
    const login = uniqueLogin("otp-signup-in-existing");
    // Pre-create user via regular signup
    await otpSignupEmail(login);
    const { body: otpBody } = await getEmulatorOtp(login);
    await otpVerifyEmail(login, otpBody.code as string);

    // signup-in should work and not fail with conflict
    const { status, body } = await otpSignupInEmail(login);
    expect(status).toBe(200);
    expect(typeof body.code).toBe("string");
  });
});

describe("POST /v1/auth/otp/signup-in/sms", () => {
  it("creates new phone user and returns code", async () => {
    const phone = "+15550007001";
    const { status, body } = await otpSignupInPhone(phone);
    expect(status).toBe(200);
    expect(typeof body.maskedPhone).toBe("string");
    expect((body.code as string).length).toBe(6);
    // Verify round-trip
    const { status: vs } = await otpVerifyPhone(phone, body.code as string);
    expect(vs).toBe(200);
  });

  it("signs in existing phone user without duplicate", async () => {
    const phone = "+15550007002";
    await otpSignupPhone(phone);
    const { body: otpBody } = await getEmulatorOtp(phone);
    await otpVerifyPhone(phone, otpBody.code as string);

    const { status } = await otpSignupInPhone(phone);
    expect(status).toBe(200);
  });
});
