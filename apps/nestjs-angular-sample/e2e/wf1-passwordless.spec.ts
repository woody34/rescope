/**
 * WF1 — Passwordless (OTP / Magic Link)
 */
import { test, expect } from './fixtures';
import {
  resetEmulator,
  createTestUser,
  generateMagicLink,
  uniqueLogin,
  injectSession,
} from './helpers/emulator';

const BASE_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';

test.beforeEach(async () => {
  await resetEmulator();
});

test.describe('WF1 — Passwordless', () => {
  test('magic link verify produces valid session; authenticated user can add todos', async ({ page }) => {
    const login = uniqueLogin('wf1');

    await createTestUser(login);
    const token = await generateMagicLink(login);

    const res = await fetch(`${BASE_URL}/v1/auth/magiclink/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token }),
    });
    expect(res.ok).toBe(true);
    const { sessionJwt, refreshJwt } = await res.json() as { sessionJwt: string; refreshJwt: string };

    await page.goto('/wf1');
    await injectSession(page, sessionJwt, refreshJwt);
    await page.reload();

    // Authenticated: user bar visible, add-input rendered (todo section loaded)
    await expect(page.locator('#wf1-user-bar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#todo-input')).toBeVisible({ timeout: 10_000 });

    // Add a todo so the table renders
    await page.fill('#todo-input', 'WF1 test task');
    await expect(page.locator('#todo-input')).toHaveValue('WF1 test task');
    await page.click('#add-todo-btn');
    await expect(page.locator('#todo-table')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#todo-table')).toContainText('WF1 test task');

    // JWT claims panel shows sub
    await expect(page.locator('#jwt-claims-panel')).toBeVisible();
    await expect(page.locator('[data-claim="sub"]')).toBeVisible();
  });

  test('unauthenticated user sees descope-wc component', async ({ page }) => {
    await page.goto('/wf1');
    await expect(page.locator('descope-wc#wf1-descope-wc')).toBeAttached({ timeout: 15_000 });
    await expect(page.locator('#wf1-user-bar')).not.toBeVisible();
  });

  test('logout clears session and returns to sign-in state', async ({ page }) => {
    const login = uniqueLogin('wf1-logout');
    await createTestUser(login);
    const token = await generateMagicLink(login);

    const res = await fetch(`${BASE_URL}/v1/auth/magiclink/verify`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token }),
    });
    const { sessionJwt, refreshJwt } = await res.json() as { sessionJwt: string; refreshJwt: string };

    await page.goto('/wf1');
    await injectSession(page, sessionJwt, refreshJwt);
    await page.reload();

    await expect(page.locator('#wf1-logout-btn')).toBeVisible({ timeout: 10_000 });
    await page.click('#wf1-logout-btn');

    await expect(page.locator('descope-wc')).toBeAttached({ timeout: 10_000 });
    await expect(page.locator('#wf1-user-bar')).not.toBeVisible();
  });
});
