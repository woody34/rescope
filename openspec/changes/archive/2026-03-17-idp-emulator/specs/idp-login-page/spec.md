## ADDED Requirements

### Requirement: User picker HTML page
The system SHALL serve an HTML page at the IdP authorize (OIDC) and SSO (SAML) endpoints that lists all users in the tenant associated with the SSO request. Each user SHALL be displayed with their name, email, and a "Login" button.

#### Scenario: User picker shows tenant users
- **WHEN** a browser navigates to the IdP authorize/SSO endpoint for tenant "acme" which has users alice@acme.com and bob@acme.com
- **THEN** the page SHALL display both users with their names and emails

#### Scenario: Empty tenant shows no-users message
- **WHEN** a browser navigates to the IdP authorize endpoint for a tenant with no users
- **THEN** the page SHALL display a message indicating no users are available

### Requirement: User selection triggers protocol-appropriate redirect
The system SHALL redirect to the appropriate callback URL when a user is selected on the login page, using the correct protocol (OIDC code redirect or SAML POST binding).

#### Scenario: OIDC user selection redirects with code
- **WHEN** a user clicks "Login" on the OIDC IdP user picker
- **THEN** the browser SHALL redirect to `redirect_uri?code=<code>&state=<state>`

#### Scenario: SAML user selection auto-submits form
- **WHEN** a user clicks "Login" on the SAML IdP user picker
- **THEN** the browser SHALL auto-submit a form POST to the ACS URL with `SAMLResponse` and `RelayState`

### Requirement: Login page visual design
The system SHALL style the login page consistent with Rescope's dark theme, clearly labeling it as an emulated IdP login page (not a real one).

#### Scenario: Page identifies itself as emulated
- **WHEN** the user picker page is rendered
- **THEN** the page SHALL contain visible text indicating this is a Rescope IdP emulator (e.g., "Rescope IdP Emulator" header)
