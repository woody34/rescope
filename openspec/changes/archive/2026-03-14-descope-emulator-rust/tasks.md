## 1. Project Bootstrap

- [x] 1.1 Initialize `Cargo.toml` with `[lib]` and `[[bin]]` targets, add all Rust dependencies (`axum`, `tokio`, `serde`, `serde_json`, `jsonwebtoken`, `rsa`, `bcrypt`, `uuid`, `tower-http`, `rand`, `thiserror`, `scopeguard`, `hex`, `sha2`, `base64`)
- [x] 1.2 Add dev dependencies (`cargo-llvm-cov`) and `[features]` section for `parity`
- [x] 1.3 Create `src/lib.rs` with all `pub mod` declarations and `src/main.rs` thin entry point
- [x] 1.4 Create `src/config.rs` — `EmulatorConfig` struct read from env vars with defaults (port 4600, project ID, management key, TTLs, seed file, key file paths)
- [x] 1.5 Create `src/error.rs` — `EmulatorError` enum with `thiserror`, implement `IntoResponse` mapping each variant to Descope JSON error shape and HTTP status code
- [x] 1.6 Create `src/types.rs` — shared types: `User`, `Tenant`, `TokenEntry`, `TokenType`, `UserResponse`, `JwtResponse`, `AuthenticationResponse`, `SdkResponse<T>`
- [x] 1.7 Initialize `package.json` in project root with `vitest`, `@descope/node-sdk`, and TypeScript dev dependencies
- [x] 1.8 Create `integration/vitest.config.ts` with `globalSetup`/`globalTeardown` pointing to setup helpers
- [x] 1.9 Implement `integration/setup/global-setup.ts` — spawn the Rust binary, poll `GET /health` until ready (timeout 15s), expose base URL to tests
- [x] 1.10 Implement `integration/setup/global-teardown.ts` — kill the emulator process
- [x] 1.11 Implement `integration/helpers/client.ts` — typed `fetch` wrapper for the emulator base URL
- [x] 1.12 Implement `integration/helpers/sdk-client.ts` — `@descope/node-sdk` client pointed at the emulator for SDK-level integration tests

## 2. JWT Engine

- [x] 2.1 Write failing tests for key generation (fresh key pair, load from PEM file, invalid PEM error)
- [x] 2.2 Implement `src/jwt/key_manager.rs` — generate 2048-bit RSA key pair with `rsa` crate, export to PKCS8 PEM, create `EncodingKey` and `DecodingKey`, compute `kid` as `hex(sha256(der))[..16]`
- [x] 2.3 Write failing tests for JWKS serialization (valid JWK Set shape, `n`/`e` base64url encoded, matches signing key)
- [x] 2.4 Implement JWKS builder in `key_manager.rs` — serialize public key to JWK `{ kty, use, alg, kid, n, e }`
- [x] 2.5 Write failing tests for JWT signing (correct claims, correct `alg`/`kid` header, session TTL, refresh TTL)
- [x] 2.6 Implement `src/jwt/token_generator.rs` — sign session JWT and refresh JWT with all claims from design spec
- [x] 2.7 Write failing tests for JWT verification (valid token passes, expired token fails, wrong key fails)
- [x] 2.8 Implement `src/jwt/token_validator.rs` — verify RS256 signature, expiry, extract claims

## 3. In-Memory Stores

