# rescope-core

Framework-agnostic Rust library containing all Rescope business logic: user management, tenant management, auth flows (password, OTP, magic link, SAML, OIDC), JWT/JWKS, roles, permissions, access keys, connectors, and custom attributes.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Test | `cargo test -p rescope-core` |
| Test (no native crypto) | `cargo test -p rescope-core --no-default-features` |

## Conventions

- WASM-compatible by default: `default-features = false` skips ring/jsonwebtoken/rcgen
- Feature flags: `native-crypto` (JWT signing, SAML certs), `axum` (IntoResponse for errors)
- `apps/api` enables both features; `apps/rescope-wasm` uses defaults only
- All state management is in-memory with no external dependencies
- Error types defined via `thiserror`; Axum integration is behind the `axum` feature gate
- RSA key generation uses the `rsa` crate (pure Rust, WASM-compatible)