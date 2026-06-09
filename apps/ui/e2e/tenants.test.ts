/**
 * Feature: Tenants Management
 *
 * Workflows:
 *  - Empty state: Page shows empty state when no tenants exist
 *  - Create: Open dialog → fill tenant ID + name → submit → row appears in table
 *  - Delete: Confirm delete dialog → tenant row removed from table
 *  - Multiple: Create several tenants → all rows visible simultaneously
 *
 * Tenants enable multi-tenancy. Each tenant has a unique slug ID, a display
 * name, optional domains, and an auth type (none, saml, oidc). Users can be
 * associated with one or more tenants and granted tenant-scoped roles.
 */
import { test, expect } from "@playwright/test";

import { TenantsPage } from "./pom/TenantsPage";
import { resetEmulator } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Tenants Page", () => {
  test("shows empty state with no tenants", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();
    await expect(tenants.locators.emptyTitle).toBeVisible();
  });

  test("create tenant → appears in table", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();

    await tenants.createTenant({ id: "acme-corp", name: "Acme Corp" });

    await expect(tenants.tenantRow("acme-corp")).toBeVisible();
  });

  test("delete tenant → removed from table", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();
    await tenants.createTenant({ id: "to-delete", name: "To Delete" });

    await tenants.deleteTenant("to-delete");

    await expect(tenants.locators.emptyTitle).toBeVisible();
  });

  test("multiple tenants → all rows visible", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();

    await tenants.createTenant({ id: "tenant-a", name: "Tenant A" });
    await tenants.createTenant({ id: "tenant-b", name: "Tenant B" });

    expect(await tenants.getVisibleRowCount()).toBe(2);
    await expect(tenants.tenantRow("tenant-a")).toBeVisible();
    await expect(tenants.tenantRow("tenant-b")).toBeVisible();
  });
});
