# Descope Emulator — Feature Summaries

_Generated: 2026-03-14_

Per-feature documentation of what the emulator implements and how it behaves.

---

## 1. Infrastructure

### 1.1 Health Check — `GET /health`

Returns `{ "ok": true }` immediately. Used by test runners and CI to poll for emulator readiness before running tests. This endpoint does not exist on the real Descope API — it is an emulator-specific extension.

### 1.2 State Reset — `POST /emulator/reset`

Clears all in-memory state (users, tenants, pending tokens, revocation set) and re-applies the seed file if one was configured at startup. This is an emulator-specific extension with no Descope equivalent. It is the primary mechanism for test isolation — call it between test suites to get a clean slate.

### 1.3 JWKS — `GET /.well-known/jwks.json` and `GET /v2/keys/:project_id`

Both endpoints return an RFC 7517 JWK Set containing the emulator's RSA public key. All Descope SDKs call this endpoint at startup to obtain the key used to verify session JWTs. The emulator generates an RSA key pair at startup (or loads one from `DESCOPE_KEY_FILE`) and uses it to sign all tokens. The two paths exist because different SDK versions use different URL patterns.

---

## 2. Auth — Password

### 2.1 Sign Up — `POST /v1/auth/password/signup`

Creates a new user from a `loginId` and `password`, optionally enriched by a `user` object containing `email`, `phone`, `name`, `givenName`, and `familyName`. The password is bcrypt-hashed (cost 10). On success, issues a session JWT and refresh JWT and sets `DS`/`DSR` cookies. Returns the full `AuthenticationResponse` shape including `sessionJwt`, `refreshJwt`, `user`, and cookie metadata. Fails with `409 Conflict` if `loginId` is already taken.

### 2.2 Sign In — `POST /v1/auth/password/signin`

Authenticates an existing user with `loginId` and `password`. Verifies the bcrypt hash stored on the user. Returns the same `AuthenticationResponse` as sign up. Fails with `401 Unauthorized` if the password is wrong or the user is not found.

### 2.3 Replace Password — `POST /v1/auth/password/replace`

Changes a user's password given their current (`oldPassword`) and new (`newPassword`) passwords, both alongside `loginId`. Verifies the old password hash before writing the new one. Returns a fresh `AuthenticationResponse` (new session and refresh tokens). Used by an authenticated change-password flow where the user already knows their current password.

### 2.4 Initiate Password Reset — `POST /v1/auth/password/reset`

Accepts a `loginId` and stores a short-lived reset token in the token store. **Deviation from Descope:** The emulator returns the reset token directly in the response body (`{ "token": "..." }`) for testability — the real Descope API sends an email and returns nothing meaningful. Also returns a masked email address (`al***@acme.com`).

### 2.5 Complete Password Reset — `POST /v1/auth/password/update`

Accepts a `loginId`, `newPassword`, and `token`. The token may arrive either in the `Authorization: Bearer projectId:token` header (as the SDK sends it) or in the request body. Consumes the token (single-use), then updates the bcrypt hash. Returns `{ "ok": true }`.

---

## 3. Auth — Magic Link

### 3.1 Initiate Sign-In — `POST /v1/auth/magiclink/signin/email`

Accepts a `loginId` and optional `URI`. Verifies the user exists, generates a random magic-link token, stores it associated with the user's `userId`, and returns the masked email. **Deviation from Descope:** The response includes the raw `token` for testability — the real API sends an email. Used by both the Node SDK (`magicLink.signIn.email`) and from test code.

### 3.2 Verify Token — `POST /v1/auth/magiclink/verify`

Accepts a `token`. Looks it up in the token store, retrieves the associated user, removes the token (single-use), and returns a full `AuthenticationResponse`. This endpoint is the completion step for all magic link flows — used by both browser login pages and backend test automation.

### 3.3 Update Email — `POST /v1/auth/magiclink/update/email`

Accepts `loginId`, `email`, and optionally an existing session `token` for authorization. If a token is provided it validates it as a refresh JWT. **Known gap:** The email field on the user record is not actually updated — the endpoint returns `{ "ok": true }` without persisting the new email. This is sufficient for the current email-verification flow but would not survive a subsequent `me()` call expecting the new address.

---

## 4. Auth — OTP

### 4.1 Update Phone via SMS — `POST /v1/auth/otp/update/phone/sms`

