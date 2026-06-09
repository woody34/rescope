import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

export class RolesPage extends BasePage {
  static readonly url = "roles";
  static readonly path = "/roles";

  get locators() {
    return {
      createPermissionBtn: this.page.getByTestId("create-permission-btn"),
      permNameInput: this.page.getByTestId("perm-name-input"),
      permDescInput: this.page.getByTestId("perm-desc-input"),
      createRoleBtn: this.page.getByTestId("create-role-btn"),
      roleNameInput: this.page.getByTestId("role-name-input"),
      roleDescInput: this.page.getByTestId("role-desc-input"),
      createSubmit: this.page.getByTestId("create-submit"),
      confirmDelete: this.page.getByTestId("confirm-delete-btn"),
      rows: this.page.locator("table tbody tr"),
      permissionsTab: this.page.locator("[role='tab']:has-text('Permissions')"),
      rolesTab: this.page.locator("[role='tab']:has-text('Roles')"),
    };
  }

  permissionRow(name: string): Locator {
    return this.page.locator(`table tbody tr:has-text("${name}")`);
  }

  roleRow(name: string): Locator {
    return this.page.locator(`table tbody tr:has(strong:text("${name}"))`);
  }

  permCheckbox(name: string): Locator {
    return this.page.locator(
      `label:has(input[type="checkbox"]):has-text("${name}") input`,
    );
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(RolesPage.url);
    await this.locators.createPermissionBtn.waitFor();
    return this;
  }

  async switchToPermissions() {
    await this.locators.permissionsTab.click();
    await this.locators.createPermissionBtn.waitFor();
  }

  async switchToRoles() {
    await this.locators.rolesTab.click();
    await this.locators.createRoleBtn.waitFor();
  }

  // ── Permissions ───────────────────────────────────────────────────────────

  async openCreatePermissionDialog() {
    await this.locators.createPermissionBtn.click();
    await this.locators.permNameInput.waitFor();
  }

  async fillPermissionForm(opts: { name: string; description?: string }) {
    await this.locators.permNameInput.fill(opts.name);
    if (opts.description)
      await this.locators.permDescInput.fill(opts.description);
  }

  async submitPermission() {
    await this.locators.createSubmit.click();
    await this.locators.permNameInput.waitFor({ state: "detached" });
  }

  async createPermission(opts: { name: string; description?: string }) {
    await this.openCreatePermissionDialog();
    await this.fillPermissionForm(opts);
    await this.submitPermission();
  }

  async deletePermission(name: string) {
    await this.permissionRow(name).getByTestId("delete-btn").click();
    await this.locators.confirmDelete.click();
    await this.permissionRow(name).waitFor({ state: "detached" });
  }

  // ── Roles ─────────────────────────────────────────────────────────────────

  async openCreateRoleDialog() {
    await this.locators.createRoleBtn.click();
    await this.locators.roleNameInput.waitFor();
  }

  async fillRoleForm(opts: {
    name: string;
    description?: string;
    permissionNames?: string[];
  }) {
    await this.locators.roleNameInput.fill(opts.name);
    if (opts.description)
      await this.locators.roleDescInput.fill(opts.description);
    for (const perm of opts.permissionNames ?? []) {
      await this.permCheckbox(perm).check();
    }
  }

  async submitRole() {
    await this.locators.createSubmit.click();
    await this.locators.roleNameInput.waitFor({ state: "detached" });
  }

  async createRole(opts: {
    name: string;
    description?: string;
    permissionNames?: string[];
  }) {
    await this.openCreateRoleDialog();
    await this.fillRoleForm(opts);
    await this.submitRole();
  }

  async deleteRole(name: string) {
    await this.roleRow(name).getByTestId("delete-btn").click();
    await this.locators.confirmDelete.click();
    await this.roleRow(name).waitFor({ state: "detached" });
  }

  // ── State helpers ─────────────────────────────────────────────────────────

  async getRowCount(): Promise<number> {
    return this.locators.rows.count();
  }
}
