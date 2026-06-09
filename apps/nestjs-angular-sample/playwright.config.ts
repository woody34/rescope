import { defineConfig, devices } from '@playwright/test';
import * as dotenv from 'dotenv';
dotenv.config();

const EMULATOR_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';
const ANGULAR_URL = 'http://127.0.0.1:4444';
const NEST_URL = 'http://localhost:3333';

export default defineConfig({
  testDir: './e2e',
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: 'list',
  use: {
    baseURL: ANGULAR_URL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    navigationTimeout: 60_000,
    actionTimeout: 15_000,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: [
    {
      // Emulator — reuse if already running on :4600
      command: 'cargo run',
      url: `${EMULATOR_URL}/health`,
      reuseExistingServer: true,
      timeout: 120_000,
      cwd: '../..',
    },
    {
      // NestJS backend — GET /health returns 200 when ready
      command: 'npm run start:dev',
      url: `${NEST_URL}/health`,
      reuseExistingServer: true,
      timeout: 60_000,
      cwd: './backend',
    },
    {
      // Angular frontend — built and served via 'npx serve' (Angular 21 dev
      // server is incompatible with Node.js v22 — times out on all connections)
      command: 'npm run serve:e2e',
      url: ANGULAR_URL,
      reuseExistingServer: true,
      timeout: 180_000,
      cwd: './frontend',
    },
  ],
});
