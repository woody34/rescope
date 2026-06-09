/**
 * Cross-SDK integration test: JS SDK login → Node SDK token validation.
 *
 * Signs up / signs in via @descope/core-js-sdk (browser-like, stateless),
 * then validates the resulting session JWT via @descope/node-sdk (server-side).
 */
import { describe, it, expect, beforeEach } from "vitest";
import createJsSdk from "@descope/core-js-sdk";
import { createSdkClient } from "../helpers/sdk-client";
import { resetEmulator, uniqueLogin } from "../helpers/client";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";

function createJsClient() {
  return createJsSdk({ projectId: PROJECT_ID, baseUrl: BASE_URL });
}

beforeEach(() => resetEmulator());

describe("cross-SDK: JS SDK login → Node SDK validation", () => {
  it("signup via JS SDK, validate session via Node SDK", async () => {
    const js = createJsClient();
    const node = createSdkClient();
    const login = uniqueLogin("cross-signup");

    // Sign up with JS SDK
    const signupRes = await js.password.signUp(login, "CrossTest1!");
    expect(signupRes.ok).toBe(true);
    const sessionJwt = signupRes.data!.sessionJwt as string;
    expect(sessionJwt.split(".").length).toBe(3);

    // Validate session with Node SDK
    const validated = await node.validateSession(sessionJwt);
    expect(validated).toBeTruthy();
    expect(validated.token.sub).toBeTruthy();
  });

  it("signin via JS SDK, validate session via Node SDK", async () => {
    const js = createJsClient();
    const node = createSdkClient();
    const login = uniqueLogin("cross-signin");

    // Sign up first
    await js.password.signUp(login, "CrossTest1!");

    // Sign in with JS SDK
    const signinRes = await js.password.signIn(login, "CrossTest1!");
    expect(signinRes.ok).toBe(true);
    const sessionJwt = signinRes.data!.sessionJwt as string;

    // Validate session with Node SDK
    const validated = await node.validateSession(sessionJwt);
    expect(validated).toBeTruthy();
    expect(validated.token.sub).toBeTruthy();
  });

  it("JS SDK refresh token works with Node SDK refreshSession", async () => {
    const js = createJsClient();
    const node = createSdkClient();
    const login = uniqueLogin("cross-refresh");

    const signupRes = await js.password.signUp(login, "CrossTest1!");
    const refreshJwt = signupRes.data!.refreshJwt as string;

    // Refresh via Node SDK using the JS SDK's refresh token
    const refreshed = await node.refreshSession(refreshJwt);
    expect(refreshed).toBeTruthy();
    expect(refreshed.token.sub).toBeTruthy();
  });

  it("JS SDK session is rejected by Node SDK after logout", async () => {
    const js = createJsClient();
    const node = createSdkClient();
    const login = uniqueLogin("cross-logout");

    const signupRes = await js.password.signUp(login, "CrossTest1!");
    const sessionJwt = signupRes.data!.sessionJwt as string;
    const refreshJwt = signupRes.data!.refreshJwt as string;

    // Validate works before logout
    const pre = await node.validateSession(sessionJwt);
    expect(pre.token.sub).toBeTruthy();

    // Logout via Node SDK
    await node.logout(refreshJwt);
    await new Promise((r) => setTimeout(r, 100));

    // Refresh should now be rejected
    await expect(node.refreshSession(refreshJwt)).rejects.toThrow();
  });
});