- [x] 3.1 Write failing tests for user store (create, duplicate loginId, load by loginId, load by additional loginId, load by userId, not found error)
- [x] 3.2 Implement `src/store/user_store.rs` — `HashMap` with four indices (loginId, userId, email, phone), full CRUD, test-user flagging, password storage
- [x] 3.3 Write failing tests for user search (by email, by phone, by customAttributes, withTestUser flag)
- [x] 3.4 Implement user search filtering in `user_store.rs`
- [x] 3.5 Write failing tests for user update/patch (update clears fields, patch preserves fields)
- [x] 3.6 Implement update (full replace) and patch (partial merge) in `user_store.rs`
- [x] 3.7 Write failing tests for deletion (delete by loginId, by userId, delete all test users, idempotent delete)
- [x] 3.8 Implement delete operations in `user_store.rs`
- [x] 3.9 Write failing tests for bcrypt password storage and verification (correct password passes, wrong password fails)
- [x] 3.10 Implement `set_password` and `verify_password` in `user_store.rs` using `tokio::task::spawn_blocking`
- [x] 3.11 Write failing tests for tenant store (insert, load all, find by email domain, domain not found)
- [x] 3.12 Implement `src/store/tenant_store.rs`
- [x] 3.13 Write failing tests for token store (insert, consume single-use, consume non-existent, token generation format, cap eviction with logging)
- [x] 3.14 Implement `src/store/token_store.rs` with 10,000 entry cap and oldest-first eviction
- [x] 3.15 Write failing tests for revocation store (revoke, is_revoked, reset clears set)
- [x] 3.16 Implement `src/store/revocation_store.rs`

## 4. State and Server Wiring

- [ ] 4.1 Implement `src/state.rs` — `EmulatorState` with all stores wrapped in `Arc<tokio::sync::RwLock<T>>`
- [ ] 4.2 Implement `src/server.rs` — `build_router(state)` function constructing the full Axum router with CORS (`AllowOrigin::mirror_request()`, `allow_credentials(true)`), all route groups nested
- [ ] 4.3 Verify `build_router(state)` is fully decoupled from `main.rs` so the Vitest `globalSetup` can spawn the binary and the Rust unit tests can test routing logic independently

## 5. Infrastructure Routes

- [ ] 5.1 Write failing tests for JWKS endpoints (`/.well-known/jwks.json`, `/v2/keys/{projectId}`)
- [ ] 5.2 Implement `src/routes/jwks.rs`
- [ ] 5.3 Write failing tests for health endpoint (`GET /health` returns `{ "ok": true }`)
- [ ] 5.4 Write failing tests for reset endpoint (clears users, clears tokens, clears revocation set, re-applies seed)
- [ ] 5.5 Implement `src/routes/emulator.rs` — health and reset handlers
- [ ] 5.6 Implement `src/cookies.rs` — `Set-Cookie` header builder for `DS` (session JWT) and `DSR` (refresh JWT) with `HttpOnly`, `SameSite=Lax`, `Path=/`

## 6. Auth Routes — Password

- [ ] 6.1 Write failing integration tests for `POST /v1/auth/password/signup` (success, duplicate loginId error)
- [x] 6.2 Implement `signup` handler in `src/routes/auth/password.rs`
- [x] 6.3 Write failing integration tests for `POST /v1/auth/password/signin` (valid credentials, wrong password, unknown user)
- [x] 6.4 Implement `signin` handler
- [x] 6.5 Write failing tests for `POST /v1/auth/password/replace` (correct old password, wrong old password)
- [x] 6.6 Implement `replace` handler
- [x] 6.7 Write failing tests for `POST /v1/auth/password/reset` (existing user returns masked email, unknown user error)
- [x] 6.8 Implement `send_reset` handler
- [x] 6.9 Write failing tests for `POST /v1/auth/password/update` (valid reset token, exhausted token, invalid token)
- [x] 6.10 Implement `update` handler

## 7. Auth Routes — Magic Link

- [x] 7.1 Write failing tests for `POST /v1/auth/magiclink/signin/email` (existing user, non-existent user)
- [x] 7.2 Implement magic link signin handler in `src/routes/auth/magic_link.rs`
- [x] 7.3 Write failing tests for `POST /v1/auth/magiclink/verify` (valid token returns session, single-use, invalid token)
- [x] 7.4 Implement magic link verify handler
- [x] 7.5 Write failing tests for `POST /v1/auth/magiclink/update/email` (valid refresh token, invalid token)
- [x] 7.6 Implement magic link update email handler

## 8. Auth Routes — SAML and OTP

