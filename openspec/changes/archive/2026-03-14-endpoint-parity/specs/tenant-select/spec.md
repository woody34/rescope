## ADDED Requirements

### Requirement: Tenant selection re-issues tokens with active tenant claim
`POST /v1/auth/tenant/select` SHALL accept `{ tenant }` and a valid refresh JWT. It SHALL verify the user belongs to the specified tenant, then issue new `sessionJwt` and `refreshJwt` with the tenant's roles injected into the session claims.

#### Scenario: Valid tenant selection returns new tokens
- **WHEN** `POST /v1/auth/tenant/select` is called with a valid refresh JWT and a tenant the user belongs to
- **THEN** response returns new `sessionJwt` and `refreshJwt` with updated tenant claims

#### Scenario: Unknown tenant returns 404
- **WHEN** `POST /v1/auth/tenant/select` is called with a tenant the user is not a member of
- **THEN** response is 404

#### Scenario: Invalid JWT returns 401
- **WHEN** `POST /v1/auth/tenant/select` is called with an invalid or expired refresh JWT
- **THEN** response is 401
