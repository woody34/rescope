import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator } from "../helpers/client";

// ─── Helpers ─────────────────────────────────────────────────────────────────

const BASE = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

let idpId: string;

async function setupOidcIdp() {
  // Create tenant with OIDC config
  await client.mgmtPost("/v1/mgmt/tenant/create", {
    id: "acme",
    name: "Acme Corp",
    selfProvisioningDomains: ["acme.com"],
    authType: "oidc",
    oidcConfig: {
      clientId: "acme-client",
      clientSecret: "acme-secret",
    },
  });

  // Create IdP emulator
  const idpRes = await client.mgmtPost("/v1/mgmt/idp", {
    protocol: "oidc",
    displayName: "Mock Okta",
    tenantId: "acme",
    attributeMapping: {
      email: "user.email",
      name: "user.name",
      given_name: "user.givenName",
    },
  });
  const { idp } = await idpRes.json();
  idpId = idp.id;

  // Create user in tenant
  await client.mgmtPost("/v1/mgmt/user/create", {
    loginId: "alice@acme.com",
    email: "alice@acme.com",
    name: "Alice Smith",
    givenName: "Alice",
    tenants: [{ tenantId: "acme" }],
  });
}

beforeEach(async () => {
  await resetEmulator();
  await setupOidcIdp();
});

// ─── Discovery ───────────────────────────────────────────────────────────────

