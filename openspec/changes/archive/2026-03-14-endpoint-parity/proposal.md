## Why

The emulator covers 100% of its original PRD scope but a crosswalk against the official Descope OpenAPI spec (`Descope_API.yaml`) reveals ~25% overall endpoint coverage (~43/175 endpoints). Before public release, full parity is required so that any Descope SDK integration test can run against the emulator without workarounds. The gaps that are testable without third-party dependencies — OTP signup-in variants, magic link SMS, tenant CRUD, password policy, user field updates, `signup-in` composite flows, and SAML route name alignment — are the targets for this change.

## What Changes

- **New**: `POST /v1/auth/otp/signup-in/email` — create-or-sign-in composite via OTP email
- **New**: `POST /v1/auth/otp/signup-in/sms` — create-or-sign-in composite via OTP SMS
- **New**: `POST /v1/auth/magiclink/signup-in/email` — create-or-sign-in composite via magic link
- **New**: `POST /v1/auth/magiclink/signup/sms` — sign up via magic link SMS (returns token in body; no SMS sent)
- **New**: `POST /v1/auth/magiclink/signin/sms` — sign in via magic link SMS (returns token in body)
- **New**: `POST /v1/auth/magiclink/signup-in/sms` — create-or-sign-in via magic link SMS
- **New**: `POST /v1/auth/magiclink/update/phone/sms` — update phone via magic link; returns token in body
- **New**: `GET /v1/auth/password/policy` — returns a static permissive policy object (minLength: 6)
- **RENAMED**: `POST /v1/auth/saml/start` → `POST /v1/auth/saml/authorize` — aligns to spec; old path kept as alias for backward compat
- **New**: `POST /v1/auth/sso/authorize` — generic SSO start; same implementation as SAML authorize
- **New**: `POST /v1/auth/sso/exchange` — generic SSO exchange; same implementation as SAML exchange
- **New**: `GET /v1/auth/me/history` — returns empty `[]` (emulator has no login history)
- **New**: `POST /v1/auth/tenant/select` — accepts tenant selection, re-issues tokens with tenant claim
- **New**: `POST /v1/mgmt/tenant/create` — create tenant at runtime
- **New**: `POST /v1/mgmt/tenant/update` — update tenant name/settings
- **New**: `DELETE /v1/mgmt/tenant` — delete tenant by id
- **New**: `GET /v1/mgmt/tenant` — load single tenant by id
- **New**: `POST /v1/mgmt/tenant/search` — filter tenants by name/id
- **New**: `POST /v1/mgmt/user/update/name` — update display name via management API
- **New**: `POST /v1/mgmt/user/update/phone` — update phone via management API
- **New**: `POST /v1/mgmt/user/update/loginid` — rename a user's primary login ID
- **New**: `POST /v1/mgmt/user/update/role/set` — set user's global roles
- **New**: `POST /v1/mgmt/user/update/role/remove` — remove user's global roles
- **New**: `POST /v1/mgmt/user/create/batch` — create multiple users in one call
- **New**: `DELETE /v1/mgmt/user/delete/batch` — delete multiple users by login ID
- **New**: `POST /v1/mgmt/user/logout` — force logout a user (mgmt)
- **New**: `POST /v1/mgmt/user/password/expire` — mark password as expired (returns ok; no actual enforcement needed for emulator)
- **New**: `POST /v1/mgmt/user/password/set/temporary` — same as set/active but marks password as temporary
- **New**: `POST /v1/mgmt/tests/generate/enchantedlink` — returns a token that can be consumed via `/v1/auth/magiclink/verify` (emulator shortcut)
- **New**: `GET /v2/mgmt/user/search` — v2 search (same behavior as v1 with `nextPage` cursor support)
- **New**: `POST /v1/mgmt/jwt/update` — returns the provided JWT with updated custom claims merged in

## Capabilities

### New Capabilities

- `otp-signup-in`: Composite OTP sign-up-or-sign-in flows for email and SMS channels
- `magiclink-sms`: Magic link flows for SMS channel (signup, signin, signup-in, update-phone)
- `magiclink-signup-in`: Magic link composite create-or-sign-in for email and SMS
- `password-policy`: Static password policy retrieval endpoint
- `sso-generic`: Generic SSO authorize/exchange endpoints (non-SAML)
- `tenant-select`: Tenant selection flow — re-issues tokens with chosen tenant claim
- `tenant-crud`: Full runtime tenant create/update/delete/load/search via management API
- `user-field-updates`: Granular user field update endpoints: name, phone, loginId, global roles
- `user-batch-ops`: Batch create and batch delete of users via management API
- `user-mgmt-extended`: Force logout, password expiry marking, temporary password set
- `enchanted-link-test`: Test utility for enchanted link token generation (mgmt escape hatch)
- `jwt-update`: Management API for updating custom claims on an existing JWT

### Modified Capabilities

- `saml-auth`: Route renamed from `/start` to `/authorize` with backward-compat alias; `signup-in` variant added
- `magiclink-auth`: `update/email` now persists the new email on the user record (was a no-op)
- `session-flows`: `me/history` stub added; `tenant/select` re-issue flow added

## Impact

- **Modified**: `src/routes/auth/otp.rs` — add `signup-in` handlers for email and SMS
- **Modified**: `src/routes/auth/magic_link.rs` — add SMS variants, `signup-in`, fix `update_email` persistence, add `update_phone_sms`
- **Modified**: `src/routes/auth/saml.rs` — rename handler registration + alias
- **Modified**: `src/routes/auth/session.rs` — add `me_history` stub, `tenant_select`
- **New**: `src/routes/auth/sso.rs` — generic SSO authorize/exchange (thin wrappers on SAML logic)
- **New**: `src/routes/auth/password_policy.rs` — static policy handler
- **Modified**: `src/routes/mgmt/tenant.rs` — add create, update, delete, load, search handlers
- **Modified**: `src/routes/mgmt/user.rs` — add name, phone, loginId, roles updates; batch create/delete; logout; password expire/temporary; enchanted link test generate
- **New**: `src/routes/mgmt/jwt.rs` — JWT update handler
- **Modified**: `src/server.rs` — register all new routes
- **Modified**: `src/store/user_store.rs` — add `update_login_id`, `set_roles`, `remove_roles`, `batch_insert`, `batch_delete` methods
- **Modified**: `src/store/tenant_store.rs` (if exists, else new) — full CRUD
- **New integration tests**: cover all new endpoints across existing test files
