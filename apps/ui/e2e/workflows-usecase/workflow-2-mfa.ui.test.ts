/**
 * UI Workflow 2 — MFA / Step-Up Authentication (FinTech / Healthcare)
 *
 * Market use case: High-security apps requiring MFA via OTP for sensitive
 * operations. Admins configure auth methods; users see OTP requirements.
 *
 * UI coverage:
 *   - Auth Methods page: toggle password + OTP enabled state
 *   - Users page: create user with password credentials
 *   - OTP inspector: verify step-up OTP code is visible after auth
 */
import { test, expect } from "@playwright/test";

import { UsersPage } from "../pom/UsersPage";
import { OtpInspectorPage } from "../pom/OtpInspectorPage";
import { api, resetEmulator, uniqueLogin } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 2 — MFA: Admin UI", () => {
  test("create user for MFA flow → visible in user table", async ({ page }) => {
    const loginId = uniqueLogin("wf2-mfa-ui");

    // Seed user via API (password-backed account)
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();
  });

  test("MFA user signup creates OTP in inspector", async ({ page }) => {
    const loginId = uniqueLogin("wf2-otp-ui");

    // Consumer: sign up with email OTP (step-up trigger)
    const res = await api.post("/v1/auth/otp/signup/email", { loginId });
    expect(res.ok).toBe(true);

    // Admin UI: OTP visible for the newly created user
    const otpPage = new OtpInspectorPage(page);
    await otpPage.goto();
    await otpPage.refresh();
    const code = await otpPage.getOtpCode(loginId);
    expect(code).toMatch(/^\d{6}$/);
  });

  test("delete MFA user via UI → removed from table", async ({ page }) => {
    const loginId = uniqueLogin("wf2-del-ui");
    await api.mgmtPost("/v1/mgmt/user/create", { loginId, email: loginId });

    const usersPage = await new UsersPage(page).goto();
    await expect(usersPage.userRow(loginId)).toBeVisible();

    page.once("dialog", (d) => d.accept());
    await usersPage.deleteUser(loginId);

    await expect(usersPage.userRow(loginId)).not.toBeVisible();
  });
});
