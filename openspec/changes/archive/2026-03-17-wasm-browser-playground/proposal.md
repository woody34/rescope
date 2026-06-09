## Why

Rescope is a local dev tool that emulates the Descope auth platform. Today, trying Rescope requires downloading a binary and running it locally. A zero-install browser playground — where the entire emulator runs inside a web page — would let developers try Rescope instantly from documentation, demos, or interactive tutorials with zero setup.

## What Changes

- Extract core business logic (stores, JWT, auth handlers) into a `rescope-core` library crate that compiles to both native and `wasm32-unknown-unknown`
- Create a WASM bridge crate (`rescope-wasm`) that exposes an HTTP-request-in/response-out interface via `wasm-bindgen`
- Build a Service Worker that intercepts `fetch()` calls matching Descope API patterns and routes them to the WASM core in-memory
- Create a playground host page that loads the WASM module, registers the service worker, and hosts the Descope SDK + admin UI entirely in-browser

## Capabilities

### New Capabilities
- `core-lib-extraction`: Extract platform-agnostic business logic from the axum routing layer into a standalone library crate that compiles to both native and WASM targets
- `wasm-bridge`: WASM-bindgen bridge that accepts serialized HTTP requests and returns serialized responses, translating between JS and the Rust core
- `service-worker-interception`: Service worker that intercepts fetch requests matching Descope API URL patterns and routes them through the WASM bridge instead of the network
- `playground-host`: Embeddable web page that bootstraps the WASM module, service worker, and renders either the admin UI or a Descope SDK demo app

### Modified Capabilities

_(none — no existing spec-level behavior changes)_

## Impact

- **Workspace Cargo.toml** — new workspace members: `libs/rescope-core`, `apps/rescope-wasm`
- **`apps/api/`** — refactored to depend on `rescope-core` for all business logic; `main.rs`/`server.rs` become thin axum wiring
- **New: `libs/rescope-core/`** — pure-Rust library with stores, JWT, auth, types
- **New: `apps/rescope-wasm/`** — `wasm-bindgen` bridge crate
- **New: `apps/playground/`** — small web app with service worker + WASM loader
- **Dependencies** — `reqwest` must be made optional or abstracted behind a trait (it uses native TLS which doesn't compile to WASM); `tokio` replaced with sync alternatives in the core crate
