## ADDED Requirements

### Requirement: Generic SSO authorize is an alias for SAML start
`POST /v1/auth/sso/authorize` SHALL accept `{ tenant, redirectUrl? }` and behave identically to `POST /v1/auth/saml/authorize`. It SHALL return `{ url }` containing a redirect URL with `?code=<token>`.

#### Scenario: SSO authorize returns redirect URL with code
- **WHEN** `POST /v1/auth/sso/authorize` is called with a valid tenant and user
- **THEN** response returns `{ url }` containing `?code=<token>`

### Requirement: Generic SSO exchange is an alias for SAML exchange
`POST /v1/auth/sso/exchange` SHALL accept `{ code }` and behave identically to `POST /v1/auth/saml/exchange`. It SHALL return session and refresh JWTs.

#### Scenario: SSO exchange returns tokens
- **WHEN** `POST /v1/auth/sso/exchange` is called with a valid code from SSO authorize
- **THEN** response returns `sessionJwt`, `refreshJwt`, and `user` object
