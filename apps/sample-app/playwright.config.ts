import { defineConfig, devices } from "@playwright/test";

const EMULATOR_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4600";
const SAMPLE_APP_URL = "http://localhost:3001";

export default defineConfig({
  testDir: "./e2e",
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: "list",
  use: {
    baseURL: SAMPLE_APP_URL,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  // Two webServers:
  //   1. Rust emulator — reuse if already running on :4600
  //   2. This React app — started on :3001
  webServer: [
    {
      command: "cargo run",
      url: `${EMULATOR_URL}/health`,
      reuseExistingServer: true,
      timeout: 120_000,
      cwd: "../..",
    },
    {
      command: "PORT=3001 BROWSER=none npm start",
      url: SAMPLE_APP_URL,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
      cwd: ".",
    },
  ],
  globalSetup: require.resolve("./e2e/auth.setup"),
  globalTeardown: require.resolve("./e2e/auth.teardown"),
});
