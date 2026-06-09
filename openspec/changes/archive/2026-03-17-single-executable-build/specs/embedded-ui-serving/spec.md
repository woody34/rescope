## ADDED Requirements

### Requirement: Embedded UI asset serving
The server SHALL embed the React UI dist output into the binary at compile time when built with the `embed-ui` Cargo feature, and serve those assets to browsers without any filesystem dependency.

#### Scenario: Production binary serves UI
- **WHEN** the binary is built with `--features embed-ui` and a user navigates to `http://localhost:<port>/`
- **THEN** the server SHALL respond with the embedded `index.html` and all referenced JS/CSS/asset files with correct MIME types

#### Scenario: SPA fallback routing
- **WHEN** a user navigates to a client-side route (e.g., `/identity-providers`) and no API route matches
- **THEN** the server SHALL respond with the embedded `index.html` to support React Router

#### Scenario: Dev mode preserves filesystem serving
- **WHEN** the binary is built WITHOUT the `embed-ui` feature (default `cargo run`)
- **THEN** the server SHALL continue serving from `apps/ui/dist/` on disk, preserving the existing dev workflow

#### Scenario: API routes take precedence over embedded assets
- **WHEN** a request matches both an API route (e.g., `/health`) and a potential static file path
- **THEN** the API route handler SHALL take priority
