## Why

Rescope currently starts silently, logs nothing to stdout, and requires developers to either inspect the admin UI or call escape-hatch APIs to retrieve OTP codes and magic link tokens. For a tool whose primary value proposition is developer productivity, this is a significant gap — developers spend unnecessary time wondering "did my request arrive?", "what went wrong?", and "what's the OTP code?"

Additionally, the README documents env var names (`EMULATOR_PORT`, `EMULATOR_PROJECT_ID`) that differ from what the code actually reads (`DESCOPE_EMULATOR_PORT`, `DESCOPE_PROJECT_ID`). This will be the first bug every new user files.

## What Changes

- **Startup banner** — On boot, print a colorized summary: port, project ID, management key, loaded seed file, number of seeded users/tenants, and emulator-only endpoint hints.
- **Request logging** — Add `tower-http` tracing layer with colorized HTTP method, path, status code, and response time on every request. Use `tracing` + `tracing-subscriber` with `fmt` layer.
- **Console OTP/magic link output** — When an OTP code or magic link token is generated, print it to stdout (e.g., `📧 OTP for alice@test.com: 123456`). Developers shouldn't need to leave their terminal.
- **Env var naming reconciliation** — Update `README.md` to match what `config.rs` actually reads. The code is the source of truth; the README is wrong.
- **Startup config validation** — Warn on startup if seed file is configured but doesn't exist, or if port is already in use, instead of failing silently or with a cryptic error.

## Capabilities

### New Capabilities
- `request-logging`: Structured HTTP request/response logging with timing via `tracing` and `tower-http`
- `startup-banner`: Colorized startup output with configuration summary and helpful hints
- `console-auth-codes`: Print OTP codes and magic link tokens to stdout when generated

### Modified Capabilities
_(none — no existing spec-level requirements are changing)_

## Impact

- **Crate dependencies**: Add `tracing`, `tracing-subscriber`, `tower-http` (tracing feature). These are already standard in the Axum ecosystem and may be partially present.
- **Files modified**: `main.rs` (startup banner), `server.rs` (tracing layer), `routes/auth/otp.rs` (console output), `routes/auth/magic_link.rs` (console output), `routes/emulator/mod.rs` (console output for escape-hatch OTP retrieval), `README.md` (env var fix), `config.rs` (validation).
- **No API surface changes** — All modifications are server-side observability. No request/response formats change.
- **No breaking changes.**
