## 1. Store Extensions — UserStore & TenantStore

- [x] 1.1 Write unit tests for `UserStore::update_login_id(old, new)` — success renames index entry and `login_ids` vec; new already taken → conflict; unknown old → not found; load by old loginId after rename → not found
- [x] 1.2 Implement `UserStore::update_login_id` — remove old key from `by_login_id`, insert new key, update `login_ids` vec; fail if new key already exists
- [x] 1.3 Write unit tests for `UserStore::set_roles(login_id, roles)` and `UserStore::remove_roles(login_id, roles_to_remove)`
- [x] 1.4 Implement `UserStore::set_roles` (replaces `role_names`) and `UserStore::remove_roles` (subtracts given roles from `role_names`)
- [x] 1.5 Write unit tests for `UserStore::batch_insert(users)` — all succeed returns all users; first duplicate stops and returns conflict; users before the error ARE persisted (non-transactional, assert via `load`)
- [x] 1.6 Implement `UserStore::batch_insert` — loop calling `insert`; return first `Err` if any
- [x] 1.7 Write unit tests for `TenantStore::create`, `update`, `delete`, `load`, `search` — no new Tenant fields needed, `Tenant` struct already has `id`, `name`, `self_provisioning_domains`, `auth_type`
- [x] 1.8 Implement `TenantStore::create(id?, name, domains?)`, `update(id, name?, domains?)`, `delete(id)`, `load(id) -> Result`, `search(ids?, names?)` in `src/store/tenant_store.rs`
- [x] 1.9 Update `src/store/mod.rs` to re-export `tenant_store` if missing

## 2. OTP Signup-In Handlers (TDD)

- [x] 2.1 Write failing unit tests for `otp::signup_in_email` — existing user gets OTP without creating a new user (assert user count unchanged); new user is created and gets OTP; returned OTP is verifiable via `otp::verify_email`
- [x] 2.2 Write failing unit tests for `otp::signup_in_phone_sms` — same pattern for phone channel
- [x] 2.3 Implement `otp::signup_in_email` in `src/routes/auth/otp.rs` — try `load` first; if `UserNotFound` then create; generate OTP, return `{ maskedEmail, code }`
- [x] 2.4 Implement `otp::signup_in_phone_sms` — same load-or-create pattern, return `{ maskedPhone, code }`
- [x] 2.5 Register `POST /v1/auth/otp/signup-in/email` and `POST /v1/auth/otp/signup-in/sms` in `src/server.rs`
- [x] 2.6 Add `signup-in` integration tests to `integration/sdk-js/tests/otp.sdk.test.ts` — email and SMS signup-in round trips (new user created, existing user signed in, token is valid)

## 3. Magic Link SMS & Signup-In Handlers (TDD)

- [x] 3.1 Write failing unit tests for `magic_link::signup_sms` — new user created + token returned; duplicate → 409
- [x] 3.2 Write failing unit tests for `magic_link::signin_sms` — existing user gets token; unknown → 401
- [x] 3.3 Write failing unit tests for `magic_link::signup_in_email` — existing user → token only (no new user, assert user count unchanged); new user → created + token
- [x] 3.4 Write failing unit tests for `magic_link::signup_in_sms` — same pattern for phone
- [x] 3.5 Write failing unit tests for `magic_link::update_phone_sms` — valid JWT + phone → phone updated on user record + token returned; invalid JWT → 401
- [x] 3.6 Implement `magic_link::signup_sms`, `signin_sms`, `signup_in_email`, `signup_in_sms`, `update_phone_sms` in `src/routes/auth/magic_link.rs`
- [x] 3.7 Register all new magic link routes in `src/server.rs`:
  - `POST /v1/auth/magiclink/signup/sms`
  - `POST /v1/auth/magiclink/signin/sms`
  - `POST /v1/auth/magiclink/signup-in/email`
  - `POST /v1/auth/magiclink/signup-in/sms`
  - `POST /v1/auth/magiclink/update/phone/sms`
- [x] 3.8 Add SMS magic link integration tests to `integration/sdk-js/tests/magic-link.sdk.test.ts` — signup, signin, signup-in via SMS round trips; token is valid

## 4. Bug Fix: magiclink/update/email Persistence (TDD)

- [x] 4.1 Write failing unit test: after `POST /v1/auth/magiclink/update/email` with `{ loginId, email }` and valid refresh JWT, `UserStore::load(loginId)` returns user with updated email field
- [x] 4.2 Fix `update_email` in `src/routes/auth/magic_link.rs` — add `users.patch(login_id, UserPatch { email: Some(req.email), .. })` after JWT validation; this was previously a no-op
- [x] 4.3 Verify no existing integration tests rely on the no-op behavior (search `update/email` in `integration/` tests)
- [x] 4.4 Add `update_email` persistence integration test to `integration/sdk-js/tests/magic-link.sdk.test.ts` — after update, subsequent `GET /v1/auth/me` returns new email

