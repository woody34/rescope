## ADDED Requirements

### Requirement: Single-function WASM entry point
The `rescope-wasm` crate SHALL expose a single `wasm-bindgen` function that accepts a serialized HTTP request and returns a serialized HTTP response.

#### Scenario: Successful request handling
- **WHEN** `handle_request` is called with a JSON-serialized `WasmRequest` containing method `POST`, path `/v1/auth/otp/signup/email`, and a valid JSON body
- **THEN** it SHALL return a JSON-serialized `WasmResponse` with the appropriate status code and body

#### Scenario: Malformed input
- **WHEN** `handle_request` is called with invalid JSON
- **THEN** it SHALL return a `WasmResponse` with status 400 and an error message

### Requirement: WASM module initialization
The WASM bridge SHALL expose an `init` function that creates the `rescope-core` state (stores, key manager, config) and returns a handle for subsequent `handle_request` calls.

#### Scenario: Initialization with default config
- **WHEN** `init` is called with a JSON config (project ID, management key)
- **THEN** it SHALL create the core state with generated RSA keys and empty stores, and return successfully

#### Scenario: Multiple requests share state
- **WHEN** `init` is called once and then `handle_request` is called multiple times
- **THEN** state changes from earlier requests (e.g., user creation) SHALL be visible to later requests
