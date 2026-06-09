## ADDED Requirements

### Requirement: JWT custom claims can be updated via management API
`POST /v1/mgmt/jwt/update` SHALL accept `{ refreshJwt, customClaims }`. It SHALL validate the refresh JWT, extract the userId, load the user, and issue a new session JWT with the provided `customClaims` merged into the standard claims. Returns `{ sessionJwt }`.

#### Scenario: Custom claims are merged into new session JWT
- **WHEN** `POST /v1/mgmt/jwt/update` is called with a valid refresh JWT and `{ "appRole": "admin" }`
- **THEN** response returns a new `sessionJwt` whose decoded claims contain `"appRole": "admin"`

#### Scenario: Invalid refresh JWT returns 401
- **WHEN** `POST /v1/mgmt/jwt/update` is called with an invalid JWT
- **THEN** response is 401
