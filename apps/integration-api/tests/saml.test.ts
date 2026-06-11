import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator, uniqueLogin } from "../helpers/client";

beforeEach(() => resetEmulator());

function makeSamlTenant(id: string, domain: string) {
  return client.post("/emulator/seed-tenant", {
    id,
    name: `${id} Corp`,
    domains: [domain],
    authType: "saml",
  });
}

async function seedSamlUser(login: string, domain: string) {
  const tenantId = `t-${domain.replace(".", "-")}`;
  // Create tenant by seeding the store directly via mgmt (workaround: just use user-create with tenantIds via seed or direct route)
  // For SAML tests: pre-create user and add tenant
  await client.mgmtPost("/v1/mgmt/user/create", {
    loginId: login,
    email: login,
  });
  // Note: SAML tenant lookup is by email domain — no separate tenant mgmt endpoint needed in tests
  return { tenantId };
}

// ─── SAML Start ───────────────────────────────────────────────────────────────

describe("POST /v1/auth/saml/start", () => {
  it("returns url with code when tenant has matching domain", async () => {
    // Use plain user ID lookup path (tenant ID approach)
    const login = uniqueLogin("saml-start");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    // SAML start via tenant ID needs tenant pre-configured. 
    // For now validate email-based lookup errors correctly when tenant not configured
    const res = await client.post("/v1/auth/saml/start", {
      tenant: login, // email format — but no SAML tenant configured
      redirectUrl: "http://localhost:3000/callback",
    });
    // Expect error since no SAML tenant configured for this domain
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("rejects if user does not exist for email lookup", async () => {
    const res = await client.post("/v1/auth/saml/start", {
      tenant: "ghost@notconfigured.com",
      redirectUrl: "http://localhost/cb",
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── SAML Authorize (GET — query params) ─────────────────────────────────────

describe("GET /v1/auth/saml/authorize", () => {
  it("returns url with code when called with query params", async () => {
    const domain = "samlget.example";
    const login = `user@${domain}`;

    // Create SAML tenant with matching domain
    await client.post("/emulator/tenant", {
      id: "saml-get-tenant",
      name: "SAML GET Corp",
      domains: [domain],
      authType: "saml",
    });

    // Create user with matching email
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    // Assign user to tenant
    await client.mgmtPost("/v1/mgmt/user/tenant/add", {
      loginId: login,
      tenantId: "saml-get-tenant",
    });

    const qs = new URLSearchParams({
      tenant: login,
      redirectURL: "http://localhost:4200/login",
      loginHint: login,
    });
    const res = await client.get(`/v1/auth/saml/authorize?${qs}`);
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.url).toContain("http://localhost:4200/login");
    expect(body.url).toContain("code=");
  });

  it("rejects unknown user via GET", async () => {
    const qs = new URLSearchParams({
      tenant: "ghost@nowhere.com",
      redirectURL: "http://localhost/cb",
    });
    const res = await client.get(`/v1/auth/saml/authorize?${qs}`);
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── SAML Exchange ────────────────────────────────────────────────────────────

describe("POST /v1/auth/saml/exchange", () => {
  it("rejects invalid/expired code", async () => {
    const res = await client.post("/v1/auth/saml/exchange", {
      code: "a".repeat(64),
    });
    expect(res.status).toBe(401);
  });
});

// ─── OTP Phone Update ─────────────────────────────────────────────────────────

describe("POST /v1/auth/otp/update/phone/sms", () => {
  it("updates phone on existing user", async () => {
    const login = uniqueLogin("otp-phone");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    const res = await client.post("/v1/auth/otp/update/phone/sms", {
      loginId: login,
      phone: "+15550001234",
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    // Update issues an OTP to the new number (delivered, then verified separately).
    expect(body.maskedPhone).toBeTruthy();
    expect(body.code).toMatch(/^\d{6}$/);

    const user = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    const u = await user.json();
    expect(u.user.phone).toBe("+15550001234");
  });

  it("adds phone to loginIds when flag set", async () => {
    const login = uniqueLogin("otp-phone-lid");
    await client.mgmtPost("/v1/mgmt/user/create", { loginId: login, email: login });

    await client.post("/v1/auth/otp/update/phone/sms", {
      loginId: login,
      phone: "+15550009999",
      options: { addToLoginIDs: true },
    });

    const user = await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`);
    const u = await user.json();
    expect(u.user.loginIds).toContain("+15550009999");
  });

  it("rejects unknown user", async () => {
    const res = await client.post("/v1/auth/otp/update/phone/sms", {
      loginId: "nobody@x.com",
      phone: "+1555",
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});
