import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

describe("GET /health", () => {
  it("returns ok", async () => {
    const res = await client.get("/health");
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.ok).toBe(true);
  });
});

describe("POST /emulator/reset", () => {
  it("clears users between tests", async () => {
    const login = uniqueLogin("lifecycle");

    // Create user
    const create = await client.mgmtPost("/v1/mgmt/user/create", {
      loginId: login,
      email: login,
    });
    expect(create.status).toBe(200);

    // Reset
    const reset = await client.post("/emulator/reset", {});
    expect(reset.status).toBe(200);

    // User should no longer exist
    const load = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    expect(load.status).toBe(400);
  });

  it("re-applies seed data when configured (skips if no seed)", async () => {
    // Just verify reset doesn't crash with no seed configured
    const res = await client.post("/emulator/reset", {});
    expect(res.status).toBe(200);
  });
});

describe("CORS", () => {
  it("returns CORS headers on POST", async () => {
    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/health`, {
      headers: { Origin: "http://localhost:3000" },
    });
    expect(res.headers.get("access-control-allow-origin")).toBe("http://localhost:3000");
  });

  it("handles OPTIONS preflight", async () => {
    const res = await fetch(`${process.env.EMULATOR_BASE_URL}/v1/auth/password/signin`, {
      method: "OPTIONS",
      headers: {
        Origin: "http://localhost:3000",
        "Access-Control-Request-Method": "POST",
        "Access-Control-Request-Headers": "Content-Type,Authorization",
      },
    });
    expect([200, 204]).toContain(res.status);
    expect(res.headers.get("access-control-allow-origin")).toBe("http://localhost:3000");
  });
});

describe("GET /.well-known/jwks.json", () => {
  it("returns a valid JWKS document", async () => {
    const res = await client.get("/.well-known/jwks.json");
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(Array.isArray(body.keys)).toBe(true);
    const key = body.keys[0];
    expect(key.kty).toBe("RSA");
    expect(key.use).toBe("sig");
    expect(key.alg).toBe("RS256");
    expect(typeof key.kid).toBe("string");
    expect(key.kid.length).toBe(16);
    expect(typeof key.n).toBe("string");
    expect(typeof key.e).toBe("string");
  });

  it("kid matches the session JWT kid header", async () => {
    const login = uniqueLogin("jwks");
    await client.post("/v1/auth/password/signup", { loginId: login, password: "Hunter2!" });

    const signinRes = await client.post("/v1/auth/password/signin", {
      loginId: login,
      password: "Hunter2!",
    });
    const body = await signinRes.json();
    const [header] = body.sessionJwt.split(".");
    const decoded = JSON.parse(atob(header.replace(/-/g, "+").replace(/_/g, "/")));

    const jwksRes = await client.get("/.well-known/jwks.json");
    const jwks = await jwksRes.json();
    expect(jwks.keys[0].kid).toBe(decoded.kid);
  });
});
