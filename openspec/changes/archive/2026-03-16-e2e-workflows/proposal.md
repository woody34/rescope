## Why

The descope-emulator has comprehensive Vitest integration test coverage of all API endpoints, but zero coverage of user-facing workflows driven through a real browser. As the Admin UI is built out, there's no automated gate to verify that UI actions actually produce correct emulator state — and no coverage of the unique emulator workflows like the OTP Inspector. A full E2E suite is needed to validate the emulator as a drop-in replacement for real Descope in CI.

## What Changes

- Add a top-level `e2e/` workspace with a standalone Playwright config that covers both API-seeded state setup and browser-driven UI interaction in the same test files
- Add a `test-app/` — a minimal HTML page using the Descope Web Component SDK (`@descope/web-component`) pointed at the emulator — for testing authentication flows end-to-end from the browser login UI through to session JWTs
- Add 7 Playwright workflow test suites covering: snapshot round-trip, users admin, permissions & roles, OTP Inspector, auth-method toggle, access keys, and tenant lifecycle
- Add a `test:e2e:playwright` script to the root `package.json` and wire Playwright into CI
- Rename the existing confusingly-named `"e2e"` root npm script to `"test:integration"` to remove ambiguity

## Capabilities

### New Capabilities

- `e2e-snapshot-workflow`: Export, reset, and import emulator snapshot via Admin UI; verify state restored via API
- `e2e-users-admin-workflow`: Create user via mgmt API, see in Users page, edit in UI, verify via API
- `e2e-permissions-roles-workflow`: Create permission and role in UI, assign to user, verify in JWT claims
- `e2e-otp-inspector-workflow`: Trigger OTP send via API, read pending OTP code from OTP Inspector UI page, complete verification
- `e2e-auth-method-toggle-workflow`: Disable an auth method in Admin UI, verify API returns 4xx, re-enable, verify works
- `e2e-access-keys-workflow`: Create access key via Admin UI, use key to call mgmt API successfully
- `e2e-tenant-lifecycle-workflow`: Create tenant in UI, assign user, verify tenant claims appear in session JWT
- `e2e-auth-sdk-workflow`: Full browser-driven auth flow using Descope Web Component SDK pointed at emulator (password, OTP, magic link)

### Modified Capabilities

- (none — all new)

## Impact

- New directory: `e2e/` at root (Playwright config, workflow test files, POMs, test-app HTML)
- New npm scripts in root `package.json`
- Root `"e2e"` script renamed to `"test:integration"` (**BREAKING** for anyone calling that script directly)
- Playwright added as a devDependency at root workspace level
- `@descope/web-component` added to `e2e/test-app/` for SDK auth flow tests
- `ui/e2e/` existing tests and POMs remain; top-level `e2e/` is the new home for workflow suites
