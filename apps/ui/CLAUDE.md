# rescope-ui

React/Vite admin UI for inspecting and managing emulator state (users, tenants, roles, OTP codes, access keys, identity providers).

## Commands

| Task | Command (from repo root) |
|------|---------|
| Dev | `npx nx run ui:dev` (or `cd apps/ui && npm run dev`) |
| Build | `npx nx run ui:build` |
| Test (E2E) | `npx nx run ui:test` |
| Lint | `npx nx run ui:lint` |
| Format | `npx nx run ui:format` |
| Format check | `npx nx run ui:format-check` |

## Conventions

- React 19 with TypeScript, Vite 8 bundler
- UI components built on Radix UI primitives (dialog, tabs, dropdown, toast, etc.)
- Routing via react-router-dom v7
- Styling with plain CSS + clsx for conditional classes
- E2E tests use Playwright (`e2e/` directory); one-time setup: `npx playwright install chromium`
- Dev server runs on port 5173, expects emulator API on port 4600
- `build:e2e` target builds in development mode for E2E test runs
