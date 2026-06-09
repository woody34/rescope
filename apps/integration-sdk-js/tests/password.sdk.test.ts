/**
 * Password flows via @descope/core-js-sdk.
 *
 * core-js-sdk is STATELESS. Tokens (sessionJwt, refreshJwt) come back
 * in res.data and must be passed explicitly to subsequent calls.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin, getEmulatorToken } from "../helpers/sdk";

beforeEach(() => resetEmulator());

describe("sdk.password.signUp", () => {
  it("signs up and returns sessionJwt + refreshJwt", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-signup");

    const res = await sdk.password.signUp(login, "Hunter2!");
    expect(res.ok).toBe(true);
    expect(typeof res.data?.sessionJwt).toBe("string");
    expect(typeof res.data?.refreshJwt).toBe("string");
    expect(res.data?.user?.loginIds).toContain(login);
  });

  it("rejects duplicate signup", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-dup");

    await sdk.password.signUp(login, "Hunter2!");
    const res2 = await sdk.password.signUp(login, "Hunter2!");
    expect(res2.ok).toBe(false);
  });
});

describe("sdk.password.signIn", () => {
  it("returns sessionJwt and refreshJwt for valid credentials", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-signin");

    await sdk.password.signUp(login, "Hunter2!");
    const res = await sdk.password.signIn(login, "Hunter2!");
    expect(res.ok).toBe(true);
    expect(res.data?.sessionJwt.split(".").length).toBe(3);
    expect(res.data?.refreshJwt?.split(".").length).toBe(3);
  });

  it("rejects wrong password with 401", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-bad");

    await sdk.password.signUp(login, "Hunter2!");
    const res = await sdk.password.signIn(login, "wrongpassword");
    expect(res.ok).toBe(false);
    expect(res.code).toBe(401);
  });

  it("rejects unknown user", async () => {
    const sdk = createClient();
    const res = await sdk.password.signIn("ghost@sdk.example", "any");
    expect(res.ok).toBe(false);
    expect(res.code).toBeGreaterThanOrEqual(400);
  });
});

describe("sdk.password.replace", () => {
  it("replaces password — old fails, new works", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-replace");

    await sdk.password.signUp(login, "OldPass1!");
    const replaceRes = await sdk.password.replace(login, "OldPass1!", "NewPass2!");
    expect(replaceRes.ok).toBe(true);

    const badRes = await sdk.password.signIn(login, "OldPass1!");
    expect(badRes.ok).toBe(false);
    expect(badRes.code).toBe(401);

    const goodRes = await sdk.password.signIn(login, "NewPass2!");
    expect(goodRes.ok).toBe(true);
  });

  it("rejects wrong old password with 401", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-replace-bad");

    await sdk.password.signUp(login, "OldPass1!");
    const res = await sdk.password.replace(login, "wrongOld", "NewPass2!");
    expect(res.ok).toBe(false);
    expect(res.code).toBe(401);
  });
});

describe("sdk.password.sendReset + update", () => {
  it("resets password via token and new credentials work", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-reset");

    await sdk.password.signUp(login, "OldPass1!");
    const sendRes = await sdk.password.sendReset(login, "http://localhost/reset");
    expect(sendRes.ok).toBe(true);

    // emulator returns token in body for offline tests
    const token = await getEmulatorToken("/v1/auth/password/reset", { loginId: login });
    const updateRes = await sdk.password.update(login, "NewPass1!", token);
    expect(updateRes.ok).toBe(true);

    const signinRes = await sdk.password.signIn(login, "NewPass1!");
    expect(signinRes.ok).toBe(true);
  });

  it("reset token is single-use", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pwd-reset-once");

    await sdk.password.signUp(login, "OldPass1!");
    const token = await getEmulatorToken("/v1/auth/password/reset", { loginId: login });

    await sdk.password.update(login, "NewPass1!", token);
    const res2 = await sdk.password.update(login, "NewPass2!", token);
    expect(res2.ok).toBe(false);
    expect(res2.code).toBe(401);
  });
});

// ─── GET /v1/auth/password/policy ─────────────────────────────────────────────

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

describe("GET /v1/auth/password/policy", () => {
  it("returns static permissive policy with expected shape", async () => {
    const res = await fetch(`${BASE_URL}/v1/auth/password/policy`);
    expect(res.status).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body.active).toBe(true);
    expect(body.minLength).toBe(6);
    expect(body.maxLength).toBe(128);
  });
});
