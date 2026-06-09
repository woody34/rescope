/**
 * Feature: Access Keys
 *
 * Workflows:
 *  - Empty state: Page shows empty state when no access keys exist
 *  - Create: Open dialog → fill name/expiry → submit → cleartext shown once
 *  - Delete: Click delete on a key row → row removed from table
 *  - Multiple: Create several keys → table row count increments correctly
 *  - Banner polish: Cleartext banner is readable (dark theme compatible)
 *  - Curl example: Test example shown with actual key and emulator URL
 *
 * Access keys are long-lived API credentials for machine-to-machine auth.
 * The cleartext secret is displayed exactly once after creation; after
 * dismissal it cannot be retrieved (the emulator stores only a hash).
 */
import { test, expect } from "@playwright/test";

import { AccessKeysPage } from "./pom/AccessKeysPage";
import { resetEmulator } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Access Keys Page", () => {
  test("shows empty state with no keys", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await expect(keys.locators.emptyTitle).toBeVisible();
  });

  test("create key → cleartext shown once, then row visible", async ({
    page,
  }) => {
    const keys = await new AccessKeysPage(page).goto();

    await keys.createKey({ name: "my-ci-key" });

    await expect(keys.locators.cleartextBanner).toBeVisible();
    const cleartext = await keys.getCleartextKey();
    expect(cleartext.length).toBeGreaterThan(10);

    await keys.dismissCleartextBanner();
    await expect(keys.keyRow("my-ci-key")).toBeVisible();
  });

  test("delete key → removed from table", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "temp-key" });
    await keys.dismissCleartextBanner();

    await keys.deleteKey("temp-key");

    await expect(keys.locators.emptyTitle).toBeVisible();
  });

  test("multiple keys → count increments", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();

    await keys.createKey({ name: "key-one" });
    await keys.dismissCleartextBanner();
    await keys.createKey({ name: "key-two" });
    await keys.dismissCleartextBanner();

    expect(await keys.getVisibleRowCount()).toBe(2);
  });
});

test.describe("Access Keys Banner & Curl Example", () => {
  test("banner uses dark-theme-compatible colors (not light-on-light)", async ({
    page,
  }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "color-test-key" });

    // Verify the banner container is visible and uses dark bg (not transparent green)
    const banner = keys.locators.cleartextBanner;
    await expect(banner).toBeVisible();

    // Code block should NOT have white background
    const codeBlock = keys.locators.cleartextCode;
    const codeBgColor = await codeBlock.evaluate(
      (el) => getComputedStyle(el).backgroundColor,
    );
    expect(codeBgColor).not.toBe("rgb(255, 255, 255)");
  });

  test("curl example is displayed after key creation", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "curl-test-key" });

    await expect(keys.locators.curlExample).toBeVisible();
    const curlText = await keys.getCurlExample();
    expect(curlText).toContain("curl");
    expect(curlText).toContain("/v1/mgmt/user/search");
  });

  test("curl command contains the actual cleartext key", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "key-in-curl" });

    const cleartext = await keys.getCleartextKey();
    const curlText = await keys.getCurlExample();
    expect(curlText).toContain(cleartext);
  });

  test("curl command contains the correct emulator URL", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "url-test-key" });

    const curlText = await keys.getCurlExample();
    // The curl command should contain the emulator's base URL
    expect(curlText).toContain("localhost:4600");
  });

  test("copy curl button copies the command to clipboard", async ({
    page,
    context,
  }) => {
    // Grant clipboard permissions
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "copy-curl-key" });

    await keys.copyCurlCommand();

    const clipboardText = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboardText).toContain("curl");
    expect(clipboardText).toContain("/v1/mgmt/user/search");
    expect(clipboardText).toContain("Authorization: Bearer");
  });
});

