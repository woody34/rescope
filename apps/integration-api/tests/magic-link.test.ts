import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

async function createAndGetToken(login: string) {
  await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });
  const signInRes = await client.post("/v1/auth/magiclink/signin/email", { loginId: login });
  const body = await signInRes.json();
  return { token: body.token as string, maskedEmail: body.maskedEmail as string };
}

// ─── Signin / initiation ─────────────────────────────────────────────────────

describe("POST /v1/auth/magiclink/signin/email", () => {
  it("returns masked email and test token for existing user", async () => {
    const login = uniqueLogin("ml-signin");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.post("/v1/auth/magiclink/signin/email", { loginId: login });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.ok).toBe(true);
    expect(typeof body.maskedEmail).toBe("string");
    expect(body.maskedEmail).toContain("@");
    expect(typeof body.token).toBe("string");
    expect(body.token.length).toBe(64);
  });

  it("auto-provisions unknown user (sign-in is sign-up-in)", async () => {
    const res = await client.post("/v1/auth/magiclink/signin/email", {
      loginId: "ghost@nowhere.com",
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.token).toBe("string");
  });
});

// ─── Verify ───────────────────────────────────────────────────────────────────

describe("POST /v1/auth/magiclink/verify", () => {
  it("exchanges token for session and refresh JWTs", async () => {
    const login = uniqueLogin("ml-verify");
    const { token } = await createAndGetToken(login);

    const res = await client.post("/v1/auth/magiclink/verify", { token });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
    expect(body.user.loginIds).toContain(login);
  });

  it("sets DS and DSR cookies on verify", async () => {
    const login = uniqueLogin("ml-verify-cookie");
    const { token } = await createAndGetToken(login);
    const res = await client.post("/v1/auth/magiclink/verify", { token });
    const cookies = res.headers.getSetCookie?.() ?? [];
    const all = cookies.join(" ");
    expect(all).toContain("DS=");
    expect(all).toContain("DSR=");
  });

  it("token is single-use", async () => {
    const login = uniqueLogin("ml-single");
    const { token } = await createAndGetToken(login);
    await client.post("/v1/auth/magiclink/verify", { token });
    const res2 = await client.post("/v1/auth/magiclink/verify", { token });
    expect(res2.status).toBe(401);
  });

  it("rejects invalid token", async () => {
    const res = await client.post("/v1/auth/magiclink/verify", { token: "deadbeef".repeat(8) });
    expect(res.status).toBe(401);
  });
});

// ─── Update email ─────────────────────────────────────────────────────────────

describe("POST /v1/auth/magiclink/update/email", () => {
  it("accepts request with valid refresh token", async () => {
    const login = uniqueLogin("ml-update-email");
    const { token } = await createAndGetToken(login);
    const verifyRes = await client.post("/v1/auth/magiclink/verify", { token });
    const { refreshJwt } = await verifyRes.json();

    const res = await client.post("/v1/auth/magiclink/update/email", {
      loginId: login,
      email: "new@example.com",
      token: refreshJwt,
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.ok).toBe(true);
  });

  it("returns ok even without a token (emulator is lenient)", async () => {
    const login = uniqueLogin("ml-update-no-token");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });
    const res = await client.post("/v1/auth/magiclink/update/email", {
      loginId: login,
      email: "updated@x.com",
    });
    expect(res.status).toBe(200);
  });
});