Accepts `loginId`, `phone`, and optional `options.addToLoginIDs`. Updates the `phone` field on the user record and sets `verifiedPhone: true`. If `addToLoginIDs` is `true`, also pushes the phone number into the user's `loginIds` array (making the phone a valid login identifier). No SMS is ever sent. Returns `{ "ok": true }`.

Full OTP sign-up, sign-in, and verify flows for both email and SMS are also implemented. See the API Endpoints section in the README for the complete list.

---

## 5. Auth — SAML / SSO

### 5.1 Start SAML Flow — `POST /v1/auth/saml/start`

Accepts a `tenant` (either a raw tenant ID or a user's email address) and optional `redirectUrl`.

- **Email input:** Looks up the user by email, finds their SAML/OIDC-typed tenant, generates a one-time code, and stores it keyed to the `userId`.
- **Tenant ID input:** Resolves the tenant, validates it has `authType: saml` or `oidc`, generates a code keyed to `"tenant:{id}"` (cannot be exchanged for a session without a real user).

Returns `{ "url": "{redirectUrl}?code={code}" }`. The real Descope API redirects the browser to an actual IdP — the emulator skips that step, handing the code directly to the caller. This works because the consuming app's SSO flow extracts the `code` from the URL and calls `saml.exchange` directly.

### 5.2 Exchange SAML Code — `POST /v1/auth/saml/exchange`

Accepts a `code`. Looks up the code in the token store. If valid and resolved to a real user ID (not a `"tenant:"` prefix), retrieves the user and issues a full `AuthenticationResponse`. Codes are single-use. Fails if the code resolves to a tenant-level reference (i.e., `saml.start` was called with a tenant ID, not a user email).

---

## 6. Session Management

### 6.1 Refresh Session — `POST /v1/auth/refresh`

Issues new session and refresh JWTs given a valid, non-revoked refresh JWT. The refresh token may be provided via:

1. `Authorization: Bearer projectId:jwtToken` header (SDK default, strips the `projectId:` prefix)
2. `DSR` cookie (browser-based flows)
3. `refreshJwt` in the request body

Checks the token against the revocation set before validating the JWT signature. On success, returns a fresh `AuthenticationResponse` with new tokens and updated `Set-Cookie` headers.

### 6.2 Logout — `POST /v1/auth/logout`

Accepts the same token sources as refresh. Validates the JWT signature first, then adds the raw token string to an in-memory revocation set. Subsequent calls to `refresh` or `me` with that token will fail with `401`. Returns `{ "ok": true }`.

### 6.3 Get Current User — `GET /v1/auth/me`

Reads the refresh token from the `Authorization` header or `DSR` cookie, checks revocation, validates the JWT, and returns `{ "user": UserResponse }` for the associated user. Used by the web app's `descopeSdk.me()` to reload the user profile after attribute changes.

### 6.4 Validate Session — `POST /v1/auth/validate`

Accepts a `sessionJwt` in the body. Verifies the signature and expiry using the emulator's RSA public key (the same key exposed via JWKS). Returns `{ "jwt": "<token>", "token": <decoded_claims>, "cookies": [] }`. This is the endpoint called by the Node SDK's `sdk.validateSession()` — it is the critical path for the backend's auth guard.

---

## 7. Management — Users

All management endpoints require `Authorization: Bearer {projectId}:{managementKey}`. The emulator validates this against `EMULATOR_PROJECT_ID` and `EMULATOR_MANAGEMENT_KEY`.

### 7.1 Create User — `POST /v1/mgmt/user/create`

Creates a regular user with a generated UUID `userId`. Accepts: `loginId`, `email`, `phone`, `name`, `givenName`, `familyName`, `roleNames`, `userTenants`, `customAttributes`, `verifiedEmail`, `verifiedPhone`. Returns `{ "user": UserResponse }`. Fails if `loginId` already exists.

### 7.2 Create Test User — `POST /v1/mgmt/user/create/test`

Same as create, but sets the internal `_is_test_user` flag. Test users are excluded from `search` by default unless `withTestUser: true`. They can be bulk-deleted via `DELETE /v1/mgmt/user/test/delete/all`. This flag enables the `POST /v1/mgmt/tests/generate/magiclink` endpoint.

### 7.3 Load User by LoginId — `GET /v1/mgmt/user?loginid=…`

Looks up a user where any entry in their `loginIds` array matches the provided value. Returns `{ "user": UserResponse }` or 404.

### 7.4 Load User by UserId — `GET /v1/mgmt/user/userid?userid=…`

Looks up a user by their internal UUID. Returns `{ "user": UserResponse }` or 404.

### 7.5 Search Users — `POST /v1/mgmt/user/search`

Filters the in-memory store. Supported filters:

- `emails`: Match users whose `email` is in the provided list
- `phones`: Match users whose `phone` is in the provided list
- `customAttributes`: Partial match — all provided key/value pairs must exist on the user
- `withTestUser`: When `true`, includes test-flagged users in results (default: excluded)
- `page` (0-indexed) and `limit` (default 100) for pagination

Returns `{ "users": UserResponse[] }`.

### 7.6 Update User — `POST /v1/mgmt/user/update`

Full replace of all mutable user fields. Fields not provided in the payload are cleared/reset to their zero value. Accepts the same field set as `create` plus `middleName`, `picture`, `verifiedEmail`, `verifiedPhone`. Returns `{ "user": UserResponse }`.

### 7.7 Patch User — `PATCH /v1/mgmt/user/patch`

Partial update — only fields present in the payload are written. Omitted fields preserve their current values. Uses the same request shape as `update`. Returns `{ "user": UserResponse }`.

### 7.8 Update Email — `POST /v1/mgmt/user/update/email`

Convenience endpoint for changing a user's email and verification state together. Accepts `loginId`, `email`, and optional `verified` boolean. Implemented via the same patch logic as `user_patch`. Returns `{ "user": UserResponse }`.

### 7.9 Set Active Password — `POST /v1/mgmt/user/password/set/active`

Allows management code to set a user's password directly (bypasses the reset flow). Bcrypt-hashes the provided `password` and stores it on the user. The user can then authenticate via `password.signIn`. Returns `{ "ok": true }`.

### 7.10 Delete by LoginId — `DELETE /v1/mgmt/user?loginid=…`

Removes the user matching the given `loginId`. Idempotent — deleting a non-existent user returns `{ "ok": true }`.

### 7.11 Delete by UserId — `DELETE /v1/mgmt/user/userid?userid=…`

Removes the user matching the given `userId`. Idempotent.

### 7.12 Delete All Test Users — `DELETE /v1/mgmt/user/test/delete/all`

Removes every user with `_is_test_user: true`. Regular users are untouched. Used for test cleanup.

### 7.13 Add Tenant — `POST /v1/mgmt/user/tenant/add`

Associates a user with a tenant. Accepts `loginId`, `tenantId`, and optional `roleNames`. Resolves the tenant name from the tenant store if the tenant exists. The `UserTenant` entry is appended to the user's `userTenants` array. This information appears in the user's session JWT claims and in `UserResponse.userTenants`. Idempotent.

### 7.14 Generate Embedded Link — `POST /v1/mgmt/user/embeddedlink`

Generates a single-use `Embedded` token for a user identified by `loginId`. Returns `{ "token": "..." }`. The token can be used in a magic-link-style verify flow to create a session without user interaction — used for link-based login scenarios.

---

## 8. Management — Test Flows

### 8.1 Generate Magic Link for Test User — `POST /v1/mgmt/tests/generate/magiclink`

Generates a magic link token for a **test user** (created via `POST /v1/mgmt/user/create/test`). Fails if the user is not flagged as a test user. Accepts `loginId` and optional `URI`. Returns:

```json
{
  "link": "{URI}?token={token}",
  "token": "{token}",
  "maskedEmail": "{email}"
}
```

The `link` field contains the full URL with the token query parameter. The `token` can be passed directly to `POST /v1/auth/magiclink/verify` to obtain a session. This is the primary mechanism for automated test logins in the backend's test service.

---

## 9. Management — Tenants

### 9.1 Load All Tenants — `GET /v1/mgmt/tenant/all`

Returns all tenants from the in-memory tenant store: `{ "tenants": Tenant[] }`. The tenant store is populated at startup from the seed file or via the management API. Each tenant has `id`, `name`, `domains`, and `authType`. Full tenant CRUD is implemented — see the API Endpoints section in the README.

---

## 10. JWT & Token Architecture

### Session JWT

Short-lived (default 3600 seconds). Signed with the emulator's RSA private key. Contains:

- `sub` — userId
- `iss` — projectId
- `exp` — expiry timestamp
- `email`, `name`, `phone`, `email_verified`, `phone_verified`
- `tenants` — map of tenantId → `{ roleNames }` for all userTenants
- `roles` — global role names array
- `custom_attributes` — user's custom attribute map

### Refresh JWT

Long-lived (default 2592000 seconds / 30 days). Contains only `sub` (userId), `iss`, and `exp`. Used exclusively to obtain new session JWTs. The revocation set stores raw refresh JWT strings — revoked tokens are rejected without signature checks.

### Token Store

One-time-use tokens (magic link, SAML code, password reset, embedded link) are stored in a keyed map. Each entry has: `token`, `userId`, and `tokenType`. Tokens are consumed (deleted) on first use using an atomic read-then-delete operation. Token types: `Magic`, `Saml`, `Reset`, `Embedded`.

---

## 11. Seed File

Users and tenants can be pre-loaded via `DESCOPE_SEED_FILE`. At startup (and on `POST /emulator/reset`), the file is parsed and its contents replace the in-memory store.

**User seed fields**: `loginId`, `email`, `name`, `phone`, `password` (plain text — hashed on load), `verifiedEmail`, `verifiedPhone`, `isTestUser`, `customAttributes` (object), `tenantIds` (resolved to `userTenants` entries), `roleNames`.

**Tenant seed fields**: `id`, `name`, `domains` (array), `authType` (`"none"` | `"saml"` | `"oidc"`).

**IdP emulator seed fields**: `protocol` (`"oidc"` | `"saml"`), `displayName`, `tenantId`, `attributeMapping` (object mapping IdP claim names to user fields).

Seeded users with a `password` can immediately sign in via `password.signIn`. Seeded users with `isTestUser: true` can immediately use `POST /v1/mgmt/tests/generate/magiclink`.

---

## 12. CORS & Cookie Behavior

The emulator enables CORS for all origins using `AllowOrigin::mirror_request` (mirrors the incoming `Origin` header back) with `Access-Control-Allow-Credentials: true`. This is required for browser-based apps (e.g. running on `:4200` or `:3000`) that make credentialed cross-origin requests to the emulator on `:4600`.

Authentication responses that issue tokens also set two `Set-Cookie` headers:

- `DS=<sessionJwt>; HttpOnly; SameSite=Lax; Path=/`
- `DSR=<refreshJwt>; HttpOnly; SameSite=Lax; Path=/; Max-Age=<refresh_ttl>`

`SameSite=Lax` is used (not `Strict`) because cross-port requests on `localhost` are treated as cross-site by browsers under the `Strict` policy.

---

## 13. IdP Emulator

Rescope can emulate external OIDC and SAML identity providers for SSO testing — no real Okta, Azure AD, or Auth0 account required.

### 13.1 Identity Provider Management

IdP emulators are managed via `POST /v1/mgmt/idp` (create), `GET /v1/mgmt/idp/all` (list), `POST /v1/mgmt/idp/update` (update), and `POST /v1/mgmt/idp/delete` (delete). Each IdP has a `protocol` (oidc or saml), `displayName`, `tenantId`, and `attributeMapping` (maps claim names to user fields). The admin UI provides a full management interface at `/identity-providers`.

### 13.2 OIDC IdP

The emulator exposes standard OIDC endpoints per IdP: discovery (`/.well-known/openid-configuration`), JWKS, authorize, and token. The authorize endpoint renders a user picker UI (or accepts a `login_id` query param for programmatic use). On user selection, it generates an authorization code and redirects to the `redirect_uri`. The token endpoint exchanges codes for `id_token` and `access_token` JWTs signed with a separate IdP key pair.

### 13.3 SAML IdP

The emulator serves SAML EntityDescriptor metadata XML and an SSO endpoint per IdP. The SSO endpoint renders the same user picker UI. On user selection, it generates a SAML Response XML (with NameID, AuthnStatement, and AttributeStatement populated from the attribute mapping) and returns an auto-submit HTML form that POSTs to the tenant's ACS URL. X.509 certificates are generated via `rcgen` (no native dependencies).

### 13.4 SP Callback Flow

Both OIDC and SAML flows complete by redirecting through an SP-side callback that generates an SP authorization code. This code can be exchanged for session JWTs via `POST /v1/auth/saml/exchange` or `POST /v1/auth/sso/exchange`, matching the real Descope SDK flow.
