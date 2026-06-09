## ADDED Requirements

### Requirement: Transparent fetch interception
The service worker SHALL intercept `fetch()` requests from the page that match Descope API URL patterns and route them to the WASM bridge instead of the network.

#### Scenario: API request intercepted
- **WHEN** the page (or an SDK) calls `fetch("/v1/auth/otp/signup/email", { method: "POST", body: ... })`
- **THEN** the service worker SHALL intercept the request, serialize it to a `WasmRequest`, call the WASM bridge, and return the `WasmResponse` as a standard `Response` object

#### Scenario: Non-API request passes through
- **WHEN** the page calls `fetch("https://cdn.example.com/script.js")`
- **THEN** the service worker SHALL NOT intercept it and SHALL let it pass to the network

#### Scenario: Management API requests intercepted
- **WHEN** the page calls `fetch("/v1/mgmt/user/create", ...)`
- **THEN** the service worker SHALL intercept and route to the WASM bridge

### Requirement: Immediate activation
The service worker SHALL activate immediately without waiting for existing clients to close.

#### Scenario: First visit activation
- **WHEN** a user visits the playground page for the first time
- **THEN** the service worker SHALL register, install, and activate using `skipWaiting()` + `clients.claim()` so that interception begins immediately without requiring a page reload

### Requirement: WASM module lifecycle
The service worker SHALL load and initialize the WASM module during its `install` event and keep it alive for the duration of the service worker's lifetime.

#### Scenario: WASM initialized during install
- **WHEN** the service worker installs
- **THEN** it SHALL load the WASM binary, call `init()` with default playground config, and hold the resulting state handle in memory

#### Scenario: State persists across requests
- **WHEN** a user creates a user via the management API and then searches for that user
- **THEN** the WASM state SHALL contain both operations' effects
