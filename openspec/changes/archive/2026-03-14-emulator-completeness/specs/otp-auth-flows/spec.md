## ADDED Requirements

### Requirement: OTP sign-up creates a new user and returns a code

`POST /v1/auth/otp/signup/email` SHALL accept `{ "loginId": "<email>" }`, create a new user with `loginId` as both login ID and email, generate a random 6-digit numeric code, store it associated with the user's `userId`, and return `{ "maskedEmail": "<masked>", "code": "<code>" }`. SHALL fail with `409 Conflict` if `loginId` already exists.

`POST /v1/auth/otp/signup/phone/sms` SHALL behave identically for phone/SMS, accepting `{ "loginId": "<phone>" }`. The code is stored associated with the `userId`.

#### Scenario: Email OTP signup creates user and returns code

- **WHEN** `POST /v1/auth/otp/signup/email` is called with `{ "loginId": "new@test.com" }`
- **THEN** the response SHALL be `200 OK` with `{ "maskedEmail": "ne**@test.com", "code": "<6-digit-string>" }`
- **AND** a subsequent `GET /v1/mgmt/user?loginid=new@test.com` SHALL return the created user

#### Scenario: Duplicate loginId returns conflict

- **WHEN** `POST /v1/auth/otp/signup/email` is called for a `loginId` that already exists
- **THEN** the response SHALL be `409 Conflict`

### Requirement: OTP sign-in issues a code for an existing user

`POST /v1/auth/otp/signin/email` SHALL accept `{ "loginId": "<email>" }`, verify the user exists, generate and store a code, and return `{ "maskedEmail": "<masked>", "code": "<code>" }`. SHALL fail with `401` if user is not found.

`POST /v1/auth/otp/signin/phone/sms` SHALL behave identically for phone.

#### Scenario: Sign-in for existing user returns code

- **WHEN** `POST /v1/auth/otp/signin/email` is called with a known `loginId`
- **THEN** the response SHALL be `200 OK` with `code` in the body

#### Scenario: Sign-in for unknown user returns 401

- **WHEN** `POST /v1/auth/otp/signin/email` is called with an unknown `loginId`
- **THEN** the response SHALL be `401 Unauthorized`

### Requirement: OTP verify exchanges a code for a session

`POST /v1/auth/otp/verify/email` and `POST /v1/auth/otp/verify/phone/sms` SHALL accept `{ "loginId": "<id>", "code": "<code>" }`, look up the stored code for that user, consume it (single-use), and return a full `AuthenticationResponse` (sessionJwt, refreshJwt, user, cookies). SHALL fail with `401` if code is wrong, expired, or already used.

#### Scenario: Valid code returns session tokens

- **WHEN** the code from sign-in is submitted to `POST /v1/auth/otp/verify/email`
- **THEN** the response SHALL be `200 OK` with `sessionJwt` and `refreshJwt`

#### Scenario: Code is single-use

- **WHEN** the same code is submitted twice to `/v1/auth/otp/verify/email`
- **THEN** the second response SHALL be `401 Unauthorized`

#### Scenario: Wrong code returns 401

- **WHEN** an incorrect code is submitted
- **THEN** the response SHALL be `401 Unauthorized`

### Requirement: Emulator provides an OTP code retrieval endpoint

`GET /emulator/otp/:loginId` SHALL return `{ "code": "<last-issued-code>" }` for the given loginId without consuming the code (non-destructive read). SHALL return `404` if no pending code exists for that user.

#### Scenario: Code is retrievable without being consumed

- **WHEN** `GET /emulator/otp/alice@test.com` is called after an OTP sign-in
- **THEN** the response SHALL contain the same code that was issued
- **AND** the code SHALL still be valid for a subsequent call to `/v1/auth/otp/verify/email`
