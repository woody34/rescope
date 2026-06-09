## ADDED Requirements

### Requirement: OIDC discovery endpoint
The system SHALL serve an OpenID Connect discovery document at `GET /emulator/idp/:idp_id/.well-known/openid-configuration` containing `issuer`, `authorization_endpoint`, `token_endpoint`, `jwks_uri`, and supported response types/grant types.

#### Scenario: Discovery document is valid
- **WHEN** a GET request is made to `/emulator/idp/mock-okta/.well-known/openid-configuration`
- **THEN** the response SHALL be JSON with `issuer` matching the IdP's base URL, and `authorization_endpoint`, `token_endpoint`, `jwks_uri` pointing to the correct IdP paths

### Requirement: OIDC JWKS endpoint
The system SHALL serve the IdP's public key at `GET /emulator/idp/:idp_id/jwks` in JWKS format. This key SHALL be separate from the SP's signing key.

#### Scenario: JWKS contains IdP public key
- **WHEN** a GET request is made to `/emulator/idp/mock-okta/jwks`
- **THEN** the response SHALL be a valid JWKS JSON with at least one RSA key that can verify id_tokens signed by this IdP

### Requirement: OIDC authorize endpoint
The system SHALL serve `GET /emulator/idp/:idp_id/authorize` accepting standard OIDC parameters (`client_id`, `redirect_uri`, `response_type`, `scope`, `state`, `nonce`). It SHALL render an HTML user picker page listing all users in the tenant linked to this IdP.

#### Scenario: Authorize renders user picker
- **WHEN** a browser navigates to `/emulator/idp/mock-okta/authorize?client_id=acme&redirect_uri=http://localhost:3000/callback&response_type=code&state=xyz`
- **THEN** the page SHALL display a list of users from the linked tenant with clickable login buttons

#### Scenario: User selection redirects with code
- **WHEN** a user clicks a login button on the IdP user picker page
- **THEN** the browser SHALL redirect to the `redirect_uri` with `?code=<auth_code>&state=<original_state>` parameters

#### Scenario: Missing client_id returns error
- **WHEN** the authorize endpoint is called without a `client_id` parameter
- **THEN** the response SHALL be a 400 error

### Requirement: OIDC token endpoint
The system SHALL serve `POST /emulator/idp/:idp_id/token` accepting `grant_type=authorization_code`, `code`, `redirect_uri`, `client_id`, and `client_secret`. It SHALL return `id_token`, `access_token`, and `token_type`.

#### Scenario: Valid code exchange returns tokens
- **WHEN** a valid authorization code is exchanged at the token endpoint with correct client credentials
- **THEN** the response SHALL include a signed `id_token` JWT (signed with IdP key, not SP key) containing user claims based on the attribute mapping, plus an `access_token`

#### Scenario: Invalid code returns error
- **WHEN** an invalid or expired code is submitted to the token endpoint
- **THEN** the response SHALL be a 400 error with `error: "invalid_grant"`

#### Scenario: Wrong client_secret returns error
- **WHEN** a valid code is submitted with an incorrect `client_secret`
- **THEN** the response SHALL be a 401 error with `error: "invalid_client"`

### Requirement: OIDC SSO wired into saml/start and sso/start
The system SHALL modify the `saml/start` and `sso/start` handlers so that when a tenant's `auth_type` is `oidc` and its `oidc_config.discovery_url` points to an emulated IdP, the returned `url` SHALL redirect the browser to the IdP's authorize endpoint with proper OIDC parameters.

#### Scenario: SSO start redirects to OIDC IdP
- **WHEN** `POST /v1/auth/sso/authorize` is called with tenant "acme" (configured with OIDC IdP)
- **THEN** the response `url` SHALL point to the emulated IdP's authorize endpoint with `client_id`, `redirect_uri`, `response_type=code`, `state`, and `nonce` parameters

### Requirement: OIDC token exchange wired into saml/exchange
The system SHALL modify `saml/exchange` and `sso/exchange` to accept codes issued by the OIDC IdP, validate the `id_token` against the IdP's JWKS, and apply attribute mapping to populate user fields before issuing SP-side session/refresh JWTs.

#### Scenario: Exchange with OIDC IdP code issues session
- **WHEN** `POST /v1/auth/sso/exchange` is called with a code from the OIDC IdP
- **THEN** the SP SHALL validate the id_token, resolve/create the user, apply attribute mapping, and return session + refresh JWTs
