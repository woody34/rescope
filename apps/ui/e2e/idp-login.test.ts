import { test, expect } from "@playwright/test";
import { api, resetEmulator } from "./helpers/api";
import { IdpLoginPage } from "./pom/IdpLoginPage";

// ─── Helpers ─────────────────────────────────────────────────────────────────

const BASE = "http://localhost:4600";

async function createOidcIdp(): Promise<string> {
  await api.mgmtPost("/v1/mgmt/tenant/create", {
    id: "oidc-tenant",
    name: "OIDC Corp",
    selfProvisioningDomains: ["oidc.com"],
    authType: "oidc",
    oidcConfig: { clientId: "oidc-client", clientSecret: "oidc-secret" },
  });
  const res = await api.mgmtPost("/v1/mgmt/idp", {
    protocol: "oidc",
    displayName: "Test OIDC Provider",
    tenantId: "oidc-tenant",
    attributeMapping: { email: "user.email", name: "user.name" },
  });
  const { idp } = await res.json();

  // Create users
  await api.mgmtPost("/v1/mgmt/user/create", {
    loginId: "alice@oidc.com",
    email: "alice@oidc.com",
    name: "Alice",
    tenants: [{ tenantId: "oidc-tenant" }],
  });
  await api.mgmtPost("/v1/mgmt/user/create", {
    loginId: "bob@oidc.com",
    email: "bob@oidc.com",
    name: "Bob",
    tenants: [{ tenantId: "oidc-tenant" }],
  });

  return idp.id;
}

async function createSamlIdp(): Promise<string> {
  await api.mgmtPost("/v1/mgmt/tenant/create", {
    id: "saml-tenant",
    name: "SAML Corp",
    selfProvisioningDomains: ["saml.com"],
    authType: "saml",
    samlConfig: { acsUrl: `${BASE}/emulator/idp/saml/acs` },
  });
  const res = await api.mgmtPost("/v1/mgmt/idp", {
    protocol: "saml",
    displayName: "Test SAML Provider",
    tenantId: "saml-tenant",
    attributeMapping: { email: "user.email" },
  });
  const { idp } = await res.json();

  await api.mgmtPost("/v1/mgmt/user/create", {
    loginId: "carol@saml.com",
    email: "carol@saml.com",
    name: "Carol",
    tenants: [{ tenantId: "saml-tenant" }],
  });

  return idp.id;
}

// ─── Tests ───────────────────────────────────────────────────────────────────

test.describe("IdP Login Page — OIDC", () => {
  let idpId: string;

  test.beforeEach(async () => {
    await resetEmulator();
    idpId = await createOidcIdp();
  });

  test("renders user picker with branding and users", async ({ page }) => {
    const loginPage = new IdpLoginPage(page);
    await loginPage.gotoOidc(idpId, {
      clientId: "oidc-client",
      redirectUri: "http://localhost:9999/cb",
    });

    await expect(loginPage.locators.heading).toContainText("Rescope IdP Emulator");
    await expect(loginPage.locators.subtitle).toContainText("Test OIDC Provider");
    await expect(loginPage.locators.userRows).toHaveCount(2);
    await expect(loginPage.userRow("Alice")).toBeVisible();
    await expect(loginPage.userRow("Bob")).toBeVisible();
  });

  test("clicking Login redirects with code and state", async ({ page }) => {
    const loginPage = new IdpLoginPage(page);
    await loginPage.gotoOidc(idpId, {
      clientId: "oidc-client",
      redirectUri: `${BASE}/emulator/idp/callback`,
      state: `${BASE}/dashboard`,
      nonce: "test-nonce",
    });

    await loginPage.loginAs("Alice");

    // The callback redirects to state URL with code param
    await page.waitForURL(/code=/, { timeout: 5000 });
    expect(page.url()).toContain("code=");
  });
});

test.describe("IdP Login Page — SAML", () => {
  let idpId: string;

  test.beforeEach(async () => {
    await resetEmulator();
    idpId = await createSamlIdp();
  });

  test("renders user picker with SAML badge", async ({ page }) => {
    const loginPage = new IdpLoginPage(page);
    await loginPage.gotoSaml(idpId, { relayState: "http://app" });

    await expect(loginPage.locators.heading).toContainText("Rescope IdP Emulator");
    await expect(loginPage.locators.badge).toContainText("SAML");
    await expect(loginPage.locators.subtitle).toContainText("Test SAML Provider");
    await expect(loginPage.locators.userRows).toHaveCount(1);
    await expect(loginPage.userRow("Carol")).toBeVisible();
  });

  test("clicking Login auto-submits SAML Response", async ({ page }) => {
    const loginPage = new IdpLoginPage(page);
    await loginPage.gotoSaml(idpId, { relayState: `${BASE}/dashboard` });

    await loginPage.loginAs("Carol");

    // ACS callback redirects to RelayState with code param
    await page.waitForURL(/code=/, { timeout: 5000 });
    expect(page.url()).toContain("code=");
  });
});

test.describe("IdP Login Page — Empty State", () => {
  test("shows empty message when no users exist", async ({ page }) => {
    await resetEmulator();

    // Create OIDC tenant + IdP but NO users
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "empty-t",
      name: "Empty Corp",
      authType: "oidc",
      oidcConfig: { clientId: "c", clientSecret: "s" },
    });
    const res = await api.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "Empty IdP",
      tenantId: "empty-t",
      attributeMapping: {},
    });
    const { idp } = await res.json();

    const loginPage = new IdpLoginPage(page);
    await loginPage.gotoOidc(idp.id, {
      clientId: "c",
      redirectUri: "http://localhost:9999/cb",
    });

    await expect(loginPage.locators.emptyMessage).toContainText("No users");
    await expect(loginPage.locators.loginButtons).toHaveCount(0);
  });
});
