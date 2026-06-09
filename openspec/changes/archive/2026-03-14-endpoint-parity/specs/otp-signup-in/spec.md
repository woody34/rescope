## ADDED Requirements

### Requirement: OTP signup-in creates-or-signs-in via email OTP
`POST /v1/auth/otp/signup-in/email` SHALL accept `{ loginId }`. If a user with that loginId exists, it SHALL generate an OTP code and treat it as a sign-in. If no user exists, it SHALL create a new user with `email = loginId` and generate an OTP code. The response SHALL include `maskedEmail` and `code`.

#### Scenario: Existing user receives OTP sign-in code
- **WHEN** `POST /v1/auth/otp/signup-in/email` is called with a loginId that already exists
- **THEN** the response returns `{ maskedEmail, code }` with a 6-digit code and no new user is created

#### Scenario: New user is created and receives OTP code
- **WHEN** `POST /v1/auth/otp/signup-in/email` is called with a loginId that does not exist
- **THEN** a new user is created with `email = loginId` and the response returns `{ maskedEmail, code }`

#### Scenario: OTP code from signup-in can be verified
- **WHEN** the code from `signup-in/email` is submitted to `POST /v1/auth/otp/verify/email`
- **THEN** the response returns valid `sessionJwt` and `refreshJwt`

### Requirement: OTP signup-in creates-or-signs-in via SMS OTP
`POST /v1/auth/otp/signup-in/sms` SHALL behave identically to `signup-in/email` but use `phone = loginId` and return `maskedPhone`.

#### Scenario: Existing user receives OTP SMS code
- **WHEN** `POST /v1/auth/otp/signup-in/sms` is called with an existing phone loginId
- **THEN** response returns `{ maskedPhone, code }` without creating a duplicate user

#### Scenario: New user created via SMS signup-in
- **WHEN** `POST /v1/auth/otp/signup-in/sms` is called with a new phone loginId
- **THEN** a new user is created and response returns `{ maskedPhone, code }`
