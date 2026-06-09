## ADDED Requirements

### Requirement: IdP emulator configuration store
The system SHALL provide an `IdpEmulator` data model with fields: `id` (string), `protocol` (enum: oidc | saml), `display_name` (string), `tenant_id` (string, links to tenant), and `attribute_mapping` (map of IdP claim name → user field path).

#### Scenario: Create an OIDC IdP emulator
- **WHEN** a new IdP emulator is created with protocol "oidc" and tenant_id "acme"
- **THEN** the store SHALL persist the configuration and it SHALL be retrievable by id

#### Scenario: Create a SAML IdP emulator
- **WHEN** a new IdP emulator is created with protocol "saml" and tenant_id "acme"
- **THEN** the store SHALL persist the configuration and it SHALL be retrievable by id

#### Scenario: Update an IdP emulator
- **WHEN** an existing IdP emulator's attribute_mapping is updated
- **THEN** the store SHALL persist the updated mapping

#### Scenario: Delete an IdP emulator
- **WHEN** an IdP emulator is deleted by id
- **THEN** it SHALL no longer be retrievable from the store

#### Scenario: List all IdP emulators
- **WHEN** multiple IdP emulators exist
- **THEN** listing SHALL return all configured emulators

### Requirement: IdP configuration in snapshot export/import
The system SHALL include `idpEmulators` in the `EmulatorSnapshot` struct so that IdP configurations survive export/import cycles.

#### Scenario: Export includes IdP emulators
- **WHEN** the emulator state is exported via `GET /emulator/snapshot`
- **THEN** the JSON SHALL include an `idpEmulators` array with all configured IdP emulators

#### Scenario: Import restores IdP emulators
- **WHEN** a snapshot JSON containing `idpEmulators` is imported via `POST /emulator/snapshot`
- **THEN** the IdP store SHALL contain exactly the emulators from the snapshot

### Requirement: IdP configuration in seed file
The system SHALL accept `idpEmulators` in the seed file JSON to pre-configure IdP emulators on startup.

#### Scenario: Seed file with IdP emulators
- **WHEN** the emulator starts with a seed file containing `idpEmulators`
- **THEN** the IdP store SHALL be populated with the seeded configurations
