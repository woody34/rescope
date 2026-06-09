## ADDED Requirements

### Requirement: Demo profile definition contract
Each demo profile SHALL be a TypeScript object conforming to the `DemoProfile` interface, defining seed data, login UI type, and display metadata.

#### Scenario: Profile has all required fields
- **WHEN** a profile is loaded
- **THEN** it SHALL have an `id`, `label`, `icon`, `description`, `seed` (with at least one user), and `loginUI.type`

### Requirement: Email + Password profile
The system SHALL include an "Email + Password" profile that seeds users with passwords and presents a password login form.

#### Scenario: Password profile seeds correct data
- **WHEN** the "Email + Password" profile is selected
- **THEN** the emulator SHALL be seeded with at least two users with email login IDs and pre-set passwords, and at least one role

#### Scenario: Password profile shows login form
- **WHEN** the "Email + Password" profile is active
- **THEN** the login panel SHALL display email + password fields pre-filled with the seed user's credentials

### Requirement: OTP (Email) profile
The system SHALL include an "OTP" profile that seeds users and presents an OTP flow with visible code retrieval.

#### Scenario: OTP profile exposes the code
- **WHEN** a user submits the OTP request form
- **THEN** the panel SHALL fetch `GET /demo-wasm/emulator/otp/:loginId` and display the code alongside the verify input, making the emulator's superpower visible

### Requirement: Magic Link profile
The system SHALL include a "Magic Link" profile that demonstrates the token-based link flow.

#### Scenario: Magic link profile shows token
- **WHEN** a user requests a magic link
- **THEN** the panel SHALL retrieve and display the token from the emulator, then let the user click "Verify" to complete the flow

### Requirement: Multi-tenant profile
The system SHALL include a "Multi-tenant" profile seeding two tenants, multiple users with tenant role bindings.

#### Scenario: Multi-tenant profile seeds org structure
- **WHEN** the "Multi-tenant" profile is selected
- **THEN** the emulator SHALL be seeded with two tenants, at least two users, and tenant-role bindings visible in the resource tree
