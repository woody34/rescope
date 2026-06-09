import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── Auth helper ─────────────────────────────────────────────────────────────

async function signupAndSignin(login: string, password = "Hunter2!") {
  await client.post("/v1/auth/password/signup", { loginId: login, password, user: { email: login } });
  const res = await client.post("/v1/auth/password/signin", { loginId: login, password });
  return res.json() as Promise<{ sessionJwt: string; refreshJwt: string }>;
}

// ─── /v1/auth/refresh ────────────────────────────────────────────────────────

describe("POST /v1/auth/refresh", () => {
  it("issues a new session and refresh jwt", async () => {
    const login = uniqueLogin("refresh");
    const { refreshJwt } = await signupAndSignin(login);

    const res = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
  });

  it("fails with invalid refresh token", async () => {
    const res = await client.post("/v1/auth/refresh", { refreshJwt: "bad.jwt.here" });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("fails after logout (revoked)", async () => {
    const login = uniqueLogin("refresh-revoked");
    const { refreshJwt } = await signupAndSignin(login);

    await client.post("/v1/auth/logout", { refreshJwt });

    const res = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── /v1/auth/logout ─────────────────────────────────────────────────────────

describe("POST /v1/auth/logout", () => {
  it("returns ok and revokes the refresh token", async () => {
    const login = uniqueLogin("logout");
    const { refreshJwt } = await signupAndSignin(login);

    const res = await client.post("/v1/auth/logout", { refreshJwt });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.ok).toBe(true);

    // Subsequent refresh must fail
    const refreshRes = await client.post("/v1/auth/refresh", { refreshJwt });
    expect(refreshRes.status).toBeGreaterThanOrEqual(400);
  });

  it("rejects invalid token at logout", async () => {
    const res = await client.post("/v1/auth/logout", { refreshJwt: "garbage" });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── /v1/auth/me ─────────────────────────────────────────────────────────────

describe("GET /v1/auth/me", () => {
  it("returns user profile for valid refresh token", async () => {
    const login = uniqueLogin("me");
    const { refreshJwt } = await signupAndSignin(login);

    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/me`, {
      headers: {
        Authorization: `Bearer ${refreshJwt}`,
      },
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.loginIds).toContain(login);
  });

  it("rejects request with no token", async () => {
    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/me`);
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("rejects revoked refresh token", async () => {
    const login = uniqueLogin("me-revoked");
    const { refreshJwt } = await signupAndSignin(login);
    await client.post("/v1/auth/logout", { refreshJwt });

    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/me`, {
      headers: { Authorization: `Bearer ${refreshJwt}` },
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── /v1/auth/validate ───────────────────────────────────────────────────────

describe("POST /v1/auth/validate", () => {
  it("validates a fresh session JWT", async () => {
    const login = uniqueLogin("validate");
    const { sessionJwt } = await signupAndSignin(login);

    const res = await client.post("/v1/auth/validate", { sessionJwt });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.jwt).toBe("string");
    expect(body.token.sub).toBeTruthy();
    expect(body.token.email).toContain("@");
  });

  it("rejects an expired or invalid session JWT", async () => {
    const res = await client.post("/v1/auth/validate", { sessionJwt: "not.a.jwt" });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Cookie extraction ────────────────────────────────────────────────────────

describe("Session cookie flow", () => {
  it("refresh token from DSR cookie works", async () => {
    const login = uniqueLogin("cookie-flow");
    const signinRes = await client.post("/v1/auth/password/signup", {
      loginId: login,
      password: "Hunter2!",
      user: { email: login },
    });
    const cookies = signinRes.headers.getSetCookie?.() ?? [];
    const dsrCookie = cookies.find((c) => c.startsWith("DSR="));
    expect(dsrCookie).toBeTruthy();
    const dsr = dsrCookie!.split(";")[0].split("=").slice(1).join("=");

    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Cookie: `DSR=${dsr}` },
      body: JSON.stringify({}),
    });
    expect(res.status).toBe(200);
  });
});