## 5. SAML Authorize Alias & SSO Generic (TDD)

- [x] 5.1 Write failing unit test: `POST /v1/auth/saml/authorize` returns same response as `/start` for a valid SAML user
- [x] 5.2 Register `POST /v1/auth/saml/authorize` in `src/server.rs` pointing to existing `saml::start` handler (keep `/start` alias)
- [x] 5.3 Write failing unit test: `POST /v1/auth/sso/authorize` → `{ url }` with code; `POST /v1/auth/sso/exchange` with that code → tokens
- [x] 5.4 Register `POST /v1/auth/sso/authorize` and `POST /v1/auth/sso/exchange` in `src/server.rs` pointing to `saml::start` and `saml::exchange`
- [x] 5.5 Add SAML `/authorize` alias integration test to `integration/sdk-js/tests/saml.sdk.test.ts`
- [x] 5.6 Add SSO generic round-trip integration test (authorize + exchange) to saml test file

## 6. Password Policy & Session Stubs (TDD)

- [x] 6.1 Write failing unit test: `GET /v1/auth/password/policy` → `{ active: true, minLength: 6, maxLength: 128 }`
- [x] 6.2 Implement `password_policy` handler in `src/routes/auth/password.rs` (or new file)
- [x] 6.3 Register `GET /v1/auth/password/policy` in `src/server.rs`
- [x] 6.4 Write failing unit test: `GET /v1/auth/me/history` with valid JWT → `{ users: [] }`; without JWT → 401
- [x] 6.5 Implement `me_history` stub handler in `src/routes/auth/session.rs`
- [x] 6.6 Register `GET /v1/auth/me/history` in `src/server.rs`
- [x] 6.7 Add `password/policy` integration test to `integration/sdk-js/tests/password.sdk.test.ts`
- [x] 6.8 Add `me/history` integration test to `integration/sdk-js/tests/session.sdk.test.ts` — valid JWT → 200 with empty list; no JWT → 401

## 7. Tenant Select (TDD)

- [x] 7.1 Write failing unit test: `POST /v1/auth/tenant/select` with valid refresh JWT + tenant user belongs to → new `sessionJwt` containing `dct = tenantId` claim; tenant not in user's tenants → 404; invalid JWT → 401
- [x] 7.2 Add optional `extra_claims: Option<HashMap<String, Value>>` parameter to `generate_session_jwt` (or a companion overload) to inject `dct` and any other top-level claims
- [x] 7.3 Implement `tenant_select` handler in `src/routes/auth/session.rs` — validate refresh JWT, verify user tenant membership, call updated `generate_session_jwt` with `dct = tenant_id`, return full JWTResponse (sessionJwt + refreshJwt + user)
- [x] 7.4 Register `POST /v1/auth/tenant/select` in `src/server.rs`
- [x] 7.5 Add tenant select integration test to `integration/sdk-js/tests/session.sdk.test.ts` — sign in, select tenant, decode token and assert `dct` claim matches tenant id; unknown tenant → 404

## 8. Tenant CRUD Management Endpoints (TDD)

- [x] 8.1 Write failing unit tests for `mgmt::tenant::create` — creates tenant; duplicate id → 409; auto-id generation
- [x] 8.2 Write failing unit tests for `mgmt::tenant::update` — updates name; unknown id → 404
- [x] 8.3 Write failing unit tests for `mgmt::tenant::delete` — removes tenant; idempotent
- [x] 8.4 Write failing unit tests for `mgmt::tenant::load` — returns single tenant; unknown → 404
- [x] 8.5 Write failing unit tests for `mgmt::tenant::search` — filters by name; empty filter returns all
- [x] 8.6 Implement all five handlers in `src/routes/mgmt/tenant.rs`
- [x] 8.7 Register new routes in `src/server.rs`:
  - `POST /v1/mgmt/tenant/create`
  - `POST /v1/mgmt/tenant/update`
  - `DELETE /v1/mgmt/tenant`
  - `GET /v1/mgmt/tenant`
  - `POST /v1/mgmt/tenant/search`
- [x] 8.8 Add tenant CRUD integration tests to `integration/sdk-js/tests/mgmt.sdk.test.ts` — create, load, update, search, delete round trip

## 9. User Field Update Endpoints (TDD)

