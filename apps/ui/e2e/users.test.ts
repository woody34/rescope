/**
 * Feature: Users Management
 *
 * Workflows:
 *  - Empty state: Page shows empty state when no users exist
 *  - Create: Open dialog → fill login ID → submit → user row appears in table
 *  - Search: Typing in the search box filters visible rows by login ID / name / email
 *  - Delete: Confirm delete dialog → user row removed from table
 *  - Edit: Click row → modal opens → edit name → save → table updated
 *  - Status toggle: Toggle status in modal header → badge updates
 *  - Column picker: Toggle columns → table columns change → localStorage persists
 *  - Custom attributes sub-tab: Navigate to /users/attributes → manage attribute defs
 *  - Tenant/role: Add tenant in modal → verify in assignment list
 *
 * Users are the core identity primitive. Each user has one or more login IDs
 * (email addresses or phone numbers) and can be assigned to tenants and roles.
 */
import { test, expect } from "@playwright/test";

import { UsersPage } from "./pom/UsersPage";
import { api, resetEmulator, uniqueLogin } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Users Page", () => {
  test("shows empty state with no users", async ({ page }) => {
    const users = await new UsersPage(page).goto();
    await expect(users.locators.emptyTitle).toBeVisible();
  });

  test("create user → appears in table", async ({ page }) => {
    const users = await new UsersPage(page).goto();
    const loginId = uniqueLogin("create");

    await users.createUser({ loginId });

    await expect(users.userRow(loginId)).toBeVisible();
  });

  test("search filters visible rows", async ({ page }) => {
    const alice = uniqueLogin("alice");
    const bob = uniqueLogin("bob");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: alice,
      email: alice,
    });
    await api.mgmtPost("/v1/mgmt/user/create", { loginId: bob, email: bob });

    const users = await new UsersPage(page).goto();
    await expect(users.locators.rows).toHaveCount(2);

    await users.search("alice");
    await expect(users.locators.rows).toHaveCount(1);
  });

  test("delete user → removed from table", async ({ page }) => {
    const loginId = uniqueLogin("delete");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const users = await new UsersPage(page).goto();
    await expect(users.userRow(loginId)).toBeVisible();

    page.once("dialog", (d) => d.accept());
    await users.deleteUser(loginId);

    await expect(users.userRow(loginId)).not.toBeVisible();
  });
});

test.describe("User Edit Modal", () => {
  test("clicking row opens edit modal with pre-filled data", async ({
    page,
  }) => {
    const loginId = uniqueLogin("edit");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      name: "Test User",
      email: loginId,
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    await expect(users.locators.modalTitle).toHaveText("Edit User");
    await expect(users.locators.loginIdInput).toBeDisabled();
    await expect(users.locators.nameInput).toHaveValue("Test User");
  });

  test("edit user name → save → table reflects change", async ({ page }) => {
    const loginId = uniqueLogin("edit-name");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      name: "Old Name",
      email: loginId,
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);
    await users.locators.nameInput.fill("New Name");
    await users.submitEdit();

    // Verify the new name shows in the table
    const row = users.userRow(loginId);
    await expect(row).toContainText("New Name");
  });

  test("toggle status via modal header", async ({ page }) => {
    const loginId = uniqueLogin("status");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    // Initially enabled
    await expect(users.locators.statusToggle).toHaveText("enabled");

    // Toggle to disabled
    await users.toggleStatus();
    await expect(users.locators.statusToggle).toHaveText("disabled");
  });

  test("add tenant to user via modal", async ({ page }) => {
    const loginId = uniqueLogin("tenant");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "t-test",
      name: "Test Tenant",
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    await users.locators.addTenantBtn.click();
    await users.locators.newTenantSelect.selectOption("t-test");
    await users.locators.confirmAddTenant.click();

    // Verify the tenant assignment appears
    await expect(
      page.getByTestId("tenant-assignment-t-test"),
    ).toBeVisible();
  });

  test("create tenant inline when none exist", async ({ page }) => {
    const loginId = uniqueLogin("inline-tenant");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    // Click add tenant → select "create new"
    await users.locators.addTenantBtn.click();
    await users.locators.newTenantSelect.selectOption("__create__");

    // Fill new tenant name and create — should auto-assign
    await page.getByTestId("new-tenant-name-input").fill("My Dev Tenant");
    await page.getByTestId("create-tenant-inline-btn").click();

    // Verify the tenant assignment appears immediately (no second "Add" click)
    await expect(
      page.getByTestId("tenant-assignment-my-dev-tenant"),
    ).toBeVisible();
  });

  test("create role inline auto-assigns to tenant", async ({ page }) => {
    const loginId = uniqueLogin("inline-role");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "t-role-test",
      name: "Role Test Tenant",
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    // Add tenant first
    await users.locators.addTenantBtn.click();
    await users.locators.newTenantSelect.selectOption("t-role-test");
    await users.locators.confirmAddTenant.click();

    // Create a role inline — should auto-assign to the tenant
    await page.getByTestId("new-role-input").fill("developer");
    await page.getByTestId("create-role-inline-btn").click();

    // Role should be selected in the multi-select for this tenant
    const roleSelect = page.getByTestId("tenant-roles-t-role-test");
    await expect(roleSelect).toHaveValues(["developer"]);
  });

  test("remove tenant assignment via button", async ({ page }) => {
    const loginId = uniqueLogin("remove-tenant");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      id: "t-remove",
      name: "Remove Me",
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    // Add tenant
    await users.locators.addTenantBtn.click();
    await users.locators.newTenantSelect.selectOption("t-remove");
    await users.locators.confirmAddTenant.click();
    await expect(page.getByTestId("tenant-assignment-t-remove")).toBeVisible();

    // Remove it
    await page.getByTestId("remove-tenant-t-remove").click();
    await expect(page.getByTestId("tenant-assignment-t-remove")).not.toBeVisible();
  });
});

