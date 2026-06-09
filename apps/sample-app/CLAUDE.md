# sample-app

Sample React app (Create React App) demonstrating Descope React SDK integration with the Rescope emulator.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Start | `npx nx run sample-app:start` (port 3001) |
| E2E tests | `npx nx run sample-app:test` |

## Conventions

- Create React App (react-scripts) with TypeScript
- Uses `@descope/react-sdk` for auth flows
- Playwright E2E tests in root directory
- Runs on port 3001 with BROWSER=none (headless)
