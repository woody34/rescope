## ADDED Requirements

### Requirement: WASM build pipeline for docs
The docs build process SHALL include a step that compiles `apps/rescope-wasm` via `wasm-pack` and copies the output to `apps/docs/public/wasm/`.

#### Scenario: WASM assets present after build
- **WHEN** `npm run docs:build:wasm` is run
- **THEN** `apps/docs/public/wasm/rescope_wasm.js` and `apps/docs/public/wasm/rescope_wasm_bg.wasm` SHALL exist and be valid

### Requirement: Demo service worker scoped to /demo-wasm/
The docs site SHALL include a service worker (`demo-sw.js`) registered with scope `/demo-wasm/` that intercepts requests and routes them to the WASM emulator.

#### Scenario: Service worker does not intercept docs navigation
- **WHEN** the demo SW is registered
- **THEN** requests to `/getting-started/`, `/guides/`, search endpoints, and all other non-`/demo-wasm/` paths SHALL NOT be intercepted by the service worker

#### Scenario: Service worker routes emulator calls to WASM
- **WHEN** the demo React island makes a request to `/demo-wasm/v1/auth/password/signin`
- **THEN** the service worker SHALL intercept the request and route it to `wasm_bindgen.handle_request()`, returning the WASM response

### Requirement: WASM initialization with loading state
The demo React island SHALL initialize the WASM module lazily when the demo page is visited, showing progress during initialization.

#### Scenario: Loading indicator shown during WASM init
- **WHEN** a user navigates to `/demo` for the first time
- **THEN** a loading indicator SHALL be shown while the WASM module fetches and initializes

#### Scenario: Graceful fallback when WASM unavailable
- **WHEN** the WASM assets are not present (e.g., docs built without `build:wasm`)
- **THEN** the demo page SHALL display a fallback message directing users to run Rescope locally, without crashing
