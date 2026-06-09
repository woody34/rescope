## Context

Rescope is a Rust/Axum emulator for the Descope authentication API. It already uses `tracing` and `tracing-subscriber` (in `Cargo.toml` and `main.rs`), and `tower-http` for CORS and static file serving. The current startup output is a single `tracing::info!` line. There is no HTTP request logging, no console output for generated auth codes, and the README documents incorrect env var names.

Current dependencies already present:
- `tracing = "0.1"` ✅
- `tracing-subscriber = "0.3"` with `env-filter` ✅
- `tower-http = "0.5"` with `cors`, `fs` features ✅ (need to add `trace` feature)

## Goals / Non-Goals

**Goals:**
- Make every HTTP request visible in the terminal with method, path, status, and timing
- Print a useful startup banner with current configuration
- Print OTP codes and magic link tokens to stdout when they are generated
- Fix the README env var naming to match the code
- Warn on obviously broken configs at startup (missing seed file, etc.)

**Non-Goals:**
- Structured JSON logging (overkill for a dev tool — plain colorized text is more readable)
- Metrics collection or OpenTelemetry integration
- Log file rotation or persistence
- Changing any actual env var names in `config.rs` (the code is correct; the README is wrong)
- Logging request/response bodies (security concern — JWTs in bodies)

## Decisions

### 1. Use `tower-http::trace::TraceLayer` for HTTP logging

**Decision**: Add the `trace` feature to the existing `tower-http` dependency, and layer `TraceLayer` onto the Axum router.

**Alternatives considered**:
- Custom middleware: More control but more code to maintain. `TraceLayer` is the Axum ecosystem standard.
- `tower-http::logging`: Doesn't exist as a separate feature; `TraceLayer` is the canonical approach.

**Rationale**: `TraceLayer` integrates directly with the `tracing` ecosystem already in use. It provides method, path, status, and latency out of the box. Minimal code change (~5 lines in `server.rs`).

### 2. Print auth codes with `tracing::info!` (not `println!`)

**Decision**: Use `tracing::info!` with a distinctive target (e.g., `target: "rescope::auth_code"`) for OTP and magic link output.

**Alternatives considered**:
- `println!`: Simpler, but bypasses the tracing subscriber. Can't be filtered or suppressed in tests.
- Separate log level: Using `warn!` to stand out — but these aren't warnings, they're informational.

**Rationale**: Keeps all output flowing through one system. Can be filtered via `RUST_LOG` if someone wants to suppress codes. The tracing subscriber's `fmt` layer will emit them to stdout with timestamps.

### 3. Startup banner as formatted `info!` lines (not raw `println!`)

**Decision**: Use multiple `tracing::info!` calls with structured fields for the startup banner. Add a visual separator line.

**Rationale**: Consistent with the rest of the output. Structured fields mean the banner data is filterable. No mixing of `println!` and `tracing` output (which can interleave unpredictably with async).

### 4. README is wrong, code is right

**Decision**: Update the README env var table to match what `config.rs` actually reads. Do not change the code.

**Rationale**: The `DESCOPE_*` prefix convention makes more sense for discoverability (users coming from Descope know to look for `DESCOPE_` vars). The `EMULATOR_` prefix in the README was likely an earlier naming scheme.

## Risks / Trade-offs

**[Request logging adds noise]** → Mitigation: Use `RUST_LOG=rescope=info,tower_http=info` as default. Users can set `RUST_LOG=rescope=info,tower_http=off` to suppress request logs. Document this in the README.

**[Auth codes in logs are a "security" concern]** → Mitigation: This is a local dev emulator. The codes are not real. If someone runs this in a shared environment, they're misusing the tool. Add a note in the startup banner: "⚠️ Do not use in production."

**[Startup validation failures]** → Mitigation: Use `tracing::warn!` for non-fatal issues (missing seed file). Use `anyhow` errors for fatal issues (port already in use is already handled by `TcpListener::bind`).
