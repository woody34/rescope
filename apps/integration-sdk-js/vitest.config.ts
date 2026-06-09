import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // Node environment — core-js-sdk is stateless and doesn't require DOM APIs.
    // happy-dom enforces CORS (Origin:null) which blocks response bodies from
    // localhost, causing all SDK calls to fail with JSON parse errors.
    environment: "node",

    // Reuse the same global-setup as the raw HTTP integration suite.
    // Use a relative path string — __dirname is unavailable in ESM context.
    globalSetup: "../integration-api/setup/global-setup.ts",

    // Single fork so all tests share the same emulator process
    pool: "forks",
    poolOptions: {
      forks: { singleFork: true },
    },

    include: ["tests/**/*.sdk.test.ts"],

    // Longer timeout — SDK may do multiple round-trips per test + bcrypt
    testTimeout: 30_000,
    hookTimeout: 30_000,

    globals: true,
  },
});
