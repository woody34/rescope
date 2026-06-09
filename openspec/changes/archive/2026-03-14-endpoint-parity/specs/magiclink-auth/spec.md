## MODIFIED Requirements

### Requirement: magiclink/update/email persists the new email on the user record
`POST /v1/auth/magiclink/update/email` SHALL accept `{ loginId, email }` with a valid refresh JWT. After validating the JWT it SHALL update the user's email field in UserStore and add the new email to `loginIds`. Returns `{ ok: true }`.

#### Scenario: Email is updated and persisted
- **WHEN** `POST /v1/auth/magiclink/update/email` is called with a valid JWT and new email
- **THEN** the user's email field is updated; `GET /v1/auth/me` returns the new email

#### Scenario: Invalid JWT returns 401
- **WHEN** `POST /v1/auth/magiclink/update/email` is called with an invalid JWT
- **THEN** response is 401
