/**
 * WF3 — B2B Multi-Tenant SSO
 */
import { test, expect } from './fixtures';
import {
  resetEmulator, createUserWithPassword, createTenant, addUserToTenant, uniqueLogin,
} from './helpers/emulator';

test.beforeEach(async () => { await resetEmulator(); });

test.describe('WF3 — B2B Tenancy', () => {
  test('password sign-in + tenant select → dct claim visible in JWT panel', async ({ page }) => {
    const login = uniqueLogin('wf3');
    const password = 'TenantPass1!';
    const tenantId = `acme-corp-${Date.now()}`;

    await createUserWithPassword(login, password);
    await createTenant(tenantId, 'Acme Corp');
    await addUserToTenant(login, tenantId, ['admin']);

    await page.goto('/wf3');
    await page.fill('#wf3-login-id', login);
    await page.fill('#wf3-password', password);
    await page.click('#wf3-signin-btn');

    await expect(page.locator('#wf3-tenant-id')).toBeVisible({ timeout: 10_000 });
    await page.fill('#wf3-tenant-id', tenantId);
    await page.click('#wf3-select-tenant-btn');

    await expect(page.locator('#wf3-user-bar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#todo-input')).toBeVisible({ timeout: 5_000 });

    // Add a todo so the table renders
    await page.fill('#todo-input', 'WF3 tenant task');
    await expect(page.locator('#todo-input')).toHaveValue('WF3 tenant task');
    await page.click('#add-todo-btn');
    await expect(page.locator('#todo-table')).toBeVisible({ timeout: 5_000 });

    // dct claim should show the tenant ID
    await expect(page.locator('[data-claim="dct"]')).toBeVisible();
    await expect(page.locator('[data-claim="dct"]')).toContainText(tenantId);
  });

  test('selecting a tenant not associated with user shows error', async ({ page }) => {
    const login = uniqueLogin('wf3-isolation');
    await createUserWithPassword(login, 'IsoPass1!');

    await page.goto('/wf3');
    await page.fill('#wf3-login-id', login);
    await page.fill('#wf3-password', 'IsoPass1!');
    await page.click('#wf3-signin-btn');

    await expect(page.locator('#wf3-tenant-id')).toBeVisible({ timeout: 10_000 });
    await page.fill('#wf3-tenant-id', 'nonexistent-tenant');
    await page.click('#wf3-select-tenant-btn');

    await expect(page.locator('#wf3-error')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#wf3-user-bar')).not.toBeVisible();
  });
});
