# rescope (API)

Rust/Axum HTTP server that emulates the Descope authentication API. Core binary of the Rescope monorepo.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Build (debug) | `cargo build` |
| Build (release) | `cargo build --release` |
| Unit tests | `cargo test --lib` |
| Lint | `cargo clippy -- -D warnings` |
| Format check | `cargo fmt --check` |
| Coverage | `cargo llvm-cov --lib --fail-under-coverage 95` |

## Conventions

- Business logic lives in `libs/rescope-core`; this crate handles HTTP routing, server setup, and Axum integration
- Routes organized under `src/routes/` by domain: `auth/`, `mgmt/`, `emulator/`, `jwks.rs`
- Management endpoints require auth via `mgmt_auth.rs` extractor
- State is in-memory (`src/state.rs`), reset via `/emulator/reset`
- UI assets embedded at compile time via `rust-embed` (`src/embedded_ui.rs`)
- Seed file support via `src/seed.rs` (loaded from DESCOPE_EMULATOR_SEED_FILE)
- JWT signing uses RSA keys from `rescope-core`; JWKS served at `/.well-known/jwks.json`
- `dev-ui` feature flag: enables dev-mode UI asset serving
- Tests use `axum-test` crate for in-process HTTP testing
