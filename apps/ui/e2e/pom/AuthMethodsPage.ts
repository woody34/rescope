import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

export type AuthMethodKey =
  | "otp"
  | "magicLink"
  | "password"
  | "totp"
  | "passkeys"
  | "sso"
  | "enchantedLink"
  | "embeddedLink"
  | "notp";

export class AuthMethodsPage extends BasePage {
  static readonly url = "auth-methods";
  static readonly path = "/auth-methods";

  readonly apiRoute = "/v1/mgmt/config/auth-methods";

  get locators() {
    return {
      firstToggle: this.page.getByTestId("toggle-otp"),
      toggleBtn: (key: AuthMethodKey): Locator =>
        this.page.getByTestId(`toggle-${key}`),
      statusBadge: (key: AuthMethodKey): Locator =>
        this.page.getByTestId(`badge-${key}`),
    };
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(AuthMethodsPage.url);
    await this.locators.firstToggle.waitFor();
    return this;
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async toggle(key: AuthMethodKey) {
    // Register BEFORE click to avoid race condition where fast response is missed
    const done = this.page.waitForResponse(
      (r) => r.url().includes(this.apiRoute) && r.request().method() === "PUT",
    );
    await this.locators.toggleBtn(key).click();
    await done;
  }

  async enable(key: AuthMethodKey) {
    const text = await this.locators.toggleBtn(key).textContent();
    if (text?.trim() === "Enable") await this.toggle(key);
  }

  async disable(key: AuthMethodKey) {
    const text = await this.locators.toggleBtn(key).textContent();
    if (text?.trim() === "Disable") await this.toggle(key);
  }

  // ── State helpers ─────────────────────────────────────────────────────────

  async isEnabled(key: AuthMethodKey): Promise<boolean> {
    const text = await this.locators.statusBadge(key).textContent();
    return text?.trim() === "Enabled";
  }
}
