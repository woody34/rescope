# Rescope

A standalone Rust emulator for the Descope authentication API. Nx monorepo with a Rust/Axum backend, React/Vite admin UI, and Astro/Starlight docs site.

## Tech Stack

- **Languages**: Rust, TypeScript
- **Frameworks**: Axum (API), React 19 (admin UI), Astro/Starlight (docs)
- **Build**: Cargo, Vite 8, Nx 20
- **Test**: cargo test (unit), Vitest (integration), Playwright (E2E)
- **Lint**: Clippy + cargo fmt (Rust), ESLint + Prettier (TypeScript)
- **CI**: GitHub Actions

## Project Structure

```
/
├── apps/
│   ├── api/                    ← Rust/Axum backend emulator (binary + lib)
│   ├── ui/                     ← React/Vite admin UI (Radix UI components)
│   ├── docs/                   ← Astro/Starlight documentation site
│   ├── integration-api/        ← Vitest HTTP integration tests
│   ├── integration-sdk-js/     ← JS SDK integration tests
│   ├── integration-sdk-nodejs/ ← Node SDK integration tests
│   ├── platform-tests/         ← Platform-level tests
│   ├── nestjs-angular-sample/  ← NestJS + Angular sample app
│   └── sample-app/             ← Sample consumer app
├── libs/
│   └── rescope-core/           ← Shared Rust core (framework-agnostic, WASM-compatible)
├── Cargo.toml                  ← Cargo workspace (apps/api + libs/rescope-core)
├── nx.json                     ← Nx workspace config
└── Makefile                    ← Rust-specific build/test/lint targets
```

## Build & Run

| Task | Command |
|------|---------|
| Dev (API + UI) | `npm run dev` |
| Build all | `npm run build` |
| Build API only | `cargo build` (from root) |
| Build UI only | `cd apps/ui && npm run build` |
| Release binary | `npx nx run api:build-release` |
| Docs dev | `npm run docs:dev` |

## Testing

- **Rust unit**: `npm run test:unit` — cargo test --lib, no emulator needed (~5s)
- **API integration**: `npm run test:api` — Vitest, auto-starts emulator on port 4501 (~30s)
- **SDK integration**: `npm run test:sdk-js` / `npm run test:sdk-nodejs`
- **E2E**: `npm run test:e2e` — Playwright, builds UI + starts emulator on port 4600 (~2m)
- **All tests**: `npm run test`
- **Coverage**: `npm run api:coverage` (requires cargo-llvm-cov, 95% floor)
- **Parity**: `make test-parity` (requires DESCOPE_PARITY_PROJECT_ID + DESCOPE_PARITY_MANAGEMENT_KEY)

## Key Conventions

- Monorepo orchestrated by Nx; run all commands from repo root
- Rust workspace: `apps/api` (binary) depends on `libs/rescope-core` (shared logic)
- `rescope-core` is framework-agnostic and WASM-compatible (no ring/rcgen in default features)
- Management API auth: `Authorization: Bearer <project_id>:<management_key>` (default: `emulator-project:emulator-key`)
- Emulator state is in-memory; `POST /emulator/reset` clears and re-seeds
- Integration tests auto-build the Rust binary before running
- E2E setup: `cd apps/ui && npx playwright install chromium` (one-time)
- Port 4600 = dev/E2E emulator, port 4501 = integration test emulator

## Architecture

```
┌─────────────┐     ┌──────────────────────────────────┐
│  React UI   │────▶│         Axum HTTP Server          │
│ (Vite, :5173)│     │         (rescope, :4600)          │
└─────────────┘     │                                    │
                    │  ┌─────────────────────────────┐  │
┌─────────────┐     │  │       rescope-core           │  │
│ Descope SDKs│────▶│  │  (users, tenants, tokens,    │  │
│ (JS/Node/Go)│     │  │   roles, OTP, magic link,    │  │
└─────────────┘     │  │   SAML/OIDC, connectors)     │  │
                    │  └─────────────────────────────┘  │
┌─────────────┐     │  ┌──────────┐  ┌──────────────┐  │
│  curl / HTTP│────▶│  │ JWT/JWKS │  │ IdP Emulator │  │
└─────────────┘     │  └──────────┘  │ (OIDC + SAML)│  │
                    │                └──────────────┘  │
                    └──────────────────────────────────┘
```
