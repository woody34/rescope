# integration-sdk-nodejs

Integration tests exercising the Descope Node SDK (`@descope/node-sdk`) against the Rescope emulator.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Test | `npm run test:sdk-nodejs` (or `npx nx run integration-sdk-nodejs:test`) |
| Watch | `cd apps/integration-sdk-nodejs && npx vitest` |

## Conventions

- Vitest test runner with TypeScript
- Uses `@descope/node-sdk` (server-side SDK)
- Emulator auto-started on port 4501 during test runs
