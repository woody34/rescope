# nestjs-angular-sample

Sample full-stack app demonstrating Descope integration with NestJS (backend) and Angular (frontend), running against the Rescope emulator.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Dev (all 3) | `npx nx run nestjs-angular-sample:dev` (emulator + backend + frontend in parallel) |
| Build backend | `npx nx run nestjs-angular-sample:build-backend` |
| Build frontend | `npx nx run nestjs-angular-sample:build-frontend` |
| E2E tests | `npm run test:nestjs-sample` (or `npx nx run nestjs-angular-sample:e2e`) |
| E2E (headed) | `npm run test:nestjs-sample:watch` |

## Conventions

- Backend: NestJS 11 with `@descope/node-sdk`, Jest for unit tests, port 3000
- Frontend: Angular 21 with `@descope/angular-sdk` + Descope web components, port 4444
- E2E: Playwright (root-level config), tests in `e2e/` directory
- `dev` target starts the Rescope emulator, NestJS backend, and Angular frontend in parallel
- Backend uses Prettier + ESLint; frontend uses Angular CLI conventions
