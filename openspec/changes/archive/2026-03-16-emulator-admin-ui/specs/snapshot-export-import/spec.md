## ADDED Requirements

### Requirement: Snapshot export produces complete versioned JSON
The system SHALL export all emulator state as a single versioned JSON document via `GET /emulator/snapshot`. The export SHALL include: project config, auth method config (all 13 methods), permissions, roles, JWT templates, connector definitions, custom user attribute schemas, tenants (with SAML/OIDC config), users (with bcrypt password hashes), and access keys. Connector secrets SHALL be included (opaque, not stripped — emulator-local use). The export SHALL include the RSA key pair PEM strings under an optional `keys` field.

#### Scenario: Full export includes all stores
- **WHEN** `GET /emulator/snapshot` is called with a valid auth header
- **THEN** response is `200 OK` with `Content-Type: application/json`
- **THEN** body includes `version: 1`, `exported_at`, `users`, `tenants`, `roles`, `permissions`, `auth_methods`, `jwt_templates`, `connectors`, `custom_user_attributes`, `access_keys`, `project_config`, and `keys`

#### Scenario: Users export includes password hashes
- **WHEN** emulator has a user with a bcrypt password hash
- **WHEN** `GET /emulator/snapshot` is called
- **THEN** user entry in snapshot includes `_password_hash` field containing the bcrypt string

### Requirement: Snapshot import replaces all emulator state
The system SHALL accept a snapshot JSON via `POST /emulator/snapshot` and replace all in-memory stores with the contents. If the `keys` field is present, the RSA key pair SHALL be replaced. If `keys` is absent, the existing key pair SHALL be retained. After import, `GET /emulator/snapshot` SHALL return the imported state.

#### Scenario: Import replaces users
- **WHEN** emulator has existing users A and B
- **WHEN** `POST /emulator/snapshot` is called with a snapshot containing only user C
- **THEN** users A and B no longer exist
- **THEN** user C exists with the imported data

#### Scenario: Import accepts bcrypt hashes without re-hashing
- **WHEN** snapshot contains a user with `_password_hash: "$2b$10$..."` and `password: true`
- **WHEN** `POST /emulator/snapshot` is called  
- **THEN** the user is stored with the existing bcrypt hash
- **THEN** a subsequent password sign-in with the matching plaintext succeeds

#### Scenario: Import without keys field retains current key pair
- **WHEN** `POST /emulator/snapshot` is called with a snapshot that has no `keys` field
- **THEN** the emulator's RSA key pair is unchanged
- **THEN** JWTs signed before the import remain valid

#### Scenario: Invalid version in snapshot returns 400
- **WHEN** `POST /emulator/snapshot` is called with `version: 99`
- **THEN** response is `400 Bad Request` with an error describing the unsupported version
