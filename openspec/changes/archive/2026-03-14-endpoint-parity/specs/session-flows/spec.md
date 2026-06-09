## ADDED Requirements

### Requirement: me/history returns empty list stub
`GET /v1/auth/me/history` SHALL accept a valid session or refresh JWT and return `{ users: [] }`. The emulator has no login event history so an empty list is always returned.

#### Scenario: me/history returns empty list for authenticated user
- **WHEN** `GET /v1/auth/me/history` is called with a valid refresh JWT
- **THEN** response is `{ users: [] }`

#### Scenario: me/history requires authentication
- **WHEN** `GET /v1/auth/me/history` is called without a JWT
- **THEN** response is 401
