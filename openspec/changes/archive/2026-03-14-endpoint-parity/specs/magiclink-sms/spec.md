## ADDED Requirements

### Requirement: Magic link sign-up via SMS
`POST /v1/auth/magiclink/signup/sms` SHALL accept `{ loginId }` where loginId is a phone number. If a user with that loginId already exists it SHALL return 409. If not, it SHALL create the user and return `{ maskedPhone, token }`. No SMS is sent.

#### Scenario: New user signup via SMS magic link
- **WHEN** `POST /v1/auth/magiclink/signup/sms` is called with a new phone loginId
- **THEN** a user is created and response returns `{ maskedPhone, token }`

#### Scenario: Duplicate phone returns conflict
- **WHEN** `POST /v1/auth/magiclink/signup/sms` is called with an existing phone loginId
- **THEN** response is 409 Conflict

### Requirement: Magic link sign-in via SMS
`POST /v1/auth/magiclink/signin/sms` SHALL accept `{ loginId }`, verify the user exists, generate a token, and return `{ maskedPhone, token }`. No SMS is sent.

#### Scenario: Existing user receives SMS magic link token
- **WHEN** `POST /v1/auth/magiclink/signin/sms` is called with an existing phone user
- **THEN** response returns `{ maskedPhone, token }`

#### Scenario: Unknown user returns 401
- **WHEN** `POST /v1/auth/magiclink/signin/sms` is called with an unknown loginId
- **THEN** response is 401

### Requirement: Magic link update phone via SMS
`POST /v1/auth/magiclink/update/phone/sms` SHALL accept `{ loginId, phone }` with a valid refresh JWT in the Authorization header. It SHALL generate a token stored in TokenStore, update the user's phone field in UserStore, and return `{ maskedPhone, token }`. The token can be consumed via `magiclink/verify`.

#### Scenario: Phone is updated and token is returned
- **WHEN** `POST /v1/auth/magiclink/update/phone/sms` is called with a valid refresh JWT and new phone
- **THEN** the user's phone is updated and response returns `{ maskedPhone, token }`

#### Scenario: Invalid JWT returns 401
- **WHEN** `POST /v1/auth/magiclink/update/phone/sms` is called without a valid refresh JWT
- **THEN** response is 401
