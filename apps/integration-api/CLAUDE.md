# integration-api

HTTP integration tests for the Rescope emulator API. Tests auth flows, management endpoints, and emulator-specific features using the Descope Node SDK.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Test | `npm run test:api` (or `npx nx run integration-api:test`) |
| Watch | `cd apps/integration-api && npx vitest` |

## Conventions

- Vitest test runner with TypeScript
- Tests auto-build the emulator binary and start it on port 4501
- Uses `@descope/node-sdk` to exercise the real SDK against the emulator
- Each test file should call `/emulator/reset` in beforeEach for clean state
