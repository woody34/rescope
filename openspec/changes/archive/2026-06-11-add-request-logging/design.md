# Design — Add Request Logging

## Context

The emulator (`apps/api`, Rust/Axum) serves four kinds of traffic from one router: the Descope API surface (`/v1/*`, `/v2/keys/*`, `/.well-known/*`, `/oauth/*`), emulator escape hatches (`/emulator/*`, `/health`), docs (`/docs`, `/openapi.json`), and the embedded admin UI (everything else). Logging today is `tracing` + `tracing-subscriber` with a default env filter of `rescope=info` (`main.rs`). A `tower_http::TraceLayer` is attached in `server.rs` but emits under the `tower_http::trace` target, so its output is invisible by default — and even when enabled it is verbose, uncategorized, and body-less.

Constraints:

- Repo convention: `rescope-core` is framework-agnostic; HTTP/Axum concerns live in `apps/api`. This feature is pure HTTP plumbing → `apps/api` only.
- Output must be readable in a terminal: one line per request, truncated bodies.
- The emulator is explicitly dev-only ("Do not use in production"), which changes the secrets-in-logs calculus: seeing credentials/OTP codes in request bodies is a debugging feature, not a leak.

## Goals / Non-Goals

**Goals:**

- Every HTTP request produces exactly one log line, on completion, visible by default for API/emulator traffic with zero `RUST_LOG` configuration.
- Lines are categorized (`[API]`, `[EMU]`, `[DOCS]`, `[UI]`) and include method, path, status, latency, and (where applicable) truncated bodies.
- Noise control via log levels, not omission — everything is logged, defaults show what matters.

**Non-Goals:**

- No queryable request store, `/emulator/requests` endpoint, or admin-UI requests page (explicitly deferred; would be a separate change).
- No emulation of Descope's audit API (`/v1/mgmt/audit/search`) — that is event-shaped, not request-shaped.
- No log files, rotation, or structured JSON output — stdout via `tracing` only.
- No masking of secrets inside bodies (dev-only tool; decided with user).

## Decisions

### D1: Custom `middleware::from_fn` instead of configuring `TraceLayer`

A single `axum::middleware::from_fn` (`apps/api/src/request_log.rs`) replaces the `TraceLayer` in `server.rs`.

- *Why not keep `TraceLayer` with custom `MakeSpan`/`OnResponse`?* Its callbacks don't get easy access to buffered request/response bodies, and per-category log levels + custom targets fight the span model. The closures end up bigger than a plain middleware.
- The middleware logs under the **`rescope::http`** target, which the existing default filter (`rescope=info`) already enables — this is what makes it work out of the box.
- `tower-http`'s `trace` feature can be dropped from `Cargo.toml` (the `cors` and `fs` features remain in use).
- **Amendment (found in smoke testing):** `main.rs` previously force-added the `rescope=info` directive on top of `RUST_LOG`, which silently overrode a user-supplied `rescope=debug` and made the DEBUG categories unreachable. It now uses `EnvFilter::try_from_default_env()` with `rescope=info` as the unset-fallback, so `RUST_LOG` governs fully when present (this also makes the integration harness's `RUST_LOG=error` actually silence the emulator).
- The layer is attached **after** the UI merge at the end of `build_router`, not where `TraceLayer` sat — the old position never saw `[UI]` traffic at all.

### D2: Category from path prefix, level from category

Classification is a pure function `fn classify(path: &str) -> Category` (unit-testable, no HTTP types):

| Prefix | Category | Level |
|---|---|---|
| `/v1/`, `/v2/keys/`, `/.well-known/`, `/oauth/` | `[API]` | INFO |
| `/emulator/` | `[EMU]` | INFO |
| `/health` | `[EMU]` | DEBUG (polled by harnesses) |
| `/docs`, `/openapi.json` | `[DOCS]` | DEBUG |
| anything else | `[UI]` | DEBUG |

`tracing` requires the level to be const per `event!` call site; the middleware branches (`info!` vs `debug!`) on the computed level rather than passing it dynamically.

### D3: Body capture — request always (for API/EMU), response only on errors

- **Request body**: buffered with `axum::body::to_bytes` limited to `cap + slack` (slack ≈ 1 KiB so we can tell "exactly cap" from "truncated"), then the request is reconstructed with `Request::from_parts` before calling the inner service. Only for `[API]`/`[EMU]` requests whose `Content-Type` is JSON / form-urlencoded / `text/*`; binary and multipart bodies are never buffered or logged.
- **Response body**: captured only when status ≥ 400, same content-type and cap rules. Success responses pass through untouched — no buffering cost on the hot path, and happy-path lines stay short.
- **Formatting**: bodies are collapsed to one line (newlines/runs of whitespace squashed) and truncated at the cap with a trailing `…`. No JSON re-serialization — raw text squashing is cheaper and never fails on invalid JSON.
- **Bound on memory**: buffering is capped — chunks are read from the body stream only until the cap is exceeded, so memory held is at most the cap plus one in-flight chunk. *(Implemented via `into_data_stream()` + a chunk loop rather than `to_bytes`, which fails rather than partially reads at a limit. When the body continues past the cap, the buffered chunks are replayed and chained onto the unread remainder of the stream — `futures_util::StreamExt::chain` — so the handler always receives the full, byte-identical body.)*
- `Authorization` headers (and headers generally) are not logged.

### D4: Configuration via env vars, read once

- `RESCOPE_LOG_BODY=0` → disable all body logging.
- `RESCOPE_LOG_BODY_MAX=<n>` → truncation cap in bytes (default 256).

Parsed once at router build time (in `build_router` or a `OnceLock`), not per request. These piggyback on the existing `EmulatorConfig::from_env` pattern if convenient, but they are logging-only knobs and may stay local to `request_log.rs` to avoid widening `EmulatorConfig`.

### D5: Line format

```
INFO  rescope::http: [API] POST /v1/auth/otp/verify/email 401 2ms body={"loginId":"x@y.com","code":"00000"} resp={"errorCode":"E061102"…
DEBUG rescope::http: [UI] GET /assets/index-Bx2.js 200 0ms
```

`method path status latency` always; ` body=…` when a request body was captured; ` resp=…` when status ≥ 400 and a response body was captured. Query strings are included in the path as received.

## Risks / Trade-offs

- **[Buffering changes request flow]** Handlers now receive a replayed body for text-like API requests. → Mitigated by exact reconstruction from the buffered bytes; integration suites (`npm run test:api`, SDK suites) exercise every route and would surface regressions immediately.
- **[Large text bodies]** A multi-MB JSON body is read only up to `cap + slack`; the remainder streams through unlogged. → Acceptable: log shows `body=…<truncated>`; handlers are unaffected.
- **[Secrets in terminal/CI logs]** Passwords and OTP codes appear in plaintext. → Accepted deliberately (dev-only emulator, explicit startup warning). `RESCOPE_LOG_BODY=0` exists for CI environments that archive logs.
- **[Log volume in integration runs]** Hundreds of INFO lines during `test:api`. → Tests run the binary with default filter; volume is bounded and aids debugging failures. Can be silenced with `RUST_LOG=rescope=warn` if it ever bothers CI.
- **[Latency reporting honesty]** Latency is measured around the inner service call only (excludes body buffering time). → Negligible at emulator scale; noted so nobody chases phantom discrepancies.

## Open Questions

_None — scope and behavior were settled in explore mode with the user._
