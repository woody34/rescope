# platform-tests

Platform-level integration tests that verify cross-cutting emulator behavior using the Descope Node SDK.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Test | `npm run test:platform` (or `npx nx run platform-tests:test`) |
| Watch | `cd apps/platform-tests && npx vitest` |

## Conventions

- Vitest test runner with TypeScript
- Uses `@descope/node-sdk`
- Emulator auto-started on port 4501 during test runs
