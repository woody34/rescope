/**
 * WF7 — Marketplace & Inbound App Integrations
 */
import { test, expect } from './fixtures';
import {
  resetEmulator, createUserWithPassword, createTenant, addUserToTenant, uniqueLogin,
} from './helpers/emulator';

test.beforeEach(async () => { await resetEmulator(); });

test.describe('WF7 — Marketplace Integrations', () => {
  test('admin signs in + selects tenant → tenant-scoped todos; dct claim present', async ({ page }) => {
    const adminLogin = uniqueLogin('wf7-admin');
    const tenantId = `marketplace-${Date.now()}`;

    await createUserWithPassword(adminLogin, 'MarketAdmin1!');
    await createTenant(tenantId, 'Marketplace Vendor Co');
    await addUserToTenant(adminLogin, tenantId, ['marketplace-admin']);

    await page.goto('/wf7');
    await page.fill('#wf7-login-id', adminLogin);
    await page.fill('#wf7-password', 'MarketAdmin1!');
    await page.fill('#wf7-tenant-id', tenantId);
    await page.click('#wf7-signin-btn');

    await expect(page.locator('#wf7-user-bar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#wf7-user-bar')).toContainText(tenantId);

    // Add a todo so the table renders
    await expect(page.locator('#todo-input')).toBeVisible({ timeout: 5_000 });
    await page.fill('#todo-input', 'WF7 marketplace task');
    await expect(page.locator('#todo-input')).toHaveValue('WF7 marketplace task');
    await page.click('#add-todo-btn');
    await expect(page.locator('#todo-table')).toBeVisible({ timeout: 5_000 });

    // dct claim present in JWT claims panel
    await expect(page.locator('[data-claim="dct"]')).toContainText(tenantId);
  });

  test('two tenant users have isolated todo lists', async ({ page }) => {
    const adminLogin = uniqueLogin('wf7-admin2');
    const viewerLogin = uniqueLogin('wf7-viewer');
    const tenantId = `isolated-tenant-${Date.now()}`;

    await createUserWithPassword(adminLogin, 'AdminPass1!');
    await createUserWithPassword(viewerLogin, 'ViewerPass1!');
    await createTenant(tenantId, 'Isolated Corp');
    await addUserToTenant(adminLogin, tenantId, ['admin']);
    await addUserToTenant(viewerLogin, tenantId, ['viewer']);

    // Sign in as admin + create a todo
    await page.goto('/wf7');
    await page.fill('#wf7-login-id', adminLogin);
    await page.fill('#wf7-password', 'AdminPass1!');
    await page.fill('#wf7-tenant-id', tenantId);
    await page.click('#wf7-signin-btn');
    await expect(page.locator('#wf7-user-bar')).toBeVisible({ timeout: 10_000 });

    await page.fill('#todo-input', 'Shared tenant task');
    await expect(page.locator('#todo-input')).toHaveValue('Shared tenant task');
    await page.click('#add-todo-btn');
    await expect(page.locator('#todo-table')).toContainText('Shared tenant task', { timeout: 5_000 });

    // Log out, sign in as viewer — should see same tenant todo
    await page.click('#wf7-logout-btn');
    await expect(page.locator('#wf7-signin-btn')).toBeVisible({ timeout: 5_000 });

    await page.fill('#wf7-login-id', viewerLogin);
    await page.fill('#wf7-password', 'ViewerPass1!');
    await page.fill('#wf7-tenant-id', tenantId);
    await page.click('#wf7-signin-btn');
    await expect(page.locator('#wf7-user-bar')).toBeVisible({ timeout: 10_000 });

    // Wait for todo table to load (already has the shared task)
    await expect(page.locator('#todo-table')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#todo-table')).toContainText('Shared tenant task');
  });
});
