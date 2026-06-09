## ADDED Requirements

### Requirement: Management API can generate an OTP for a test user

`POST /v1/mgmt/tests/generate/otp` SHALL accept `{ "loginId": "<id>" }`, require the user to be flagged as a test user, generate and store a 6-digit code, and return `{ "code": "<code>", "loginId": "<id>" }`. SHALL fail with `400` if the user is not a test user.

#### Scenario: OTP generated for test user

- **WHEN** `POST /v1/mgmt/tests/generate/otp` is called with a `loginId` that belongs to a test user
- **THEN** the response SHALL be `200 OK` with `code` in the body

#### Scenario: OTP generation fails for non-test user

- **WHEN** the `loginId` belongs to a regular (non-test) user
- **THEN** the response SHALL be `400 Bad Request`
