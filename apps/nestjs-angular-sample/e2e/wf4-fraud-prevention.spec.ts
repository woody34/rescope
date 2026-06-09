/**
 * WF4 — Fraud Prevention
 */
import { test, expect } from './fixtures';
import {
  resetEmulator, createUserWithPassword, setUserStatus, forceLogout, uniqueLogin, injectSession,
} from './helpers/emulator';

const BASE_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';

test.beforeEach(async () => { await resetEmulator(); });

test.describe('WF4 — Fraud Prevention', () => {
  test('disabled account shows blocked state; re-enabled account allows sign-in', async ({ page }) => {
    const login = uniqueLogin('wf4');
    const password = 'FraudTest1!';
    await createUserWithPassword(login, password);

    // Initial sign in succeeds
    await page.goto('/wf4');
    await page.fill('#wf4-login-id', login);
    await page.fill('#wf4-password', password);
    await page.click('#wf4-signin-btn');
    await expect(page.locator('#wf4-user-bar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#wf4-account-status')).toContainText('Active');

    // Log out
    await page.click('#wf4-logout-btn');
    await expect(page.locator('#wf4-signin-btn')).toBeVisible({ timeout: 5_000 });

    // ADMIN: disable account
    await setUserStatus(login, 'disabled');

    // Sign in now blocked
    await page.fill('#wf4-login-id', login);
    await page.fill('#wf4-password', password);
    await page.click('#wf4-signin-btn');
    await expect(page.locator('#wf4-error')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#wf4-error')).toContainText('disabled');

    // ADMIN: re-enable
    await setUserStatus(login, 'enabled');

    // Sign in succeeds again
    await page.fill('#wf4-login-id', login);
    await page.fill('#wf4-password', password);
    await page.click('#wf4-signin-btn');
    await expect(page.locator('#wf4-user-bar')).toBeVisible({ timeout: 10_000 });
  });

  test('force-logout revokes existing session; NestJS returns 401 on next API call', async ({ page }) => {
    const login = uniqueLogin('wf4-forcelogout');
    const password = 'Force1!';
    await createUserWithPassword(login, password);

    // Sign in and get session via API directly
    const signinRes = await fetch(`${BASE_URL}/v1/auth/password/signin`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ loginId: login, password }),
    });
    const { sessionJwt, refreshJwt } = await signinRes.json() as { sessionJwt: string; refreshJwt: string };

    // Inject into page
    await page.goto('/wf4');
    await injectSession(page, sessionJwt, refreshJwt);
    await page.reload();
    await expect(page.locator('#wf4-user-bar')).toBeVisible({ timeout: 10_000 });

    // ADMIN: force logout
    await forceLogout(login);
    await new Promise(r => setTimeout(r, 1100)); // wait for 1s revocation resolution

    // NestJS should now return 401 for todos
    const todosRes = await page.request.get('http://localhost:3333/api/todos', {
      headers: { Authorization: `Bearer ${sessionJwt}` },
    });
    expect(todosRes.status()).toBe(401);
  });
});
