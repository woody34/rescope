/**
 * UI Workflow 1 — Passwordless (Consumer App)
 *
 * Market use case: Consumer apps replacing passwords with magic links and OTP.
 * This test suite covers the admin-side UI flows that support the passwordless
 * workflow: creating users, inspecting OTP codes, and verifying users exist in
 * the dashboard after authentication.
 *
 * UI coverage:
 *   - Users page: create user via UI → user row visible
 *   - OTP Inspector: after signUpOrIn, the emulator OTP inspector shows the code
 *   - Users page: seeded test users appear in user list (admin visibility)
 */
import { test, expect } from "@playwright/test";

import { UsersPage } from "../pom/UsersPage";
import { OtpInspectorPage } from "../pom/OtpInspectorPage";
import { api, resetEmulator, uniqueLogin } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 1 — Passwordless: Admin UI", () => {
  test("create test user via UI → row visible in user table", async ({
    page,
  }) => {
    const usersPage = await new UsersPage(page).goto();
    const loginId = uniqueLogin("wf1-ui");

    await usersPage.createUser({ loginId });

    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("OTP inspector shows pending code after signUpOrIn via API", async ({
    page,
  }) => {
    const loginId = uniqueLogin("wf1-otp-ui");

    // Consumer action: trigger OTP via auth API
    const signupRes = await api.post("/v1/auth/otp/signup-in/email", {
      loginId,
    });
    expect(signupRes.ok).toBe(true);

    // Admin UI: verify the OTP code is visible in the inspector
    const otpPage = new OtpInspectorPage(page);
    await otpPage.goto();

    await otpPage.refresh();
    const code = await otpPage.getOtpCode(loginId);
    expect(code).toMatch(/^\d{6}$/);
  });

  test("seeded test user appears in Users dashboard", async ({ page }) => {
    const loginId = uniqueLogin("wf1-seed-ui");

    // ADMIN CONFIG: create test user directly via management API
    await api.mgmtPost("/v1/mgmt/user/create/test", {
      loginId,
      email: loginId,
    });

    // Admin UI: user row visible
    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("multiple passwordless users all appear in table", async ({ page }) => {
    const user1 = uniqueLogin("wf1-multi-a");
    const user2 = uniqueLogin("wf1-multi-b");

    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: user1,
      email: user1,
    });
    await api.mgmtPost("/v1/mgmt/user/create", {
      loginId: user2,
      email: user2,
    });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(user1)).toBeVisible();
    await expect(usersPage.userRow(user2)).toBeVisible();
  });
});
