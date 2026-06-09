## ADDED Requirements

### Requirement: Platform-agnostic core library
The `rescope-core` library crate SHALL contain all business logic (stores, types, JWT operations, auth flows, management handlers) and SHALL compile to both native (`x86_64`/`aarch64`) and `wasm32-unknown-unknown` targets.

#### Scenario: Core compiles to native
- **WHEN** `cargo build -p rescope-core` is run
- **THEN** the crate SHALL compile successfully with no errors

#### Scenario: Core compiles to WASM
- **WHEN** `cargo build -p rescope-core --target wasm32-unknown-unknown` is run
- **THEN** the crate SHALL compile successfully with no errors

#### Scenario: No tokio dependency in core
- **WHEN** the `rescope-core` dependency tree is inspected
- **THEN** it SHALL NOT depend on `tokio`, `axum`, or `tower-http`

### Requirement: Synchronous store API
All store operations in `rescope-core` SHALL be synchronous. Concurrency primitives SHALL be selected via `cfg` attributes — `std::sync::RwLock` on native, `RefCell` on WASM.

#### Scenario: Native uses RwLock
- **WHEN** `rescope-core` is compiled for a native target
- **THEN** store access SHALL use `std::sync::RwLock` for thread safety

#### Scenario: WASM uses RefCell
- **WHEN** `rescope-core` is compiled for `wasm32-unknown-unknown`
- **THEN** store access SHALL use `RefCell` (single-threaded)

### Requirement: Request router in core
`rescope-core` SHALL include a synchronous request router that maps method + path pairs to handler functions and returns a response, without depending on any HTTP framework.

#### Scenario: Route matching
- **WHEN** a request with method `POST` and path `/v1/auth/otp/signup/email` is dispatched to the core router
- **THEN** the core router SHALL invoke the OTP signup email handler and return its response

#### Scenario: Unknown route returns 404
- **WHEN** a request with an unrecognized path is dispatched
- **THEN** the core router SHALL return a 404 status response

### Requirement: Connector invoker trait abstraction
Network-dependent functionality (connector invocation) SHALL be abstracted behind a trait so that native and WASM targets can provide different implementations.

#### Scenario: Native uses reqwest
- **WHEN** the emulator runs as a native binary
- **THEN** the connector invoker SHALL use `reqwest` for outbound HTTP calls

#### Scenario: WASM uses no-op
- **WHEN** the emulator runs in WASM
- **THEN** the connector invoker SHALL use a no-op implementation that returns an error indicating connectors are unavailable in playground mode
