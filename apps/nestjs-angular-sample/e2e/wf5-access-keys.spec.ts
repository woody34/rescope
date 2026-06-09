import { test, expect } from './fixtures';
import { resetEmulator, deleteAllAccessKeys } from './helpers/emulator';

test.beforeEach(async () => {
  await resetEmulator();
  await deleteAllAccessKeys(); // reset does NOT clear access keys
});

test.describe('WF5 — Access Keys', () => {
  test('create access key → appears in table → delete → removed', async ({ page }) => {
    await page.goto('/wf5');

    // Table initially empty
    await expect(page.locator('.empty')).toBeVisible({ timeout: 10_000 });

    // Create a key
    const keyName = `test-agent-${Date.now()}`;
    await page.fill('#wf5-key-name', keyName);
    await page.click('#wf5-create-key-btn');

    // Cleartext shown once
    await expect(page.locator('#wf5-cleartext')).toBeVisible({ timeout: 5_000 });
    const cleartext = await page.locator('#wf5-cleartext code').textContent();
    expect(cleartext?.length).toBeGreaterThan(10);

    // Key appears in table
    await expect(page.locator('#wf5-key-table')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#wf5-key-table')).toContainText(keyName);

    // Delete the key
    await page.locator('[data-testid^="delete-"]').first().click();

    // Table is empty again
    await expect(page.locator('.empty')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('#wf5-key-table')).not.toBeVisible();
  });

  test('multiple keys can coexist; deleting one does not affect others', async ({ page }) => {
    await page.goto('/wf5');
    await expect(page.locator('.empty')).toBeVisible({ timeout: 10_000 });

    for (const name of ['agent-a', 'agent-b', 'agent-c']) {
      await page.fill('#wf5-key-name', name);
      await page.click('#wf5-create-key-btn');
      await expect(page.locator('#wf5-key-table')).toContainText(name, { timeout: 5_000 });
    }

    const rows = page.locator('#wf5-key-table tbody tr');
    await expect(rows).toHaveCount(3);

    // Delete first key
    await page.locator('[data-testid^="delete-"]').first().click();
    await expect(rows).toHaveCount(2, { timeout: 5_000 });
  });
});
