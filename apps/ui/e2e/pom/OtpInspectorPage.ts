import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

export class OtpInspectorPage extends BasePage {
  static readonly url = "otp-inspector";
  static readonly path = "/otp-inspector";

  get locators() {
    return {
      refreshBtn: this.page.getByTestId("refresh-otps-btn"),
      emptyState: this.page.getByTestId("empty-state"),
      rows: this.page.locator("table tbody tr"),
    };
  }

  otpRow(userId: string): Locator {
    return this.page.getByTestId(`otp-row-${userId}`);
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(OtpInspectorPage.url);
    // Wait for loading spinner to disappear, then either empty state or table is visible
    await this.page.locator(".loading-spinner").waitFor({ state: "hidden" });
    return this;
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async refresh() {
    await this.locators.refreshBtn.click();
  }

  // ── State helpers ─────────────────────────────────────────────────────────

  async getOtpCode(userId: string): Promise<string | null> {
    const row = this.otpRow(userId);
    if (!(await row.isVisible())) return null;
    return (await row.locator("code").textContent())?.trim() ?? null;
  }

  async getVisibleRowCount(): Promise<number> {
    return this.locators.rows.count();
  }
}
