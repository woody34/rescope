/**
 * SAML/SSO integration tests.
 * Tests saml.start (with user email) → code extraction → saml.exchange → JWTs.
 * The emulator returns the SAML code in the redirect URL query param: ?code=...
 * Uses POST /emulator/tenant to seed a SAML-configured tenant before each test.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { resetEmulator, uniqueLogin, mgmtAuth } from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

beforeEach(() => resetEmulator());

const H = { "Content-Type": "application/json", Authorization: mgmtAuth };

// ─── helpers ──────────────────────────────────────────────────────────────────

const SAML_DOMAIN = "saml-corp.example";
const TENANT_ID = "saml-tenant-01";

async function seedSamlTenant() {
  await fetch(`${BASE_URL}/emulator/tenant`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ id: TENANT_ID, name: "SAML Corp", domains: [SAML_DOMAIN], authType: "saml" }),
  });
}

async function createSamlUser(loginId: string) {
  await fetch(`${BASE_URL}/v1/mgmt/user/create`, {
    method: "POST", headers: H,
    body: JSON.stringify({ loginId, email: loginId }),
  });
}

async function samlStart(loginOrTenant: string, redirectUrl = "http://localhost/callback") {
  const res = await fetch(`${BASE_URL}/v1/auth/saml/start`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ tenant: loginOrTenant, redirectUrl }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

async function samlExchange(code: string) {
  const res = await fetch(`${BASE_URL}/v1/auth/saml/exchange`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code }),
  });
  return { status: res.status, body: await res.json() as Record<string, unknown> };
}

// ─── SAML email-based flow (user + SAML tenant seeded) ───────────────────────

describe("POST /v1/auth/saml/start (email-based user lookup)", () => {
  it("returns a URL with code for existing SAML user", async () => {
    await seedSamlTenant();
    const login = `user@${SAML_DOMAIN}`;
    await createSamlUser(login);

    const { status, body } = await samlStart(login);
    expect(status).toBe(200);
    expect(typeof body.url).toBe("string");
    expect((body.url as string).includes("code=")).toBe(true);
  });

  it("returns 400 for unknown user email", async () => {
    const { status } = await samlStart("ghost@saml.example");
    expect(status).toBe(400);
  });
});

describe("POST /v1/auth/saml/exchange (email-based round-trip)", () => {
  it("exchanges SAML code for sessionJwt + refreshJwt", async () => {
    await seedSamlTenant();
    const login = `user2@${SAML_DOMAIN}`;
    await createSamlUser(login);

    const { body: startBody } = await samlStart(login);
    const url = startBody.url as string;
    const code = new URL(url).searchParams.get("code")!;
    expect(code).toBeTruthy();

    const { status, body } = await samlExchange(code);
    expect(status).toBe(200);
    expect(typeof body.sessionJwt).toBe("string");
    expect(typeof body.refreshJwt).toBe("string");
  });

  it("code is single-use (second exchange → 401)", async () => {
    await seedSamlTenant();
    const login = `user3@${SAML_DOMAIN}`;
    await createSamlUser(login);

    const { body: startBody } = await samlStart(login);
    const code = new URL(startBody.url as string).searchParams.get("code")!;

    await samlExchange(code);
    const { status } = await samlExchange(code);
    expect(status).toBe(401);
  });

  it("returns 401 for invalid code", async () => {
    const { status } = await samlExchange("invalid-code-12345");
    expect(status).toBe(401);
  });

  it("disabled user cannot exchange SAML code", async () => {
    await seedSamlTenant();
    const login = `user4@${SAML_DOMAIN}`;
    await createSamlUser(login);

    const { body: startBody } = await samlStart(login);
    const code = new URL(startBody.url as string).searchParams.get("code")!;

    // Disable user
    await fetch(`${BASE_URL}/v1/mgmt/user/status`, {
      method: "POST", headers: H,
      body: JSON.stringify({ loginId: login, status: "disabled" }),
    });

    const { status } = await samlExchange(code);
    expect(status).toBe(403);
  });
});

// ─── SAML /authorize alias ────────────────────────────────────────────────────

describe("POST /v1/auth/saml/authorize (alias for /start)", () => {
  it("returns URL with code for existing SAML user — same as /start", async () => {
    await seedSamlTenant();
    const login = `user5@${SAML_DOMAIN}`;
    await createSamlUser(login);

    const res = await fetch(`${BASE_URL}/v1/auth/saml/authorize`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tenant: login, redirectUrl: "http://localhost/callback" }),
    });
    expect(res.status).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(typeof body.url).toBe("string");
    expect((body.url as string).includes("code=")).toBe(true);
  });
});

// ─── Generic SSO /authorize + /exchange ──────────────────────────────────────

describe("POST /v1/auth/sso/authorize + /v1/auth/sso/exchange", () => {
  it("full round-trip: sso/authorize → code → sso/exchange → JWTs", async () => {
    await seedSamlTenant();
    const login = `user6@${SAML_DOMAIN}`;
    await createSamlUser(login);

    const authorizeRes = await fetch(`${BASE_URL}/v1/auth/sso/authorize`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tenant: login, redirectUrl: "http://localhost/callback" }),
    });
    expect(authorizeRes.status).toBe(200);
    const authorizeBody = await authorizeRes.json() as { url: string };
    const code = new URL(authorizeBody.url).searchParams.get("code")!;
    expect(code).toBeTruthy();

    const exchangeRes = await fetch(`${BASE_URL}/v1/auth/sso/exchange`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ code }),
    });
    expect(exchangeRes.status).toBe(200);
    const exchangeBody = await exchangeRes.json() as Record<string, unknown>;
    expect(typeof exchangeBody.sessionJwt).toBe("string");
    expect(typeof exchangeBody.refreshJwt).toBe("string");
  });
});
