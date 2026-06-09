/**
 * UI Workflow 7 — Marketplace / Multi-Vendor Integrations
 *
 * Market use case: Platforms where each marketplace vendor is a tenant,
 * and third-party integrations use access keys as service accounts.
 * Admins provision vendors, manage their access keys, and onboard end-users.
 *
 * UI coverage:
 *   - Tenants page: create vendor tenant → row visible
 *   - Users page: provision vendor admin → user row visible
 *   - Access Keys page: create integration service key → cleartext shown
 *   - Access Keys page: integration key row visible in table
 *   - Tenants page: decommission vendor → tenant deleted from UI
 */
import { test, expect } from "@playwright/test";

import { TenantsPage } from "../pom/TenantsPage";
import { UsersPage } from "../pom/UsersPage";
import { AccessKeysPage } from "../pom/AccessKeysPage";
import { api, resetEmulator, uniqueLogin } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 7 — Marketplace Integrations: Admin UI", () => {
  test("full marketplace lifecycle: create vendor tenant → provision admin → service key → decommission", async ({
    page,
  }) => {
    const vendorAdmin = uniqueLogin("wf7-vendor-admin-ui");

    // ── ADMIN CONFIG: create vendor tenant ────────────────────────────────────
    const tenantsPage = await new TenantsPage(page).goto();
    await tenantsPage.createTenant({
      id: "vendor-acme",
      name: "Acme Marketplace",
    });
    await expect(tenantsPage.tenantRow("vendor-acme")).toBeVisible();

    // ── ADMIN CONFIG: provision vendor admin user ─────────────────────────────
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: vendorAdmin,
      email: vendorAdmin,
    });
    await api.mgmtPost("/v1/mgmt/user/tenant/add", {
      loginId: vendorAdmin,
      tenantId: "vendor-acme",
      roleNames: ["admin"],
    });

    // UI: admin user visible in Users table
    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(vendorAdmin)).toBeVisible();

    // ── ADMIN ACTION: create service access key for integration ───────────────
    const keysPage = await new AccessKeysPage(page).goto();
    await keysPage.createKey({ name: "acme-integration-key" });
    await expect(keysPage.locators.cleartextBanner).toBeVisible();
    const cleartext = await keysPage.getCleartextKey();
    expect(cleartext.length).toBeGreaterThan(10);
    await keysPage.dismissCleartextBanner();
    await expect(keysPage.keyRow("acme-integration-key")).toBeVisible();

    // ── ADMIN ACTION: decommission vendor (churn/offboarding) ─────────────────
    const tenantsPageFinal = await new TenantsPage(page).goto();
    await tenantsPageFinal.deleteTenant("vendor-acme");
    await expect(tenantsPageFinal.locators.emptyTitle).toBeVisible();
  });

  test("multiple vendors can coexist as separate tenants", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();

    await tenants.createTenant({
      id: "vendor-alpha",
      name: "Alpha Marketplace",
    });
    await tenants.createTenant({ id: "vendor-beta", name: "Beta Commerce" });

    expect(await tenants.getVisibleRowCount()).toBe(2);
    await expect(tenants.tenantRow("vendor-alpha")).toBeVisible();
    await expect(tenants.tenantRow("vendor-beta")).toBeVisible();
  });

  test("service key per vendor — each vendor gets own access key", async ({
    page,
  }) => {
    const keys = await new AccessKeysPage(page).goto();

    await keys.createKey({ name: "alpha-integration" });
    await keys.dismissCleartextBanner();
    await keys.createKey({ name: "beta-integration" });
    await keys.dismissCleartextBanner();

    expect(await keys.getVisibleRowCount()).toBe(2);
    await expect(keys.keyRow("alpha-integration")).toBeVisible();
    await expect(keys.keyRow("beta-integration")).toBeVisible();
  });

  test("tenant isolation — deleting vendor removes their tenant row only", async ({
    page,
  }) => {
    const tenants = await new TenantsPage(page).goto();

    await tenants.createTenant({
      id: "staying-vendor",
      name: "Staying Vendor",
    });
    await tenants.createTenant({
      id: "leaving-vendor",
      name: "Leaving Vendor",
    });
    expect(await tenants.getVisibleRowCount()).toBe(2);

    await tenants.deleteTenant("leaving-vendor");

    expect(await tenants.getVisibleRowCount()).toBe(1);
    await expect(tenants.tenantRow("staying-vendor")).toBeVisible();
    await expect(tenants.tenantRow("leaving-vendor")).not.toBeVisible();
  });
});
