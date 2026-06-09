/**
 * Magic link flows via @descope/core-js-sdk (stateless).
 * Tokens are extracted from res.data and passed explicitly.
 */
import { describe, it, expect, beforeEach } from "vitest";
import {
  createClient,
  resetEmulator,
  uniqueLogin,
  mgmtCreateUser,
  mgmtCreateTestUser,
  mgmtGenerateMagicLink,
  getEmulatorToken,
} from "../helpers/sdk";

beforeEach(() => resetEmulator());

describe("sdk.magicLink.signIn.email", () => {
  it("initiates magic link for existing user", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-signin");
    await mgmtCreateUser(login);

    const res = await sdk.magicLink.signIn.email(login, "http://localhost/verify");
    expect(res.ok).toBe(true);
  });

  it("auto-provisions unknown user (sign-in is sign-up-in)", async () => {
    const sdk = createClient();
    const res = await sdk.magicLink.signIn.email("nobody@nope.com", "http://localhost/verify");
    expect(res.ok).toBe(true);
  });
});

describe("sdk.magicLink.verify", () => {
  it("exchanges token for sessionJwt + refreshJwt", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-verify");
    await mgmtCreateUser(login);

    const token = await getEmulatorToken("/v1/auth/magiclink/signin/email", { loginId: login });
    const res = await sdk.magicLink.verify(token);
    expect(res.ok).toBe(true);
    expect(res.data?.sessionJwt.split(".").length).toBe(3);
    expect(res.data?.user?.loginIds).toContain(login);
  });

  it("token is single-use", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-single");
    await mgmtCreateUser(login);

    const token = await getEmulatorToken("/v1/auth/magiclink/signin/email", { loginId: login });
    await sdk.magicLink.verify(token);
    const res2 = await sdk.magicLink.verify(token);
    expect(res2.ok).toBe(false);
    expect(res2.code).toBe(401);
  });

  it("rejects invalid token", async () => {
    const sdk = createClient();
    const res = await sdk.magicLink.verify("deadbeef".repeat(8));
    expect(res.ok).toBe(false);
    expect(res.code).toBe(401);
  });
});

describe("mgmt-generated magic link for test user", () => {
  it("verifies successfully and returns user", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-testuser");
    await mgmtCreateTestUser(login);

    const token = await mgmtGenerateMagicLink(login);
    const res = await sdk.magicLink.verify(token);
    expect(res.ok).toBe(true);
    expect(res.data?.user?.loginIds).toContain(login);
  });
});

// ─── Magic link signUp (new user) ─────────────────────────────────────────────

describe("sdk.magicLink.signUp.email", () => {
  it("initiates sign-up for a brand-new login ID", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-signup");

    const res = await sdk.magicLink.signUp.email(login, "http://localhost/verify");
    expect(res.ok).toBe(true);
  });

  it("newly-signed-up user can verify token and receive JWTs", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-signup-verify");
    const BASE = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

    // signup/email auto-creates the user and returns a token
    const signupRes = await fetch(`${BASE}/v1/auth/magiclink/signup/email`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ loginId: login }),
    });
    expect(signupRes.ok).toBe(true);
    const signupBody = await signupRes.json() as { token: string };
    const token = signupBody.token;
    expect(typeof token).toBe("string");

    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    expect(typeof verifyRes.data?.sessionJwt).toBe("string");
  });
});

// ─── Magic link SMS flows ─────────────────────────────────────────────────────

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

async function mlPost(path: string, body: Record<string, unknown>) {
  const res = await fetch(`${BASE_URL}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

describe("POST /v1/auth/magiclink/signup/sms", () => {
  it("creates new phone user and returns token", async () => {
    const sdk = createClient();
    const { status, body } = await mlPost("/v1/auth/magiclink/signup/sms", { loginId: "+15550030001" });
    expect(status).toBe(200);
    expect(typeof body.token).toBe("string");
    expect(typeof body.maskedPhone).toBe("string");
    // Token is verifiable
    const verifyRes = await sdk.magicLink.verify(body.token as string);
    expect(verifyRes.ok).toBe(true);
    expect(typeof verifyRes.data?.sessionJwt).toBe("string");
  });

  it("returns 400 for duplicate phone number", async () => {
    await mlPost("/v1/auth/magiclink/signup/sms", { loginId: "+15550030002" });
    const { status } = await mlPost("/v1/auth/magiclink/signup/sms", { loginId: "+15550030002" });
    expect(status).toBe(400);
  });
});

describe("POST /v1/auth/magiclink/signin/sms", () => {
  it("returns token for existing phone user", async () => {
    const sdk = createClient();
    // Create user via SMS signup first
    await mlPost("/v1/auth/magiclink/signup/sms", { loginId: "+15550030003" });
    // Consume the signup token so state is clean
    const { body: signupBody } = await mlPost("/v1/auth/magiclink/signup/sms", { loginId: "+15550030004" });
    await sdk.magicLink.verify(signupBody.token as string);

    // Now signin
    const { status, body } = await mlPost("/v1/auth/magiclink/signin/sms", { loginId: "+15550030003" });
    expect(status).toBe(200);
    expect(typeof body.token).toBe("string");
  });

  it("returns 400 for unknown phone", async () => {
    const { status } = await mlPost("/v1/auth/magiclink/signin/sms", { loginId: "+10000000099" });
    expect(status).toBe(400);
  });
});

describe("POST /v1/auth/magiclink/signup-in/email", () => {
  it("creates new user and token is verifiable", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-signup-in-email-new");
    const { status, body } = await mlPost("/v1/auth/magiclink/signup-in/email", { loginId: login });
    expect(status).toBe(200);
    expect(typeof body.token).toBe("string");
    const verifyRes = await sdk.magicLink.verify(body.token as string);
    expect(verifyRes.ok).toBe(true);
  });

  it("signs in existing user without conflict", async () => {
    const login = uniqueLogin("ml-signup-in-email-existing");
    await mgmtCreateUser(login);
    const { status, body } = await mlPost("/v1/auth/magiclink/signup-in/email", { loginId: login });
    expect(status).toBe(200);
    expect(typeof body.token).toBe("string");
  });
});

describe("POST /v1/auth/magiclink/signup-in/sms", () => {
  it("creates new phone user and token is verifiable", async () => {
    const sdk = createClient();
    const { status, body } = await mlPost("/v1/auth/magiclink/signup-in/sms", { loginId: "+15550030010" });
    expect(status).toBe(200);
    expect(typeof body.token).toBe("string");
    const verifyRes = await sdk.magicLink.verify(body.token as string);
    expect(verifyRes.ok).toBe(true);
  });

  it("signs in existing phone user without conflict", async () => {
    await mlPost("/v1/auth/magiclink/signup/sms", { loginId: "+15550030011" });
    const { status } = await mlPost("/v1/auth/magiclink/signup-in/sms", { loginId: "+15550030011" });
    expect(status).toBe(200);
  });
});
