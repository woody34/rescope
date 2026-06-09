## ADDED Requirements

### Requirement: OTP codes are printed to stdout when generated
The emulator SHALL print OTP codes to stdout when they are generated via any OTP auth endpoint.

#### Scenario: OTP signup prints code
- **WHEN** a client calls `POST /v1/auth/otp/signup/email` with `loginId: "alice@test.com"`
- **THEN** stdout SHALL contain a log line with the login ID and the generated OTP code

#### Scenario: OTP signin prints code
- **WHEN** a client calls `POST /v1/auth/otp/signin/email` for an existing user
- **THEN** stdout SHALL contain a log line with the login ID and the generated OTP code

#### Scenario: OTP code printed for SMS flow
- **WHEN** a client calls `POST /v1/auth/otp/signup/phone/sms` with a phone number
- **THEN** stdout SHALL contain a log line with the phone number and the generated OTP code

### Requirement: Magic link tokens are printed to stdout when generated
The emulator SHALL print magic link tokens to stdout when they are generated via any magic link auth endpoint.

#### Scenario: Magic link signup prints token
- **WHEN** a client calls `POST /v1/auth/magiclink/signup/email` with `loginId: "bob@test.com"`
- **THEN** stdout SHALL contain a log line with the login ID and the generated magic link token

#### Scenario: Magic link signin prints token
- **WHEN** a client calls `POST /v1/auth/magiclink/signin/email` for an existing user
- **THEN** stdout SHALL contain a log line with the login ID and the generated magic link token

### Requirement: Test-user generated codes are printed to stdout
The emulator SHALL print codes/tokens to stdout when generated via the management test-user endpoints.

#### Scenario: Test user OTP generation prints code
- **WHEN** a client calls `POST /v1/mgmt/tests/generate/otp` for a test user
- **THEN** stdout SHALL contain a log line with the login ID and the generated OTP code

#### Scenario: Test user magic link generation prints token
- **WHEN** a client calls `POST /v1/mgmt/tests/generate/magiclink` for a test user
- **THEN** stdout SHALL contain a log line with the login ID and the generated magic link token
