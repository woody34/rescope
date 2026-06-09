import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

/**
 * POM for the Identity Providers admin page (/identity-providers).
 */
export class IdpPage extends BasePage {
  static readonly url = "identity-providers";
  static readonly path = "/identity-providers";

  get locators() {
    return {
      createBtn: this.page.getByTestId("create-idp-btn"),
      nameInput: this.page.getByTestId("idp-name-input"),
      protocolSelect: this.page.getByTestId("idp-protocol-select"),
      tenantSelect: this.page.getByTestId("idp-tenant-select"),
      createSubmit: this.page.getByTestId("create-idp-submit"),
      table: this.page.getByTestId("idp-table"),
      emptyState: this.page.getByTestId("idp-empty-state"),
      rows: this.page.locator("table tbody tr"),
      mappingKeyInput: this.page.getByTestId("mapping-key-input"),
      mappingValueInput: this.page.getByTestId("mapping-value-input"),
      addMappingBtn: this.page.getByTestId("add-mapping-btn"),
    };
  }

  idpRow(id: string): Locator {
    return this.page.getByTestId(`idp-row-${id}`);
  }

  testSsoBtn(id: string): Locator {
    return this.page.getByTestId(`test-sso-${id}`);
  }

  deleteBtn(id: string): Locator {
    return this.page.getByTestId(`delete-idp-${id}`);
  }

  mappingEditor(id: string): Locator {
    return this.page.getByTestId(`mapping-editor-${id}`);
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(IdpPage.url);
    await this.page.waitForLoadState("networkidle");
    return this;
  }

  // ── Create ────────────────────────────────────────────────────────────────

  async openCreateDialog() {
    await this.locators.createBtn.click();
    await this.locators.nameInput.waitFor();
  }

  async fillCreateForm(opts: {
    name: string;
    protocol?: "oidc" | "saml";
    tenantId?: string;
  }) {
    await this.locators.nameInput.fill(opts.name);
    if (opts.protocol) {
      await this.locators.protocolSelect.selectOption(opts.protocol);
    }
    if (opts.tenantId) {
      await this.locators.tenantSelect.selectOption(opts.tenantId);
    }
  }

  async submitCreate() {
    await this.locators.createSubmit.click();
    // Wait for dialog to close
    await this.page.locator(".dialog-content").waitFor({ state: "detached" });
  }

  async createIdp(opts: {
    name: string;
    protocol?: "oidc" | "saml";
    tenantId?: string;
  }) {
    await this.openCreateDialog();
    await this.fillCreateForm(opts);
    await this.submitCreate();
  }

  // ── Delete ────────────────────────────────────────────────────────────────

  async deleteIdp(id: string) {
    this.page.once("dialog", (d) => d.accept());
    await this.deleteBtn(id).click();
  }
}
