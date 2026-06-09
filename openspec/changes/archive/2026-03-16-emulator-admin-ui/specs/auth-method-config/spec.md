## ADDED Requirements

### Requirement: Auth method configs are stored per method and enforced at runtime
The emulator SHALL maintain a single `AuthMethodConfig` store with one configuration object per auth method (OTP, magic link, enchanted link, embedded link, TOTP, passkeys, OAuth, SSO, passwords, security questions, recovery codes, device auth, nOTP). The config SHALL be readable and writable at runtime. Auth route handlers SHALL read config on each request and enforce: expiry windows, retry limits, lockout thresholds, and method-enabled state.

#### Scenario: Disabled method returns 403
- **WHEN** `auth_methods.otp.enabled = false` is configured
- **WHEN** an OTP sign-in is attempted via SDK
- **THEN** response is `403 Forbidden` with an error indicating the method is disabled

#### Scenario: OTP code expires after configured window
- **WHEN** `auth_methods.otp.expiration_seconds = 10` is configured
- **WHEN** an OTP code is issued
- **WHEN** verification is attempted 11 or more seconds later
- **THEN** response is `401 Unauthorized` with an expiry error

### Requirement: Password policy is enforced at signup and password change
The emulator SHALL enforce all password policy fields during password sign-up and password update: `min_length`, `require_lowercase`, `require_uppercase`, `require_number`, `require_non_alphanumeric`, and `strength_enforcement` level. A password violating any enabled constraint SHALL be rejected with `400 Bad Request`.

#### Scenario: Password shorter than min_length is rejected
- **WHEN** `auth_methods.password.min_length = 12` is configured
- **WHEN** a signup is attempted with an 8-character password
- **THEN** response is `400 Bad Request` with a policy violation message

### Requirement: Account lockout is enforced after configured failed attempts
The emulator SHALL track failed auth attempts per login ID for methods with lockout enabled (OTP, passwords, security questions, recovery codes). After `lockout_attempts` failures within the configured timeframe, the account SHALL be locked for `lockout_duration` seconds. Locked accounts SHALL return `429 Too Many Requests`.

#### Scenario: Account locks after threshold failures (OTP)
- **WHEN** `auth_methods.otp.max_retries = 3` is configured
- **WHEN** 3 consecutive OTP verifications fail for the same login ID
- **WHEN** a 4th OTP verification is attempted
- **THEN** response is `429 Too Many Requests`

### Requirement: Auth method config is configurable via admin API and UI
The emulator SHALL expose `GET /v1/mgmt/config/auth-methods` (returns all method configs) and `PUT /v1/mgmt/config/auth-methods` (replaces all method configs). The admin UI SHALL save changes via these endpoints. Both endpoints require management key authentication.

#### Scenario: Auth method config is persisted in memory and survives reset
- **WHEN** `PUT /v1/mgmt/config/auth-methods` sets `otp.expiration_seconds = 60`
- **WHEN** `POST /emulator/reset` is called (re-applies seed)
- **THEN** `GET /v1/mgmt/config/auth-methods` still returns `otp.expiration_seconds = 60`
- **NOTE**: Auth method config is not cleared on reset (only user/tenant/token stores are reset)
