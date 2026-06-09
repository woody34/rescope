import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

export class UsersPage extends BasePage {
  static readonly url = "users";
  static readonly path = "/users";

  get locators() {
    return {
      createBtn: this.page.getByTestId("create-user-btn"),
      loginIdInput: this.page.getByTestId("user-login-id"),
      nameInput: this.page.getByTestId("user-name-input"),
      givenNameInput: this.page.getByTestId("user-given-name-input"),
      familyNameInput: this.page.getByTestId("user-family-name-input"),
      emailInput: this.page.getByTestId("user-email-input"),
      phoneInput: this.page.getByTestId("user-phone-input"),
      createSubmit: this.page.getByTestId("create-submit"),
      searchInput: this.page.getByTestId("user-search"),
      rows: this.page.locator("table tbody tr"),
      emptyTitle: this.page.locator(".empty-state-title"),
      modalTitle: this.page.getByTestId("user-modal-title"),
      statusToggle: this.page.getByTestId("status-toggle"),
      columnPickerBtn: this.page.getByTestId("column-picker-btn"),
      columnPickerDropdown: this.page.getByTestId("column-picker-dropdown"),
      tabUsers: this.page.getByTestId("tab-users"),
      tabAttributes: this.page.getByTestId("tab-attributes"),
      addTenantBtn: this.page.getByTestId("add-tenant-btn"),
      newTenantSelect: this.page.getByTestId("new-tenant-select"),
      confirmAddTenant: this.page.getByTestId("confirm-add-tenant"),
      noTenantsMsg: this.page.getByTestId("no-tenants-msg"),
    };
  }

  userRow(loginId: string): Locator {
    return this.page.getByTestId(`user-row-${loginId}`);
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(UsersPage.url);
    await this.page.waitForSelector('[data-testid="create-user-btn"]');
    return this;
  }

  async gotoAttributes(): Promise<this> {
    await this.page.goto("users/attributes");
    await this.page.waitForSelector('[data-testid="tab-attributes"]');
    return this;
  }

  // ── Create ────────────────────────────────────────────────────────────────

  async openCreateDialog() {
    await this.locators.createBtn.click();
    await this.locators.loginIdInput.waitFor();
  }

  async fillCreateForm(opts: {
    loginId: string;
    name?: string;
    email?: string;
    phone?: string;
  }) {
    await this.locators.loginIdInput.fill(opts.loginId);
    if (opts.name) await this.locators.nameInput.fill(opts.name);
    if (opts.email) await this.locators.emailInput.fill(opts.email);
    if (opts.phone) await this.locators.phoneInput.fill(opts.phone);
  }

  async submitCreate() {
    await this.locators.createSubmit.click();
    await this.locators.loginIdInput.waitFor({ state: "detached" });
  }

  async createUser(opts: {
    loginId: string;
    name?: string;
    email?: string;
    phone?: string;
  }) {
    await this.openCreateDialog();
    await this.fillCreateForm(opts);
    await this.submitCreate();
  }

  // ── Edit ──────────────────────────────────────────────────────────────────

  async openEditModal(loginId: string) {
    const row = this.userRow(loginId);
    await row.click();
    await this.locators.modalTitle.waitFor();
  }

  async submitEdit() {
    await this.locators.createSubmit.click();
    await this.locators.modalTitle.waitFor({ state: "detached" });
  }

  // ── Status ────────────────────────────────────────────────────────────────

  async toggleStatus() {
    await this.locators.statusToggle.click();
  }

  async getStatus(): Promise<string> {
    return (await this.locators.statusToggle.textContent())?.trim() ?? "";
  }

  // ── Column picker ─────────────────────────────────────────────────────────

  async openColumnPicker() {
    await this.locators.columnPickerBtn.click();
    await this.locators.columnPickerDropdown.waitFor();
  }

  async toggleColumn(key: string) {
    await this.page.getByTestId(`col-toggle-${key}`).locator("input").click();
  }

  // ── Search ────────────────────────────────────────────────────────────────

  async search(term: string) {
    await this.locators.searchInput.fill(term);
  }

  // ── Delete ────────────────────────────────────────────────────────────────

  async deleteUser(loginId: string) {
    const row = this.userRow(loginId);
    await row.getByTestId("delete-user-btn").click();
    await row.waitFor({ state: "detached" });
  }
}
