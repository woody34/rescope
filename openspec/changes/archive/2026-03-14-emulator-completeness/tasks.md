## 1. Infrastructure — OtpStore + User Status + logoutAll State

- [x] 1.1 Create `src/store/otp_store.rs` — `OtpStore` with `store(user_id, code)`, `peek(user_id) -> Option<&str>`, `consume(user_id, code) -> Result<(), EmulatorError>`, `reset()`
- [x] 1.2 Write unit tests for `OtpStore` — store+peek, store+consume (correct), consume wrong code, consume after consume, reset
- [x] 1.3 Add `otps: Arc<RwLock<OtpStore>>` to `EmulatorState` in `src/state.rs`
- [x] 1.4 Add `user_revocations: Arc<RwLock<HashMap<String, u64>>>` to `EmulatorState`
- [x] 1.5 Update `reset_stores()` to also reset `otps` and `user_revocations`
- [x] 1.6 Add `UserStore::disable(login_id)` — sets `user.status = "disabled"`; add `UserStore::enable(login_id)` — sets `user.status = "enabled"`
- [x] 1.7 Write unit tests for `UserStore::enable` / `UserStore::disable`
- [x] 1.8 Update `src/store/mod.rs` to re-export `otp_store`

## 2. OTP Auth Handlers

- [x] 2.1 Write failing test: `POST /v1/auth/otp/signup/email` — creates user with `email == loginId`, `verified_email: false`; duplicate loginId → 409
- [x] 2.2 Write failing test: `POST /v1/auth/otp/signup/phone/sms` — creates user, duplicate → 409
- [x] 2.3 Write failing test: `POST /v1/auth/otp/signin/email` — generates code for existing user; response has `maskedEmail` and `code`; unknown user → 401
- [x] 2.4 Write failing test: `POST /v1/auth/otp/signin/phone/sms` — same pattern for phone
- [x] 2.5 Write failing test: `POST /v1/auth/otp/verify/email` — valid code → `AuthenticationResponse` with `sessionJwt`; sets `verified_email: true`; single-use (second verify → 401); wrong code → 401
- [x] 2.6 Write failing test: `POST /v1/auth/otp/verify/phone/sms` — same pattern, sets `verified_phone: true`
- [x] 2.7 Implement all six OTP auth handlers in `src/routes/auth/otp.rs` — all tests pass
- [x] 2.8 Register six new routes in `src/server.rs`

## 3. OTP Emulator Escape Hatch + Mgmt

- [x] 3.1 Write failing test: `GET /emulator/otp/:loginId` — returns code without consuming; unknown loginId → 404
- [x] 3.2 Implement `GET /emulator/otp/:loginId` handler in `src/routes/emulator.rs` (lookup userId from loginId → peek OtpStore)
- [x] 3.3 Write failing test: `POST /v1/mgmt/tests/generate/otp` — request `{ deliveryMethod: "email", loginId }` → response `{ code, loginId }`; non-test user → 400
- [x] 3.4 Implement `POST /v1/mgmt/tests/generate/otp` handler in `src/routes/mgmt/user.rs`
- [x] 3.5 Register new routes in `src/server.rs`

## 4. Disabled User Blocking

- [x] 4.1 Write failing test: disable user via `user/status`, attempt `password/signin` → 403
- [x] 4.2 Add `if user.status == "disabled" { return Err(Forbidden) }` check to `password/signin` handler
- [x] 4.3 Add disabled check to `magiclink/verify`
- [x] 4.4 Add disabled check to `otp/verify/email` and `otp/verify/phone/sms`
- [x] 4.5 Add disabled check to `saml/exchange`
- [x] 4.6 Add disabled check to `auth/refresh` (after user load)

## 5. User Status Management Endpoint

- [x] 5.1 Write failing test: `POST /v1/mgmt/user/status` with `{ loginId, status: "disabled" }` → `{ user }` with `status: "disabled"`; then `{ status: "enabled" }` → `{ user }` with `status: "enabled"`; unknown loginId → 404
- [x] 5.2 Implement `status_update` handler in `src/routes/mgmt/user.rs` — calls `users.disable()` or `users.enable()` based on `status` field
- [x] 5.3 Register `POST /v1/mgmt/user/status` route in `src/server.rs`

## 6. Tenant Membership Mutations

- [x] 6.1 Write failing test: `POST /v1/mgmt/user/tenant/remove` — `{ loginId, tenantId }` → removes entry, returns `{ user }`; repeated call is idempotent; unknown user → 404
- [x] 6.2 Write failing test: `POST /v1/mgmt/user/tenant/setRole` — `{ loginId, tenantId, roleNames }` → replaces roles, returns `{ user }`; user not in tenant → 404
- [x] 6.3 Add `UserStore::remove_tenant(login_id, tenant_id)` and `UserStore::set_tenant_roles(login_id, tenant_id, role_names)` methods
- [x] 6.4 Implement `tenant_remove` and `tenant_set_role` handlers in `src/routes/mgmt/user.rs`
- [x] 6.5 Register both routes in `src/server.rs`

## 7. logoutAll

- [x] 7.1 Write failing test: `POST /v1/auth/logoutall` — revokes all tokens for user; a second session's refresh token is also rejected; fresh token after re-login works; invalid token → 401
- [x] 7.2 Implement `logout_all` handler in `src/routes/auth/session.rs` — extracts refresh JWT, validates, records `user_revocations[userId] = now_secs()`
- [x] 7.3 Add `user_revocations` timestamp check to `refresh` handler (after string-level revocation check): `if claims.iat <= revoked_at { return Err(TokenExpired) }`
- [x] 7.4 Add same timestamp check to `me` handler
- [x] 7.5 Register `POST /v1/auth/logoutall` route in `src/server.rs`

## 8. Integration Tests + Final Coverage Gate

- [ ] 8.1 Create `integration/sdk-js/tests/otp.sdk.test.ts` — email sign-up/sign-in/verify round-trip; escape hatch (`GET /emulator/otp/:loginId`); single-use code; wrong code; `mgmt.generateOTPForTestUser()`
- [ ] 8.2 Add `logoutall` test to `integration/sdk-js/tests/session.sdk.test.ts`
- [ ] 8.3 Add user status tests (disable blocks sign-in, enable restores) to `integration/sdk-js/tests/cross-flow.sdk.test.ts`
- [ ] 8.4 Add tenant remove + setRole tests to existing management integration tests
- [ ] 8.5 Run `make coverage` — verify 95%+ gate passes (`cargo llvm-cov --lib --fail-under-coverage 95`)
- [ ] 8.6 Run `make test-integration` — all integration tests pass
