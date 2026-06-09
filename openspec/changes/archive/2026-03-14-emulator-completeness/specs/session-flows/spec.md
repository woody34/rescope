## ADDED Requirements

### Requirement: logoutAll revokes all sessions for a user

`POST /v1/auth/logoutall` SHALL accept a refresh JWT (via `Authorization: Bearer projectId:jwt`, `DSR` cookie, or `refreshJwt` body field), resolve the `userId` from the token claims, and mark a per-user revocation timestamp equal to the current time. All refresh tokens issued before or at this timestamp SHALL be rejected by subsequent `refresh` and `me` calls. Returns `{ "ok": true }`.

#### Scenario: After logoutAll, all existing refresh tokens are rejected

- **WHEN** a user has multiple active sessions and calls `POST /v1/auth/logoutall` with one token
- **THEN** all other refresh tokens for that user SHALL also be rejected by `POST /v1/auth/refresh`

#### Scenario: New tokens issued after logoutAll are valid

- **WHEN** a user calls `logoutAll`, then signs in again to get a fresh refresh token
- **THEN** the new refresh token SHALL be accepted by `POST /v1/auth/refresh`

#### Scenario: logoutAll with invalid token returns 401

- **WHEN** `POST /v1/auth/logoutall` is called with a malformed or expired refresh JWT
- **THEN** the response SHALL be `401 Unauthorized`
