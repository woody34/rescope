## ADDED Requirements

### Requirement: Identity Providers admin page
The system SHALL provide an "Identity Providers" page in the admin UI accessible from the sidebar navigation, listing all configured IdP emulators.

#### Scenario: Page shows list of configured IdPs
- **WHEN** the user navigates to the Identity Providers page with 2 IdP emulators configured
- **THEN** the page SHALL display both emulators with their display name, protocol, and linked tenant

#### Scenario: Empty state
- **WHEN** no IdP emulators are configured
- **THEN** the page SHALL display an empty state with guidance on creating an IdP emulator

### Requirement: Create IdP emulator dialog
The system SHALL provide a dialog for creating a new IdP emulator with fields for display name, protocol (OIDC/SAML dropdown), and linked tenant (dropdown of existing tenants).

#### Scenario: Create OIDC IdP emulator
- **WHEN** the user fills in display name "Mock Okta", selects protocol "OIDC", selects tenant "acme", and clicks Create
- **THEN** a new IdP emulator SHALL be created and appear in the list, and the tenant's `oidcConfig` SHALL be auto-configured to point to the emulated IdP

#### Scenario: Create SAML IdP emulator
- **WHEN** the user fills in display name "Mock Azure AD", selects protocol "SAML", selects tenant "acme", and clicks Create
- **THEN** a new IdP emulator SHALL be created and the tenant's `samlConfig` SHALL be auto-configured

### Requirement: Attribute mapping editor
The system SHALL provide a table-based editor for configuring attribute mapping on each IdP emulator. Each row maps an IdP claim name to a user field path.

#### Scenario: Add attribute mapping
- **WHEN** the user adds a mapping row with IdP claim "email" → user field "user.email"
- **THEN** the mapping SHALL be saved to the IdP emulator configuration

#### Scenario: Default mappings populated
- **WHEN** a new IdP emulator is created
- **THEN** the attribute mapping editor SHALL pre-populate with common defaults: `email` → `user.email`, `name` → `user.name`

### Requirement: Delete IdP emulator
The system SHALL allow deleting an IdP emulator from the admin UI with a confirmation dialog.

#### Scenario: Delete IdP emulator
- **WHEN** the user clicks Delete on an IdP emulator and confirms
- **THEN** the emulator SHALL be removed from the list and the linked tenant's SSO config SHALL be cleared

### Requirement: Test SSO button
The system SHALL provide a "Test SSO" button on each IdP emulator that opens the IdP login page in a new tab, allowing the user to walk through the full SSO flow interactively.

#### Scenario: Test SSO opens login page
- **WHEN** the user clicks "Test SSO" on an OIDC IdP emulator
- **THEN** a new browser tab SHALL open showing the IdP user picker page for the linked tenant
