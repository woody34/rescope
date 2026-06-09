## Context

The Descope Emulator is a greenfield Rust project that replicates the Descope REST API for local development and testing. There is no existing codebase to migrate. The primary reference documents are `prd-descope-emulator.md` and `architecture-diagram.md` in the project root.

The emulator must faithfully replicate the HTTP contract consumed by Descope's client SDKs (`@descope/node-sdk`, `@descope/angular-sdk`, `@descope/nextjs-sdk`, `@descope/react-native-sdk`) so that any application using those SDKs can point its base URL at the emulator and work transparently offline.

**Two-layer testing strategy**: Rust unit tests cover internal logic (stores, JWT engine). Vitest integration tests cover the full HTTP API externally using TypeScript — same language and tooling as the real consumer apps, with the option to use actual Descope TS SDKs as clients.

## Goals / Non-Goals

**Goals:**

- Implement the full Descope REST API surface used by the target SDKs
- All state in-memory; ephemeral by default, resettable via `POST /emulator/reset`
- RS256 JWT signing and JWKS endpoint compatible with `@descope/node-sdk`'s `validateSession`
- 95% Rust unit test coverage (line + branch) enforced in CI via `cargo-llvm-cov`
- External HTTP API integration tests written in TypeScript with **Vitest** — mirrors real consumer SDK usage
- TDD throughout: specs → failing tests → implementation
- Optional parity test harness (`--features parity`) for structural diffing against live Descope
- Single-crate (`[lib] + [[bin]]`) structure for easy future integration into an Nx monorepo

**Non-Goals:**

- Persistence across restarts (seed file is the only startup state mechanism)
- Descope admin console UI or Flow screens
- OAuth provider login, TOTP, WebAuthn, enchanted links
- Rate limiting emulation
- Production deployment

## Decisions

### 1. HTTP Framework: Axum + Tokio

**Decision**: Use `axum` (v0.7+) with `tokio` runtime.

**Rationale**: Axum is built on Tower + Tokio. Its extractor pattern (`Json<T>`, `State<S>`, `TypedHeader`) eliminates boilerplate across the ~35 endpoints. `tower-http` provides CORS middleware. Router nesting maps cleanly onto the Descope URL structure.

**Alternatives considered**: `actix-web` (own runtime, less ergonomic), raw `hyper` (too low-level).

---

### 2. Shared State: `tokio::sync::RwLock<T>` per store

**Decision**: Each store is an `Arc<tokio::sync::RwLock<Store>>` injected via Axum `State`. `tokio`'s `RwLock` is mandatory — `std::sync::RwLock` guards are not `Send` across `.await` points and will not compile in async handlers.

```rust
#[derive(Clone)]
pub struct EmulatorState {
    pub users:   Arc<tokio::sync::RwLock<UserStore>>,
    pub tenants: Arc<tokio::sync::RwLock<TenantStore>>,
    pub tokens:  Arc<tokio::sync::RwLock<TokenStore>>,
    pub revoked: Arc<tokio::sync::RwLock<RevocationStore>>,
    pub keys:    Arc<KeyManager>,   // immutable after startup, no lock needed
    pub config:  Arc<EmulatorConfig>,
}
```

Reads vastly outnumber writes. `RwLock` allows concurrent reads while keeping writes exclusive. Each store is independently lockable.

---

### 3. Error Handling: `thiserror` typed enum

