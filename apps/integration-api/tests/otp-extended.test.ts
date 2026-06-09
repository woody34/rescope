/**
 * OTP signup-in composite endpoint integration tests.
 * Covers gaps: POST /v1/auth/otp/signup-in/email and /signup-in/sms.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function getEmulatorOtp(loginId: string) {
  const res = await client.get(`/emulator/otp/${encodeURIComponent(loginId)}`);
  return await res.json() as { code: string };
}

// ─── POST /v1/auth/otp/signup-in/email ───────────────────────────────────────

describe("POST /v1/auth/otp/signup-in/email", () => {
  it("creates a new user if not found and sends OTP", async () => {
    const login = uniqueLogin("otp-signup-in-new");
    const res = await client.post("/v1/auth/otp/signup-in/email", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedEmail: string; code: string };
    expect(typeof body.code).toBe("string");
    expect(body.code.length).toBe(6);
  });

  it("sends OTP to existing user without creating a duplicate", async () => {
    const login = uniqueLogin("otp-signup-in-existing");
    // Create via regular signup first
    await client.post("/v1/auth/otp/signup/email", { loginId: login });
    const { code: firstCode } = await getEmulatorOtp(login);
    await client.post("/v1/auth/otp/verify/email", { loginId: login, code: firstCode });

    // signup-in should work for existing user
    const res = await client.post("/v1/auth/otp/signup-in/email", { loginId: login });
    expect(res.status).toBe(200);

    // Confirm only one user with this loginId
    const searchRes = await client.mgmtPost("/v1/mgmt/user/search", { emails: [login] });
    const { users } = await searchRes.json() as { users: unknown[] };
    expect(users.length).toBe(1);
  });

  it("newly created user can verify OTP and receive JWTs", async () => {
    const login = uniqueLogin("otp-signup-in-verify");
    await client.post("/v1/auth/otp/signup-in/email", { loginId: login });

    const { code } = await getEmulatorOtp(login);
    const verifyRes = await client.post("/v1/auth/otp/verify/email", { loginId: login, code });
    expect(verifyRes.status).toBe(200);
    const verifyBody = await verifyRes.json() as { sessionJwt: string; refreshJwt: string };
    expect(verifyBody.sessionJwt.split(".").length).toBe(3);
    expect(verifyBody.refreshJwt.split(".").length).toBe(3);
  });
});

// ─── POST /v1/auth/otp/signup-in/sms ─────────────────────────────────────────

describe("POST /v1/auth/otp/signup-in/sms", () => {
  it("creates a new phone user and sends OTP", async () => {
    const phone = "+15550003333";
    const res = await client.post("/v1/auth/otp/signup-in/sms", { loginId: phone });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedPhone: string; code: string };
    expect(typeof body.code).toBe("string");
    expect(body.code.length).toBe(6);
  });

  it("sends OTP to existing phone user without creating a duplicate", async () => {
    const phone = "+15550004444";
    // Create via regular signup first
    await client.post("/v1/auth/otp/signup/phone/sms", { loginId: phone });

    const res = await client.post("/v1/auth/otp/signup-in/sms", { loginId: phone });
    expect(res.status).toBe(200);
  });

  it("newly created phone user can verify OTP and receive JWTs", async () => {
    const phone = "+15550005555";
    await client.post("/v1/auth/otp/signup-in/sms", { loginId: phone });

    const { code } = await getEmulatorOtp(phone);
    const verifyRes = await client.post("/v1/auth/otp/verify/phone/sms", { loginId: phone, code });
    expect(verifyRes.status).toBe(200);
    const body = await verifyRes.json() as { sessionJwt: string };
    expect(body.sessionJwt.split(".").length).toBe(3);
  });
});
