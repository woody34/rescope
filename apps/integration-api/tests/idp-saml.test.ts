import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator } from "../helpers/client";

// ─── Helpers ─────────────────────────────────────────────────────────────────

const BASE = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

let idpId: string;

async function setupSamlIdp() {
  // Create tenant with SAML config
  await client.mgmtPost("/v1/mgmt/tenant/create", {
    id: "contoso",
    name: "Contoso Corp",
    selfProvisioningDomains: ["contoso.com"],
    authType: "saml",
    samlConfig: {
      acsUrl: `${BASE}/emulator/idp/saml/acs`,
    },
  });

  // Create SAML IdP emulator
  const idpRes = await client.mgmtPost("/v1/mgmt/idp", {
    protocol: "saml",
    displayName: "Mock Azure AD",
    tenantId: "contoso",
    attributeMapping: {
      email: "user.email",
      firstName: "user.name",
    },
  });
  const { idp } = await idpRes.json();
  idpId = idp.id;

  // Create user
  await client.mgmtPost("/v1/mgmt/user/create", {
    loginId: "bob@contoso.com",
    email: "bob@contoso.com",
    name: "Bob Smith",
    tenants: [{ tenantId: "contoso" }],
  });
}

beforeEach(async () => {
  await resetEmulator();
  await setupSamlIdp();
});

// ─── Metadata ────────────────────────────────────────────────────────────────

describe("GET /emulator/idp/:idp_id/metadata", () => {
  it("returns valid EntityDescriptor XML", async () => {
    const res = await client.get(`/emulator/idp/${idpId}/metadata`);
    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toContain("application/xml");
    const xml = await res.text();
    expect(xml).toContain("EntityDescriptor");
    expect(xml).toContain("IDPSSODescriptor");
    expect(xml).toContain("X509Certificate");
    expect(xml).toContain("SingleSignOnService");
    expect(xml).toContain(idpId);
  });

  it("rejects OIDC IdP requesting SAML metadata", async () => {
    // Create an OIDC IdP
    const oidcRes = await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "OIDC Only",
      tenantId: "contoso",
      attributeMapping: {},
    });
    const { idp: oidcIdp } = await oidcRes.json();
    const res = await client.get(`/emulator/idp/${oidcIdp.id}/metadata`);
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── SSO (Programmatic) ─────────────────────────────────────────────────────

describe("GET /emulator/idp/:idp_id/sso (programmatic)", () => {
  it("returns auto-submit form with SAML Response when login_id provided", async () => {
    const url = `/emulator/idp/${idpId}/sso?RelayState=${encodeURIComponent("http://localhost:9999/app")}&login_id=bob@contoso.com`;
    const res = await client.get(url);
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("SAMLResponse");
    expect(html).toContain("RelayState");
    expect(html).toContain("onload");

    // Extract and verify the SAMLResponse
    const match = html.match(/name="SAMLResponse" value="([^"]+)"/);
    expect(match).toBeTruthy();
    const samlB64 = match![1];
    const samlXml = atob(samlB64);
    expect(samlXml).toContain("samlp:Response");
    expect(samlXml).toContain("saml:Assertion");
    expect(samlXml).toContain("bob@contoso.com"); // NameID
    expect(samlXml).toContain("saml:AttributeStatement");
  });

  it("shows user picker HTML in browser mode (no login_id)", async () => {
    const url = `/emulator/idp/${idpId}/sso?RelayState=abc`;
    const res = await client.get(url);
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("Rescope IdP Emulator");
    expect(html).toContain("SAML");
    expect(html).toContain("bob@contoso.com");
    expect(html).toContain("Mock Azure AD");
  });
});

// ─── SAML ACS Callback (SP-side) ────────────────────────────────────────────

describe("POST /emulator/idp/saml/acs", () => {
  it("exchanges SAML Response for SP code and redirects", async () => {
    // Get SAML Response via programmatic SSO
    const ssoUrl = `/emulator/idp/${idpId}/sso?RelayState=${encodeURIComponent("http://localhost:9999/app")}&login_id=bob@contoso.com`;
    const ssoRes = await client.get(ssoUrl);
    const html = await ssoRes.text();
    const samlMatch = html.match(/name="SAMLResponse" value="([^"]+)"/);
    const relayMatch = html.match(/name="RelayState" value="([^"]+)"/);
    expect(samlMatch).toBeTruthy();

    // POST to ACS
    const acsRes = await fetch(`${BASE}/emulator/idp/saml/acs`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: `SAMLResponse=${encodeURIComponent(samlMatch![1])}&RelayState=${encodeURIComponent(relayMatch?.[1] ?? "")}`,
      redirect: "manual",
    });
    expect(acsRes.status).toBe(303);
    const location = acsRes.headers.get("location") ?? "";
    expect(location).toContain("http://localhost:9999/app");
    expect(location).toContain("code=");

    // Extract SP code and exchange for session JWTs
    const spCodeMatch = location.match(/code=([^&]+)/);
    const spCode = spCodeMatch?.[1] ?? "";
    expect(spCode).toBeTruthy();

    const exchangeRes = await client.post("/v1/auth/saml/exchange", { code: spCode });
    expect(exchangeRes.status).toBe(200);
    const session = await exchangeRes.json();
    expect(session.sessionJwt).toBeTruthy();
    expect(session.refreshJwt).toBeTruthy();
  });
});
