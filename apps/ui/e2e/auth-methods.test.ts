/**
 * Feature: Auth Methods Configuration
 *
 * Workflows:
 *  - List: All supported auth methods render with a toggle button and status badge
 *  - Disable: Clicking the toggle on an enabled method disables it (button → "Enable", badge → "Disabled")
 *  - Re-enable: Toggling an already-disabled method restores its enabled state
 *  - Persistence: Disabled state is persisted and reflected back by the config API
 *
 * Auth methods control which login mechanisms are available in the project.
 * Changes are immediately persisted to the emulator and reflected in the
 * management config API at GET /v1/mgmt/config/auth-methods.
 */
import { test, expect } from "@playwright/test";

import { AuthMethodsPage } from "./pom/AuthMethodsPage";
import { api, resetEmulator } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Auth Methods Page", () => {
  test("all methods render with a toggle button", async ({ page }) => {
    const authMethods = await new AuthMethodsPage(page).goto();
    for (const method of ["otp", "magicLink", "password"] as const) {
      await expect(authMethods.locators.toggleBtn(method)).toBeVisible();
    }
  });

  test("disable OTP → toggle changes to Enable and badge shows Disabled", async ({
    page,
  }) => {
    const authMethods = await new AuthMethodsPage(page).goto();

    await authMethods.disable("otp");

    // Button now says "Enable"
    await expect(authMethods.locators.toggleBtn("otp")).toContainText("Enable");
    // Badge now says "Disabled"
    await expect(authMethods.locators.statusBadge("otp")).toContainText(
      "Disabled",
    );
  });

  test("toggle off then on restores Enabled state", async ({ page }) => {
    const authMethods = await new AuthMethodsPage(page).goto();

    await authMethods.disable("magicLink");
    await expect(authMethods.locators.toggleBtn("magicLink")).toContainText(
      "Enable",
    );

    await authMethods.enable("magicLink");
    await expect(authMethods.locators.toggleBtn("magicLink")).toContainText(
      "Disable",
    );
    await expect(authMethods.locators.statusBadge("magicLink")).toContainText(
      "Enabled",
    );
  });

  test("disabled auth method state is persisted in the API", async ({
    page,
  }) => {
    const authMethods = await new AuthMethodsPage(page).goto();
    await authMethods.disable("otp");

    // Confirm the config API reflects the disabled state
    const res = await api.mgmtGet("/v1/mgmt/config/auth-methods");
    const body = await res.json();
    expect(body.authMethods?.otp?.enabled).toBe(false);
  });
});
