## ADDED Requirements

### Requirement: Playground route with WASM embed
The docs site SHALL have a `/playground` route that loads the WASM emulator and renders the Rescope admin UI inline, providing a zero-install interactive experience.

#### Scenario: Playground loads WASM module
- **WHEN** a user navigates to `/playground`
- **THEN** the page SHALL lazy-load the WASM module, register the service worker, and display a loading indicator during initialization

#### Scenario: Playground becomes interactive
- **WHEN** the WASM module and service worker are both ready
- **THEN** the loading indicator SHALL be replaced with a fully functional Rescope admin UI where users can create tenants, users, and test auth flows

#### Scenario: Playground graceful fallback
- **WHEN** the WASM module is not yet available (pre-WASM change) or fails to load
- **THEN** the playground page SHALL display a fallback message directing users to install Rescope locally

### Requirement: Playground isolation
The WASM playground SHALL be isolated so it does not interfere with the docs site's own navigation or network requests.

#### Scenario: Service worker scoping
- **WHEN** the playground service worker is registered
- **THEN** it SHALL be scoped to the playground route only and SHALL NOT intercept requests from other docs pages
