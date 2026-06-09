/**
 * Feature: Session Management
 *
 * Tests unauthenticated state and session behaviour:
 *   1. Unauthenticated user sees the Descope sign-in component
 *   2. Authenticated user can log out (session is cleared)
 *
 * The unauthenticated tests override storageState with empty state to simulate
 * a fresh browser with no JWT in localStorage.
 */
import { expect, test } from "./fixtures";

test.describe("Unauthenticated state", () => {
  // Override global auth state — simulate a fresh, logged-out browser
  test.use({ storageState: { cookies: [], origins: [] } });

  test("unauthenticated user sees the Descope sign-in component", async ({
    page,
  }) => {
    await page.goto("/");

    // Wait for app to load — in unauthenticated state the Descope web
    // component is mounted by React. We verify it's attached to the DOM.
    // Note: the component may be visually hidden if flow assets can't load
    // from localhost (Phase 2: flow rendering support). The important thing
    // for Phase 1 is that React chose to render the unauthenticated branch.
    await expect(page.locator("descope-wc")).toBeAttached({ timeout: 15_000 });

    // The welcome / greeting content should NOT be visible
    await expect(page.getByText(/Hello, /i)).not.toBeVisible();
  });


  test("unauthenticated user does not see authenticated content", async ({
    page,
  }) => {
    await page.goto("/");
    // Loading spinner should resolve and then show sign-in, not greeting
    await expect(page.getByText(/Loading\.\./i)).not.toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByText(/Hello, /i)).not.toBeVisible();
  });
});

test.describe("Session lifecycle", () => {
  test.use({ storageState: "playwright/.auth/user.json" });

  test("clearing localStorage removes authenticated state on reload", async ({
    page,
  }) => {
    await page.goto("/");
    // Confirm authenticated
    await expect(page.getByText(/Hello, /i)).toBeVisible({ timeout: 10_000 });

    // Simulate logout by clearing local storage (what the SDK's logout does)
    await page.evaluate(() => {
      window.localStorage.removeItem("DS");
      window.localStorage.removeItem("DSR");
    });
    await page.reload();

    // After reload without JWTs, the app should show the sign-in component
    await expect(page.getByText(/Hello, /i)).not.toBeVisible({ timeout: 10_000 });
  });
});
