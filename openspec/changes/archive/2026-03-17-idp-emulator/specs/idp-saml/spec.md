## ADDED Requirements

### Requirement: SAML metadata endpoint
The system SHALL serve a SAML EntityDescriptor XML document at `GET /emulator/idp/:idp_id/metadata` containing the IdP's entity ID, SSO endpoint URL, and X.509 signing certificate.

#### Scenario: Metadata returns valid EntityDescriptor XML
- **WHEN** a GET request is made to `/emulator/idp/mock-azure/metadata`
- **THEN** the response SHALL be XML with content type `application/xml`, containing an `EntityDescriptor` with `IDPSSODescriptor`, `SingleSignOnService` URL, and `KeyDescriptor` with the IdP's X.509 certificate

### Requirement: SAML SSO endpoint
The system SHALL serve `GET /emulator/idp/:idp_id/sso` accepting a `SAMLRequest` query parameter (base64-encoded, deflated AuthnRequest) and a `RelayState` parameter. It SHALL render an HTML user picker page listing all users in the linked tenant.

#### Scenario: SSO endpoint renders user picker
- **WHEN** a browser navigates to `/emulator/idp/mock-azure/sso?SAMLRequest=<encoded>&RelayState=xyz`
- **THEN** the page SHALL display a list of users from the linked tenant with clickable login buttons

#### Scenario: User selection posts SAML Response to ACS
- **WHEN** a user clicks a login button on the SAML IdP user picker page
- **THEN** the browser SHALL auto-submit a form POST to the tenant's ACS URL with `SAMLResponse` (base64-encoded, signed XML) and `RelayState`

### Requirement: SAML Response generation
The system SHALL generate valid SAML 2.0 Response XML containing a signed Assertion with `NameID`, `AuthnStatement`, and `AttributeStatement` (populated from attribute mapping). The Assertion SHALL be signed using the IdP's X.509 certificate and private key via the `samael` crate.

#### Scenario: SAML Response contains correct user attributes
- **WHEN** a SAML Response is generated for user "alice@acme.com" with attribute mapping `{email: user.email, firstName: user.givenName}`
- **THEN** the AttributeStatement SHALL contain `email` = "alice@acme.com" and `firstName` = Alice's given name

#### Scenario: SAML Assertion is properly signed
- **WHEN** a SAML Response is generated
- **THEN** the Assertion SHALL contain an enveloped XML signature verifiable with the IdP's X.509 certificate from the metadata endpoint

### Requirement: Self-signed X.509 certificate
The system SHALL auto-generate a self-signed X.509 certificate and RSA key pair for SAML signing on startup, separate from the SP's signing key. The certificate SHALL be included in the SAML metadata.

#### Scenario: Certificate is generated on startup
- **WHEN** the emulator starts
- **THEN** an IdP X.509 certificate SHALL be available for SAML signing

### Requirement: SAML SSO wired into saml/start
The system SHALL modify `saml/start` and `sso/authorize` so that when a tenant's `auth_type` is `saml` and its `saml_config.metadata_url` points to an emulated IdP, the returned `url` SHALL redirect the browser to the IdP's SSO endpoint with a `SAMLRequest` and `RelayState`.

#### Scenario: SSO start redirects to SAML IdP
- **WHEN** `POST /v1/auth/saml/start` is called with tenant "acme" (configured with SAML IdP)
- **THEN** the response `url` SHALL point to the emulated IdP's SSO endpoint with `SAMLRequest` and `RelayState` parameters

### Requirement: SAML Response processing in exchange
The system SHALL modify `saml/exchange` to accept SAML Response data from the IdP, validate the signature against the IdP's certificate, extract user attributes from the Assertion, apply attribute mapping, and issue SP-side session/refresh JWTs.

#### Scenario: Exchange with SAML IdP response issues session
- **WHEN** `POST /v1/auth/saml/exchange` is called after a SAML Response is received from the IdP
- **THEN** the SP SHALL validate the response, resolve/create the user, apply attribute mapping, and return session + refresh JWTs
