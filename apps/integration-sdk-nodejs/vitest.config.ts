import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",

    // Shared emulator lifecycle with sdk-js suite
    globalSetup: "../integration-api/setup/global-setup.ts",

    // Single fork — all tests share the same emulator process
    pool: "forks",
    poolOptions: {
      forks: { singleFork: true },
    },

    include: ["tests/**/*.node.test.ts"],

    // Generous timeout — Node SDK may do multiple round-trips + bcrypt
    testTimeout: 30_000,
    hookTimeout: 30_000,

    globals: true,
  },
});
