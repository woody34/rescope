/**
 * JWT management API integration tests.
 * Covers: POST /v1/mgmt/jwt/update — merges custom claims into session JWT.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client.js";

beforeEach(() => resetEmulator());

// ─── helpers ──────────────────────────────────────────────────────────────────

async function signedInJwt(login: string): Promise<{ sessionJwt: string; refreshJwt: string }> {
  await client.post("/v1/auth/password/signup", { loginId: login, password: "Test1234!" });
  const res = await client.post("/v1/auth/password/signin", { loginId: login, password: "Test1234!" });
  return await res.json() as { sessionJwt: string; refreshJwt: string };
}

function decodeJwtPayload(jwt: string): Record<string, unknown> {
  const parts = jwt.split(".");
  const payload = parts[1].replace(/-/g, "+").replace(/_/g, "/");
  return JSON.parse(atob(payload));
}

// ─── POST /v1/mgmt/jwt/update ────────────────────────────────────────────────

describe("POST /v1/mgmt/jwt/update", () => {
  it("merges custom claims into a new session JWT", async () => {
    const login = uniqueLogin("jwt-update");
    const { sessionJwt } = await signedInJwt(login);

    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { appRole: "admin", orgId: "org-123" },
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { jwt: string };
    expect(typeof body.jwt).toBe("string");
    expect(body.jwt.split(".").length).toBe(3);

    const claims = decodeJwtPayload(body.jwt);
    expect(claims["appRole"]).toBe("admin");
    expect(claims["orgId"]).toBe("org-123");
  });

  it("preserves standard claims (sub, iss, email) in updated JWT", async () => {
    const login = uniqueLogin("jwt-update-preserve");
    const { sessionJwt } = await signedInJwt(login);

    const original = decodeJwtPayload(sessionJwt);
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { foo: "bar" },
    });
    const body = await res.json() as { jwt: string };
    const updated = decodeJwtPayload(body.jwt);

    expect(updated["sub"]).toBe(original["sub"]);
    expect(updated["email"]).toBe(original["email"]);
    expect(updated["foo"]).toBe("bar");
  });

  it("works with empty customClaims (returns equivalent JWT)", async () => {
    const login = uniqueLogin("jwt-update-empty");
    const { sessionJwt } = await signedInJwt(login);

    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: {},
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { jwt: string };
    expect(body.jwt.split(".").length).toBe(3);
  });

  it("returns 401 for invalid JWT", async () => {
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: "not.a.jwt",
      customClaims: {},
    });
    expect(res.status).toBe(401);
  });

  it("returns 401 with invalid management auth", async () => {
    const login = uniqueLogin("jwt-update-noauth");
    const { sessionJwt } = await signedInJwt(login);

    const res = await fetch(`${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/v1/mgmt/jwt/update`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer wrong-project:wrong-key",
      },
      body: JSON.stringify({ jwt: sessionJwt, customClaims: {} }),
    });
    expect(res.status).toBe(401);
  });
});
