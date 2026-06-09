## ADDED Requirements

### Requirement: Self-contained playground page
The playground host SHALL be a standalone web application that bootstraps the WASM module, registers the service worker, and renders a functional Descope emulator experience in-browser.

#### Scenario: Zero-install startup
- **WHEN** a user navigates to the playground URL in a browser
- **THEN** the page SHALL load the WASM module, register the service worker, and display a ready state — without any downloads, CLI commands, or server processes

#### Scenario: Admin UI loads in-browser
- **WHEN** the playground is ready
- **THEN** the user SHALL be able to interact with the Rescope admin UI (users, tenants, roles, permissions pages) with all operations handled by the in-browser WASM core

### Requirement: Loading state
The playground SHALL display a loading indicator while the WASM module initializes and the service worker activates.

#### Scenario: Loading feedback
- **WHEN** the playground page is first loaded and the WASM module is initializing
- **THEN** the page SHALL display a loading indicator with status text (e.g., "Loading emulator…")

#### Scenario: Ready state
- **WHEN** the WASM module is initialized and the service worker is active
- **THEN** the loading indicator SHALL be replaced with the functional playground UI

### Requirement: Embeddable via iframe
The playground SHALL be embeddable in external sites via an `<iframe>` tag.

#### Scenario: Iframe embedding
- **WHEN** an external site includes `<iframe src="https://playground.rescope.dev"></iframe>`
- **THEN** the playground SHALL load and function correctly within the iframe, including service worker registration
