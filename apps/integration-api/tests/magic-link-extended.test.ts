/**
 * Magic link SMS and composite (signup-in) endpoint integration tests.
 * Covers gaps: signup/sms, signin/sms, signup-in/email, signup-in/sms,
 *              update/phone/sms, update/email.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

const PHONE = "+15550001234";
const PHONE2 = "+15550005678";

// ─── Helpers ──────────────────────────────────────────────────────────────────

async function signedInTokens(login: string) {
  await client.post("/v1/auth/password/signup", { loginId: login, password: "Test1234!" });
  const res = await client.post("/v1/auth/password/signin", { loginId: login, password: "Test1234!" });
  return await res.json() as { sessionJwt: string; refreshJwt: string };
}

// ─── POST /v1/auth/magiclink/signup/sms ──────────────────────────────────────

describe("POST /v1/auth/magiclink/signup/sms", () => {
  it("creates a phone user and returns maskedPhone + token", async () => {
    const res = await client.post("/v1/auth/magiclink/signup/sms", { loginId: PHONE });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedPhone: string; token: string };
    expect(typeof body.maskedPhone).toBe("string");
    expect(typeof body.token).toBe("string");
  });

  it("returns 400 for duplicate phone (signup semantics)", async () => {
    await client.post("/v1/auth/magiclink/signup/sms", { loginId: PHONE2 });
    const res = await client.post("/v1/auth/magiclink/signup/sms", { loginId: PHONE2 });
    expect(res.status).toBe(400);
  });

  it("token is consumable via /v1/auth/magiclink/verify", async () => {
    const { body: signupBody } = await (async () => {
      const res = await client.post("/v1/auth/magiclink/signup/sms", { loginId: PHONE });
      return { status: res.status, body: await res.json() as { token: string } };
    })();
    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token: signupBody.token });
    expect(verifyRes.status).toBe(200);
    const verifyBody = await verifyRes.json() as { sessionJwt: string };
    expect(typeof verifyBody.sessionJwt).toBe("string");
  });
});

// ─── POST /v1/auth/magiclink/signin/sms ──────────────────────────────────────

describe("POST /v1/auth/magiclink/signin/sms", () => {
  it("sends magic link to existing phone user and returns token", async () => {
    // Create phone user via signup/sms first
    const signupRes = await client.post("/v1/auth/magiclink/signup/sms", { loginId: PHONE });
    const { token: signupToken } = await signupRes.json() as { token: string };
    // Consume to activate the user
    await client.post("/v1/auth/magiclink/verify", { token: signupToken });

    const res = await client.post("/v1/auth/magiclink/signin/sms", { loginId: PHONE });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedPhone: string; token: string };
    expect(typeof body.token).toBe("string");
    expect(body.maskedPhone.includes("*")).toBe(true);
  });

  it("returns 400 for unknown phone", async () => {
    const res = await client.post("/v1/auth/magiclink/signin/sms", { loginId: "+19999999999" });
    expect(res.status).toBe(400);
  });
});

// ─── POST /v1/auth/magiclink/signup-in/email ─────────────────────────────────

describe("POST /v1/auth/magiclink/signup-in/email", () => {
  it("creates a new user if not found and returns token", async () => {
    const login = uniqueLogin("ml-signup-in-email");
    const res = await client.post("/v1/auth/magiclink/signup-in/email", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedEmail: string; token: string };
    expect(typeof body.token).toBe("string");

    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token: body.token });
    expect(verifyRes.status).toBe(200);
  });

  it("sends magic link to existing user without creating a duplicate", async () => {
    const login = uniqueLogin("ml-signup-in-existing");
    // Create user first
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res1 = await client.post("/v1/auth/magiclink/signup-in/email", { loginId: login });
    expect(res1.status).toBe(200);

    // Confirm only one user exists with this loginId
    const searchRes = await client.mgmtPost("/v1/mgmt/user/search", { emails: [login] });
    const searchBody = await searchRes.json() as { users: unknown[] };
    expect(searchBody.users.length).toBe(1);
  });
});

// ─── POST /v1/auth/magiclink/signup-in/sms ───────────────────────────────────

describe("POST /v1/auth/magiclink/signup-in/sms", () => {
  it("creates user for new phone and returns token", async () => {
    const phone = "+15550007777";
    const res = await client.post("/v1/auth/magiclink/signup-in/sms", { loginId: phone });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedPhone: string; token: string };
    expect(typeof body.token).toBe("string");

    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token: body.token });
    expect(verifyRes.status).toBe(200);
  });

  it("returns token for existing phone user without duplicate", async () => {
    const phone = "+15550008888";
    // First call creates + sends
    const first = await client.post("/v1/auth/magiclink/signup-in/sms", { loginId: phone });
    const { token: firstToken } = await first.json() as { token: string };
    await client.post("/v1/auth/magiclink/verify", { token: firstToken });

    // Second call should succeed (user exists, send new link)
    const second = await client.post("/v1/auth/magiclink/signup-in/sms", { loginId: phone });
    expect(second.status).toBe(200);
  });
});

// ─── POST /v1/auth/magiclink/update/phone/sms ────────────────────────────────

describe("POST /v1/auth/magiclink/update/phone/sms", () => {
  it("persists new phone on user record and returns token", async () => {
    const login = uniqueLogin("ml-update-phone");
    const { refreshJwt } = await signedInTokens(login);

    const res = await fetch(`${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/v1/auth/magiclink/update/phone/sms`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ loginId: login, phone: "+15550006666" }),
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { maskedPhone: string; token: string };
    expect(typeof body.token).toBe("string");

    // Verify the phone was persisted
    const user = await (await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`)).json() as { user: Record<string, unknown> };
    expect(user.user.phone).toBe("+15550006666");
  });
});

// ─── POST /v1/auth/magiclink/update/email ────────────────────────────────────

describe("POST /v1/auth/magiclink/update/email", () => {
  it("persists new email on user record", async () => {
    const login = uniqueLogin("ml-update-email");
    const { refreshJwt } = await signedInTokens(login);
    const newEmail = uniqueLogin("new-email");

    const res = await fetch(`${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/v1/auth/magiclink/update/email`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${refreshJwt}` },
      body: JSON.stringify({ loginId: login, email: newEmail }),
    });
    expect(res.status).toBe(200);

    const user = await (await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`)).json() as { user: Record<string, unknown> };
    expect(user.user.email).toBe(newEmail);
  });
});
