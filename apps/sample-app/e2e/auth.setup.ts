/**
 * Global Setup — creates a test user via the Descope Node SDK (pointed at the
 * local emulator), generates a magic link, verifies it to obtain session JWTs,
 * then injects them into browser localStorage so every test starts authenticated.
 *
 * Adapted from:
 *   https://github.com/descope-sample-apps/descope-playwright-react-example/blob/main/e2e/auth.setup.ts
 *
 * Key differences vs the upstream:
 *   - baseURL is passed to Descope SDK so calls go to :4600, not api.descope.com
 *   - Token extracted with `?t=` (matches real Descope format)
 *   - Env vars aligned with other integration apps (EMULATOR_PROJECT_ID etc.)
 */
import DescopeClient from "@descope/node-sdk";
import { chromium, type FullConfig } from "@playwright/test";
import * as crypto from "crypto";
import * as path from "path";
import * as fs from "fs";

require("dotenv").config();

export const authFile = "playwright/.auth/user.json";

const EMULATOR_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4600";
const PROJECT_ID =
  process.env.EMULATOR_PROJECT_ID ??
  process.env.REACT_APP_DESCOPE_PROJECT_ID ??
  "emulator-project";
const MGMT_KEY =
  process.env.EMULATOR_MANAGEMENT_KEY ??
  process.env.DESCOPE_MANAGEMENT_KEY ??
  "emulator-key";
const SAMPLE_APP_URL = "http://localhost:3001";

async function globalSetup(_config: FullConfig) {
  // Ensure auth directory exists
  const authDir = path.dirname(authFile);
  if (!fs.existsSync(authDir)) {
    fs.mkdirSync(authDir, { recursive: true });
  }

  const testUser = `test-${crypto.randomBytes(8).toString("hex")}`;
  process.env.TEST_USER = testUser;

  // Point ALL Node SDK calls at the local emulator
  const descope = DescopeClient({
    projectId: PROJECT_ID,
    managementKey: MGMT_KEY,
    baseUrl: EMULATOR_URL,
  });

  // 1. Create a test user
  const createRes = await descope.management.user.createTestUser(testUser, {
    email: `${testUser}@test.local`,
  });
  if (!createRes.ok) {
    throw new Error(`Failed to create test user: ${JSON.stringify(createRes.error)}`);
  }

  // 2. Generate a magic link for the test user
  const magicRes = await descope.management.user.generateMagicLinkForTestUser(
    "email",
    testUser,
    "http://localhost:3001/verify",
  );
  if (!magicRes.ok || !magicRes.data?.link) {
    throw new Error(`Failed to generate magic link: ${JSON.stringify(magicRes.error)}`);
  }

  // 3. Extract token — emulator returns ?t=<value> (matches real Descope)
  const linkUrl = magicRes.data.link;
  const token =
    new URL(linkUrl).searchParams.get("t") ??
    linkUrl.split("?t=")[1];
  if (!token) {
    throw new Error(`Could not extract token from link: ${linkUrl}`);
  }

  // 4. Verify the magic link — get session JWTs
  const authRes = await descope.magicLink.verify(token);
  if (!authRes.ok || !authRes.data?.sessionJwt) {
    throw new Error(`Magic link verification failed: ${JSON.stringify(authRes.error)}`);
  }

  // 5. Inject JWTs into browser localStorage and save storage state
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto(SAMPLE_APP_URL);

  const { sessionJwt, refreshJwt } = authRes.data;
  await page.evaluate(
    ([ds, dsr]: [string, string | undefined]) => {
      window.localStorage.setItem("DS", ds);
      if (dsr) window.localStorage.setItem("DSR", dsr);
    },
    [sessionJwt, refreshJwt ?? undefined] as [string, string | undefined],
  );

  await page.context().storageState({ path: authFile });
  await browser.close();

  console.log(`[auth.setup] Test user "${testUser}" created and authenticated.`);
}

export default globalSetup;
