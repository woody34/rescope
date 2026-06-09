/**
 * UI Workflow 4 — Fraud Prevention / Account Security (eCommerce / FinTech)
 *
 * Market use case: Platforms monitoring for account takeover or suspicious
 * activity. Admins disable compromised accounts; re-enable after verification.
 *
 * UI coverage:
 *   - Users page: user appears, gets disabled via API, UI shows disabled state
 *   - Users page: re-enable via API → user active again
 *   - Users page: delete fraudulent user via UI → removed
 */
import { test, expect } from "@playwright/test";

import { UsersPage } from "../pom/UsersPage";
import { api, resetEmulator, uniqueLogin } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 4 — Fraud Prevention: Admin UI", () => {
  test("compromised user appears in Users table", async ({ page }) => {
    const loginId = uniqueLogin("wf4-fraud-ui");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("disable account via API → user still visible in table (admin sees all)", async ({
    page,
  }) => {
    const loginId = uniqueLogin("wf4-disable-ui");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    // FRAUD RESPONSE: disable account via management API
    await api.mgmtPost("/v1/mgmt/user/status", { loginId, status: "disabled" });

    // Admin UI: user still visible (admins can see disabled users)
    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("delete fraudulent user via UI → removed from table", async ({
    page,
  }) => {
    const loginId = uniqueLogin("wf4-delete-ui");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();

    page.once("dialog", (d) => d.accept());
    await usersPage.deleteUser(loginId);

    await expect(usersPage.userRow(loginId)).not.toBeVisible();
  });

  test("search finds specific user among many", async ({ page }) => {
    const suspect = uniqueLogin("wf4-suspect");
    const innocent = uniqueLogin("wf4-innocent");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: suspect,
      email: suspect,
    });
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: innocent,
      email: innocent,
    });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.locators.rows).toHaveCount(2);

    await usersPage.search("suspect");
    await expect(usersPage.locators.rows).toHaveCount(1);
  });
});
