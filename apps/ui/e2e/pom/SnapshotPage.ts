import { BasePage } from "./_BasePage";

export class SnapshotPage extends BasePage {
  static readonly url = "snapshot";
  static readonly path = "/snapshot";

  get locators() {
    return {
      exportBtn: this.page.getByTestId("export-snapshot-btn"),
      importInput: this.page.getByTestId("import-snapshot-input"),
      resetBtn: this.page.locator("#reset-emulator-btn"),
    };
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(SnapshotPage.url);
    await this.locators.exportBtn.waitFor();
    return this;
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async export(): Promise<string> {
    const [download] = await Promise.all([
      this.page.waitForEvent("download"),
      this.locators.exportBtn.click(),
    ]);
    const path = await download.path();
    if (!path) throw new Error("Download failed — no path returned");
    return path;
  }

  async importFile(filePath: string): Promise<void> {
    await this.locators.importInput.setInputFiles(filePath);
    await this.page
      .getByText("imported successfully")
      .waitFor({ timeout: 5000 });
  }

  async reset(): Promise<void> {
    this.page.once("dialog", (dialog) => dialog.accept());
    await this.locators.resetBtn.click();
    await this.page.getByText("runtime state reset").waitFor({ timeout: 5000 });
  }
}