**Decision**: A single `EmulatorError` enum defined with `thiserror`, implementing Axum's `IntoResponse`. Every handler returns `Result<Json<T>, EmulatorError>`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum EmulatorError {
    #[error("user not found")]
    UserNotFound,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid token")]
    InvalidToken,
    #[error("token expired")]
    TokenExpired,
    #[error("tenant not found")]
    TenantNotFound,
    #[error("user is not configured for SSO")]
    NotSsoUser,
    #[error("user is not a test user")]
    NotTestUser,
    #[error("internal error: {0}")]
    Internal(String),
}
```

Each variant maps to a specific Descope error JSON shape and HTTP status code in `IntoResponse`. This gives us a single place to audit all error paths, which is critical for the 95% coverage target.

---

### 4. JWT: `jsonwebtoken` v9 only — no separate `rsa` crate

**Decision**: Use `jsonwebtoken` v9+ exclusively. Use the `rsa` crate only for key pair generation. Bridge via PKCS8 PEM.

**Rationale**: `jsonwebtoken` v9 introduced a `jwk` module for JWKS serialization. Its `EncodingKey::from_rsa_pem()` accepts PKCS8 PEM. The `rsa` crate generates the key pair and exports to PEM; `jsonwebtoken` takes it from there. No other RSA crate is needed.

**Key lifecycle**:

```
Startup:
  RsaPrivateKey::new(&mut OsRng, 2048)       // rsa crate
    → private_key.to_pkcs8_pem()              // rsa + pkcs8 features
    → EncodingKey::from_rsa_pem(pem_bytes)    // jsonwebtoken
    → DecodingKey::from_rsa_pem(pub_pem)      // jsonwebtoken
    → JWK serialized from public key components (n, e)  // manual + base64url

kid: hex(sha256(der_encoded_public_key))[..16]
```

**Cargo features required on `rsa`**: `pkcs8`, `pem`.
**Cargo features required on `jsonwebtoken`**: `use_pem` + one of `aws_lc_rs` or `ring` as crypto backend. Use `ring` for broader platform compatibility (no native libs required).

**Alternatives considered**: Using `openssl` crate — rejected (requires system OpenSSL, poor CI portability). Using `jose-jwt` — less mature.

---

### 5. Password Hashing: `bcrypt` with `spawn_blocking`

**Decision**: Use the `bcrypt` crate, cost factor 10. Every call site that hashes or verifies a password **must** use `tokio::task::spawn_blocking` to avoid blocking the Tokio executor. This applies to: `password/signup`, `password/signin`, `password/replace`, `mgmt/user/password/set/active`, and seed loader.

**Alternatives considered**: `argon2` — more modern but not implied by the PRD, heavier. Adequate for a dev tool.

---

### 6. Token Store: Capped at 10,000 entries with eviction logging

**Decision**: The token store (magic links, SAML codes, embedded tokens, reset tokens) is capped at **10,000 entries**. When the cap is reached, the oldest entries (by `created_at`) are evicted to make room. Evictions are logged at `WARN` level with the token type and count. Unverified tokens do not need a TTL — the cap prevents unbounded growth.

**Rationale**: A dev tool's process lifetime is short. Eviction is simpler than a background TTL cleanup task (which would require a Tokio `interval` and coordinated locking). Logging evictions surfaces the issue without crashing.

---

### 7. Project Structure: Single `[lib] + [[bin]]` crate

**Decision**: One `Cargo.toml` with `[lib]` (all emulator logic) and `[[bin]]` (`main.rs` thin entry point). This is the correct pattern for future Nx monorepo integration — Nx Rust plugins (`@monodon/rust`) treat each `Cargo.toml` as a project, so the whole crate drops into `libs/descope-emulator/` or `apps/descope-emulator/` in the monorepo with no restructuring needed. Integration tests import from the lib crate directly.

```
Cargo.toml          # [lib] + [[bin]]
src/
  lib.rs            # pub mod declarations
  main.rs           # thin: config → init → server::start()
  server.rs
  state.rs
  config.rs
  error.rs
  types.rs
  routes/
    auth/
      password.rs
      magic_link.rs
      saml.rs
      otp.rs
      session.rs
    mgmt/
      user.rs
      tenant.rs
    jwks.rs
    emulator.rs
  store/
    user_store.rs
    tenant_store.rs
    token_store.rs
    revocation_store.rs
  jwt/
    key_manager.rs
    token_generator.rs
    token_validator.rs
  seed.rs
  cookies.rs

tests/                        # integration tests, import from lib
  password_test.rs
  magic_link_test.rs
  saml_test.rs
  session_test.rs
  user_mgmt_test.rs
  jwt_test.rs
  seed_test.rs
  lifecycle_test.rs

tests/parity/                 # --features parity only
  runner.rs
  password_parity.rs
  magic_link_parity.rs
  saml_parity.rs
  session_parity.rs
  user_mgmt_parity.rs
