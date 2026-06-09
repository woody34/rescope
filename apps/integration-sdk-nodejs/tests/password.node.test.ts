/**
 * Password authentication via @descope/node-sdk
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin } from "../helpers/sdk.js";

beforeEach(() => resetEmulator());

describe("password.signUp + signIn", () => {
  it("signUp returns sessionJwt and refreshJwt", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pw");
    const res = await sdk.password.signUp(login, "Pass1!", { email: login });
    expect(res.ok).toBe(true);
    expect(res.data?.sessionJwt).toBeTruthy();
    expect(res.data?.refreshJwt).toBeTruthy();
  });

  it("signIn with correct password returns tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pw");
    await sdk.password.signUp(login, "Pass1!", { email: login });
    const res = await sdk.password.signIn(login, "Pass1!");
    expect(res.ok).toBe(true);
    expect(res.data?.sessionJwt).toBeTruthy();
  });

  it("signIn with wrong password fails", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pw");
    await sdk.password.signUp(login, "Pass1!", { email: login });
    const res = await sdk.password.signIn(login, "wrong-pw");
    expect(res.ok).toBe(false);
  });
});

describe("password.replace", () => {
  it("replace issues new tokens and old password no longer works", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pw");
    await sdk.password.signUp(login, "OldPass1!", { email: login });
    const res = await sdk.password.replace(login, "OldPass1!", "NewPass1!");
    expect(res.ok).toBe(true);
    expect(res.data?.sessionJwt).toBeTruthy();

    // Old password no longer works
    const bad = await sdk.password.signIn(login, "OldPass1!");
    expect(bad.ok).toBe(false);

    // New password works
    const good = await sdk.password.signIn(login, "NewPass1!");
    expect(good.ok).toBe(true);
  });
});

describe("password.sendReset + update", () => {
  it("sendReset returns ok; update with reset token changes password", async () => {
    const sdk = createClient();
    const login = uniqueLogin("pw");
    await sdk.password.signUp(login, "Pass1!", { email: login });

    // Emulator returns reset token in body (test convenience — real Descope sends email)
    const resetRes = await sdk.password.sendReset(login, "http://localhost/reset");
    expect(resetRes.ok).toBe(true);
    const token = (resetRes.data as Record<string, unknown>)?.token as string;
    expect(token).toBeTruthy();

    // update(loginId, newPassword, refreshToken)
    const updateRes = await sdk.password.update(login, "NewPass2!", token);
    expect(updateRes.ok).toBe(true);

    const signinRes = await sdk.password.signIn(login, "NewPass2!");
    expect(signinRes.ok).toBe(true);
  });
});

describe("password.policy", () => {
  it("returns a policy object with expected shape", async () => {
    const sdk = createClient();
    // The auth client exposes this as sdk.password.policy() (not getPolicy)
    const res = await sdk.password.policy();
    expect(res.ok).toBe(true);
    expect(res.data).toBeDefined();
    expect(typeof res.data?.minLength).toBe("number");
  });
});
