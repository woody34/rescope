/**
 * UI Workflow 3 — B2B Multi-Tenant SSO (SaaS Platforms)
 *
 * Market use case: SaaS platforms selling to businesses. Each customer
 * organisation is a separate tenant; users belong to one or more tenants
 * and get tenant-scoped JWTs.
 *
 * UI coverage:
 *   - Tenants page: create tenant → row visible
 *   - Users page: create user, verify user row shows
 *   - Tenants page: delete tenant → row removed
 *   - Multiple tenants: both rows visible simultaneously
 */
import { test, expect } from "@playwright/test";

import { TenantsPage } from "../pom/TenantsPage";
import { UsersPage } from "../pom/UsersPage";
import { api, resetEmulator, uniqueLogin } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 3 — B2B Multi-Tenant SSO: Admin UI", () => {
  test("create tenant via UI → appears in tenant table", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();

    await tenants.createTenant({ id: "acme-saas", name: "Acme SaaS" });

    await expect(tenants.tenantRow("acme-saas")).toBeVisible();
  });

  test("provision B2B user and verify visible in Users table", async ({
    page,
  }) => {
    const loginId = uniqueLogin("wf3-b2b-ui");

    // ADMIN CONFIG: create tenant + provision user
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "wf3-tenant",
      name: "B2B Corp",
    });
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });
    await api.mgmtPost("/v1/mgmt/user/tenant/add", {
      loginId,
      tenantId: "wf3-tenant",
      roleNames: ["admin"],
    });

    // UI: user and tenant both visible
    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("delete tenant via UI → removed from table", async ({ page }) => {
    const tenants = await new TenantsPage(page).goto();
    await tenants.createTenant({ id: "to-remove", name: "To Remove" });
    await expect(tenants.tenantRow("to-remove")).toBeVisible();

    await tenants.deleteTenant("to-remove");

    await expect(tenants.locators.emptyTitle).toBeVisible();
  });

  test("multiple B2B tenants → all rows visible simultaneously", async ({
    page,
  }) => {
    const tenants = await new TenantsPage(page).goto();

    await tenants.createTenant({ id: "vendor-alpha", name: "Alpha Corp" });
    await tenants.createTenant({ id: "vendor-beta", name: "Beta Ltd" });

    expect(await tenants.getVisibleRowCount()).toBe(2);
    await expect(tenants.tenantRow("vendor-alpha")).toBeVisible();
    await expect(tenants.tenantRow("vendor-beta")).toBeVisible();
  });
});
