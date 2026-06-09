## Why

The emulator covers 100% of its original PRD scope, but a gap analysis against the full Descope REST API surface revealed missing endpoints that limit testing fidelity for consumers. This change implements the next tier of endpoints: full OTP authentication flows, user status management, tenant role management, and bulk session revocation — bringing the emulator to production-grade completeness for the most common Descope integration patterns.

## What Changes

- **New**: `POST /v1/auth/otp/signup/email` — Sign up a new user via OTP; emulator returns `code` in body
- **New**: `POST /v1/auth/otp/signin/email` — Sign in via email OTP; emulator returns `code` in body
- **New**: `POST /v1/auth/otp/verify/email` — Verify email OTP code → session + refresh JWTs
- **New**: `POST /v1/auth/otp/signup/phone/sms` — Sign up via SMS OTP; emulator returns `code` in body
- **New**: `POST /v1/auth/otp/signin/phone/sms` — Sign in via SMS OTP; emulator returns `code` in body
- **New**: `POST /v1/auth/otp/verify/phone/sms` — Verify SMS OTP code → session + refresh JWTs
- **New**: `GET /emulator/otp/:loginId` — Emulator-specific; returns the last generated OTP code for a loginId (escape hatch for SDK-driven OTP flows)
- **New**: `POST /v1/mgmt/tests/generate/otp` — Generate OTP for a test user (management API)
- **New**: `POST /v1/auth/logoutall` — Revoke all refresh tokens for a given user (adds all known refresh tokens for that userId to the revocation set)
- **New**: `POST /v1/mgmt/user/tenant/remove` — Remove a user from a tenant
- **New**: `POST /v1/mgmt/user/tenant/setRole` — Replace a user's roles within a specific tenant
- **New**: `POST /v1/mgmt/user/status/enable` — Enable a disabled user (allows sign-in)
- **New**: `POST /v1/mgmt/user/status/disable` — Disable a user (blocks sign-in with `403 Forbidden`)

## Capabilities

### New Capabilities

- `otp-auth-flows`: Full OTP authentication: sign-up, sign-in, and verify for email and phone/SMS channels. Emulator deviation: OTP code returned directly in response body and via `GET /emulator/otp/:loginId`
- `otp-mgmt`: Management API extension for OTP: `POST /v1/mgmt/tests/generate/otp` for test-user OTP scenarios
- `user-status`: User enable/disable management — `status.enable` and `status.disable` endpoints; disabled users are blocked from all auth flows
- `tenant-mgmt`: Tenant membership mutations — `tenant/remove` and `tenant/setRole` for runtime tenant role management

### Modified Capabilities

- `session-flows`: `logoutAll` added — revokes all active refresh tokens for a user
- `mgmt-api`: Two new user management sub-endpoints (`tenant/remove`, `tenant/setRole`, `status/enable`, `status/disable`)

## Impact

- **New file**: `src/routes/auth/otp.rs` — expands existing OTP handler (currently only `update/phone/sms`) with sign-up/sign-in/verify for email and SMS
- **Modified**: `src/routes/auth/session.rs` — add `logoutall` handler
- **Modified**: `src/routes/mgmt/user.rs` — add `tenant/remove`, `tenant/setRole`, `status/enable`, `status/disable` handlers
- **Modified**: `src/store/user_store.rs` — add `disabled` flag to `User` struct; add `remove_tenant`, `set_tenant_roles`, `enable`, `disable` methods
- **Modified**: `src/server.rs` — register new routes
- **Modified**: `src/routes/emulator.rs` — add `GET /emulator/otp/:loginId` lookup handler
- **New integration tests**: `integration/sdk-js/tests/otp.sdk.test.ts`, additions to `session.sdk.test.ts`, additions to user management tests
- **Documentation**: README deviations table updated, OTP code retrieval section filled in
