/**
 * Feature: OTP Inspector
 *
 * Workflows:
 *  - Empty state: Page shows empty state when no OTPs are pending
 *  - Pending OTP: After a sign-up via email OTP, a code appears in the inspector table
 *  - Cleared OTP: After verifying the OTP, the row is removed from the inspector
 *
 * The OTP Inspector is an emulator-only debug view that exposes pending one-time
 * passcodes without requiring real email/SMS delivery. It mirrors the data from
 * the GET /emulator/otps endpoint.
 */
import { test, expect } from "@playwright/test";

import { OtpInspectorPage } from "./pom/OtpInspectorPage";
import { api, resetEmulator } from "./helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("OTP Inspector Page", () => {
  test("shows empty state when no OTPs pending", async ({ page }) => {
    const otpPage = await new OtpInspectorPage(page).goto();
    await expect(otpPage.locators.emptyState).toBeVisible();
  });

  test("OTP code appears after signup", async ({ page }) => {
    // Trigger OTP signup
    await api.post("/v1/auth/otp/signup/email", {
      loginId: "otp-user@example.com",
    });

    // Get the pending OTPs directly from the emulator inspector API
    const otpData = await (await api.mgmtGet("/emulator/otps")).json();
    const entries = Object.entries(otpData.otps ?? {});
    expect(entries.length).toBeGreaterThan(0);
    const [userId] = entries[0] as [string, string];

    const otpPage = await new OtpInspectorPage(page).goto();

    const code = await otpPage.getOtpCode(userId);
    expect(code).toMatch(/^\d{6}$/);
  });

  test("OTP code is removed after verify", async ({ page }) => {
    await api.post("/v1/auth/otp/signup/email", {
      loginId: "verify-me@example.com",
    });

    // Get code from emulator API
    const otpData = await (await api.mgmtGet("/emulator/otps")).json();
    const entries = Object.entries(otpData.otps ?? {});
    expect(entries.length).toBeGreaterThan(0);
    const [userId, code] = entries[0] as [string, string];

    // Verify to consume the OTP
    await api.post("/v1/auth/otp/verify/email", {
      loginId: "verify-me@example.com",
      code,
    });

    const otpPage = await new OtpInspectorPage(page).goto();
    expect(await otpPage.getOtpCode(userId)).toBeNull();
    await expect(otpPage.locators.emptyState).toBeVisible();
  });
});