describe("GET /emulator/idp/:idp_id/.well-known/openid-configuration", () => {
  it("returns valid OIDC discovery document", async () => {
    const res = await client.get(`/emulator/idp/${idpId}/.well-known/openid-configuration`);
    expect(res.status).toBe(200);
    const doc = await res.json();
    expect(doc.issuer).toContain(idpId);
    expect(doc.authorization_endpoint).toContain("/authorize");
    expect(doc.token_endpoint).toContain("/token");
    expect(doc.jwks_uri).toContain("/jwks");
    expect(doc.response_types_supported).toContain("code");
    expect(doc.id_token_signing_alg_values_supported).toContain("RS256");
  });

  it("returns 400 for unknown IdP", async () => {
    const res = await client.get("/emulator/idp/ghost/.well-known/openid-configuration");
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── JWKS ────────────────────────────────────────────────────────────────────

describe("GET /emulator/idp/:idp_id/jwks", () => {
  it("returns JWKS with RSA key", async () => {
    const res = await client.get(`/emulator/idp/${idpId}/jwks`);
    expect(res.status).toBe(200);
    const jwks = await res.json();
    expect(jwks.keys).toBeDefined();
    expect(jwks.keys.length).toBeGreaterThan(0);
    expect(jwks.keys[0].kty).toBe("RSA");
    expect(jwks.keys[0].alg).toBe("RS256");
  });
});

// ─── Authorize (Programmatic) ────────────────────────────────────────────────

describe("GET /emulator/idp/:idp_id/authorize (programmatic)", () => {
  it("redirects with code when login_id is provided", async () => {
    const url = `/emulator/idp/${idpId}/authorize?client_id=acme-client&redirect_uri=${encodeURIComponent("http://localhost:9999/callback")}&response_type=code&state=xyz&nonce=n1&login_id=alice@acme.com`;
    const res = await fetch(`${BASE}${url}`, { redirect: "manual" });
    expect(res.status).toBe(303);
    const location = res.headers.get("location") ?? "";
    expect(location).toContain("http://localhost:9999/callback");
    expect(location).toContain("code=");
    expect(location).toContain("state=xyz");
  });

  it("returns 400 without client_id", async () => {
    const res = await client.get(`/emulator/idp/${idpId}/authorize?redirect_uri=http://x&response_type=code`);
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("shows user picker HTML in browser mode (no login_id)", async () => {
    const url = `/emulator/idp/${idpId}/authorize?client_id=acme-client&redirect_uri=${encodeURIComponent("http://localhost:9999/callback")}&response_type=code`;
    const res = await client.get(url);
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("Rescope IdP Emulator");
    expect(html).toContain("alice@acme.com");
    expect(html).toContain("Mock Okta");
  });
});

// ─── Token Endpoint ──────────────────────────────────────────────────────────

describe("POST /emulator/idp/:idp_id/token", () => {
  async function getOidcCode(): Promise<string> {
    const url = `/emulator/idp/${idpId}/authorize?client_id=acme-client&redirect_uri=${encodeURIComponent("http://localhost:9999/callback")}&response_type=code&state=xyz&nonce=n1&login_id=alice@acme.com`;
    const res = await fetch(`${BASE}${url}`, { redirect: "manual" });
    const location = res.headers.get("location") ?? "";
    const codeMatch = location.match(/code=([^&]+)/);
    return codeMatch?.[1] ?? "";
  }

  it("exchanges code for id_token", async () => {
    const code = await getOidcCode();
    expect(code).toBeTruthy();

    const res = await fetch(`${BASE}/emulator/idp/${idpId}/token`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: `grant_type=authorization_code&code=${code}&redirect_uri=${encodeURIComponent("http://localhost:9999/callback")}&client_id=acme-client&client_secret=acme-secret`,
    });
    expect(res.status).toBe(200);
    const body = await res.json();

    expect(body.id_token).toBeTruthy();
    expect(body.access_token).toBeTruthy();
    expect(body.token_type).toBe("Bearer");

    // Decode id_token and check claims
    const [, payloadB64] = body.id_token.split(".");
    const payload = JSON.parse(atob(payloadB64.replace(/-/g, "+").replace(/_/g, "/")));
    expect(payload.email).toBe("alice@acme.com");
    expect(payload.name).toBe("Alice Smith");
    expect(payload.nonce).toBe("n1");
    expect(payload.iss).toContain(idpId);
    expect(payload.aud).toBe("acme-client");
  });

  it("returns error for invalid code", async () => {
    const res = await fetch(`${BASE}/emulator/idp/${idpId}/token`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: "grant_type=authorization_code&code=invalid&client_id=acme-client&client_secret=acme-secret",
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("code is single-use", async () => {
    const code = await getOidcCode();
    // First exchange succeeds
    await fetch(`${BASE}/emulator/idp/${idpId}/token`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: `grant_type=authorization_code&code=${code}&client_id=acme-client&client_secret=acme-secret`,
    });
    // Second exchange fails
    const res2 = await fetch(`${BASE}/emulator/idp/${idpId}/token`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: `grant_type=authorization_code&code=${code}&client_id=acme-client&client_secret=acme-secret`,
    });
    expect(res2.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Callback (SP-side) ─────────────────────────────────────────────────────

describe("GET /emulator/idp/callback", () => {
  it("exchanges OIDC code and redirects with SP code", async () => {
    // Get an OIDC code directed to the emulator callback
    const authUrl = `/emulator/idp/${idpId}/authorize?client_id=acme-client&redirect_uri=${encodeURIComponent(`${BASE}/emulator/idp/callback`)}&response_type=code&state=${encodeURIComponent("http://localhost:9999/app")}&nonce=n1&login_id=alice@acme.com`;
    const authRes = await fetch(`${BASE}${authUrl}`, { redirect: "manual" });
    const location = authRes.headers.get("location") ?? "";
    const codeMatch = location.match(/code=([^&]+)/);
    const oidcCode = codeMatch?.[1] ?? "";

    // Hit the callback
    const callbackUrl = `${BASE}/emulator/idp/callback?code=${oidcCode}&state=${encodeURIComponent("http://localhost:9999/app")}`;
    const cbRes = await fetch(callbackUrl, { redirect: "manual" });
    expect(cbRes.status).toBe(303);

    const cbLocation = cbRes.headers.get("location") ?? "";
    expect(cbLocation).toContain("http://localhost:9999/app");
    expect(cbLocation).toContain("code=");

    // Extract SP code and exchange for session
    const spCodeMatch = cbLocation.match(/code=([^&]+)/);
    const spCode = spCodeMatch?.[1] ?? "";
    expect(spCode).toBeTruthy();

    // Exchange SP code for session JWTs
    const exchangeRes = await client.post("/v1/auth/saml/exchange", { code: spCode });
    expect(exchangeRes.status).toBe(200);
    const session = await exchangeRes.json();
    expect(session.sessionJwt).toBeTruthy();
    expect(session.refreshJwt).toBeTruthy();
  });
});
