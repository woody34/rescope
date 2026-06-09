## ADDED Requirements

### Requirement: Parity test harness (feature-flagged)

The parity test harness SHALL be compiled only when `--features parity` is passed to `cargo test`. It SHALL NOT be included in the default build or the standard test suite. It lives under `tests/parity/`.

#### Scenario: Parity tests do not run in default cargo test

- **WHEN** `cargo test` is run without `--features parity`
- **THEN** no parity test files are compiled or executed

#### Scenario: Parity tests run when feature is enabled

- **WHEN** `cargo test --features parity` is run with required env vars set
- **THEN** all parity test scenarios execute against both live Descope and the local emulator

---

### Requirement: Structural response diffing

For each parity scenario, the harness SHALL send the same request to both the live Descope API and the local emulator and compare:

- HTTP status code (exact match)
- Response JSON field presence and types (not values)
- Error `errorCode` field (exact match when present)
- Cookie header presence (`Set-Cookie: DS=...` and `Set-Cookie: DSR=...`)
- JWT claim key presence (not values) when the response contains a JWT

The harness SHALL NOT compare raw field values (JWTs, UUIDs, timestamps, masked emails differ by design).

#### Scenario: Matching response shapes pass parity

- **WHEN** both live Descope and the emulator return `{ "ok": true, "sessionJwt": "...", "user": { "userId": "...", ... } }`
- **THEN** the parity check passes (same keys, same types, same `ok` value)

#### Scenario: Mismatched error codes fail parity

- **WHEN** live Descope returns `errorCode: "E062108"` and the emulator returns a different or missing code
- **THEN** the parity check fails with a diff report

---

### Requirement: Parity test coverage

The parity harness SHALL have one scenario for every endpoint the emulator implements. The minimum required scenarios correspond to the 62 contract test cases listed in the PRD (Section 10.5).

#### Scenario: Every emulator endpoint has a parity scenario

- **WHEN** a new endpoint is added to the emulator
- **THEN** a corresponding parity scenario MUST be added to `tests/parity/` before the PR is merged

---

### Requirement: Parity test cleanup

Each parity scenario SHALL clean up any Descope test state it creates (delete created users, consume tokens) in an `after_each` teardown step. The cleanup MUST run even if the scenario fails.

#### Scenario: Test users are deleted after parity scenario

- **WHEN** a parity scenario creates a user in live Descope and the scenario completes (pass or fail)
- **THEN** the user is deleted from live Descope via `management.user.delete`

---

### Requirement: Parity test environment configuration

Parity tests SHALL read live Descope credentials from environment variables. Tests SHALL be skipped (not failed) if these variables are unset.

| Variable                        | Description                                              |
| ------------------------------- | -------------------------------------------------------- |
| `DESCOPE_PARITY_PROJECT_ID`     | Descope test project ID                                  |
| `DESCOPE_PARITY_MANAGEMENT_KEY` | Management key for the test project                      |
| `DESCOPE_PARITY_BASE_URL`       | Descope API base URL (default `https://api.descope.com`) |

#### Scenario: Parity tests skip gracefully without credentials

- **WHEN** `DESCOPE_PARITY_PROJECT_ID` is not set
- **THEN** all parity tests are skipped with a clear message, not failed
