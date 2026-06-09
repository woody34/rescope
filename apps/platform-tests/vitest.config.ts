import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",

    // Reuse the same global setup that spawns the emulator binary and
    // waits for /health before any test file runs.
    globalSetup: "../integration-api/setup/global-setup.ts",

    // All scenario tests run in a single fork so they share one
    // emulator process — avoids port collisions and speeds things up.
    pool: "forks",
    poolOptions: {
      forks: { singleFork: true },
    },

    include: ["tests/**/*.scenario.test.ts"],

    // Scenarios are more complex than unit tests — generous timeouts
    // because each scenario does multiple SDK round-trips.
    testTimeout: 45_000,
    hookTimeout: 30_000,

    globals: true,
  },
});
