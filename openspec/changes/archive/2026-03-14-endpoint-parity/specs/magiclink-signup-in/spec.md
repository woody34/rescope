## ADDED Requirements

### Requirement: Magic link signup-in composite for email
`POST /v1/auth/magiclink/signup-in/email` SHALL accept `{ loginId }`. If the user exists it behaves as sign-in; if not it creates the user. Returns `{ maskedEmail, token }`.

#### Scenario: Existing user receives sign-in magic link
- **WHEN** `POST /v1/auth/magiclink/signup-in/email` is called with an existing user's email
- **THEN** response returns `{ maskedEmail, token }` without creating a duplicate user

#### Scenario: New user is created and receives magic link
- **WHEN** `POST /v1/auth/magiclink/signup-in/email` is called with a new email
- **THEN** a new user is created and response returns `{ maskedEmail, token }`

#### Scenario: Token from signup-in is consumable
- **WHEN** the token from `signup-in/email` is submitted to `magiclink/verify`
- **THEN** response returns valid `sessionJwt` and `refreshJwt`

### Requirement: Magic link signup-in composite for SMS
`POST /v1/auth/magiclink/signup-in/sms` SHALL behave the same as `signup-in/email` but use `phone = loginId` and return `maskedPhone`.

#### Scenario: Existing phone user receives SMS magic link
- **WHEN** `POST /v1/auth/magiclink/signup-in/sms` is called with an existing phone user
- **THEN** response returns `{ maskedPhone, token }` without creating a duplicate

#### Scenario: New phone user is created
- **WHEN** `POST /v1/auth/magiclink/signup-in/sms` is called with a new phone loginId
- **THEN** a new user is created and response returns `{ maskedPhone, token }`
