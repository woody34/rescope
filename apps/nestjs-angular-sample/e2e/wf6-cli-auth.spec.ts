/**
 * WF6 — CLI Auth (Magic Link as PKCE proxy)
 */
import { test, expect } from './fixtures';
import {
  resetEmulator, createTestUser, generateMagicLink, uniqueLogin,
} from './helpers/emulator';

const BASE_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';

test.beforeEach(async () => { await resetEmulator(); });

test.describe('WF6 — CLI Auth', () => {
  test('paste magic link token → authenticated; add todo; logout revokes session', async ({ page }) => {
    const login = uniqueLogin('wf6');
    await createTestUser(login);
    const token = await generateMagicLink(login);

    await page.goto('/wf6');
    await expect(page.locator('#wf6-token-input')).toBeVisible({ timeout: 10_000 });

    await page.fill('#wf6-token-input', token);
    await page.click('#wf6-verify-btn');

    // Authenticated
    await expect(page.locator('#wf6-user-bar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#wf6-refresh-token')).toBeVisible();

    // Add a todo so the table renders
    await expect(page.locator('#todo-input')).toBeVisible({ timeout: 5_000 });
    await page.fill('#todo-input', 'WF6 CLI task');
    await expect(page.locator('#todo-input')).toHaveValue('WF6 CLI task');
    await page.click('#add-todo-btn');
    await expect(page.locator('#todo-table')).toBeVisible({ timeout: 5_000 });

    // Get refresh token from UI for post-logout check
    const refreshJwt = await page.locator('#wf6-refresh-token code').textContent();
    expect(refreshJwt?.length).toBeGreaterThan(10);

    // Logout
    await page.click('#wf6-logout-btn');
    await expect(page.locator('#wf6-token-input')).toBeVisible({ timeout: 10_000 });

    // Post-logout: refresh should be rejected
    const postLogout = await fetch(`${BASE_URL}/v1/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${refreshJwt}` },
    });
    expect(postLogout.ok).toBe(false);
    expect(postLogout.status).toBe(401);
  });

  test('invalid token shows error message', async ({ page }) => {
    await page.goto('/wf6');
    await page.fill('#wf6-token-input', 'not-a-real-token');
    await page.click('#wf6-verify-btn');

    await expect(page.locator('#wf6-error')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#wf6-user-bar')).not.toBeVisible();
  });
});
