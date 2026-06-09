/**
 * Integration tests for custom attribute and JWT claim validation limits.
 *
 * Validates:
 * - Custom attribute machine names accept camelCase
 * - Machine name length limit (60 chars)
 * - Machine name pattern validation (letters/digits/underscores only)
 * - JWT custom claim key length limit (60 chars)
 * - JWT custom claim value length limit (500 chars)
 * - JWT custom claim key count limit (100 keys)
 */
import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client.js";

beforeEach(() => resetEmulator());

// ─── Custom Attribute Machine Name Validation ────────────────────────────────

describe("Custom Attribute Machine Names", () => {
  it("accepts camelCase machine names", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Change Password",
      machineName: "changePassword",
      attributeType: "text",
      permissions: "all",
    });
    expect(res.status).toBe(200);

    // Verify it shows up in the list
    const list = await client.mgmtGet("/v1/mgmt/user/attribute/all");
    const body = await list.json();
    const attrs = body.attributes ?? body;
    const found = (Array.isArray(attrs) ? attrs : []).find(
      (a: any) => a.machineName === "changePassword"
    );
    expect(found).toBeTruthy();
    expect(found.name).toBe("Change Password");
  });

  it("accepts snake_case machine names", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "User Plan",
      machineName: "user_plan",
      attributeType: "text",
      permissions: "all",
    });
    expect(res.status).toBe(200);
  });

  it("accepts PascalCase machine names", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Feature Flag",
      machineName: "FeatureFlag",
      attributeType: "boolean",
      permissions: "all",
    });
    expect(res.status).toBe(200);
  });

  it("accepts machine name at exactly 60 chars", async () => {
    const machineName = "a".repeat(60);
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Max Length Attr",
      machineName,
      attributeType: "text",
      permissions: "all",
    });
    expect(res.status).toBe(200);
  });

  it("rejects machine name exceeding 60 chars", async () => {
    const machineName = "a".repeat(61);
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Too Long",
      machineName,
      attributeType: "text",
      permissions: "all",
    });
    expect(res.status).toBe(400);
  });

  it("rejects machine name starting with digit", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Bad Start",
      machineName: "1startsWithDigit",
      attributeType: "text",
      permissions: "all",
    });
    expect(res.status).toBe(400);
  });

  it("rejects machine name with special characters", async () => {
    const res = await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Special Chars",
      machineName: "has-dashes",
      attributeType: "text",
      permissions: "all",
    });
    expect(res.status).toBe(400);
  });
});

// ─── User Custom Attribute Values ────────────────────────────────────────────

describe("User Custom Attribute Values via updateCustomAttribute", () => {
  it("sets a camelCase custom attribute on a user", async () => {
    const loginId = uniqueLogin("camel");

    // Define the attribute schema
    await client.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Change Password",
      machineName: "changePassword",
      attributeType: "boolean",
      permissions: "all",
    });

    // Create user
    await client.mgmtPost("/v1/mgmt/user/create", { loginId });

    // Set the attribute
    const res = await client.mgmtPost("/v1/mgmt/user/update/customAttribute", {
      loginId,
      attributeKey: "changePassword",
      attributeValue: true,
    });
    expect(res.status).toBe(200);

    // Verify
    const loadRes = await client.mgmtGet(
      `/v1/mgmt/user?loginid=${encodeURIComponent(loginId)}`
    );
    const loadBody = await loadRes.json();
    const user = loadBody.user;
    expect(user.customAttributes.changePassword).toBe(true);
  });
});

// ─── JWT Custom Claim Limits ─────────────────────────────────────────────────

describe("JWT Custom Claim Limits", () => {
  async function getSessionJwt(loginId: string): Promise<string> {
    const signupRes = await client.post("/v1/auth/password/signup", {
      loginId,
      password: "TestPass1!",
    });
    const body = await signupRes.json();
    return body.sessionJwt;
  }

  it("accepts custom claims within limits", async () => {
    const loginId = uniqueLogin("jwt-ok");
    const sessionJwt = await getSessionJwt(loginId);

    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: {
        appRole: "admin",
        tier: "enterprise",
      },
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.jwt).toBeTruthy();
  });

  it("rejects claim key exceeding 60 chars", async () => {
    const loginId = uniqueLogin("jwt-key");
    const sessionJwt = await getSessionJwt(loginId);

    const longKey = "k".repeat(61);
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { [longKey]: "value" },
    });
    expect(res.status).toBe(400);
  });

  it("accepts claim key at exactly 60 chars", async () => {
    const loginId = uniqueLogin("jwt-key-ok");
    const sessionJwt = await getSessionJwt(loginId);

    const exactKey = "k".repeat(60);
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { [exactKey]: "value" },
    });
    expect(res.status).toBe(200);
  });

  it("rejects claim value exceeding 500 chars", async () => {
    const loginId = uniqueLogin("jwt-val");
    const sessionJwt = await getSessionJwt(loginId);

    const longVal = "v".repeat(501);
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { myKey: longVal },
    });
    expect(res.status).toBe(400);
  });

  it("accepts claim value at exactly 498 chars (500 when JSON-serialised)", async () => {
    const loginId = uniqueLogin("jwt-val-ok");
    const sessionJwt = await getSessionJwt(loginId);

    // JSON serialisation of a string adds 2 quote chars => "vvv..." = 500 chars
    const exactVal = "v".repeat(498);
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: { myKey: exactVal },
    });
    expect(res.status).toBe(200);
  });

  it("rejects more than 100 custom claim keys", async () => {
    const loginId = uniqueLogin("jwt-count");
    const sessionJwt = await getSessionJwt(loginId);

    const tooManyClaims: Record<string, string> = {};
    for (let i = 0; i < 101; i++) {
      tooManyClaims[`claim${i}`] = "val";
    }
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: tooManyClaims,
    });
    expect(res.status).toBe(400);
  });

  it("accepts exactly 100 custom claim keys", async () => {
    const loginId = uniqueLogin("jwt-count-ok");
    const sessionJwt = await getSessionJwt(loginId);

    const claims: Record<string, string> = {};
    for (let i = 0; i < 100; i++) {
      claims[`claim${i}`] = "val";
    }
    const res = await client.mgmtPost("/v1/mgmt/jwt/update", {
      jwt: sessionJwt,
      customClaims: claims,
    });
    expect(res.status).toBe(200);
  });
});
