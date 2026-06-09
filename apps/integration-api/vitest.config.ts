import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // Single-fork ensures all test files share the same emulator process
    // spawned in globalSetup. Each test file gets beforeEach → resetEmulator().
    pool: "forks",
    poolOptions: {
      forks: {
        singleFork: true,
      },
    },
    globalSetup: "./setup/global-setup.ts",
    // Test files
    include: ["tests/**/*.test.ts"],
    // 30s timeout for tests that may involve bcrypt hashing
    testTimeout: 30_000,
    hookTimeout: 30_000,
  },
});
