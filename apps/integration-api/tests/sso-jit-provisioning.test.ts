import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator } from "../helpers/client";

/**
 * Just-in-time provisioning for SSO sign-ins.
 *
 * A working SSO tenant should let any address on its domains sign in, the way a
 * real identity provider does — without the user being seeded first. These
 * tests drive the wire surfaces end to end: the SAML start/exchange pair the
 * SDKs use, the identity-provider emulator endpoints, and the configuration
 * screen that collects roles and attributes for the new user.
 */

beforeEach(() => resetEmulator());

const DOMAIN = "jit.example";

async function createSsoTenant(
  id: string,
  extra: Record<string, unknown> = {},
): Promise<void> {
  const res = await client.post("/emulator/tenant", {
    id,
    name: `${id} Corp`,
    domains: [DOMAIN],
    authType: "saml",
    ...extra,
  });
  expect(res.status).toBe(200);
}

function formBody(pairs: Record<string, string>): string {
  return new URLSearchParams(pairs).toString();
}

// ─── SAML start / exchange ───────────────────────────────────────────────────

describe("POST /v1/auth/saml/start for an unseeded user", () => {
  it("returns the configuration screen URL when the tenant prompts", async () => {
    await createSsoTenant("jit-prompt");

    const res = await client.post("/v1/auth/saml/start", {
      tenant: `newcomer@${DOMAIN}`,
      redirectUrl: "http://localhost:4200/login/sso",
    });

    expect(res.status).toBe(200);
    const { url } = await res.json();
    expect(url).toContain("/emulator/sso/provision");
    expect(url).toContain(`email=newcomer%40${DOMAIN}`);
    expect(url).toContain("tenant=jit-prompt");
  });

  it("provisions and returns an exchangeable code when the sign-in skips the prompt", async () => {
    await createSsoTenant("jit-skip");
    const login = `auto@${DOMAIN}`;

    const startRes = await client.post("/v1/auth/saml/start", {
      tenant: login,
      redirectUrl: "http://localhost:4200/login/sso",
      ssoJitPrompt: false,
    });
    expect(startRes.status).toBe(200);
    const { url } = await startRes.json();
    expect(url).toContain("code=");

    const code = new URL(url).searchParams.get("code");
    const exchangeRes = await client.post("/v1/auth/saml/exchange", { code });
    expect(exchangeRes.status).toBe(200);

    const info = await exchangeRes.json();
    expect(info.sessionJwt).toBeTruthy();
    expect(info.user.email).toBe(login);
    expect(info.user.status).toBe("enabled");
    expect(info.user.verifiedEmail).toBe(true);
    expect(info.user.userTenants[0].tenantId).toBe("jit-skip");
  });

  it("honors a tenant that turns the prompt off", async () => {
    await createSsoTenant("jit-tenant-off", {
      ssoJitProvisioning: { prompt: false, defaultRoleNames: ["Member"] },
    });

    const res = await client.post("/v1/auth/saml/start", {
      tenant: `quiet@${DOMAIN}`,
      redirectUrl: "http://localhost:4200/login/sso",
    });
    const { url } = await res.json();
    expect(url).toContain("code=");

    const code = new URL(url).searchParams.get("code");
    const info = await (await client.post("/v1/auth/saml/exchange", { code })).json();
    expect(info.user.roleNames).toEqual(["Member"]);
  });

  it("still rejects an unknown user when the tenant disables provisioning", async () => {
    await createSsoTenant("jit-off", { ssoJitProvisioning: { enabled: false } });

    const res = await client.post("/v1/auth/saml/start", {
      tenant: `nobody@${DOMAIN}`,
      redirectUrl: "http://localhost:4200/login/sso",
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });

  it("still rejects an address on a domain no SSO tenant claims", async () => {
    await createSsoTenant("jit-scope");

    const res = await client.post("/v1/auth/saml/start", {
      tenant: "stranger@unclaimed.example",
      redirectUrl: "http://localhost:4200/login/sso",
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Configuration screen ────────────────────────────────────────────────────

describe("/emulator/sso/provision", () => {
  it("renders a form prefilled with the address and tenant", async () => {
    await createSsoTenant("jit-form");

    const res = await client.get(
      `/emulator/sso/provision?email=${encodeURIComponent(`jane.doe@${DOMAIN}`)}&tenant=jit-form`,
    );
    expect(res.status).toBe(200);

    const html = await res.text();
    expect(html).toContain(`value="jane.doe@${DOMAIN}"`);
    expect(html).toContain("Jane Doe");
    expect(html).toContain("jit-form Corp");
  });

  it("creates the configured user and hands back a code the app can exchange", async () => {
    await createSsoTenant("jit-submit");
    const login = `configured@${DOMAIN}`;

    const res = await fetch(
      `${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/emulator/sso/provision`,
      {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        redirect: "manual",
        body: formBody({
          email: login,
          tenant: "jit-submit",
          mode: "code",
          redirectUrl: "http://localhost:4200/login/sso",
          name: "Configured Person",
          roles: "Auditor, Reviewer",
          customAttributes: JSON.stringify({ department: "quality" }),
        }),
      },
    );

    expect(res.status).toBeGreaterThanOrEqual(300);
    expect(res.status).toBeLessThan(400);
    const location = res.headers.get("location")!;
    expect(location).toContain("http://localhost:4200/login/sso?code=");

    const code = new URL(location).searchParams.get("code");
    const info = await (await client.post("/v1/auth/saml/exchange", { code })).json();
    expect(info.user.email).toBe(login);
    expect(info.user.name).toBe("Configured Person");
    expect(info.user.roleNames).toEqual(["Auditor", "Reviewer"]);
    expect(info.user.customAttributes.department).toBe("quality");
  });
});

// ─── Identity-provider emulator ──────────────────────────────────────────────

describe("identity-provider emulator sign-in for an unseeded user", () => {
  async function createIdp(protocol: "saml" | "oidc", tenantId: string): Promise<string> {
    const res = await client.mgmtPost("/v1/mgmt/idp", {
      protocol,
      displayName: `Mock ${protocol.toUpperCase()}`,
      tenantId,
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    return body.idp?.id ?? body.id;
  }

  it("asserts a brand-new user through the SAML endpoint", async () => {
    await createSsoTenant("jit-idp-saml");
    const idpId = await createIdp("saml", "jit-idp-saml");
    const login = `saml-newcomer@${DOMAIN}`;

    const res = await client.get(
      `/emulator/idp/${idpId}/sso?login_id=${encodeURIComponent(login)}`,
    );
    expect(res.status).toBe(200);
    expect(await res.text()).toContain("SAMLResponse");

    const user = await (
      await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`)
    ).json();
    expect(user.user.status).toBe("enabled");
    expect(user.user.userTenants[0].tenantId).toBe("jit-idp-saml");
  });

  it("offers a new-user entry point on the SAML picker", async () => {
    await createSsoTenant("jit-idp-picker");
    const idpId = await createIdp("saml", "jit-idp-picker");

    const html = await (await client.get(`/emulator/idp/${idpId}/sso`)).text();
    expect(html).toContain("Sign in as a new user");
    expect(html).toContain("/emulator/sso/provision");
  });

  it("issues an authorization code for a brand-new user through the OIDC endpoint", async () => {
    await createSsoTenant("jit-idp-oidc");
    const idpId = await createIdp("oidc", "jit-idp-oidc");
    const login = `oidc-newcomer@${DOMAIN}`;

    const qs = new URLSearchParams({
      client_id: "test-client",
      redirect_uri: "http://localhost:4200/cb",
      response_type: "code",
      login_id: login,
    });
    const res = await fetch(
      `${process.env.EMULATOR_BASE_URL ?? "http://localhost:4501"}/emulator/idp/${idpId}/authorize?${qs}`,
      { redirect: "manual" },
    );
    expect(res.status).toBeGreaterThanOrEqual(300);
    expect(res.headers.get("location")).toContain("code=");

    const user = await (
      await client.mgmtGet(`/v1/mgmt/user?loginid=${encodeURIComponent(login)}`)
    ).json();
    expect(user.user.userTenants[0].tenantId).toBe("jit-idp-oidc");
  });
});
