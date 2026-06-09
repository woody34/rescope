/**
 * Feature: Authenticated Home Page
 *
 * Verifies that a signed-in user sees:
 *   - The welcome message from the app
 *   - A personalized greeting containing their user ID
 *
 * Auth state is pre-seeded by auth.setup.ts (global setup), which creates a
 * test user via the emulator's management API and injects JWTs into localStorage.
 * This validates the full emulator JWT → JWKS validation → React SDK auth chain.
 */
import { expect, test } from "./fixtures";

test.use({ storageState: "playwright/.auth/user.json" });

test.describe("Authenticated Home", () => {
  test("shows welcome message and personalized greeting", async ({ page }) => {
    await page.goto("/");
    await page.pause();

    // Wait for the React app to finish auth check (not showing loading state)
    await expect(page.getByText(/Loading\.\./i)).not.toBeVisible({
      timeout: 10_000,
    });

    // The signed-in view shows both of these
    await expect(page.getByText(/Welcome to my app!/i)).toBeVisible();
    await expect(page.getByText(/Hello, /i)).toBeVisible();
  });

  test("does not show the Descope auth component when signed in", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByText(/Loading\.\./i)).not.toBeVisible({
      timeout: 10_000,
    });

    // The <Descope flowId="sign-up-or-in" /> component should NOT be rendered
    // when the user is already authenticated
    await expect(page.locator("descope-wc")).not.toBeVisible();
  });
});
