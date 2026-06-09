## Context

The descope-emulator has 27 Vitest integration test files covering all API endpoints at the HTTP level. The Admin UI is now being built (emulator-admin-ui change, 24/68 tasks complete). The `ui/e2e/` directory has one Playwright test (snapshot round-trip) and two POMs (SnapshotPage, UsersPage). The root `"e2e"` npm script confusingly runs Vitest integration tests, not Playwright.

There are two fundamentally different E2E concerns:
1. **Admin UI workflows** — browser drives the Admin UI, API seeding sets up state, API verification confirms outcomes
2. **Auth SDK flows** — a minimal test app embeds the Descope Web Component; Playwright drives real login flows (password, OTP via OTP Inspector token) through to a session JWT

## Goals / Non-Goals

**Goals:**
- 7 Playwright workflow suites covering all Admin UI pages
- 1 Auth SDK workflow suite using a real Descope Web Component against the emulator
- OTP Inspector as a first-class tested workflow (unique to the emulator)
- Top-level `e2e/` directory with its own Playwright config, POMs, and test-app
- Root `package.json` scripts: `test:e2e` (Playwright), `test:integration` (replaces `e2e`)
- Tests written now, failing gracefully until UI pages are implemented (use `test.skip` or `test.fail` as-appropriate)

**Non-Goals:**
- OAuth/SAML E2E (require external IdP — skip)
- Load or performance testing
- Mobile browser testing
- Replacing Vitest integration tests (they stay and run separately)

## Decisions

### Decision 1: Single top-level `e2e/` over `ui/e2e/`

**Why**: The Admin UI tests need to mix API calls (mgmt endpoints, `/emulator/reset`) with browser interaction. Keeping them in `ui/` creates a tight coupling to the React app toolchain. A top-level `e2e/` directory has its own `playwright.config.ts`, deps, and runs independently of the UI build.

**Alternative considered**: Expand `ui/e2e/` — rejected because it would require running the Vite dev server alongside the emulator, adding startup complexity and making CI slower.

**Migration**: Existing `ui/e2e/smoke.test.ts` and its two POMs stay in `ui/e2e/` and continue running. The new top-level `e2e/` is additive.

---

### Decision 2: Hybrid API-seed + Playwright pattern for Admin UI tests

**Why**: Seeding state via the mgmt API is fast and reliable. Browser actions test UI correctness only for the things that actually require a browser (display, interaction, form submission). This pattern keeps tests focused and fast.

**Pattern**:
```
API Seed → Navigate to UI page → Assert display → 
Take UI action → Assert via API → Assert UI feedback
```

---

### Decision 3: `test-app/` with static Descope Web Component for auth SDK tests

**Why**: The Descope Web Component (`@descope/web-component`) is the canonical client SDK for browser-based auth. A minimal static HTML page using it pointed at `DESCOPE_PROJECT_ID` (emulator's project ID) + `baseURL` override allows real end-to-end browser auth flows without React, Vite, or any build step.

**Auth flows testable this way**:
- Password: fill email + password form → submit → assert JWT in localStorage/cookie
- OTP: fill email → submit → read OTP code from `/emulator/otps` API → fill OTP → assert JWT

**Alternative considered**: Mock the SDK or use the API directly — rejected because the point of these tests is to validate the web component works against the emulator, not just the API.

---

### Decision 4: OTP Inspector as unique emulator-specific workflow

**Pattern**:
1. POST `/v1/auth/otp/signup/email` via API → emulator stores OTP
2. Navigate to `/ui/emulator/otp-inspector` → table shows pending OTPs
3. Scrape the OTP code from the table row for our user
4. POST `/v1/auth/otp/verify/email` with scraped code → assert 200 + session JWT

This tests both the unique inspector UI and the OTP verify flow in one workflow.

---

### Decision 5: Rename root `"e2e"` script to `"test:integration"`

The current naming is misleading. The `"e2e"` script runs Vitest tests (HTTP-level API calls), not browser E2E. Renaming makes room for `"test:e2e"` to mean Playwright.

## Risks / Trade-offs

- **Admin UI pages not built yet** → Tests fail with `page.$('#selector')` not found. Mitigation: use `test.skip()` with `// TODO: un-skip when page X is built` comment. This documents intent without blocking CI.
- **Playwright flakiness** → `webServer` config waits for `/health`, but UI may take longer to render on first load. Mitigation: use `page.waitForSelector` before key assertions.
- **OTP race condition** → Two parallel test runs could see each other's OTPs. Mitigation: reset before each test, use unique email suffixes (`uniqueLogin()` pattern from Vitest helpers), filter OTP table by loginId.
- **Test app Web Component version drift** → The Descope web component may release breaking changes. Mitigation: pin to a specific version in `e2e/test-app/package.json`.

## Open Questions

- Should `test:e2e` in CI require the emulator to already be running, or should it `cargo build` itself? (Suggest: reuse existing server if running, `cargo build` in CI — same pattern as `ui/playwright.config.ts`)
- Should failing (skipped) tests block CI, or only warn? (Suggest: skip → warning, not failure)
