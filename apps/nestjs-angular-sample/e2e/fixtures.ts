/**
 * Playwright fixtures for nestjs-angular-sample tests.
 *
 * Mirrors the pattern from apps/sample-app/e2e/fixtures.ts.
 *
 * KEY: The descope SDK calls POST /v1/auth/try-refresh with `credentials: 'include'`.
 * In headless Chromium, cross-origin fetches with credentials fail at the browser level.
 * We intercept and proxy through Playwright's request API to bypass CORS restrictions.
 */
import { test as base } from '@playwright/test';

const EMULATOR_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';

export const test = base.extend<{}>({
  page: async ({ page }, use) => {
    // Proxy try-refresh to bypass headless Chromium credential restrictions
    await page.route('**/v1/auth/try-refresh**', async (route) => {
      const req = route.request();
      const url = `${EMULATOR_URL}/v1/auth/try-refresh${new URL(req.url()).search}`;
      try {
        const response = await page.request.post(url, {
          headers: {
            ...req.headers(),
            'sec-fetch-mode': 'cors',
            'sec-fetch-dest': 'empty',
          },
          data: req.postData() ?? '{}',
        });
        await route.fulfill({
          status: response.status(),
          headers: Object.fromEntries(Object.entries(response.headers())),
          body: await response.body(),
        });
      } catch {
        await route.continue();
      }
    });

    await use(page);
  },
});

export { expect } from '@playwright/test';