test.describe("Column Picker", () => {
  test("toggle column visibility", async ({ page }) => {
    const loginId = uniqueLogin("col");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });

    const users = await new UsersPage(page).goto();

    // Verify "Verified" column is not visible by default
    const headers = page.locator("table thead th");
    const headerTexts = await headers.allTextContents();
    expect(headerTexts).not.toContain("Verified");

    // Toggle "verified" column on
    await users.openColumnPicker();
    await users.toggleColumn("verified");

    // Now it should be visible
    const updatedHeaders = await headers.allTextContents();
    expect(updatedHeaders).toContain("Verified");
  });

  test("column preferences persist in localStorage", async ({ page }) => {
    const loginId = uniqueLogin("col-persist");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });

    const users = await new UsersPage(page).goto();

    // Toggle "verified" column on
    await users.openColumnPicker();
    await users.toggleColumn("verified");

    // Verify persistence
    const saved = await page.evaluate(() =>
      localStorage.getItem("rescope:users:visibleColumns"),
    );
    expect(saved).not.toBeNull();
    const parsed = JSON.parse(saved!);
    expect(parsed).toContain("verified");
  });
});

test.describe("Custom Attributes Sub-Tab", () => {
  test("navigate to custom attributes tab", async ({ page }) => {
    const users = await new UsersPage(page).goto();
    await users.locators.tabAttributes.click();

    await expect(page).toHaveURL(/\/users\/attributes/);
  });

  test("navigate back to users tab", async ({ page }) => {
    const users = await new UsersPage(page).gotoAttributes();
    await users.locators.tabUsers.click();

    await expect(page).toHaveURL(/\/users$/);
  });

  test("navigate to attributes via direct URL", async ({ page }) => {
    await page.goto("users/attributes");
    await expect(page.getByTestId("tab-attributes")).toBeVisible();
  });

  test("create custom attribute via sub-tab", async ({ page }) => {
    await page.goto("users/attributes");
    await page.getByTestId("attr-name-input").fill("Favorite Color");
    await page.getByTestId("attr-machine-name-input").fill("favorite_color");
    await page.getByTestId("attr-type-select").selectOption("text");
    await page.getByTestId("create-attr-btn").click();

    await expect(
      page.getByTestId("attr-row-favorite_color"),
    ).toBeVisible();
  });

  test("machine name with invalid characters shows error", async ({ page }) => {
    await page.goto("users/attributes");
    await page.getByTestId("attr-name-input").fill("Test Attr");
    // Clear auto-generated value and type invalid one
    await page.getByTestId("attr-machine-name-input").fill("Invalid-Name!");

    await expect(page.getByTestId("machine-name-error")).toBeVisible();
    await expect(page.getByTestId("machine-name-error")).toContainText(
      "lowercase letter",
    );
    await expect(page.getByTestId("create-attr-btn")).toBeDisabled();
  });

  test("machine name character counter is displayed", async ({ page }) => {
    await page.goto("users/attributes");
    await page.getByTestId("attr-machine-name-input").fill("test_key");

    await expect(page.getByTestId("machine-name-counter")).toHaveText(
      "8/60",
    );
  });

  test("custom attribute values appear in user modal", async ({ page }) => {
    // Create a custom attribute via API
    await api.mgmtPost("/v1/mgmt/user/attribute", {
      name: "Is VIP",
      machineName: "is_vip",
      attributeType: "boolean",
      permissions: "admin",
    });
    const loginId = uniqueLogin("attr-modal");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId,
      email: loginId,
    });

    const users = await new UsersPage(page).goto();
    await users.openEditModal(loginId);

    // The boolean attribute should be rendered as a checkbox
    await expect(page.getByTestId("attr-is_vip")).toBeVisible();
  });
});

