## Why

Developers and CI pipelines that rely on Descope for authentication cannot work offline, suffer test flakiness due to live network dependencies, and face slow feedback loops from round-trip latency to Descope's hosted servers. A local Descope emulator written in Rust eliminates all three problems by providing a fast, self-contained HTTP server that faithfully replicates the Descope API surface used by any application consuming the Descope SDKs.

## Build Methodology

This project is built with **Test-Driven Development (TDD)**. Each capability spec drives a test file before any implementation is written. The cycle is: spec → failing test → implement → passing test.

**Coverage requirement: 100%.** The CI pipeline enforces this with `cargo llvm-cov --fail-under-coverage 100`. Coverage includes both line and branch coverage. Integration tests that spin up the full HTTP server are required in addition to unit tests, ensuring all routing and middleware code is exercised.

**Parity testing**: A separate test harness (enabled via `--features parity`) runs each scenario against both the live Descope API and the local emulator simultaneously, performing structural diffing of HTTP status codes, response shapes, error codes, and header presence. This serves as both a reverse-engineering tool during development and a drift detection mechanism during maintenance. Parity tests require `DESCOPE_PARITY_PROJECT_ID` and `DESCOPE_PARITY_MANAGEMENT_KEY` environment variables to be set; they are never run in offline CI.

## What Changes

This is a new standalone project — no existing codebase is modified.

- **New**: A Rust HTTP server (Axum + Tokio) implementing the full Descope REST API surface consumed by `@descope/node-sdk`, `@descope/angular-sdk`, `@descope/nextjs-sdk`, and `@descope/react-native-sdk`
- **New**: In-memory stores for users, tenants, single-use tokens (magic links, SAML codes, embedded tokens), and a refresh token revocation set
- **New**: RSA key pair generation at startup; all JWTs signed with RS256; JWKS endpoint serves the public key for SDK-side verification
- **New**: Password hashing via bcrypt for all password-based auth flows
- **New**: CORS middleware configured for all origins with credential support (echoes `Origin` header to satisfy `Allow-Credentials: true`)
- **New**: JSON seed file support for pre-loading users and tenants at startup
- **New**: Emulator lifecycle endpoints: `GET /health` and `POST /emulator/reset`
- **New**: Optional deterministic RSA key loading from a PEM file (`DESCOPE_EMULATOR_KEY_FILE`) for reproducible JWTs across restarts
- **New**: Rust unit test suite at 95% line + branch coverage enforced by `cargo-llvm-cov`
- **New**: **Vitest integration test suite** (`integration/`) written in TypeScript — starts the Rust binary and tests the full HTTP API using `fetch` and optionally the real Descope TS SDKs as clients
- **New**: Parity test harness (`--features parity`) for structural diffing against live Descope endpoints

## Capabilities

### New Capabilities

- `jwt-engine`: RSA key pair generation, RS256 JWT signing (session + refresh tokens), JWT verification, and JWKS endpoint serialization
- `user-store`: In-memory user store with multi-index lookup (loginId, userId, email, phone), full CRUD, search/filter, and test-user flagging
- `tenant-store`: In-memory tenant store with CRUD and domain-based lookup for SAML tenant resolution
- `token-store`: Single-use token store for magic links, SAML auth codes, embedded links, and password reset tokens — all consumed on first use
- `revocation-store`: Refresh token revocation set; checked on every `refresh` and `me` call
- `auth-flows`: Authentication endpoints — password (signUp, signIn, replace, update, sendReset), magic link (signIn, verify, update), SAML/SSO (start, exchange), OTP (update phone)
- `session-flows`: Session lifecycle endpoints — refresh, logout, me, validateSession
- `mgmt-api`: Management endpoints — user CRUD (create, createTestUser, load, loadByUserId, search, update, patch, updateEmail, setActivePassword, delete, deleteByUserId, deleteAllTestUsers, addTenant, generateMagicLinkForTestUser, generateEmbeddedLink), tenant loadAll
- `seed-loader`: JSON seed file parsing and startup loading for users and tenants; clear error on invalid schema
- `emulator-lifecycle`: Health check endpoint, state reset endpoint, environment variable configuration
- `parity-testing`: Optional feature-flagged test harness that runs each endpoint scenario against both live Descope and the local emulator, performing structural diffing to detect behavioral drift and reverse-engineer undocumented behavior

### Modified Capabilities

_(none — this is a greenfield project)_

## Impact

- **New binary**: `descope-emulator` Rust binary, runnable as a standalone process on `localhost:4600` (configurable)
- **No existing code modified**: This is a self-contained project; consuming applications point their `DESCOPE_BASE_URL` (or equivalent SDK config) at the emulator
- **Dependencies**:
  - `axum` — HTTP server and routing
  - `tokio` — async runtime
  - `serde` / `serde_json` — JSON serialization
  - `jsonwebtoken` — RS256 JWT creation and verification
  - `rsa` — RSA key pair generation
  - `bcrypt` — password hashing
  - `uuid` — userId generation
  - `tower-http` — CORS middleware
  - `rand` / `hex` — random token generation (magic links, SAML codes)
  - `cargo-llvm-cov` (dev) — Rust coverage enforcement
  - `vitest`, `@descope/node-sdk` (dev, TypeScript) — external HTTP API integration tests
- **Configuration**: Environment variables — `DESCOPE_EMULATOR_PORT` (default 4600), `DESCOPE_PROJECT_ID`, `DESCOPE_MANAGEMENT_KEY`, `DESCOPE_EMULATOR_SEED_FILE`, `DESCOPE_EMULATOR_KEY_FILE`
- **Parity configuration**: `DESCOPE_PARITY_PROJECT_ID`, `DESCOPE_PARITY_MANAGEMENT_KEY` (only required for `--features parity` test runs)
- **API contract**: Matches the Descope REST API surface documented in `prd-descope-emulator.md` and `architecture-diagram.md`
