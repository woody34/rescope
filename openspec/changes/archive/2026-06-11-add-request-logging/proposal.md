# Add Request Logging

## Why

The emulator's core debugging value is showing developers what their SDK integration actually sends — yet today, request-level visibility is effectively absent. A `TraceLayer` exists in `server.rs`, but it emits under the `tower_http::trace` target, which the default env filter (`rescope=info` in `main.rs`) silently drops. Even when manually enabled via `RUST_LOG`, its output is verbose, uncategorized, and shows no request bodies — the part that matters most when an SDK call misbehaves.

## What Changes

- Add a custom Axum request-logging middleware that emits **one readable log line per request, on completion**, under the `rescope::http` target — visible out of the box with the default env filter.
- Each line includes a **category tag**, method, path, status, and latency: `[API] POST /v1/auth/otp/verify/email 401 2ms`.
- Requests are categorized by path prefix: `[API]` (Descope API surface), `[EMU]` (emulator escape hatches), `[DOCS]` (OpenAPI/Swagger), `[UI]` (admin UI assets).
- Category determines log level: `[API]`/`[EMU]` at INFO (visible by default), `[UI]`/`[DOCS]` at DEBUG (revealed via `RUST_LOG=rescope=debug`). Exception: `/health` logs at DEBUG because test harnesses poll it.
- **Truncated request bodies** are appended to `[API]`/`[EMU]` lines (single-line, capped at 256 chars, text-like content types only). **Response bodies are logged only for error statuses (≥ 400)**, same truncation.
- No masking of credentials/OTP codes in bodies — intentional for a dev-only emulator (the binary already warns "Do not use in production"). `Authorization` headers are never logged.
- Env overrides: `RESCOPE_LOG_BODY=0` disables body logging; `RESCOPE_LOG_BODY_MAX=<n>` adjusts the truncation cap.
- Remove the existing muted `TraceLayer` from `server.rs` (superseded).

## Capabilities

### New Capabilities

- `request-logging`: Per-request terminal logging for the emulator HTTP server — categorization, log levels, line format, body capture/truncation rules, and env-var configuration.

### Modified Capabilities

_None — no existing specs in `openspec/specs/`._

## Impact

- **`apps/api/src/request_log.rs`** (new): middleware, category classification, body truncation logic.
- **`apps/api/src/server.rs`**: replace `TraceLayer` with the new middleware layer (attached after the UI merge so `[UI]`/`[DOCS]` traffic is covered — the old `TraceLayer` never saw those routes).
- **`apps/api/src/lib.rs`**: register the new module.
- **`apps/api/src/main.rs`**: the env filter now uses `rescope=info` only as the fallback when `RUST_LOG` is unset. The previous `add_directive` approach silently overrode a user-supplied `rescope=debug`, which would have made the DEBUG categories unreachable (discovered during smoke testing).
- **Dependencies**: `futures-util` added as a direct dependency (already in the tree transitively via axum, same feature set — no new compilation) for the over-cap body replay; `tower-http`'s `trace` feature removed.
- **Behavioral**: stdout log volume increases (one INFO line per API request). No HTTP behavior changes; request bodies are buffered up to a small cap before handlers run, which is transparent to handlers.
- **Tests/CI**: new unit tests in `apps/api` (classification, truncation, content-type gating); coverage floor (95%) applies.
