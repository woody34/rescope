/**
 * Feature: Emulator Runtime Reset
 *
 * Workflows:
 *  - Button visible: Reset button is always visible on page load
 *  - Cancel: Dismissing the confirmation dialog leaves state intact
 *  - Confirm: Accepting the dialog clears all runtime state (users, tokens, OTPs)
 *    and shows a success message; config stores (roles, permissions, etc.) are preserved
 *
 * Reset is intended for use between test runs or manual exploration sessions.
 * It mirrors the POST /emulator/reset API endpoint.
 */
import { test, expect } from "@playwright/test";

import { ResetPage } from "./pom/ResetPage";
import { UsersPage } from "./pom/UsersPage";
import { api, resetEmulator, uniqueLogin } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Reset Page", () => {
  test("reset button is visible on load", async ({ page }) => {
    const resetPage = await new ResetPage(page).goto();
    await expect(resetPage.locators.resetBtn).toBeVisible();
  });

  test("cancel reset dialog → state is preserved", async ({ page }) => {
    const loginId = uniqueLogin("keep");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const resetPage = await new ResetPage(page).goto();
    await resetPage.cancelReset();

    await expect(resetPage.locators.successMsg).not.toBeVisible();

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("confirm reset → success message shown and runtime state cleared", async ({
    page,
  }) => {
    const loginId = uniqueLogin("gone");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const resetPage = await new ResetPage(page).goto();
    await resetPage.reset();

    await expect(resetPage.locators.successMsg).toBeVisible();

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.locators.emptyTitle).toBeVisible();
  });
});
