## 1. Helper refactoring

- [ ] 1.1 Add target switching to `helpers/platform.ts` — read `PARITY_TARGET`, configure SDK from env vars for live, keep emulator defaults. Add safety guard requiring `DESCOPE_PARITY_ACKNOWLEDGED=1` for live.
- [ ] 1.2 Add `cleanup()` with branching: emulator uses `/emulator/reset`, live uses `deleteAllTestUsers()` + flat `try/catch` loop over tracked resources. Add `trackResource(type, id)`.
- [ ] 1.3 Add `TEST_PASSWORD` constant, `signUpTestUser()` helper, and `assertMatchingKeys()` shape assertion helper.
- [ ] 1.4 Replace OTP/magic link escape hatches in helper: add functions wrapping `generateOTPForTestUser()` and `generateMagicLinkForTestUser()`.
- [ ] 1.5 Add JWT claim shorthand helper that checks both `roles`/`r`, `tenants`/`t`, `permissions`/`p`.

## 2. First test — password (prove the approach)

- [ ] 2.1 Refactor `01-password-onboarding.scenario.test.ts` — swap `reset` for `cleanup`, use `TEST_PASSWORD`, track regular users from `password.signUp()`.
- [ ] 2.2 Create `vitest.parity.config.ts` — same include pattern as existing config, no `globalSetup`.
- [ ] 2.3 Add `test:parity:emulator` and `test:parity:live` scripts to `package.json`.
- [ ] 2.4 Run password test against emulator — verify it passes.
- [ ] 2.5 Run password test against live Descope — document any gaps, adjust assertions or helpers as needed.

## 3. Remaining test refactoring (iterate based on learnings from step 2)

- [ ] 3.1 Refactor `02-otp-authentication` — replace `/emulator/otp/` with `generateOTPForTestUser`, add explicit initiation assertion.
- [ ] 3.2 Refactor `03-magic-link` — replace `/emulator/otp/` with `generateMagicLinkForTestUser`, add explicit initiation assertion.
- [ ] 3.3 Refactor `04-rbac` — swap `reset` for `cleanup`, track roles/permissions, add JWT claim shorthand checks.
- [ ] 3.4 Refactor `05-multi-tenant` — swap `reset` for `cleanup`, track tenants, add JWT claim shorthand checks.
- [ ] 3.5 Refactor `06-session-lifecycle` — swap `reset` for `cleanup`, use `signUpTestUser`.
- [ ] 3.6 Refactor `07-user-lifecycle` — swap `reset` for `cleanup`.
- [ ] 3.7 Refactor `08-jwt-enrichment` — swap `reset` for `cleanup`, parameterize `BASE_URL`/`MGMT_AUTH_HEADER`, track JWT templates.
- [ ] 3.8 Refactor `09-access-keys` — swap `reset` for `cleanup`, parameterize `BASE_URL`/`MGMT_AUTH_HEADER`.
- [ ] 3.9 Refactor `10-embedded-link` — swap `reset` for `cleanup`, parameterize `BASE_URL`/`MGMT_AUTH_HEADER`.
- [ ] 3.10 Refactor `11-marketplace-tenant-select` — swap `reset` for `cleanup`, parameterize `BASE_URL`.

## 4. Validation

- [ ] 4.1 Run full suite against emulator — all 11 tests pass.
- [ ] 4.2 Run full suite against live Descope — document parity gaps found, log error responses for error code catalog.
