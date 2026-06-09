/**
 * UI Workflow 6 — CLI / Developer Tooling Auth
 *
 * Market use case: Developer tools (CLIs, IDEs) where a human authenticates
 * interactively via a browser and the credential is handed to the CLI process.
 * Admins can create embedded links and enchanted links for developers.
 *
 * UI coverage:
 *   - Users page: create test user (the "developer" account)
 *   - Users page: dev user appears in user table; searchable
 *   - Users page: delete dev user → removed
 *   NOTE: enchanted link / embedded link generation is tested via API
 *         since these are admin-tooling flows without dedicated UI pages yet.
 */
import { test, expect } from "@playwright/test";

import { UsersPage } from "../pom/UsersPage";
import { api, resetEmulator, uniqueLogin } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 6 — CLI Auth: Admin UI", () => {
  test("create developer test user via UI → appears in table", async ({
    page,
  }) => {
    const usersPage = await new UsersPage(page).goto();
    const devLogin = uniqueLogin("wf6-dev-ui");

    await usersPage.createUser({ loginId: devLogin });

    await expect(usersPage.userRow(devLogin)).toBeVisible();
  });

  test("dev user searchable by login ID", async ({ page }) => {
    const devLogin = uniqueLogin("wf6-cli-dev");
    const otherLogin = uniqueLogin("wf6-other");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: devLogin,
      email: devLogin,
    });
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: otherLogin,
      email: otherLogin,
    });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.locators.rows).toHaveCount(2);

    await usersPage.search("cli-dev");
    await expect(usersPage.locators.rows).toHaveCount(1);
    await expect(usersPage.userRow(devLogin)).toBeVisible();
  });

  test("embedded link generation — test user created via API is visible in UI", async ({
    page,
  }) => {
    const devLogin = uniqueLogin("wf6-embed-ui");

    // Create a test user and generate an embedded link (simulates CLI onboarding)
    await api.mgmtPost("/v1/mgmt/user/create/test", {
      loginId: devLogin,
      email: devLogin,
    });
    const linkRes = await api.mgmtPost("/v1/mgmt/user/signin/embeddedlink", {
      loginId: devLogin,
    });
    expect(linkRes.ok).toBe(true);
    const { token } = (await linkRes.json()) as { token: string };
    expect(token.length).toBeGreaterThan(10);

    // UI: dev user visible in users dashboard
    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(devLogin)).toBeVisible();
  });

  test("revoke dev access — delete user via UI", async ({ page }) => {
    const devLogin = uniqueLogin("wf6-revoke-ui");
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: devLogin,
      email: devLogin,
    });

    const usersPage = await new UsersPage(page).goto();
    page.once("dialog", (d) => d.accept());
    await usersPage.deleteUser(devLogin);

    await expect(usersPage.userRow(devLogin)).not.toBeVisible();
  });
});
