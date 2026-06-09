/**
 * WF2 — MFA (Password + OTP Second Factor)
 */
import { test, expect } from './fixtures';
import {
  resetEmulator,
  createUserWithPassword,
  createTestUserWithPassword,
  generateOtp,
  uniqueLogin,
} from './helpers/emulator';

const BASE_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';

test.beforeEach(async () => {
  await resetEmulator();
});

test.describe('WF2 — MFA', () => {
  test('password + OTP second factor → authenticated user can add todos', async ({ page }) => {
    const login = uniqueLogin('wf2');
    const password = 'Factor1Pass!';

    await createTestUserWithPassword(login, password);

    await page.goto('/wf2');

    // Step 1: fill password form
    await page.fill('#wf2-login-id', login);
    await page.fill('#wf2-password', password);
    await page.click('#wf2-signin-btn');

    // App transitions to OTP step
    await expect(page.locator('#wf2-otp-code')).toBeVisible({ timeout: 10_000 });

    // Get OTP via mgmt API
    const code = await generateOtp(login);
    expect(code).toMatch(/^\d{6}$/);
    // Step 2: verify OTP
    await page.fill('#wf2-otp-code', code);
    await page.click('#wf2-verify-btn');

    // Authenticated: user bar + add-input visible
    await expect(page.locator('#wf2-user-bar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#todo-input')).toBeVisible({ timeout: 5_000 });

    // Add a todo so the table renders
    await page.fill('#todo-input', 'WF2 MFA task');
    await expect(page.locator('#todo-input')).toHaveValue('WF2 MFA task');
    await page.click('#add-todo-btn');
    await expect(page.locator('#todo-table')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#todo-table')).toContainText('WF2 MFA task');
  });

  test('wrong password shows error', async ({ page }) => {
    const login = uniqueLogin('wf2-badpw');
    await createUserWithPassword(login, 'CorrectPass1!');

    await page.goto('/wf2');
    await page.fill('#wf2-login-id', login);
    await page.fill('#wf2-password', 'WrongPass!');
    await page.click('#wf2-signin-btn');

    await expect(page.locator('#wf2-error')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#wf2-otp-code')).not.toBeVisible();
  });
});
