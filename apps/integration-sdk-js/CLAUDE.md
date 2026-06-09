# integration-sdk-js

Integration tests exercising the Descope Core JS SDK (`@descope/core-js-sdk`) against the Rescope emulator.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Test | `npm run test:sdk-js` (or `npx nx run integration-sdk-js:test`) |
| Watch | `cd apps/integration-sdk-js && npx vitest` |

## Conventions

- Vitest with happy-dom environment for browser SDK simulation
- Uses `@descope/core-js-sdk` (browser-oriented SDK)
- Emulator auto-started on port 4501 during test runs
