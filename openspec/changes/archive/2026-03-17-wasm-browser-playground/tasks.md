## 0. Prototype Spike

- [x] 0.1 Create a minimal `spike/wasm-poc/` directory with a standalone Rust crate that contains: one store (`UserStore`), one handler (`create_user`), and a sync router
- [x] 0.2 Compile the spike crate to `wasm32-unknown-unknown` using `wasm-pack build --target web`
- [x] 0.3 Create a minimal HTML page + service worker that loads the WASM, intercepts `fetch("/v1/mgmt/user/create")`, routes to WASM, and returns the response
- [x] 0.4 Validate end-to-end: open the page, call `fetch()` from the console, confirm user is created in WASM state
- [x] 0.5 Measure: WASM binary size, service worker startup time, RSA keygen latency in WASM
- [x] 0.6 Document findings and go/no-go decision for full refactor

## 1. Core Library Extraction

- [x] 1.1 Create `libs/rescope-core` crate with `Cargo.toml` (no tokio, no axum deps); add to workspace
- [x] 1.2 Move `store/` modules and `types.rs` into `rescope-core`; replace `tokio::sync::RwLock` with `std::sync::RwLock`
- [x] 1.3 Move `jwt/` (key_manager, signing, validation) into `rescope-core`
- [ ] 1.4 Move auth handler logic (password, OTP, magic link, SAML, session) into `rescope-core` as sync functions operating on `CoreRequest` → `CoreResponse`
- [ ] 1.5 Move management handler logic (user, tenant, roles, permissions, etc.) into `rescope-core`
- [ ] 1.6 Define `ConnectorInvoker` trait in core; implement `ReqwestInvoker` (native) and `NoopInvoker` (WASM)
- [ ] 1.7 Implement synchronous request router in core that maps (method, path) → handler function
- [x] 1.8 Update `apps/api` to depend on `rescope-core`; wrap all core calls in `tokio::task::spawn_blocking()`
- [ ] 1.9 Verify: `cargo build -p rescope-core --target wasm32-unknown-unknown` compiles
- [x] 1.10 Verify: all existing native tests pass (`cargo test`)

## 2. WASM Bridge Crate

- [x] 2.1 Create `apps/rescope-wasm` crate with `wasm-bindgen` dependency; add to workspace
- [x] 2.2 Implement `init(config_json: &str) -> WasmHandle` and `handle_request(handle: &WasmHandle, request_json: &str) -> String`
- [x] 2.3 Define `WasmRequest` / `WasmResponse` serialization types
- [x] 2.4 Build with `wasm-pack build --target web` and verify .wasm + .js output

## 3. Service Worker

- [x] 3.1 Create `apps/playground/` directory with `package.json` and build tooling
- [x] 3.2 Implement `sw.js` service worker: install event loads WASM + calls `init()`, fetch event intercepts API patterns and routes to `handle_request()`
- [x] 3.3 Implement URL pattern matching for Descope API routes (`/v1/auth/*`, `/v1/mgmt/*`, `/emulator/*`, `/.well-known/*`, `/health`)
- [x] 3.4 Add `skipWaiting()` + `clients.claim()` for immediate activation

## 4. Playground Host Page

- [x] 4.1 Create host page that registers the service worker and loads the WASM module
- [x] 4.2 Add loading indicator with status text during WASM initialization
- [x] 4.3 Render the admin UI once the service worker is active
- [x] 4.4 Verify: open playground in browser, confirm admin UI loads and API calls work via WASM

## 5. Integration Verification

- [x] 5.1 Create a user via the management API from the playground admin UI — confirm it appears in the user list
- [ ] 5.2 Verify OTP flow works end-to-end in the browser playground
- [x] 5.3 Verify the playground is embeddable via `<iframe>`
