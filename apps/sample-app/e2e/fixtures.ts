/**
 * Playwright fixtures for sample-app tests.
 *
 * KEY FIX: The @descope/react-sdk calls POST /v1/auth/try-refresh on init
 * with `credentials: 'include'`. In headless Chromium, cross-origin fetches
 * with credentials fail with net::ERR_FAILED before reaching the server
 * (browser CORS preflight → response → second fetch fails on credentials).
 *
 * Solution: intercept try-refresh via page.route() and proxy it through
 * Playwright's request API (which bypasses browser security restrictions).
 * This allows the emulator to validate the refresh token and return fresh
 * session JWTs, properly authenticating the SDK.
 */
import { test as base } from "@playwright/test";

const EMULATOR_URL =
  process.env.EMULATOR_BASE_URL ?? "http://localhost:4600";

export type TestFixtures = {
  /** An authenticated page with try-refresh proxied to the emulator */
  authedPage: ReturnType<typeof base["extend"]> extends { test: infer T }
    ? T
    : never;
};

/**
 * The `page` fixture in all tests automatically intercepts `/v1/auth/try-refresh`
 * and proxies it through Playwright's request API to bypass CORS restrictions.
 *
 * Usage: import { test, expect } from "./fixtures";
 *        (drop-in replacement for @playwright/test)
 */
export const test = base.extend<{}>({
  page: async ({ page }, use) => {
    // Intercept the try-refresh call and proxy it through Playwright's API
    // to bypass browser CORS/credential restrictions
    await page.route("**/v1/auth/try-refresh**", async (route) => {
      const req = route.request();
      const url = `${EMULATOR_URL}/v1/auth/try-refresh${
        new URL(req.url()).search
      }`;

      try {
        const response = await page.request.post(url, {
          headers: {
            ...req.headers(),
            // Remove browser security headers that interfere with server-side fetch
            "sec-fetch-mode": "cors",
            "sec-fetch-dest": "empty",
          },
          data: req.postData() ?? "{}",
        });

        await route.fulfill({
          status: response.status(),
          headers: Object.fromEntries(Object.entries(response.headers())),
          body: await response.body(),
        });
      } catch (e) {
        // If proxy fails, continue without interception (will fail naturally)
        await route.continue();
      }
    });

    await use(page);
  },
});

export { expect } from "@playwright/test";