- [x] 9.1 Write failing unit tests for `mgmt::user::update_name` — patches name; unknown loginId → 404
- [x] 9.2 Write failing unit tests for `mgmt::user::update_phone` — patches phone; unknown loginId → 404
- [x] 9.3 Write failing unit tests for `mgmt::user::update_login_id` — renames loginId; new loginId already taken → 409
- [x] 9.4 Write failing unit tests for `mgmt::user::set_roles` — replaces role_names; `remove_roles` subtracts specified roles
- [x] 9.5 Implement `update_name`, `update_phone`, `update_login_id`, `set_roles`, `remove_roles` in `src/routes/mgmt/user.rs`
- [x] 9.6 Register in `src/server.rs`:
  - `POST /v1/mgmt/user/update/name`
  - `POST /v1/mgmt/user/update/phone`
  - `POST /v1/mgmt/user/update/loginid`
  - `POST /v1/mgmt/user/update/role/set`
  - `POST /v1/mgmt/user/update/role/remove`
- [x] 9.7 Add `update/name`, `update/phone`, `update/loginid`, `update/role/set`, `update/role/remove` integration tests to `integration/sdk-js/tests/mgmt.sdk.test.ts`

## 10. User Batch Operations & Extended Management (TDD)

- [x] 10.1 Write failing unit tests for `mgmt::user::create_batch` — all succeed returns all users; partial failure on duplicate; first user IS persisted (non-transactional, assert via `load`)
- [x] 10.2 Write failing unit tests for `mgmt::user::delete_batch` — deletes all listed; non-existent ids silently ignored; returns `{ ok: true }`
- [x] 10.3 Write failing unit test for `mgmt::user::force_logout` — inserts user_revocations entry; subsequent `POST /v1/auth/refresh` with that user's token fails with 401/expired
- [x] 10.4 Write failing unit test for `mgmt::user::password_expire` — returns `{ ok: true }` for known user; 404 for unknown
- [x] 10.5 Write failing unit test for `mgmt::user::set_temporary_password` — bcrypt hash stored; user can sign in with that password via `password/signin`
- [x] 10.6 Write failing unit test for `mgmt::user::generate_enchanted_link_for_test_user` — test user gets token; token is consumable via `magiclink/verify` for session JWT; non-test user → 400
- [x] 10.7 Implement `create_batch`, `delete_batch`, `force_logout`, `password_expire`, `set_temporary_password`, `generate_enchanted_link_for_test_user` in `src/routes/mgmt/user.rs`
- [x] 10.8 Register in `src/server.rs` — **all batch routes are POST per Descope spec**:
  - `POST /v1/mgmt/user/create/batch`
  - `POST /v1/mgmt/user/delete/batch` ← POST not DELETE (confirmed from Descope YAML)
  - `POST /v1/mgmt/user/logout`
  - `POST /v1/mgmt/user/password/expire`
  - `POST /v1/mgmt/user/password/set/temporary`
  - `POST /v1/mgmt/tests/generate/enchantedlink`
- [x] 10.9 Add batch create/delete integration tests to `integration/sdk-js/tests/mgmt.sdk.test.ts`
- [x] 10.10 Add force-logout integration test to `integration/sdk-js/tests/session.sdk.test.ts` — after force logout, refresh token → 401
- [x] 10.11 Add `generate/enchantedlink` integration test to `integration/sdk-js/tests/mgmt.sdk.test.ts` — token is consumable via `magiclink/verify`

## 11. JWT Update Management Endpoint (TDD)

- [x] 11.1 Write failing unit test for `mgmt::jwt::update` — valid session JWT + `{ customClaims: { "appRole": "admin" } }` → new `{ jwt }` with that claim present when decoded; invalid session JWT → 401
- [x] 11.2 Create `src/routes/mgmt/jwt.rs` with `update` handler — accept `{ jwt, customClaims }`, validate session JWT, issue new session JWT with claims merged, return `{ jwt: <newSessionJwt> }` (single field, matches `managementv1.JWTResponse`)
- [x] 11.3 Register `POST /v1/mgmt/jwt/update` in `src/server.rs`
- [x] 11.4 Update `src/routes/mgmt/mod.rs` to expose `jwt` module
- [x] 11.5 Add JWT update integration test to `integration/sdk-js/tests/mgmt.sdk.test.ts` — custom claims appear in decoded session JWT

## 12. Verification

- [x] 12.1 Run `make test` (unit tests) — all pass: `cargo test --lib`
- [x] 12.2 Run `make test-integration` (SDK integration tests) — all pass
- [x] 12.3 Update `docs/gap-analysis.md` to reflect new coverage (target: ~80%+ of testable endpoints)
