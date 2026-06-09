## ADDED Requirements

### Requirement: SAML authorize is the spec-correct endpoint name
`POST /v1/auth/saml/authorize` SHALL accept the same request body as `POST /v1/auth/saml/start` and return the same `{ url }` response. Both paths SHALL be registered so existing tests continue passing.

#### Scenario: SAML authorize behaves identically to SAML start
- **WHEN** `POST /v1/auth/saml/authorize` is called with a valid tenant and email
- **THEN** response returns `{ url }` with `?code=<token>`

## MODIFIED Requirements

### Requirement: SAML start is retained as a backward-compat alias
The existing `POST /v1/auth/saml/start` endpoint SHALL continue to work identically to before. It is now an alias for `/authorize`.

#### Scenario: Existing SAML start tests still pass
- **WHEN** `POST /v1/auth/saml/start` is called with valid credentials
- **THEN** response returns `{ url }` with a valid code (unchanged behavior)
