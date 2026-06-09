/**
 * End-to-end cross-flow tests via @descope/core-js-sdk.
 * All tokens are passed explicitly — core-js-sdk is stateless.
 */
import { describe, it, expect, beforeEach } from "vitest";
import {
  createClient,
  resetEmulator,
  uniqueLogin,
  mgmtCreateTestUser,
  mgmtGenerateMagicLink,
  getEmulatorToken,
} from "../helpers/sdk";

beforeEach(() => resetEmulator());

// ─── Flow 1: Password signup → refresh → me → logout ─────────────────────────

describe("password: signup → refresh → me → logout", () => {
  it("completes the full SDK lifecycle", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-pwd");

    // Signup
    const signupRes = await sdk.password.signUp(login, "XFlow1!");
    expect(signupRes.ok).toBe(true);
    const { sessionJwt, refreshJwt } = signupRes.data!;
    expect(sessionJwt.split(".").length).toBe(3);

    // Refresh
    const refreshRes = await sdk.refresh(refreshJwt!);
    expect(refreshRes.ok).toBe(true);
    const newSessionJwt = refreshRes.data!.sessionJwt;
    expect(newSessionJwt.split(".").length).toBe(3);

    // Me
    const meRes = await sdk.me(refreshJwt!);
    expect(meRes.ok).toBe(true);
    expect(meRes.data?.user?.loginIds).toContain(login);

    // Logout
    const logoutRes = await sdk.logout(refreshJwt!);
    expect(logoutRes.ok).toBe(true);

    // Subsequent refresh fails
    const postLogout = await sdk.refresh(refreshJwt!);
    expect(postLogout.ok).toBe(false);
  });
});

// ─── Flow 2: Magic link → me → single-use ────────────────────────────────────

describe("magic link: test user → verify → me → single-use", () => {
  it("walks full test-user magic link lifecycle", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-ml");
    await mgmtCreateTestUser(login);

    const token = await mgmtGenerateMagicLink(login);
    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    expect(verifyRes.data?.user?.loginIds).toContain(login);

    const { refreshJwt } = verifyRes.data!;
    const meRes = await sdk.me(refreshJwt!);
    expect(meRes.ok).toBe(true);
    expect(meRes.data?.user?.loginIds).toContain(login);

    // Token is single-use
    const res2 = await sdk.magicLink.verify(token);
    expect(res2.ok).toBe(false);
    expect(res2.code).toBe(401);
  });
});

// ─── Flow 3: Password replace → new sign in works ────────────────────────────

describe("password replace: new credentials work, old rejected", () => {
  it("replace then signin", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-replace");

    await sdk.password.signUp(login, "OldPass1!");
    const replaceRes = await sdk.password.replace(login, "OldPass1!", "NewPass2!");
    expect(replaceRes.ok).toBe(true);

    const oldRes = await sdk.password.signIn(login, "OldPass1!");
    expect(oldRes.ok).toBe(false);
    expect(oldRes.code).toBe(401);

    const newRes = await sdk.password.signIn(login, "NewPass2!");
    expect(newRes.ok).toBe(true);
    expect(newRes.data?.sessionJwt.split(".").length).toBe(3);
  });
});

// ─── Flow 4: Reset password and use new credentials ──────────────────────────

describe("password reset flow: sendReset → update → signin", () => {
  it("full reset cycle", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-reset-flow");

    await sdk.password.signUp(login, "OldPass1!");
    await sdk.password.sendReset(login, "http://localhost/reset");

    const token = await getEmulatorToken("/v1/auth/password/reset", { loginId: login });
    const updateRes = await sdk.password.update(login, "NewPass1!", token);
    expect(updateRes.ok).toBe(true);

    const signinRes = await sdk.password.signIn(login, "NewPass1!");
    expect(signinRes.ok).toBe(true);
    expect(signinRes.data?.sessionJwt.split(".").length).toBe(3);
  });
});

// ─── Flow 5: Emulator reset invalidates sessions ─────────────────────────────

