import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

export class AccessKeysPage extends BasePage {
  static readonly url = "access-keys";
  static readonly path = "/access-keys";

  get locators() {
    return {
      createBtn: this.page.getByTestId("create-key-btn"),
      keyNameInput: this.page.getByTestId("key-name-input"),
      keyExpiryInput: this.page.getByTestId("key-expiry-input"),
      createSubmit: this.page.getByTestId("create-submit"),
      cleartextBanner: this.page.getByTestId("cleartext-banner"),
      cleartextCode: this.page.getByTestId("cleartext-code"),
      curlExample: this.page.getByTestId("curl-example"),
      copyCurlBtn: this.page.getByTestId("copy-curl-btn"),
      dismissBtn: this.page.getByTestId("dismiss-btn"),
      rows: this.page.getByTestId("key-row"),
      emptyTitle: this.page.locator(".empty-state-title"),
    };
  }

  keyRowLocator(id: string): Locator {
    return this.page.locator(`[data-testid="key-row"]:has-text("${id}")`);
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(AccessKeysPage.url);
    await this.page.waitForSelector('[data-testid="create-key-btn"]');
    return this;
  }

  // ── Create ────────────────────────────────────────────────────────────────

  async openCreateDialog() {
    await this.locators.createBtn.click();
    await this.locators.keyNameInput.waitFor();
  }

  async fillCreateForm(opts: { name: string; expiryDate?: string }) {
    await this.locators.keyNameInput.fill(opts.name);
    if (opts.expiryDate)
      await this.locators.keyExpiryInput.fill(opts.expiryDate);
  }

  async submitCreate() {
    await this.locators.createSubmit.click();
    await this.locators.cleartextBanner.waitFor();
  }

  async createKey(opts: { name: string; expiryDate?: string }) {
    await this.openCreateDialog();
    await this.fillCreateForm(opts);
    await this.submitCreate();
  }

  async getCleartextKey(): Promise<string> {
    return (await this.locators.cleartextCode.textContent())?.trim() ?? "";
  }

  async getCurlExample(): Promise<string> {
    return (await this.locators.curlExample.textContent())?.trim() ?? "";
  }

  async copyCurlCommand() {
    await this.locators.copyCurlBtn.click();
  }

  async dismissCleartextBanner() {
    await this.locators.dismissBtn.click();
    await this.locators.cleartextBanner.waitFor({ state: "detached" });
  }

  /** keyRow looks up a row by the access key's name text */
  keyRow(name: string): Locator {
    return this.keyRowLocator(name);
  }

  // ── Delete ────────────────────────────────────────────────────────────────

  async deleteKey(name: string) {
    const row = this.keyRowLocator(name);
    await row.getByTestId("delete-btn").click();
    await row.waitFor({ state: "detached" });
  }

  // ── State helpers ─────────────────────────────────────────────────────────

  async getRowCount(): Promise<number> {
    return this.locators.rows.count();
  }

  async getVisibleRowCount(): Promise<number> {
    return this.locators.rows.count();
  }
}

