import { test, expect } from "@playwright/test";
import { api, resetEmulator } from "./helpers/api";
import { IdpPage } from "./pom/IdpPage";

test.beforeEach(async () => {
  await resetEmulator();
});

test.describe("Identity Providers Page", () => {
  test("shows empty state when no IdPs exist", async ({ page }) => {
    const idpPage = new IdpPage(page);
    await idpPage.goto();

    await expect(idpPage.locators.emptyState).toBeVisible();
    await expect(idpPage.locators.emptyState).toContainText("No identity providers configured");
  });

  test("creates an OIDC IdP", async ({ page }) => {
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "t1",
      name: "Test Corp",
      authType: "oidc",
      oidcConfig: { clientId: "c", clientSecret: "s" },
    });

    const idpPage = new IdpPage(page);
    await idpPage.goto();
    await idpPage.createIdp({ name: "Mock Okta", protocol: "oidc", tenantId: "t1" });

    // Table should show the new IdP — scope to table to avoid strict mode
    await expect(idpPage.locators.table).toBeVisible();
    await expect(idpPage.locators.table.getByText("Mock Okta")).toBeVisible();
    await expect(idpPage.locators.table.getByText("OIDC")).toBeVisible();
  });

  test("creates a SAML IdP", async ({ page }) => {
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "t2",
      name: "SAML Corp",
      authType: "saml",
      samlConfig: { acsUrl: "http://localhost:4600/acs" },
    });

    const idpPage = new IdpPage(page);
    await idpPage.goto();
    await idpPage.createIdp({ name: "Mock AzureAD", protocol: "saml", tenantId: "t2" });

    await expect(idpPage.locators.table.getByText("Mock AzureAD")).toBeVisible();
    await expect(idpPage.locators.table.getByText("SAML")).toBeVisible();
  });

  test("deletes an IdP", async ({ page }) => {
    await api.mgmtPost("/v1/mgmt/tenant/create", { id: "t3", name: "Del Corp" });
    const res = await api.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "ToDelete",
      tenantId: "t3",
      attributeMapping: {},
    });
    const { idp } = await res.json();

    const idpPage = new IdpPage(page);
    await idpPage.goto();

    await expect(idpPage.locators.table.getByText("ToDelete")).toBeVisible();
    await idpPage.deleteIdp(idp.id);

    // Should show empty state after deletion
    await expect(idpPage.locators.emptyState).toBeVisible();
  });

  test("shows attribute mapping editor with default mappings", async ({ page }) => {
    await api.mgmtPost("/v1/mgmt/tenant/create", { id: "t4", name: "Mapping Corp" });
    const res = await api.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "Mapped IdP",
      tenantId: "t4",
      attributeMapping: { email: "user.email", name: "user.name" },
    });
    const { idp } = await res.json();

    const idpPage = new IdpPage(page);
    await idpPage.goto();

    const editor = idpPage.mappingEditor(idp.id);
    await expect(editor).toBeVisible();
    await expect(editor.getByText("email", { exact: true })).toBeVisible();
    await expect(editor.getByText("user.email", { exact: true })).toBeVisible();
    await expect(editor.getByText("user.name", { exact: true })).toBeVisible();
  });
});
