# Tasks — Add Request Logging

## 1. Core module

- [x] 1.1 Create `apps/api/src/request_log.rs` with `Category` enum and pure `classify(path: &str) -> Category` per the prefix table in design.md (D2), plus unit tests covering every prefix class and the `/health` exception
- [x] 1.2 Add body-formatting helpers: content-type gate (JSON / form-urlencoded / `text/*`), single-line collapse (squash newlines/whitespace runs), and truncation with `…` marker; unit tests for each (long body, multiline body, binary content-type, exact-cap boundary)
- [x] 1.3 Add config struct read once at startup (`RESCOPE_LOG_BODY`, `RESCOPE_LOG_BODY_MAX`, default cap 256) with unit tests for parsing/defaults

## 2. Middleware

- [x] 2.1 Implement the `middleware::from_fn` handler: buffer request body up to cap + slack for `[API]`/`[EMU]` text-like requests, reconstruct the request, time the inner call, capture response body only when status ≥ 400, and emit one `info!`/`debug!` event under target `rescope::http` with `[CAT] METHOD PATH STATUS LATENCYms body=… resp=…`
- [x] 2.2 Handle the over-cap path safely: bodies larger than the buffer bound must still reach the handler byte-identical (manual frame loop or equivalent — see design.md D3 note), with the log showing truncated/omitted body
- [x] 2.3 Register the module in `apps/api/src/lib.rs`

## 3. Wiring

- [x] 3.1 In `apps/api/src/server.rs`, replace the `TraceLayer` block (lines ~590–594) with `.layer(middleware::from_fn(request_log::log_request))` and remove now-unused tower-http trace imports
- [x] 3.2 Drop the `trace` feature from `tower-http` in `apps/api/Cargo.toml` (keep `cors`, `fs`)

## 4. Behavior tests (axum-test)

- [x] 4.1 In-process test: API request with JSON body → handler receives full body and responds normally (spec: "Body capture does not alter request handling")
- [x] 4.2 In-process test: request larger than the cap still processed identically (status + response match a no-logging baseline)
- [x] 4.3 Test that no log output contains an `Authorization` header value (e.g., capture tracing output or assert the line-builder never receives headers)

## 5. Verification & docs

- [x] 5.1 Run `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test --lib`, and `npm run test:api`; confirm coverage stays ≥ 95% via `npm run api:coverage` — clippy/fmt/lib (358 tests)/test:api (252 tests) all pass. Caveat: `npm run api:coverage` (coverage-all.sh) is broken repo-wide pre-existing (stale `descope-emulator` binary name → 0% report, exit 0); flagged for a separate fix. `cargo llvm-cov --lib` measures request_log.rs at 96.84% lines.
- [x] 5.2 Manual smoke (binary on scratch port; OTP signup → fetch code → failed verify; plus `/`, `/openapi.json`, `/health`): default filter shows `[API]`/`[EMU]` lines only with `body=`/`resp=`; `RUST_LOG=rescope=debug` reveals `[UI]`/`[DOCS]`/`/health` (required the main.rs env-filter fallback fix — see design.md D1 amendment)
- [x] 5.3 Document the log format and `RESCOPE_LOG_BODY*` env vars (README / docs site page that covers emulator configuration)
