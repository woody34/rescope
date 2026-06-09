import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── Helper ───────────────────────────────────────────────────────────────────

async function signup(login: string, password = "Hunter2!") {
  return client.post("/v1/auth/password/signup", {
    loginId: login,
    password,
    user: { email: login },
  });
}

async function signin(login: string, password = "Hunter2!") {
  return client.post("/v1/auth/password/signin", { loginId: login, password });
}

// ─── Signup ───────────────────────────────────────────────────────────────────

describe("POST /v1/auth/password/signup", () => {
  it("creates a user and returns JWT response", async () => {
    const login = uniqueLogin("signup");
    const res = await signup(login);
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
    expect(body.user.loginIds).toContain(login);
  });

  it("sets DS and DSR cookies", async () => {
    const login = uniqueLogin("signup-cookie");
    const res = await signup(login);
    const cookieHeader = res.headers.getSetCookie?.() ?? [];
    const all = cookieHeader.join(" ");
    expect(all).toContain("DS=");
    expect(all).toContain("DSR=");
    expect(all).toContain("HttpOnly");
  });

  it("rejects duplicate loginId", async () => {
    const login = uniqueLogin("signup-dup");
    await signup(login);
    const res2 = await signup(login);
    expect(res2.status).toBe(400);
  });
});

// ─── Signin ───────────────────────────────────────────────────────────────────

describe("POST /v1/auth/password/signin", () => {
  it("returns session and refresh JWT for valid credentials", async () => {
    const login = uniqueLogin("signin");
    await signup(login);
    const res = await signin(login);
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
  });

  it("rejects wrong password with 401", async () => {
    const login = uniqueLogin("signin-bad");
    await signup(login);
    const res = await signin(login, "wrongpass");
    expect(res.status).toBe(401);
  });

  it("rejects unknown user with 4xx", async () => {
    const res = await signin("nobody@example.com");
    expect(res.status).toBeGreaterThanOrEqual(400);
    expect(res.status).toBeLessThan(500);
  });
});

// ─── Replace ──────────────────────────────────────────────────────────────────

describe("POST /v1/auth/password/replace", () => {
  it("changes password and allows signin with new password", async () => {
    const login = uniqueLogin("replace");
    await signup(login, "OldPass1!");
    const res = await client.post("/v1/auth/password/replace", {
      loginId: login,
      oldPassword: "OldPass1!",
      newPassword: "NewPass2!",
    });
    expect(res.status).toBe(200);
    // Old password now fails
    const failRes = await signin(login, "OldPass1!");
    expect(failRes.status).toBe(401);
    // New password works
    const okRes = await signin(login, "NewPass2!");
    expect(okRes.status).toBe(200);
  });

  it("rejects with wrong old password", async () => {
    const login = uniqueLogin("replace-bad");
    await signup(login, "OldPass1!");
    const res = await client.post("/v1/auth/password/replace", {
      loginId: login,
      oldPassword: "wrong",
      newPassword: "NewPass2!",
    });
    expect(res.status).toBe(401);
  });
});

// ─── Send reset ───────────────────────────────────────────────────────────────

describe("POST /v1/auth/password/reset", () => {
  it("returns masked email and token for existing user", async () => {
    const login = uniqueLogin("reset");
    await signup(login);
    const res = await client.post("/v1/auth/password/reset", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.ok).toBe(true);
    expect(typeof body.maskedEmail).toBe("string");
    expect(body.maskedEmail).toContain("@");
    expect(typeof body.token).toBe("string"); // emulator exposes token for test convenience
  });

  it("rejects unknown user", async () => {
    const res = await client.post("/v1/auth/password/reset", { loginId: "ghost@no.com" });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Update password ─────────────────────────────────────────────────────────

describe("POST /v1/auth/password/update", () => {
  it("updates password using reset token and enables new signin", async () => {
    const login = uniqueLogin("update-pwd");
    await signup(login, "OldPass1!");

    const resetRes = await client.post("/v1/auth/password/reset", { loginId: login });
    const { token } = await resetRes.json();

    const updateRes = await client.post("/v1/auth/password/update", {
      loginId: login,
      newPassword: "Updated1!",
      token,
    });
    expect(updateRes.status).toBe(200);

    const signinRes = await signin(login, "Updated1!");
    expect(signinRes.status).toBe(200);
  });

  it("token is single-use", async () => {
    const login = uniqueLogin("update-pwd-once");
    await signup(login);
    const resetRes = await client.post("/v1/auth/password/reset", { loginId: login });
    const { token } = await resetRes.json();

    await client.post("/v1/auth/password/update", { loginId: login, newPassword: "New1!", token });
    const res2 = await client.post("/v1/auth/password/update", { loginId: login, newPassword: "New2!", token });
    expect(res2.status).toBe(401);
  });
});
