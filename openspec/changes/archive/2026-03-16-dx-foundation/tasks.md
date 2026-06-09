## 1. Request Logging

- [x] 1.1 Add `trace` feature to `tower-http` in `apps/api/Cargo.toml` (change `features = ["cors", "fs"]` to `features = ["cors", "fs", "trace"]`)
- [x] 1.2 Add `TraceLayer` to the Axum router in `server.rs` — layer it above the CORS layer so all requests are logged with method, path, status, and latency
- [x] 1.3 Verify request logging works: start the emulator, send a request, confirm stdout shows the method, path, status code, and timing

## 2. Startup Banner

- [x] 2.1 Expand the startup output in `main.rs` to print: port, project ID, management key (masked), seed file path (if configured), and a "ready" line with the full base URL
- [x] 2.2 After seed loading, log the count of seeded users and tenants (e.g., `Loaded seed: 3 users, 2 tenants`)
- [x] 2.3 Add a startup warning if `DESCOPE_EMULATOR_SEED_FILE` is set but the file does not exist (use `tracing::warn!`, do not crash)
- [x] 2.4 Verify startup banner: start the emulator with a seed file and confirm all expected lines appear in stdout

## 3. Console Auth Codes

- [x] 3.1 Add `tracing::info!` calls in `routes/auth/otp.rs` for every OTP generation (signup, signin, signup-in, update-phone) — log the login ID and the 6-digit code
- [x] 3.2 Add `tracing::info!` calls in `routes/auth/magic_link.rs` for every magic link token generation — log the login ID and the token
- [x] 3.3 Add `tracing::info!` calls in the management test-user endpoints (`routes/mgmt/user.rs`) for OTP, magic link, and enchanted link generation — log the login ID and the code/token
- [x] 3.4 Verify console auth codes: call an OTP signup endpoint and confirm the code appears in stdout without needing to call `/emulator/otp/:id`

## 4. README Env Var Fix

- [x] 4.1 Update the Environment Variables table in `README.md` to match the actual env var names read by `config.rs`: `DESCOPE_EMULATOR_PORT`, `DESCOPE_PROJECT_ID`, `DESCOPE_MANAGEMENT_KEY`, `DESCOPE_EMULATOR_SESSION_TTL`, `DESCOPE_EMULATOR_REFRESH_TTL`, `DESCOPE_EMULATOR_SEED_FILE`, `DESCOPE_EMULATOR_KEY_FILE`
- [x] 4.2 Update any other README references that use the old env var names (Quick Start section, seed file examples, etc.)
- [x] 4.3 Verify README accuracy: cross-check every `env::var(...)` call in `config.rs` against the updated README table

## 5. Verification

- [x] 5.1 Run `cargo test --lib` (Rust unit tests) and confirm all pass
- [x] 5.2 Run `npm run test:api` (API integration tests) and confirm all pass — these start the emulator and will exercise the new logging
- [ ] 5.3 Manual smoke test: start the emulator, perform a full OTP flow (signup → get code from stdout → verify), confirm no regressions
