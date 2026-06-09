## ADDED Requirements

### Requirement: Session refresh

`POST /v1/auth/refresh` SHALL accept `{ refreshJwt }` (or the `DSR` cookie), verify the refresh JWT (signature, expiry, not revoked), look up the associated user, generate a new session JWT and refresh JWT, and return an `AuthenticationResponse`.

#### Scenario: Valid refresh token returns new session

- **WHEN** `POST /v1/auth/refresh` is called with a valid, non-revoked refresh JWT
- **THEN** the response is HTTP 200 with a new `sessionJwt` and `refreshJwt`

#### Scenario: Expired refresh token is rejected

- **WHEN** `POST /v1/auth/refresh` is called with an expired refresh JWT
- **THEN** the response is HTTP 401 with `{ "ok": false }`

#### Scenario: Revoked refresh token is rejected

- **WHEN** `POST /v1/auth/refresh` is called with a refresh JWT that was previously revoked via logout
- **THEN** the response is HTTP 401 with `{ "ok": false }`

---

### Requirement: Logout

`POST /v1/auth/logout` SHALL accept `{ refreshJwt }` (or the `DSR` cookie), add the refresh JWT to the revocation store, and return `{ "ok": true }`. Subsequent refresh or me calls with the same token SHALL fail.

#### Scenario: Logout revokes the refresh token

- **WHEN** `POST /v1/auth/logout` is called with a valid refresh JWT
- **THEN** the response is HTTP 200 with `{ "ok": true }` and the token is added to the revocation set

#### Scenario: Refresh after logout is rejected

- **WHEN** `POST /v1/auth/refresh` is called after the token was logged out
- **THEN** the response is HTTP 401 with `{ "ok": false }`

---

### Requirement: Me (current user profile)

`GET /v1/auth/me` SHALL accept the `DSR` cookie or `Authorization: Bearer <refreshJwt>` header, verify the refresh JWT, and return `{ "user": UserResponse }`.

#### Scenario: Me returns correct user profile

- **WHEN** `GET /v1/auth/me` is called with a valid refresh JWT
- **THEN** the response is HTTP 200 with `{ "user": { ... } }` matching the token's `sub` user

#### Scenario: Me with invalid token is rejected

- **WHEN** `GET /v1/auth/me` is called with an invalid or revoked refresh JWT
- **THEN** the response is HTTP 401 with `{ "ok": false }`

---

### Requirement: Session validation (server-side)

`POST /v1/auth/validate` SHALL accept `{ sessionJwt }`, verify the JWT signature and expiry, and return an `AuthenticationInfo` object containing the parsed claims. This endpoint is used by `@descope/node-sdk`'s `validateSession`.

#### Scenario: Valid session JWT is validated

- **WHEN** `POST /v1/auth/validate` is called with a valid, non-expired session JWT
- **THEN** the response is HTTP 200 with `{ "jwt": "...", "token": { /* all claims */ } }`

#### Scenario: Expired session JWT is rejected

- **WHEN** `POST /v1/auth/validate` is called with an expired session JWT
- **THEN** the response is HTTP 401 with `{ "ok": false }`

---

### Requirement: Cookie-based session tokens

All authentication responses SHALL include `Set-Cookie` headers for `DS` (session JWT) and `DSR` (refresh JWT) with attributes: `HttpOnly`, `SameSite=Lax`, `Path=/`, `Secure` omitted (HTTP local dev). The `DS` cookie SHALL expire at `sessionExpiration`. The `DSR` cookie SHALL expire at `refreshExpiration`.

#### Scenario: Auth response sets DS and DSR cookies

- **WHEN** any successful authentication endpoint is called
- **THEN** the response headers include `Set-Cookie: DS=<sessionJwt>; HttpOnly; SameSite=Lax; Path=/` and `Set-Cookie: DSR=<refreshJwt>; HttpOnly; SameSite=Lax; Path=/`
