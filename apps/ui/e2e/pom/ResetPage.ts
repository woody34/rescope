import { BasePage } from "./_BasePage";

export class ResetPage extends BasePage {
  static readonly url = "reset";
  static readonly path = "/reset";

  get locators() {
    return {
      resetBtn: this.page.getByTestId("reset-btn"),
      successMsg: this.page.getByTestId("reset-success-msg"),
    };
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(ResetPage.url);
    await this.locators.resetBtn.waitFor();
    return this;
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async reset() {
    this.page.once("dialog", (d) => d.accept());
    await this.locators.resetBtn.click();
    await this.locators.successMsg.waitFor();
  }

  async cancelReset() {
    this.page.once("dialog", (d) => d.dismiss());
    await this.locators.resetBtn.click();
  }
}