- [x] 8.1 Write failing tests for `POST /v1/auth/saml/start` (email lookup → SAML tenant, tenant ID lookup, no SAML tenant error, unknown user error)
- [x] 8.2 Implement SAML start handler in `src/routes/auth/saml.rs` with dual tenant resolution
- [x] 8.3 Write failing tests for `POST /v1/auth/saml/exchange` (valid code, single-use, invalid code)
- [x] 8.4 Implement SAML exchange handler
- [x] 8.5 Write failing tests for `POST /v1/auth/otp/update/phone/sms` (phone updated, addToLoginIDs flag, unknown user)
- [x] 8.6 Implement OTP phone update handler in `src/routes/auth/otp.rs`

## 9. Session Routes

- [x] 9.1 Write failing tests for `POST /v1/auth/refresh` (valid refresh token, expired token, revoked token)
- [x] 9.2 Implement refresh handler in `src/routes/auth/session.rs`
- [x] 9.3 Write failing tests for `POST /v1/auth/logout` (revokes token, subsequent refresh fails)
- [x] 9.4 Implement logout handler
- [x] 9.5 Write failing tests for `GET /v1/auth/me` (valid token returns user, invalid/revoked token fails)
- [x] 9.6 Implement me handler
- [x] 9.7 Write failing tests for `POST /v1/auth/validate` (valid session JWT, expired JWT)
- [x] 9.8 Implement validate handler
- [x] 9.9 Write failing tests for `Set-Cookie` headers on all auth responses (DS and DSR cookies present with correct attributes)
- [x] 9.10 Wire `cookies.rs` into all auth response paths

## 10. Management Routes — User

- [x] 10.1 Write failing tests for management auth header validation (missing header → 401)
- [x] 10.2 Implement management auth middleware in `src/routes/mgmt/user.rs`
- [x] 10.3 Write failing tests for `POST /v1/mgmt/user/create` and `/create/test` (success, duplicate error, test flag)
- [x] 10.4 Implement create and createTestUser handlers
- [x] 10.5 Write failing tests for `GET /v1/mgmt/user` (by loginId) and `GET /v1/mgmt/user/userid` (by userId)
- [x] 10.6 Implement load and loadByUserId handlers
- [x] 10.7 Write failing tests for `POST /v1/mgmt/user/search` (by email, by phone, by customAttribute, withTestUser, pagination)
- [x] 10.8 Implement search handler with pagination
- [x] 10.9 Write failing tests for `POST /v1/mgmt/user/update` and `PATCH /v1/mgmt/user/patch`
- [x] 10.10 Implement update and patch handlers
- [x] 10.11 Write failing tests for `POST /v1/mgmt/user/update/email`
- [x] 10.12 Implement updateEmail handler
- [x] 10.13 Write failing tests for `POST /v1/mgmt/user/password/set/active` (enables subsequent signin)
- [x] 10.14 Implement setActivePassword handler
- [x] 10.15 Write failing tests for `DELETE /v1/mgmt/user` (idempotent), `DELETE /v1/mgmt/user/userid`, `DELETE /v1/mgmt/user/test/delete/all`
- [x] 10.16 Implement all delete handlers
- [x] 10.17 Write failing tests for `POST /v1/mgmt/user/tenant/add` (adds tenant, idempotent)
- [x] 10.18 Implement addTenant handler
- [x] 10.19 Write failing tests for `POST /v1/mgmt/tests/generate/magiclink` (test user succeeds, regular user fails)
- [x] 10.20 Implement generateMagicLinkForTestUser handler
- [x] 10.21 Write failing tests for `POST /v1/mgmt/user/embeddedlink`
- [x] 10.22 Implement generateEmbeddedLink handler

## 11. Management Routes — Tenant

- [x] 11.1 Write failing tests for `GET /v1/mgmt/tenant/all`
- [x] 11.2 Implement tenant loadAll handler in `src/routes/mgmt/tenant.rs`

## 12. Seed Loader

