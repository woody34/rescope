## Context

Rescope currently runs as a Rust server (axum + tokio) with ~60 HTTP endpoints. All state is in-memory (`Arc<RwLock<Store>>`). The business logic (stores, JWT signing, auth flows) is interleaved with HTTP routing in `apps/api/src/`. The only outbound network call is the connector invoker (`reqwest`).

The goal is to run the entire emulator in a browser tab via WASM, enabling a zero-install "try it now" experience.

### Architecture Today

```
┌──────────────────────────────────────────┐
│              apps/api                    │
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │ axum     │→ │ routes/* │→ │ store/ │ │
│  │ (HTTP)   │  │ handlers │  │ types  │ │
│  │ + tokio  │  │          │  │ jwt    │ │
│  └──────────┘  └──────────┘  └────────┘ │
└──────────────────────────────────────────┘
```

### Target Architecture

```
┌────────────────────────────────────────────────────┐
│  libs/rescope-core  (sync, std::sync::RwLock)      │
│  ┌──────────┐  ┌──────────┐  ┌────────┐           │
│  │ request  │→ │ handlers │→ │ store/ │           │
│  │ router   │  │ (sync)   │  │ types  │           │
│  │ (match)  │  │          │  │ jwt    │           │
│  └──────────┘  └──────────┘  └────────┘           │
└────────────────────────────────────────────────────┘
         ▲                      ▲
         │                      │
┌────────┴──────────┐  ┌───────┴───────────────────┐
│  apps/api         │  │  apps/rescope-wasm         │
│  (axum shell)     │  │  (wasm-bindgen bridge)     │
│  spawn_blocking   │  │  Adapts JS ↔ rescope-core  │
│  → rescope-core   │  │                            │
└───────────────────┘  └────────────────────────────┘
                                ▲
                       ┌────────┴──────────────────┐
                       │  apps/playground           │
                       │  Service Worker intercepts │
                       │  fetch() → WASM bridge     │
                       └───────────────────────────┘
```

## Goals / Non-Goals

**Goals:**
- Validate the WASM approach with a minimal prototype spike before committing to full refactor
- Extract a `rescope-core` lib crate that compiles to both native and `wasm32-unknown-unknown`
- Existing native server (`apps/api`) continues to work identically via the core lib
- WASM bridge accepts a serialized HTTP request (method, path, headers, body) and returns a serialized HTTP response
- Service worker transparently intercepts Descope SDK `fetch()` calls and routes to WASM
- Playground is embeddable in docs site or external sites

**Non-Goals:**
- Supporting `reqwest` in WASM — connectors are disabled in playground mode
- Multi-threaded WASM — single-threaded is fine for a dev playground
- Persistent storage — playground state lives in memory and resets on page reload
- WebSocket support or real-time features
- Fast RSA key generation — slow (~2-5s) is acceptable for a playground

## Decisions

### 1. Prototype spike before full refactor

**Choice:** Build a minimal end-to-end proof of concept before committing to the full core extraction. The spike will:
- Take a single store (e.g., `UserStore`) and single handler (e.g., `POST /v1/mgmt/user/create`)
- Compile to WASM
- Wire up a service worker to intercept that one endpoint
- Validate the full chain: `fetch()` → SW → WASM → response

**Rationale:** The core extraction is a 2-3 week refactor touching every file. A 1-2 day spike validates the approach works end-to-end before committing. If something is broken (WASM binary size, SW lifecycle, RSA in WASM), we find out early.

### 2. `std::sync::RwLock` in core, `spawn_blocking` in axum shell

**Choice:** `rescope-core` uses `std::sync::RwLock` for all store access (synchronous, blocking). The native axum shell wraps every core call in `tokio::task::spawn_blocking()`.

**Alternatives considered:**
- Keep `tokio::sync::RwLock` in core — can't compile to WASM (tokio runtime dependency)
- Custom `Lock` trait abstracting over async/sync — Over-engineered. `spawn_blocking` is the standard pattern for sync-in-async bridges.
- `RefCell` on WASM via `cfg` — Unnecessary. `std::sync::RwLock` works on WASM single-threaded. It compiles and functions correctly (no contention since single-threaded). One type, zero `cfg` needed.

**Rationale:** This is the same pattern used by `rusqlite` + tokio, `diesel` + actix-web, and other Rust projects that bridge sync libraries with async frameworks. It's well-understood. The core handlers must be structured as "acquire lock → do all work → release lock" with no interior async points. This is already how the rescope handlers work in practice — they don't do actual I/O inside locks.

### 3. Request/Response serialization at the WASM boundary

**Choice:** The WASM bridge exposes a single function: `handle_request(json: &str) -> String` where input/output are serialized `WasmRequest`/`WasmResponse` structs.

```rust
#[derive(Serialize, Deserialize)]
struct WasmRequest {
    method: String,       // "GET", "POST", etc.
    path: String,         // "/v1/auth/otp/signup/email"
    headers: HashMap<String, String>,
    body: Option<String>, // JSON body
}

#[derive(Serialize, Deserialize)]
struct WasmResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}
```

**Rationale:** JSON round-trip is negligible for auth API calls. Simple, testable, and the same types can be used in unit tests without a browser.

### 4. Service Worker for transparent SDK compatibility

**Choice:** A service worker intercepts `fetch()` requests matching the emulator's URL patterns and routes them to the WASM core. Descope SDKs work unmodified.

```
SDK calls: fetch("/v1/auth/otp/signup/email", { method: "POST", body: ... })
    ↓
Service Worker: intercepts, serializes to WasmRequest, calls WASM
    ↓
WASM bridge: routes to rescope-core handler, returns WasmResponse
    ↓
Service Worker: constructs Response object, returns to SDK
```

**Open concern:** If the browser kills the service worker for inactivity, the WASM state is lost. Mitigation options:
- Re-initialize WASM on SW restart (state resets — acceptable for a playground)
- Move WASM to a Web Worker that the SW postMessages to (survives SW restarts but adds latency)
- The spike (Decision 1) will validate which approach works in practice

### 5. Connector invoker trait abstraction

**Choice:** `ConnectorInvoker` becomes a trait in `rescope-core`. Native uses `reqwest`, WASM uses a no-op.

**Rationale:** Connectors call external URLs — in the playground, this is disabled. The trait boundary cleanly separates the concern.

## Risks / Trade-offs

- **Core extraction is a large refactor** — Moving ~14 stores + ~60 route handlers into a separate crate is 2-3 weeks of work. → Mitigation: Spike first (1-2 days) to validate approach. Then incremental migration with existing tests validating each step.

- **`spawn_blocking` pool exhaustion** — Under heavy load, `spawn_blocking` can exhaust the thread pool. → Mitigation: Not a concern for a dev emulator. If needed later, increase pool size or use `block_in_place`.

- **Service worker lifecycle** — Browser may kill SW for inactivity, losing WASM state. → Mitigation: Spike will test this. Playground state is ephemeral anyway — resetting is acceptable.

- **WASM binary size** — RSA crate + crypto may produce a large binary. → Mitigation: `wasm-opt`, `lto = true`. RSA being slow (~2-5s for keygen) is acceptable for a playground.

## Open Questions

1. **Should `rescope-core` extraction be its own change?** — It's a prerequisite for WASM but also improves native code organization. Could be split out for cleaner review. The spike will inform this decision.
