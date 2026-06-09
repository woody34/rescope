import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

export class TenantsPage extends BasePage {
  static readonly url = "tenants";
  static readonly path = "/tenants";

  get locators() {
    return {
      createBtn: this.page.getByTestId("create-tenant-btn"),
      idInput: this.page.getByTestId("tenant-id-input"),
      nameInput: this.page.getByTestId("tenant-name-input"),
      createSubmit: this.page.getByTestId("create-submit"),
      rows: this.page.locator("table tbody tr"),
      emptyTitle: this.page.locator(".empty-state-title"),
    };
  }

  tenantRow(id: string): Locator {
    return this.page.getByTestId(`tenant-row-${id}`);
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(TenantsPage.url);
    await this.page.waitForSelector('[data-testid="create-tenant-btn"]');
    return this;
  }

  // ── Create ────────────────────────────────────────────────────────────────

  async openCreateDialog() {
    await this.locators.createBtn.click();
    await this.locators.idInput.waitFor();
  }

  async fillCreateForm(opts: { id: string; name: string }) {
    await this.locators.idInput.fill(opts.id);
    await this.locators.nameInput.fill(opts.name);
  }

  async submitCreate() {
    await this.locators.createSubmit.click();
    await this.locators.idInput.waitFor({ state: "detached" });
  }

  async createTenant(opts: { id: string; name: string }) {
    await this.openCreateDialog();
    await this.fillCreateForm(opts);
    await this.submitCreate();
  }

  // ── Table ─────────────────────────────────────────────────────────────────

  async getVisibleRowCount(): Promise<number> {
    return this.locators.rows.count();
  }

  async deleteTenant(id: string) {
    const row = this.tenantRow(id);
    this.page.once("dialog", (d) => d.accept());
    await row.getByTestId("delete-tenant-btn").click();
    await row.waitFor({ state: "detached" });
  }
}
