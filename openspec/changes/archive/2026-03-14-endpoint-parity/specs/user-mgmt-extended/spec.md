## ADDED Requirements

### Requirement: Management API can force-logout a user
`POST /v1/mgmt/user/logout` SHALL accept `{ loginId }` and insert a `user_revocations` entry for that user invalidating all existing tokens. Returns `{ ok: true }`.

#### Scenario: Force-logout invalidates user's refresh tokens
- **WHEN** `POST /v1/mgmt/user/logout` is called for an authenticated user
- **THEN** the user's existing refresh token is rejected on next use

### Requirement: Management API can mark a password as expired
`POST /v1/mgmt/user/password/expire` SHALL accept `{ loginId }` and set a flag on the user. Returns `{ ok: true }`. The emulator does not enforce password expiry at auth time; this is a no-op stub that returns success.

#### Scenario: Password expiry call returns ok
- **WHEN** `POST /v1/mgmt/user/password/expire` is called with a valid loginId
- **THEN** response returns `{ ok: true }`

### Requirement: Management API can set a temporary password
`POST /v1/mgmt/user/password/set/temporary` SHALL accept `{ loginId, password }` and behave identically to `password/set/active` — it SHALL bcrypt-hash and store the password. Returns `{ ok: true }`.

#### Scenario: Temporary password can be used to sign in
- **WHEN** `POST /v1/mgmt/user/password/set/temporary` is called then the user signs in with that password
- **THEN** sign-in succeeds

### Requirement: Management API can generate enchanted link token for test user
`POST /v1/mgmt/tests/generate/enchantedlink` SHALL accept `{ loginId }` and generate a magic-link-style token stored in TokenStore. Returns `{ token, loginId }`. Only test users are eligible.

#### Scenario: Generates token for test user
- **WHEN** `POST /v1/mgmt/tests/generate/enchantedlink` is called for a test user
- **THEN** response returns `{ token, loginId }` and the token is consumable via `magiclink/verify`

#### Scenario: Non-test user returns 400
- **WHEN** `POST /v1/mgmt/tests/generate/enchantedlink` is called for a regular user
- **THEN** response is 400