```

---

### 8. CORS: Echo Origin for credentials compatibility

**Decision**: Use `tower-http`'s `CorsLayer` with `AllowOrigin::mirror_request()`. Do not use `allow_any_origin()` — the W3C CORS spec forbids `Allow-Origin: *` with `Allow-Credentials: true`. Mirroring the `Origin` header achieves equivalent permissiveness while satisfying the browser constraint.

---

### 9. Coverage: 95% line + branch via `cargo-llvm-cov`

**Decision**: CI enforces `cargo llvm-cov --all-features --fail-under-coverage 95`. The 95% threshold (not 100%) accounts for `serde`-derived unreachable branches and provably-infallible unwraps without requiring `#[coverage(off)]` annotations throughout the codebase. All business logic paths must be tested.

---

### 10. SAML `saml.start` — dual tenant resolution

**Decision**: The `tenant` field in `saml.start` is resolved as follows:

1. If the value contains `@`, treat as an email → look up user → find their SAML/OIDC tenant
2. Otherwise, treat as a tenant ID → look up tenant directly in the tenant store

Both paths require the resolved tenant to have `authType: "saml"` or `"oidc"`. This dual-resolution is confirmed by parity testing.

---

### 11. Parity test cleanup: `scopeguard`

**Decision**: Use the `scopeguard` crate's `defer!` macro to register Descope cleanup (user deletion) at the start of each parity scenario. `defer!` runs even on panic, guaranteeing cleanup regardless of test outcome.

---

### 12. Integration Tests: Vitest (TypeScript) for HTTP API, Rust for units

**Decision**: Two separate test layers:

**Rust unit tests** (`src/**/*.rs` `#[cfg(test)]` blocks and `tests/*.rs`): Cover stores, JWT engine, error types, seed loading. Test pure logic without HTTP.

**Vitest integration tests** (`integration/`): TypeScript test suite that starts the Rust binary as a child process, waits for `/health`, then exercises the full HTTP API via `fetch` or the actual Descope SDK clients. `globalSetup` spawns the server on a fixed test port (4501 by default, overridable via `DESCOPE_EMULATOR_TEST_PORT`). `globalTeardown` kills it.

```
integration/
  vitest.config.ts
  setup/
    global-setup.ts      # spawn emulator, poll /health
    global-teardown.ts   # kill process
  helpers/
    client.ts            # fetch wrapper with base URL
    sdk-client.ts        # optional: Descope SDK client pointed at emulator
  tests/
    password.test.ts
    magic-link.test.ts
    saml.test.ts
    session.test.ts
    mgmt-user.test.ts
    mgmt-tenant.test.ts
    lifecycle.test.ts
    cross-flow.test.ts
```

**Rationale**: Vitest tests the API the same way real consumer apps do. TypeScript allows using the actual `@descope/node-sdk` as a client — the highest-fidelity test possible. Rust unit tests stay focused on business logic.

For Rust server construction: `server::build_router(state)` is separate from `main`, allowing `--test` binary builds where additional tooling can bind the server.

## Risks / Trade-offs

- **`ring` vs `aws-lc-rs`**: `ring` requires no native libs and works everywhere. If `jsonwebtoken`'s crypto backend is changed in a future version, this may need revisiting.
- **bcrypt blocking**: Every `spawn_blocking` adds ~1 Tokio thread pool task. For test suites creating hundreds of users, bcrypt is the dominant cost. Acceptable for a dev tool.
- **Token store cap**: A test suite that generates > 10,000 unverified magic links before consuming them will see evictions. This is a pathological case. Log at WARN; document the cap.
- **PEM bridge between `rsa` and `jsonwebtoken`**: Serializing to PEM and back introduces a string allocation. Negligible for startup-time key loading. Not on the hot path.
- **CORS mirroring in CI**: When no `Origin` header is present (e.g., curl from test), no CORS headers are returned. This is correct behavior.

## Resolved Decisions

- **`POST /emulator/reset` re-applies seed**: Confirmed — reset clears all state and re-applies seed data. If no seed file, resets to empty state.
- **Token store cap**: Confirmed — cap at 10,000 entries, evict oldest, log at WARN. Good call for a dev tool.
- **Parity: `POST /v1/auth/validate`**: Required — implement and include in the parity test suite regardless of whether the node SDK calls it directly. Future consumers may call it directly.
