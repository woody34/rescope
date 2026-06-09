## ADDED Requirements

### Requirement: OTP Inspector displays pending OTPs
The OTP Inspector page at `/ui/emulator/otp-inspector` SHALL display a table of all pending (unverified) OTPs currently stored in the emulator, including the associated loginId and the OTP code.

#### Scenario: Triggered OTP appears in Inspector table
- **WHEN** `POST /v1/auth/otp/signup/email` is called for a loginId
- **THEN** navigating to the OTP Inspector page shows a row for that loginId with a 6-digit code

#### Scenario: Verified OTP is removed from Inspector
- **WHEN** an OTP is verified via `POST /v1/auth/otp/verify/email`
- **THEN** refreshing the OTP Inspector page shows no row for that loginId

---

### Requirement: OTP code from Inspector can complete authentication
The OTP code displayed in the Inspector SHALL be identical to the code the emulator would accept for verification, enabling E2E test automation without email delivery.

#### Scenario: Inspector code verifies successfully
- **WHEN** the OTP code for a loginId is read from the Inspector table and submitted to `POST /v1/auth/otp/verify/email`
- **THEN** the response is HTTP 200 with a valid `sessionJwt`

---

### Requirement: OTP Inspector auto-refreshes
The OTP Inspector page SHALL automatically refresh its data at a regular interval (or provide a manual refresh button) so that new OTPs are visible without a full page reload.

#### Scenario: New OTP appears without full reload
- **WHEN** an OTP is triggered via API while the Inspector page is already open
- **THEN** within 5 seconds, the new OTP row appears in the table (via auto-refresh or refresh button click)
