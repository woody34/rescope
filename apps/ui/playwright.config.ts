import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  // Run tests serially — the emulator is a shared in-memory instance, so
  // parallel workers would reset each other's state mid-test.
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: "html",
  use: {
    baseURL: "http://localhost:4600/",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  // Build UI in dev mode (preserves data-testid) then start the Rust emulator.
  // reuseExistingServer allows skipping the Rust restart if already running, but
  // the UI build always runs to ensure the dist is up-to-date with source changes.
  webServer: {
    command: "npm run build:e2e --prefix apps/ui && cargo run",
    url: "http://localhost:4600/health",
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
    cwd: "../..",
  },
});
