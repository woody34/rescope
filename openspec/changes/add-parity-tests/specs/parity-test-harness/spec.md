## ADDED Requirements

### Requirement: Target-agnostic SDK configuration
The `platform.ts` helper SHALL configure the `DescopeClient` based on `PARITY_TARGET`. When `"emulator"` or unset, it uses emulator defaults. When `"live"`, it reads `DESCOPE_PROJECT_ID` and `DESCOPE_MANAGEMENT_KEY` from env (required), and `DESCOPE_BASE_URL` (optional).

#### Scenario: Emulator target
- **WHEN** `PARITY_TARGET` is `"emulator"` or unset
- **THEN** SDK is configured with `baseUrl: "http://localhost:4501"`, `projectId: "emulator-project"`, `managementKey: "emulator-key"`

#### Scenario: Live target
- **WHEN** `PARITY_TARGET` is `"live"`
- **THEN** SDK is configured from `DESCOPE_PROJECT_ID` and `DESCOPE_MANAGEMENT_KEY`, with `DESCOPE_BASE_URL` used only if set

#### Scenario: Missing live credentials
- **WHEN** `PARITY_TARGET` is `"live"` and a required env var is missing
- **THEN** the helper throws an error identifying the missing variable

#### Scenario: Live safety acknowledgement
- **WHEN** `PARITY_TARGET` is `"live"` and `DESCOPE_PARITY_ACKNOWLEDGED` is not `"1"`
- **THEN** the helper throws explaining that live parity tests delete all test users project-wide

### Requirement: Branching cleanup
The `cleanup()` function SHALL use `/emulator/reset` when targeting emulator, and SDK deletion when targeting live. Live cleanup calls `deleteAllTestUsers()` then iterates tracked resources in a flat `try/catch` loop (no ordering required).

#### Scenario: Emulator cleanup
- **WHEN** `cleanup()` runs with emulator target
- **THEN** `POST /emulator/reset` is called

#### Scenario: Live cleanup
- **WHEN** `cleanup()` runs with live target
- **THEN** `deleteAllTestUsers()` is called, then each tracked resource is deleted individually with errors swallowed

#### Scenario: Resource tracking
- **WHEN** a test calls `trackResource("role", "editor")` and then `cleanup()` runs
- **THEN** the role `"editor"` is deleted via `sdk.management.role.delete("editor")`

#### Scenario: Regular user tracking for auth sign-up flows
- **WHEN** a test calls `trackResource("user", loginId)` after `password.signUp()`
- **THEN** `cleanup()` deletes the user via `sdk.management.user.delete(loginId)`

#### Scenario: Cleanup is idempotent
- **WHEN** `cleanup()` runs and a tracked resource was already deleted
- **THEN** cleanup completes without error

### Requirement: OTP code retrieval via SDK test helpers
The existing `/emulator/otp/{loginId}` fetch calls SHALL be replaced with `sdk.management.user.generateOTPForTestUser("email", loginId)`. The initiation endpoint (`otp.signUpOrIn.email()`) SHALL be called as a separate explicit assertion in the test body, not hidden inside a helper.

#### Scenario: OTP retrieval works on both targets
- **WHEN** `generateOTPForTestUser("email", loginId)` is called for an existing test user
- **THEN** a 6-digit code is returned that can be verified via `otp.verify.email()`

### Requirement: Magic link token retrieval via SDK test helpers
The existing `/emulator/otp/{loginId}` fetch for magic link tokens SHALL be replaced with `sdk.management.user.generateMagicLinkForTestUser("email", loginId, uri)`. Initiation is tested as a separate assertion.

#### Scenario: Magic link retrieval works on both targets
- **WHEN** `generateMagicLinkForTestUser("email", loginId, uri)` is called for an existing test user
- **THEN** a token is returned that can be verified via `magicLink.verify()`

### Requirement: Strong default password
The helper SHALL export a `TEST_PASSWORD` constant satisfying common password policies (min length, uppercase, lowercase, digit, special character).

#### Scenario: Password accepted on live Descope
- **WHEN** `TEST_PASSWORD` is used with `password.signUp()` or `setActivePassword()` on live
- **THEN** the operation succeeds without policy rejection

### Requirement: Authenticated test user helper
The helper SHALL export a `signUpTestUser(client, loginId, password?)` that creates a test user, sets an active password, signs in, and returns the session.

#### Scenario: Quick authenticated session
- **WHEN** `signUpTestUser(client, loginId)` is called
- **THEN** a test user exists with an active password and valid session JWTs are returned

### Requirement: Response shape assertion helper
The helper SHALL export an `assertMatchingKeys(actual, expected)` function that validates two objects have the same key structure and value types, without comparing values.

#### Scenario: Matching shapes pass
- **WHEN** `assertMatchingKeys({ id: "abc", ok: true }, { id: "xyz", ok: false })` is called
- **THEN** the assertion passes (same keys, same types)

#### Scenario: Missing key fails
- **WHEN** `assertMatchingKeys({ id: "abc" }, { id: "xyz", name: "foo" })` is called
- **THEN** the assertion fails identifying `name` as missing

#### Scenario: Type mismatch fails
- **WHEN** `assertMatchingKeys({ count: "5" }, { count: 5 })` is called
- **THEN** the assertion fails identifying `count` as string vs number

### Requirement: Separate vitest configuration for parity runs
A `vitest.parity.config.ts` SHALL run the same test files as the existing config but without `globalSetup` (no auto-started emulator).

#### Scenario: Parity config runs existing tests without emulator auto-start
- **WHEN** `vitest run -c vitest.parity.config.ts` is executed
- **THEN** the existing scenario tests run, and no emulator process is started

### Requirement: JWT claim shorthand fallback
All JWT claim assertions SHALL check both long-form and shorthand keys: `roles`/`r`, `tenants`/`t`, `permissions`/`p`.

#### Scenario: Claims found via either key
- **WHEN** a JWT contains `r: ["admin"]` but not `roles`
- **THEN** assertions that check for role presence still pass
