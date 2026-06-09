## ADDED Requirements

### Requirement: Demo page with profile selector
The docs site SHALL have a `/demo` route that renders a profile selector bar, a live resource tree panel, and a login form panel — all powered by an in-browser WASM emulator.

#### Scenario: Demo page loads and prompts profile selection
- **WHEN** a user navigates to `/demo`
- **THEN** the page SHALL display a profile selector bar with available scenarios and a prompt to select one before the panels are shown

#### Scenario: Selecting a profile initializes the emulator
- **WHEN** a user selects a profile (e.g., "Email + Password")
- **THEN** the WASM emulator SHALL initialize, the emulator SHALL be reset, the profile's seed data SHALL be applied, and both panels SHALL become visible

#### Scenario: Profile switch resets emulator
- **WHEN** a user selects a different profile while one is active
- **THEN** the emulator SHALL reset to seed state for the new profile and both panels SHALL refresh

### Requirement: Live resource tree panel
The demo page SHALL display a hacker-style file tree of the emulator's current resource state, updating in real time.

#### Scenario: Tree reflects seeded state immediately on profile load
- **WHEN** a profile is selected and the seed is applied
- **THEN** the tree SHALL display the seeded users, roles, and tenants within one second

#### Scenario: Tree updates after a login
- **WHEN** a user completes a login flow via the login form
- **THEN** the tree SHALL update to reflect any new session or token state within one second

#### Scenario: Tree structure
- **WHEN** the tree is rendered
- **THEN** it SHALL display resources grouped under `users/`, `roles/`, and `tenants/` with count badges, using a monospace hacker aesthetic

### Requirement: Inline login form panel
The demo page SHALL display an auth form appropriate to the active profile, making real HTTP calls to the WASM emulator.

#### Scenario: Password login succeeds
- **WHEN** a user enters valid credentials and submits the password form
- **THEN** the panel SHALL display the API response including the session JWT

#### Scenario: OTP form shows code retrieval
- **WHEN** a user submits their email on the OTP form
- **THEN** the panel SHALL display the OTP code returned from `GET /demo-wasm/emulator/otp/:loginId` alongside the verify step

#### Scenario: Login error is displayed
- **WHEN** a user submits invalid credentials
- **THEN** the panel SHALL display the error response from the emulator in a readable format

#### Scenario: Reset button restores seed state
- **WHEN** a user clicks the reset button
- **THEN** the emulator SHALL reset to the profile's seed state and both panels SHALL refresh
