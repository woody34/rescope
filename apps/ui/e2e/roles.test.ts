/**
 * Feature: Roles & Permissions (RBAC)
 *
 * Workflows:
 *  Permissions tab:
 *   - Create: Open dialog → fill name + description → submit → row appears in table
 *   - Delete: Click delete → confirm → permission row removed
 *   - Multiple: Create several permissions → row count increments correctly
 *
 *  Roles tab:
 *   - Create: Open dialog → fill name → submit → role row appears in table
 *   - Create with permissions: Attach permissions at create time → shown in role row
 *   - Delete: Confirm delete → role row removed
 *
 * Permissions are fine-grained action labels (e.g. "read:users"). Roles group
 * permissions and are assigned to users. Both live in config stores that survive
 * emulator runtime resets.
 */
import { test, expect } from "@playwright/test";

import { RolesPage } from "./pom/RolesPage";
import { resetEmulator } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Roles & Permissions Page", () => {
  test.describe("Permissions tab", () => {
    test("create permission → appears in table", async ({ page }) => {
      const roles = await new RolesPage(page).goto();

      await roles.createPermission({
        name: "read:all",
        description: "Read everything",
      });

      await expect(roles.permissionRow("read:all")).toBeVisible();
    });

    test("delete permission → removed from table", async ({ page }) => {
      const roles = await new RolesPage(page).goto();
      await roles.createPermission({ name: "write:all" });

      await roles.deletePermission("write:all");

      await expect(roles.permissionRow("write:all")).not.toBeVisible();
    });

    test("multiple permissions → count increments", async ({ page }) => {
      const roles = await new RolesPage(page).goto();

      await roles.createPermission({ name: "read:users" });
      await roles.createPermission({ name: "write:users" });

      expect(await roles.getRowCount()).toBe(2);
    });
  });

  test.describe("Roles tab", () => {
    test("create role → appears in table", async ({ page }) => {
      const roles = await new RolesPage(page).goto();
      await roles.switchToRoles();

      await roles.createRole({ name: "Admin", description: "Full access" });

      await expect(roles.roleRow("Admin")).toBeVisible();
    });

    test("create role with permissions → permissions shown in row", async ({
      page,
    }) => {
      const roles = await new RolesPage(page).goto();

      await roles.createPermission({ name: "read:data" });
      await roles.switchToRoles();
      await roles.createRole({
        name: "Viewer",
        permissionNames: ["read:data"],
      });

      await expect(roles.roleRow("Viewer")).toContainText("read:data");
    });

    test("delete role → removed from table", async ({ page }) => {
      const roles = await new RolesPage(page).goto();
      await roles.switchToRoles();
      await roles.createRole({ name: "Temp" });

      await roles.deleteRole("Temp");

      await expect(roles.roleRow("Temp")).not.toBeVisible();
    });
  });
});
