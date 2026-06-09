## ADDED Requirements

### Requirement: Test app serves Descope Web Component pointed at emulator
A minimal static HTML test app (`e2e/test-app/index.html`) SHALL embed the Descope Web Component (`<descope-wc>`) configured with the emulator's project ID and base URL (`http://localhost:4600`), served by the Playwright `webServer` or a simple static file server.

#### Scenario: Test app loads without errors
- **WHEN** Playwright navigates to the test app URL
- **THEN** the page loads and the `<descope-wc>` element is present in the DOM

---

### Requirement: Password authentication flow works end-to-end via Web Component
A user SHALL be able to sign up and sign in using the Descope Web Component's password flow against the emulator, resulting in a valid session JWT stored in the browser.

#### Scenario: Password signup via Web Component produces session JWT
- **WHEN** Playwright fills in email and password in the Web Component's password form and submits
- **THEN** the component fires a `success` event and a valid `sessionJwt` is accessible (via event detail or localStorage)

#### Scenario: Password signin via Web Component produces session JWT
- **WHEN** a user exists in the emulator and Playwright completes the signin form in the Web Component
- **THEN** the component fires a `success` event with a `sessionJwt` that validates against `POST /v1/auth/validate`

---

### Requirement: OTP authentication flow works end-to-end via Web Component + OTP Inspector
A user SHALL be able to complete an OTP sign-in flow by: triggering OTP via the Web Component, reading the code from the emulator's `/emulator/otps` API endpoint, and entering it back into the Web Component.

#### Scenario: OTP flow completes with Inspector-sourced code
- **WHEN** Playwright submits an email in the OTP form, fetches the code from `GET /emulator/otps`, and enters it in the OTP verification input
- **THEN** the component fires a `success` event and the resulting `sessionJwt` is valid

---

### Requirement: Magic link authentication completes via token injection
Since magic link emails are not delivered, the emulator's OTP/token store SHALL expose the pending magic link token via the management API. A test SHALL simulate clicking the magic link by navigating to the verify URL with the token.

#### Scenario: Magic link token from API completes auth flow
- **WHEN** magic link signin is triggered and the token is fetched from `GET /emulator/otps` (or a dedicated endpoint), and Playwright navigates to `/auth/magiclink/verify?token=<token>`
- **THEN** the resulting page or redirect contains a valid `sessionJwt`