describe("emulator reset: me fails after reset", () => {
  it("session stops working after emulator reset", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-emreset");

    const signupRes = await sdk.password.signUp(login, "Reset1!");
    const { refreshJwt } = signupRes.data!;

    // me works before reset
    const before = await sdk.me(refreshJwt!);
    expect(before.ok).toBe(true);

    // Reset clears all state
    await resetEmulator();

    // me fails — user no longer exists
    const after = await sdk.me(refreshJwt!);
    expect(after.ok).toBe(false);
  });
});

// ─── Flow 6: User status — disable blocks signin, enable restores ─────────────

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
import { mgmtAuth } from "../helpers/sdk";

async function mgmtStatus(loginId: string, status: "enabled" | "disabled") {
  return fetch(`${BASE_URL}/v1/mgmt/user/status`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, status }),
  });
}

describe("user status: disable/enable via mgmt/user/status", () => {
  it("disabled user cannot sign in", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-disabled");
    await sdk.password.signUp(login, "Disable1!");

    await mgmtStatus(login, "disabled");

    const res = await sdk.password.signIn(login, "Disable1!");
    expect(res.ok).toBe(false);
    expect(res.code).toBe(403);
  });

  it("re-enabled user can sign in again", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-reenable");
    await sdk.password.signUp(login, "Enable1!");

    await mgmtStatus(login, "disabled");
    await mgmtStatus(login, "enabled");

    const res = await sdk.password.signIn(login, "Enable1!");
    expect(res.ok).toBe(true);
  });

  it("status endpoint returns 400 for unknown user", async () => {
    const res = await mgmtStatus("ghost@test.com", "disabled");
    expect(res.ok).toBe(false);
  });
});

// ─── Flow 7: Tenant remove + setRole ─────────────────────────────────────────

async function mgmtAddTenant(loginId: string, tenantId: string) {
  return fetch(`${BASE_URL}/v1/mgmt/user/tenant/add`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, tenantId, roleNames: ["viewer"] }),
  });
}

async function mgmtRemoveTenant(loginId: string, tenantId: string) {
  return fetch(`${BASE_URL}/v1/mgmt/user/tenant/remove`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, tenantId }),
  });
}

async function mgmtSetTenantRole(loginId: string, tenantId: string, roleNames: string[]) {
  return fetch(`${BASE_URL}/v1/mgmt/user/tenant/setRole`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: mgmtAuth },
    body: JSON.stringify({ loginId, tenantId, roleNames }),
  });
}

describe("tenant mutations: remove + setRole", () => {
  it("tenant remove removes the tenant from user", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-tenant-rm");
    await sdk.password.signUp(login, "Tenant1!");
    await mgmtAddTenant(login, "tenant-a");

    const res = await mgmtRemoveTenant(login, "tenant-a");
    expect(res.ok).toBe(true);
    const body = await res.json() as { user: { userTenants: unknown[] } };
    expect(body.user.userTenants).toHaveLength(0);
  });

  it("tenant remove is idempotent (absent tenant is a no-op)", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-tenant-rm-idm");
    await sdk.password.signUp(login, "Tenant1!");

    const res = await mgmtRemoveTenant(login, "non-existent-tenant");
    expect(res.ok).toBe(true);
  });

  it("tenant setRole replaces existing roles", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-tenant-setrole");
    await sdk.password.signUp(login, "Tenant1!");
    await mgmtAddTenant(login, "tenant-b");

    const res = await mgmtSetTenantRole(login, "tenant-b", ["admin"]);
    expect(res.ok).toBe(true);
    const body = await res.json() as { user: { userTenants: Array<{ roleNames: string[] }> } };
    expect(body.user.userTenants[0].roleNames).toContain("admin");
  });

  it("tenant setRole returns 400 if user not in tenant", async () => {
    const sdk = createClient();
    const login = uniqueLogin("xflow-tenant-setrole-missing");
    await sdk.password.signUp(login, "Tenant1!");

    const res = await mgmtSetTenantRole(login, "non-existent-tenant", ["admin"]);
    expect(res.ok).toBe(false);
  });
});
