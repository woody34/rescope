/**
 * Feature: Dashboard Navigation
 *
 * Workflows:
 *  - Landing: Dashboard loads and displays navigation cards for all major features
 *  - Card navigation: Clicking a card navigates to that feature's page
 *  - Sidebar navigation: Clicking a sidebar link navigates to that feature's page
 *
 * Note: Access Keys is accessible via the sidebar only, not as a dashboard card.
 */
import { test, expect } from "@playwright/test";
import { DashboardPage } from "./pom/DashboardPage";
import { UsersPage } from "./pom/UsersPage";
import { RolesPage } from "./pom/RolesPage";
import { TenantsPage } from "./pom/TenantsPage";
import { AuthMethodsPage } from "./pom/AuthMethodsPage";
import { resetEmulator } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Dashboard Navigation", () => {
  test("dashboard loads with cards", async ({ page }) => {
    const dashboard = await new DashboardPage(page).goto();
    const count = await dashboard.locators.cards.count();
    expect(count).toBeGreaterThan(0);
  });

  test("Users card navigates to Users page", async ({ page }) => {
    const dashboard = await new DashboardPage(page).goto();
    const users = await dashboard.clickCard(UsersPage.path);
    expect(users).toBeInstanceOf(UsersPage);
    await expect(page).toHaveURL(new RegExp(UsersPage.path));
  });

  test("Roles card navigates to Roles page", async ({ page }) => {
    const dashboard = await new DashboardPage(page).goto();
    const roles = await dashboard.clickCard(RolesPage.path);
    expect(roles).toBeInstanceOf(RolesPage);
    await expect(page).toHaveURL(new RegExp(RolesPage.path));
  });

  test("Tenants card navigates to Tenants page", async ({ page }) => {
    const dashboard = await new DashboardPage(page).goto();
    const tenants = await dashboard.clickCard(TenantsPage.path);
    expect(tenants).toBeInstanceOf(TenantsPage);
    await expect(page).toHaveURL(new RegExp(TenantsPage.path));
  });

  test("Auth Methods card navigates to Auth Methods page", async ({ page }) => {
    const dashboard = await new DashboardPage(page).goto();
    const authMethods = await dashboard.clickCard(AuthMethodsPage.path);
    expect(authMethods).toBeInstanceOf(AuthMethodsPage);
    await expect(page).toHaveURL(new RegExp(AuthMethodsPage.path));
  });

  test("sidebar Users link navigates to Users page", async ({ page }) => {
    const dashboard = await new DashboardPage(page).goto();
    const users = await dashboard.clickSidebarLink("Users");
    expect(users).toBeInstanceOf(UsersPage);
    await expect(page).toHaveURL(new RegExp(UsersPage.path));
  });
});
