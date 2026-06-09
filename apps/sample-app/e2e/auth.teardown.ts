/**
 * Global Teardown — deletes the test user created during auth.setup.
 *
 * Adapted from:
 *   https://github.com/descope-sample-apps/descope-playwright-react-example/blob/main/e2e/auth.teardown.ts
 */
import DescopeClient from "@descope/node-sdk";

require("dotenv").config();

const EMULATOR_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4600";
const PROJECT_ID =
  process.env.EMULATOR_PROJECT_ID ??
  process.env.REACT_APP_DESCOPE_PROJECT_ID ??
  "emulator-project";
const MGMT_KEY =
  process.env.EMULATOR_MANAGEMENT_KEY ??
  process.env.DESCOPE_MANAGEMENT_KEY ??
  "emulator-key";

async function globalTeardown(_config: unknown) {
  const testUser = process.env.TEST_USER;
  if (!testUser) {
    console.warn("[auth.teardown] TEST_USER not set — nothing to clean up.");
    return;
  }

  const descope = DescopeClient({
    projectId: PROJECT_ID,
    managementKey: MGMT_KEY,
    baseUrl: EMULATOR_URL,
  });

  await descope.management.user.delete(testUser);
  console.log(`[auth.teardown] Test user "${testUser}" deleted.`);
}

export default globalTeardown;
