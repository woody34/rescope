## MODIFIED Requirements

### Requirement: Magic link update email persists the new address

When `POST /v1/auth/magiclink/update/email` is called with a `loginId`, `email`, and optional authorization token, the emulator SHALL update the `email` field on the matching user record and set `verifiedEmail: false`. The endpoint SHALL return `{ "ok": true }`. A subsequent call to `GET /v1/auth/me` SHALL reflect the new email address.

#### Scenario: New email is persisted and readable via me()

- **WHEN** a client calls `POST /v1/auth/magiclink/update/email` with `{ "loginId": "alice@old.com", "email": "alice@new.com" }`
- **THEN** the endpoint SHALL return `{ "ok": true }` and a subsequent `GET /v1/auth/me` SHALL return `email: "alice@new.com"`

#### Scenario: Invalid refresh token is rejected

- **WHEN** a client calls the endpoint with a malformed or expired token in the Authorization header
- **THEN** the endpoint SHALL return `401 Unauthorized`

#### Scenario: Unknown loginId returns 404

- **WHEN** the `loginId` in the request body does not match any user
- **THEN** the endpoint SHALL return `404 Not Found`