- [x] 12.1 Write failing tests for seed file parsing (valid file loads users and tenants, tenants loaded before users, missing file error, invalid JSON error)
- [x] 12.2 Implement `src/seed.rs` — parse JSON seed file, create tenants then users, hash passwords with `spawn_blocking`, assign tenants to users
- [x] 12.3 Write failing test: seeded user with password is authenticatable via `signin` immediately after startup
- [x] 12.4 Wire seed loading into `main.rs` startup and `POST /emulator/reset` handler

## 13. Vitest Integration Tests — Core Flows

- [x] 13.1 `integration/tests/password.test.ts` — signup, signin (valid/wrong password/unknown user), replace, reset, update
- [x] 13.2 `integration/tests/magic-link.test.ts` — signIn/email, verify (valid/single-use/invalid), update/email
- [x] 13.3 `integration/tests/saml.test.ts` — start (email lookup, tenant ID lookup, no SAML tenant error), exchange (valid/single-use/invalid)
- [x] 13.4 `integration/tests/session.test.ts` — refresh (valid/expired/revoked), logout + subsequent refresh fails, me, validate
- [x] 13.5 `integration/tests/mgmt-user.test.ts` — create, createTestUser, load, loadByUserId, search (all filter types), update, patch, updateEmail, setActivePassword, delete (idempotent), deleteByUserId, deleteAllTestUsers, addTenant, generateMagicLink, generateEmbeddedLink
- [x] 13.6 `integration/tests/mgmt-tenant.test.ts` — tenant loadAll
- [x] 13.7 `integration/tests/lifecycle.test.ts` — GET /health, POST /emulator/reset clears state, CORS headers present, OPTIONS preflight
- [x] 13.8 `integration/tests/cross-flow.test.ts`:
  - create user → set password → signin → validateSession (full backend lifecycle)
  - createTestUser → generateMagicLink → verify → session valid
  - signup → update customAttributes → load → verify attributes persisted
  - signin → logout → refresh fails
  - signin → refresh → new session JWT passes validateSession
  - saml.start (email) → extract code → saml.exchange → valid session
  - reset clears all state, seed data restored
- [x] 13.9 Add `@descope/node-sdk` SDK-level test in `cross-flow.test.ts`: point the real node SDK at the emulator and call `sdk.validateSession(sessionJwt)` — this is the highest-fidelity consumer test

## 14. Parity Test Harness

- [x] 14.1 Set up `tests/parity/runner.rs` — HTTP client wrapper that sends to both live Descope and local emulator, performs structural diff, uses `scopeguard` `defer!` for cleanup
- [x] 14.2 Implement parity scenarios for all password endpoints (signup, signin, replace, reset, update)
- [x] 14.3 Implement parity scenarios for magic link endpoints (signin/email, verify, update/email)
- [x] 14.4 Implement parity scenarios for SAML endpoints (start, exchange)
- [x] 14.5 Implement parity scenarios for session endpoints (refresh, logout, me, validate)
- [x] 14.6 Implement parity scenarios for all user management endpoints
- [x] 14.7 Implement parity scenarios for tenant, JWKS, lifecycle endpoints
- [x] 14.8 Verify parity tests skip gracefully when `DESCOPE_PARITY_PROJECT_ID` is unset
- [x] 14.9 Run full parity suite against live Descope test project and resolve all diffs

## 15. CI and Coverage

- [x] 15.1 Create `Makefile` or `justfile` with targets: `test` (Rust units only), `test-integration` (Vitest), `test-parity` (Rust parity feature), `coverage`, `lint`
- [x] 15.2 Configure `cargo llvm-cov --all-features --fail-under-coverage 95` gate for Rust unit tests in CI
- [x] 15.3 Configure Vitest CI step: build the release binary first, then run `vitest run` in `integration/`
- [x] 15.4 Add `clippy` lint check (`cargo clippy -- -D warnings`) to CI
- [x] 15.5 Document usage in `README.md`: startup, seed file format, env vars, Vitest integration test setup, parity test setup
