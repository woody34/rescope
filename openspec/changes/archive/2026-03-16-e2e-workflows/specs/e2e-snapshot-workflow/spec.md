## ADDED Requirements

### Requirement: Snapshot export via Admin UI
The emulator's Snapshot page SHALL provide an Export button that triggers a file download of the current emulator state as a JSON snapshot.

#### Scenario: Export produces valid snapshot file
- **WHEN** user navigates to `/ui/emulator/snapshot` and clicks the export button
- **THEN** a JSON file is downloaded containing all current emulator state (users, permissions, roles, tenants)

#### Scenario: Exported snapshot reflects API-seeded state
- **WHEN** a permission is created via `POST /v1/mgmt/authz/permission` and then the snapshot is exported
- **THEN** the downloaded JSON contains that permission in its `permissions` array

---

### Requirement: Emulator reset via API clears all state
The emulator SHALL clear all runtime state (users, tokens, permissions, roles) when `POST /emulator/reset` is called, but SHALL NOT lose snapshot/import capability.

#### Scenario: Post-reset, previously created permissions are gone
- **WHEN** `POST /emulator/reset` is called after seeding state
- **THEN** `GET /v1/mgmt/authz/permission/all` returns an empty permissions list

---

### Requirement: Snapshot import via Admin UI restores state
The emulator's Snapshot page SHALL provide a file input that accepts a previously exported JSON snapshot and restores the emulator to that state.

#### Scenario: Import restores permissions after reset
- **WHEN** a snapshot containing a permission is imported via the Snapshot page file input
- **THEN** `GET /v1/mgmt/authz/permission/all` returns that permission

#### Scenario: Import shows success feedback
- **WHEN** a valid snapshot file is selected in the import input
- **THEN** the page displays a success message containing "imported successfully"

---

### Requirement: Emulator reset button in Admin UI
The Snapshot page SHALL provide a Reset button that calls `POST /emulator/reset` with user confirmation and shows success feedback.

#### Scenario: Reset button requires confirmation
- **WHEN** user clicks the Reset button
- **THEN** a browser dialog (confirm) is shown before reset is executed

#### Scenario: Post-reset UI shows success
- **WHEN** user confirms the reset dialog
- **THEN** the page displays text containing "runtime state reset"
