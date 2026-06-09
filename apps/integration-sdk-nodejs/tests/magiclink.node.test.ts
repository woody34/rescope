/**
 * Magic Link authentication via @descope/node-sdk
 *
 * The emulator returns the token directly in the response body (test convenience).
 * Real Descope sends it via email/SMS.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin } from "../helpers/sdk.js";

beforeEach(() => resetEmulator());

describe("magicLink.signUpOrIn.email + verify", () => {
  it("signUpOrIn returns token; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml");
    const sendRes = await sdk.magicLink.signUpOrIn.email(login, "http://localhost/verify");
    expect(sendRes.ok).toBe(true);
    const token = (sendRes.data as Record<string, unknown>)?.token as string;
    expect(token).toBeTruthy();

    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    expect(verifyRes.data?.sessionJwt).toBeTruthy();
    expect(verifyRes.data?.refreshJwt).toBeTruthy();
  });

  it("verify with invalid token fails", async () => {
    const sdk = createClient();
    const res = await sdk.magicLink.verify("not-a-real-token");
    expect(res.ok).toBe(false);
  });

  it("token is single-use", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml");
    const sendRes = await sdk.magicLink.signUpOrIn.email(login, "http://localhost/verify");
    const token = (sendRes.data as Record<string, unknown>)?.token as string;

    await sdk.magicLink.verify(token);
    const reuse = await sdk.magicLink.verify(token);
    expect(reuse.ok).toBe(false);
  });
});

describe("magicLink.signUp.email", () => {
  it("signUp creates new user and returns token; verify gives tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-signup");
    const sendRes = await sdk.magicLink.signUp.email(login, "http://localhost/verify", { email: login });
    expect(sendRes.ok).toBe(true);
    const token = (sendRes.data as Record<string, unknown>)?.token as string;
    expect(token).toBeTruthy();

    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    expect(verifyRes.data?.sessionJwt).toBeTruthy();
  });
});

describe("magicLink.update.email", () => {
  it("updates email on authenticated user", async () => {
    const sdk = createClient();
    const login = uniqueLogin("ml-upd");
    const sendRes = await sdk.magicLink.signUpOrIn.email(login, "http://localhost/verify");
    const token = (sendRes.data as Record<string, unknown>)?.token as string;
    const verifyRes = await sdk.magicLink.verify(token);
    const refreshJwt = verifyRes.data?.refreshJwt as string;

    const newEmail = uniqueLogin("ml-new");
    // SDK signature: update.email(loginId, newEmail, redirectUrl, token?, loginOptions?)
    const updateSend = await sdk.magicLink.update.email(login, newEmail, "http://localhost/verify", refreshJwt);
    expect(updateSend.ok).toBe(true);
    const updateToken = (updateSend.data as Record<string, unknown>)?.token as string;
    expect(updateToken).toBeTruthy();

    const updateVerify = await sdk.magicLink.verify(updateToken);
    expect(updateVerify.ok).toBe(true);
  });
});