test.describe("Search Filter Dropdowns", () => {
  test("status filter shows only enabled users", async ({ page }) => {
    const enabled = uniqueLogin("enabled");
    const disabled = uniqueLogin("disabled");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: enabled,
      email: enabled,
    });
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: disabled,
      email: disabled,
    });
    await api.mgmtPost("/v1/mgmt/user/update/status", {
      loginId: disabled,
      status: "disabled",
    });

    const users = await new UsersPage(page).goto();
    await expect(users.locators.rows).toHaveCount(2);

    await page.getByTestId("status-filter").selectOption("enabled");
    await expect(users.locators.rows).toHaveCount(1);
    await expect(users.userRow(enabled)).toBeVisible();
  });

  test("tenant filter shows only users in selected tenant", async ({
    page,
  }) => {
    // Create a tenant
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      name: "AcmeCo",
      id: "acme-filter-test",
    });

    const inTenant = uniqueLogin("in-tenant");
    const noTenant = uniqueLogin("no-tenant");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: inTenant,
      email: inTenant,
    });
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: noTenant,
      email: noTenant,
    });
    await api.mgmtPost("/v1/mgmt/user/tenant/add", {
      loginId: inTenant,
      tenantId: "acme-filter-test",
    });

    const users = await new UsersPage(page).goto();
    await expect(users.locators.rows).toHaveCount(2);

    await page.getByTestId("tenant-filter").selectOption("acme-filter-test");
    await expect(users.locators.rows).toHaveCount(1);
    await expect(users.userRow(inTenant)).toBeVisible();
  });

  test("combining status + tenant filters narrows results", async ({
    page,
  }) => {
    await api.mgmtPost("/v1/mgmt/tenant/create", {
      name: "FilterCo",
      id: "filter-co",
    });

    const match = uniqueLogin("match");
    const wrongStatus = uniqueLogin("wrong-status");
    const wrongTenant = uniqueLogin("wrong-tenant");

    // Create 3 users
    for (const lid of [match, wrongStatus, wrongTenant]) {
      await api.mgmtPost("/v1/mgmt/user/create", {
        loginId: lid,
        email: lid,
      });
    }

    // Assign tenant to match and wrongStatus
    for (const lid of [match, wrongStatus]) {
      await api.mgmtPost("/v1/mgmt/user/tenant/add", {
        loginId: lid,
        tenantId: "filter-co",
      });
    }

    // Disable wrongStatus
    await api.mgmtPost("/v1/mgmt/user/update/status", {
      loginId: wrongStatus,
      status: "disabled",
    });

    const users = await new UsersPage(page).goto();
    await expect(users.locators.rows).toHaveCount(3);

    // Apply both filters
    await page.getByTestId("status-filter").selectOption("enabled");
    await page.getByTestId("tenant-filter").selectOption("filter-co");

    await expect(users.locators.rows).toHaveCount(1);
    await expect(users.userRow(match)).toBeVisible();
  });

  test("clearing filters restores full user list", async ({ page }) => {
    const a = uniqueLogin("clear-a");
    const b = uniqueLogin("clear-b");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: a,
      email: a,
    });
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: b,
      email: b,
    });
    await api.mgmtPost("/v1/mgmt/user/update/status", {
      loginId: b,
      status: "disabled",
    });

    const users = await new UsersPage(page).goto();
    await expect(users.locators.rows).toHaveCount(2);

    // Filter to enabled only
    await page.getByTestId("status-filter").selectOption("enabled");
    await expect(users.locators.rows).toHaveCount(1);

    // Clear filter
    await page.getByTestId("status-filter").selectOption("");
    await expect(users.locators.rows).toHaveCount(2);
  });
});
