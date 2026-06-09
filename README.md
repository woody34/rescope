# Rescope

A standalone Rust emulator for the [Descope](https://www.descope.com) authentication API. Run signup, login, OTP, magic links, SSO, and token validation locally — no network calls, so your tests and AI agents never touch the real Descope cloud.

> [!IMPORTANT]
> **Rescope is fully AI-generated.** It was built and tuned to support the Descope flows used by one real-world application, and *those* flows are exercised by automated integration and end-to-end tests: password auth, magic links, sessions (`/me`, refresh, validate), the management user & tenant APIs, SAML/SSO code exchange, and JWT/JWKS. **Features outside that surface may be incomplete or behave differently from real Descope** — they most likely just need more work, not a redesign.
>
> If you hit a gap or bug, please [open an issue](https://github.com/woody34/rescope/issues/new/choose) using the **Bug report** template. Support depends on you describing the technologies you've implemented and *exactly* how they call the Descope API (SDK + version, request, response). See [`docs/feature-summaries.md`](docs/feature-summaries.md) for the currently-tested surface and known gaps.

## Why Rescope?

Descope has no local development story: every auth flow requires a live request to its cloud. Rescope replaces that with a single binary.

- **Fast & offline** — auth completes in milliseconds, with no network, no rate limits, and no flaky CI.
- **Deterministic** — `POST /emulator/reset` returns to a clean, seeded state between runs.
- **Drop-in API surface** — same HTTP API, JWTs, JWKS, and management endpoints as Descope; existing SDK code works unchanged.
- **Admin UI** — inspect users, OTP codes, access keys, roles, tenants, and IdPs at `/`.
- **SSO without an IdP** — built-in OIDC and SAML emulation; no Okta, Azure AD, or Auth0 account required.

### A sandbox for AI agents

An AI coding agent pointed at a live auth API runs with real management keys and session tokens — credentials that can create or delete users, that leak into logs and chat history, and that drive real side effects at machine speed. Pointing the agent at Rescope keeps it in an isolated sandbox: no real credentials, no cloud-scoped keys, no persistent state to exfiltrate, and a clean reset every run. As agentic development becomes standard, a local auth sandbox shifts from convenience to security requirement.

## Quick Start

### Download a binary

Grab the latest release for your platform from [GitHub Releases](https://github.com/woody34/rescope/releases/latest):

```bash
# macOS (Apple Silicon)
curl -sL https://github.com/woody34/rescope/releases/latest/download/rescope-aarch64-apple-darwin.tar.gz | tar xz
./rescope

# macOS (Intel)
curl -sL https://github.com/woody34/rescope/releases/latest/download/rescope-x86_64-apple-darwin.tar.gz | tar xz
./rescope

# Linux (x86_64)
curl -sL https://github.com/woody34/rescope/releases/latest/download/rescope-x86_64-unknown-linux-gnu.tar.gz | tar xz
./rescope

# Linux (ARM64)
curl -sL https://github.com/woody34/rescope/releases/latest/download/rescope-aarch64-unknown-linux-gnu.tar.gz | tar xz
./rescope
```

The emulator listens on [http://localhost:4600](http://localhost:4600), with the admin UI at `/`. Each release also publishes `checksums-sha256.txt` for verification.

### From source

```bash
git clone https://github.com/woody34/rescope.git
cd rescope
npm install

npm run dev                    # API + admin UI in watch mode
# or build an optimized binary:
npx nx run api:build-release
./target/release/rescope
```

Other workspace commands: `npm run build` (API + UI), `npm run lint` (Clippy + ESLint), `npm run format` (cargo fmt + Prettier), `npm run graph` (Nx dependency graph).

## Configuration

| Variable | Default | Description |
| --- | --- | --- |
| `DESCOPE_EMULATOR_PORT` | `4600` | HTTP port |
| `DESCOPE_PROJECT_ID` | `emulator-project` | Project ID in the JWT `iss` claim and management auth |
| `DESCOPE_MANAGEMENT_KEY` | `emulator-key` | Management API key (`Authorization: Bearer <project>:<key>`) |
| `DESCOPE_EMULATOR_SESSION_TTL` | `3600` | Session JWT TTL (seconds) |
| `DESCOPE_EMULATOR_REFRESH_TTL` | `2592000` | Refresh JWT TTL (seconds) |
| `DESCOPE_EMULATOR_SEED_FILE` | _(none)_ | Path to a JSON seed file |
| `DESCOPE_EMULATOR_KEY_FILE` | _(none)_ | Path to a PKCS8 PEM private key (auto-generated if absent) |
| `DESCOPE_EMULATOR_CONNECTOR_MODE` | `log` | `log` (default) or `invoke` (real outbound HTTP) |

## Seed File

State is in-memory. Point `DESCOPE_EMULATOR_SEED_FILE` at a JSON file to preload tenants, users, and IdPs; `POST /emulator/reset` clears state and re-applies it.

```json
{
  "tenants": [
    { "id": "acme", "name": "Acme Corp", "domains": ["acme.com"], "authType": "saml" }
  ],
  "users": [
    {
      "loginId": "alice@acme.com",
      "email": "alice@acme.com",
      "name": "Alice",
      "password": "Secret123!",
      "verifiedEmail": true,
      "tenantIds": ["acme"],
      "roleNames": ["admin"],
      "customAttributes": { "department": "engineering" }
    }
  ],
  "idpEmulators": [
    {
      "protocol": "oidc",
      "displayName": "Mock Okta",
      "tenantId": "acme",
      "attributeMapping": { "email": "user.email", "name": "user.name" }
    }
  ]
}
```

## Management Auth

Send `Authorization: Bearer <project_id>:<management_key>` on every `/v1/mgmt/…` request (default `emulator-project:emulator-key`). Unlike production, the emulator lets unauthenticated management requests through — only *invalid* credentials return `401`.

## SDK Usage

Point any Descope SDK at the emulator's base URL; nothing else changes. JWTs are signed with the emulator's key and verify against its JWKS endpoint.

```typescript
// Node.js — @descope/node-sdk
import DescopeClient from "@descope/node-sdk";

const sdk = DescopeClient({ projectId: "emulator-project", baseUrl: "http://localhost:4600" });
const { data } = await sdk.password.signUp("alice@example.com", "Secret123!");
await sdk.validateSession(data.sessionJwt);
```

```bash
# curl — sign up, then search users via the management API
curl -s http://localhost:4600/v1/auth/password/signup \
  -H 'Content-Type: application/json' \
  -d '{"loginId":"alice@example.com","password":"Secret123!"}' | jq

curl -s http://localhost:4600/v1/mgmt/user/search \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer emulator-project:emulator-key' \
  -d '{}' | jq
```

The Python (`descope`) and Go (`descope-go-sdk`) SDKs work identically — set `base_url` / `BaseURL` to `http://localhost:4600`.

## API Reference

Rescope implements the Descope HTTP surface — auth, session, and management endpoints — plus emulator-only helpers and local IdP endpoints.

<details>
<summary><b>Full endpoint reference</b></summary>

### Emulator / Infrastructure

| Method | Path | Description |
| ------ | --------------------------- | ---------------------------------------- |
| `GET`  | `/health` | Health check |
| `POST` | `/emulator/reset` | Reset all runtime state (+ re-apply seed) |
| `GET`  | `/emulator/otp/:login_id` | Get pending OTP code for a login ID |
| `POST` | `/emulator/tenant` | Create a tenant directly (escape hatch) |
| `GET`  | `/emulator/otps` | List all pending OTP codes (userId→code) |
| `GET`  | `/emulator/snapshot` | Export full emulator state as JSON |
| `POST` | `/emulator/snapshot` | Import / restore a previously exported state |
| `GET`  | `/.well-known/jwks.json` | JWKS for JWT verification |
| `GET`  | `/v2/keys/:project_id` | JWKS (alternate path used by some SDKs) |

### Auth — Password

| Method | Path | Description |
| ------ | ---------------------------- | ---------------------------------------- |
| `POST` | `/v1/auth/password/signup` | Sign up with email + password |
| `POST` | `/v1/auth/password/signin` | Sign in with email + password |
| `POST` | `/v1/auth/password/replace` | Replace password (requires old password) |
| `POST` | `/v1/auth/password/reset` | Initiate password reset (returns token) |
| `POST` | `/v1/auth/password/update` | Complete password reset with token |
| `GET`  | `/v1/auth/password/policy` | Get password policy configuration |

### Auth — Magic Link

| Method | Path | Description |
| ------ | ------------------------------------- | ------------------------------------------------ |
| `POST` | `/v1/auth/magiclink/signup/email` | Sign up via magic link (email) |
| `POST` | `/v1/auth/magiclink/signin/email` | Sign in via magic link (email) |
| `POST` | `/v1/auth/magiclink/signup-in/email` | Sign up or sign in via magic link (email) |
| `POST` | `/v1/auth/magiclink/signup/sms` | Sign up via magic link (SMS) |
| `POST` | `/v1/auth/magiclink/signin/sms` | Sign in via magic link (SMS) |
| `POST` | `/v1/auth/magiclink/signup-in/sms` | Sign up or sign in via magic link (SMS) |
| `POST` | `/v1/auth/magiclink/verify` | Verify magic link token → session |
| `POST` | `/v1/auth/magiclink/update/email` | Update email via magic link |
| `POST` | `/v1/auth/magiclink/update/phone/sms` | Update phone number via magic link (SMS) |

### Auth — OTP

| Method | Path | Description |
| ------ | --------------------------------- | ----------------------------------------- |
| `POST` | `/v1/auth/otp/signup/email` | Sign up via OTP (email) |
| `POST` | `/v1/auth/otp/signin/email` | Sign in via OTP (email) |
| `POST` | `/v1/auth/otp/signup-in/email` | Sign up or sign in via OTP (email) |
| `POST` | `/v1/auth/otp/verify/email` | Verify OTP → session (email) |
| `POST` | `/v1/auth/otp/signup/phone/sms` | Sign up via OTP (SMS) |
| `POST` | `/v1/auth/otp/signin/phone/sms` | Sign in via OTP (SMS) |
| `POST` | `/v1/auth/otp/signup-in/sms` | Sign up or sign in via OTP (SMS) |
| `POST` | `/v1/auth/otp/verify/phone/sms` | Verify OTP → session (SMS) |
| `POST` | `/v1/auth/otp/update/phone/sms` | Update phone number via OTP |

### Auth — SAML / SSO

| Method | Path | Description |
| ------ | -------------------------- | --------------------------------------- |
| `POST` | `/v1/auth/saml/start` | Start SAML flow (returns `?code=…` URL) |
| `POST` | `/v1/auth/saml/authorize` | Alias for saml/start |
| `POST` | `/v1/auth/saml/exchange` | Exchange SAML code → session |
| `POST` | `/v1/auth/sso/authorize` | Alias for saml/start (SSO path) |
| `POST` | `/v1/auth/sso/exchange` | Alias for saml/exchange (SSO path) |

### Session

| Method | Path | Description |
| ------ | --------------------------- | ---------------------------------------- |
| `POST` | `/v1/auth/refresh` | Refresh session using refresh JWT |
| `POST` | `/v1/auth/logout` | Revoke refresh JWT (current session) |
| `POST` | `/v1/auth/logoutall` | Revoke all refresh JWTs for the user |
| `GET`  | `/v1/auth/me` | Get user profile (Bearer or DSR cookie) |
| `GET`  | `/v1/auth/me/history` | Get login history for the current user |
| `POST` | `/v1/auth/validate` | Validate session JWT → decoded claims |
| `POST` | `/v1/auth/tenant/select` | Select active tenant for the session |

### Management — User

| Method   | Path | Description |
| -------- | --------------------------------------- | ---------------------------------------------------- |
| `POST`   | `/v1/mgmt/user/create` | Create a user |
| `POST`   | `/v1/mgmt/user/create/test` | Create a test user (included in delete-all) |
| `POST`   | `/v1/mgmt/user/create/batch` | Create multiple users in one request |
| `GET`    | `/v1/mgmt/user?loginid=…` | Load user by loginId |
| `DELETE` | `/v1/mgmt/user?loginid=…` | Delete user by loginId |
| `POST`   | `/v1/mgmt/user/delete` | Delete user by loginId (SDK POST variant) |
| `GET`    | `/v1/mgmt/user/userid?userid=…` | Load user by userId |
| `DELETE` | `/v1/mgmt/user/userid?userid=…` | Delete user by userId |
| `POST`   | `/v1/mgmt/user/delete/batch` | Delete multiple users in one request |
| `DELETE` | `/v1/mgmt/user/test/delete/all` | Delete all test users |
| `POST`   | `/v1/mgmt/user/search` | Search users (filters, pagination) |
| `POST`   | `/v2/mgmt/user/search` | Search users (Node SDK alias) |
| `POST`   | `/v1/mgmt/user/update` | Full replace of user fields |
| `PATCH`  | `/v1/mgmt/user/patch` | Partial update (preserves unspecified fields) |
| `POST`   | `/v1/mgmt/user/update/email` | Update email + verified flag |
| `POST`   | `/v1/mgmt/user/update/name` | Update display name |
| `POST`   | `/v1/mgmt/user/update/phone` | Update phone number |
| `POST`   | `/v1/mgmt/user/update/loginid` | Update loginId |
| `POST`   | `/v1/mgmt/user/update/role/set` | Set roles on a user |
| `POST`   | `/v1/mgmt/user/update/role/remove` | Remove roles from a user |
| `POST`   | `/v1/mgmt/user/status` | Update user enabled/disabled status |
| `POST`   | `/v1/mgmt/user/update/status` | Alias for status update |
| `POST`   | `/v1/mgmt/user/tenant/add` | Add tenant membership to a user |
| `POST`   | `/v1/mgmt/user/tenant/remove` | Remove tenant membership from a user |
| `POST`   | `/v1/mgmt/user/tenant/setRole` | Set tenant-scoped roles for a user |
| `POST`   | `/v1/mgmt/user/logout` | Force-logout a user (revoke all sessions) |
| `POST`   | `/v1/mgmt/user/password/set/active` | Set active password for a user |
| `POST`   | `/v1/mgmt/user/password/set/temporary` | Set a temporary password (expires on next login) |
| `POST`   | `/v1/mgmt/user/password/expire` | Expire a user's password |
| `POST`   | `/v1/mgmt/user/embeddedlink` | Generate an embedded link token |
| `POST`   | `/v1/mgmt/user/signin/embeddedlink` | Alias for embedded link (Node SDK path) |

### Management — Tests

| Method | Path | Description |
| ------ | --------------------------------------- | ------------------------------------------------- |
| `POST` | `/v1/mgmt/tests/generate/magiclink` | Generate a magic link token for a test user |
| `POST` | `/v1/mgmt/tests/generate/otp` | Generate an OTP code for a test user |
| `POST` | `/v1/mgmt/tests/generate/enchantedlink` | Generate an enchanted link token for a test user |

### Management — Tenant

| Method   | Path | Description |
| -------- | ------------------------- | -------------------------------------------- |
| `GET`    | `/v1/mgmt/tenant/all` | List all tenants |
| `POST`   | `/v1/mgmt/tenant/create` | Create a tenant |
| `POST`   | `/v1/mgmt/tenant/update` | Update tenant name / domains |
| `GET`    | `/v1/mgmt/tenant?id=…` | Load tenant by ID |
| `DELETE` | `/v1/mgmt/tenant?id=…` | Delete tenant by ID |
| `POST`   | `/v1/mgmt/tenant/delete` | Delete tenant by ID (Node SDK POST variant) |
| `POST`   | `/v1/mgmt/tenant/search` | Search tenants |

### Management — Permissions

| Method | Path | Description |
| ------ | --------------------------------- | --------------------- |
| `POST` | `/v1/mgmt/authz/permission` | Create a permission |
| `GET`  | `/v1/mgmt/authz/permission/all` | List all permissions |
| `POST` | `/v1/mgmt/authz/permission/update`| Update a permission |
| `POST` | `/v1/mgmt/authz/permission/delete`| Delete a permission |

### Management — Roles

| Method | Path | Description |
| ------ | ---------------------------- | ---------------- |
| `POST` | `/v1/mgmt/authz/role` | Create a role |
| `GET`  | `/v1/mgmt/authz/role/all` | List all roles |
| `POST` | `/v1/mgmt/authz/role/update` | Update a role |
| `POST` | `/v1/mgmt/authz/role/delete` | Delete a role |

### Management — Access Keys

| Method | Path | Description |
| ------ | ----------------------------- | ----------------------------------- |
| `POST` | `/v1/mgmt/accesskey` | Create an access key |
| `GET`  | `/v1/mgmt/accesskey/all` | List all access keys |
| `POST` | `/v1/mgmt/accesskey/update` | Update access key name or expiry |
| `POST` | `/v1/mgmt/accesskey/delete` | Delete an access key |
| `POST` | `/v1/mgmt/accesskey/disable` | Disable an access key |

### Management — Auth Method Config

| Method | Path | Description |
| ------ | ------------------------------- | ----------------------------------------- |
| `GET`  | `/v1/mgmt/config/auth-methods` | Get enabled/disabled state of all methods |
| `PUT`  | `/v1/mgmt/config/auth-methods` | Update enabled/disabled state |

### Management — JWT

| Method | Path | Description |
| ------ | ------------------------------ | ---------------------------------- |
| `POST` | `/v1/mgmt/jwt/update` | Update custom claims on a JWT |
| `POST` | `/v1/mgmt/jwt/template` | Create a JWT template |
| `GET`  | `/v1/mgmt/jwt/template/all` | List all JWT templates |
| `POST` | `/v1/mgmt/jwt/template/update` | Update a JWT template |
| `POST` | `/v1/mgmt/jwt/template/delete` | Delete a JWT template |
| `POST` | `/v1/mgmt/jwt/template/set-active` | Set the active JWT template |
| `GET`  | `/v1/mgmt/jwt/template/active` | Get the currently active template |

### Management — Connectors

| Method | Path | Description |
| ------ | --------------------------- | -------------------- |
| `POST` | `/v1/mgmt/connector` | Create a connector |
| `GET`  | `/v1/mgmt/connector/all` | List all connectors |
| `POST` | `/v1/mgmt/connector/update` | Update a connector |
| `POST` | `/v1/mgmt/connector/delete` | Delete a connector |

### Management — Custom Attributes

| Method | Path | Description |
| ------ | ------------------------------ | ------------------------ |
| `POST` | `/v1/mgmt/user/attribute` | Create a custom attribute |
| `GET`  | `/v1/mgmt/user/attribute/all` | List all custom attributes |
| `POST` | `/v1/mgmt/user/attribute/delete`| Delete a custom attribute |

### Management — Identity Providers

| Method | Path | Description |
| ------ | ---------------------- | ----------------------------------- |
| `POST` | `/v1/mgmt/idp` | Create an identity provider |
| `GET`  | `/v1/mgmt/idp/all` | List all identity providers |
| `POST` | `/v1/mgmt/idp/update` | Update an identity provider |
| `POST` | `/v1/mgmt/idp/delete` | Delete an identity provider |

### Emulator — IdP OIDC

| Method | Path | Description |
| ------ | ----------------------------------------------------------- | ---------------------------------------- |
| `GET`  | `/emulator/idp/:idp_id/.well-known/openid-configuration`    | OIDC discovery document |
| `GET`  | `/emulator/idp/:idp_id/jwks` | IdP public key (JWKS) |
| `GET`  | `/emulator/idp/:idp_id/authorize` | OIDC authorize (user picker or `login_id` param) |
| `POST` | `/emulator/idp/:idp_id/token` | Exchange authorization code → tokens |
| `GET`  | `/emulator/idp/callback` | SP callback (code → SP code → redirect) |

### Emulator — IdP SAML

| Method | Path | Description |
| ------ | ------------------------------------- | ------------------------------------------ |
| `GET`  | `/emulator/idp/:idp_id/metadata` | SAML EntityDescriptor XML |
| `GET`  | `/emulator/idp/:idp_id/sso` | SAML SSO (user picker or `login_id` param) |
| `POST` | `/emulator/idp/saml/acs` | SP-side ACS callback (SAML → SP code) |

</details>

## Testing

All commands run from the repo root; Nx builds dependencies first.

| Suite | Command | Emulator | Speed |
| --- | --- | --- | --- |
| Rust unit | `npm run test:unit` | not needed | ~5s |
| API integration | `npm run test:api` | auto-starts (port 4501) | ~30s |
| SDK integration | `npm run test:sdk-js` / `test:sdk-nodejs` | auto-starts (port 4501) | ~30s |
| E2E (Playwright) | `npm run test:e2e` | auto-starts (port 4600) | ~2m |
| Everything | `npm run test` | auto | — |

- **E2E setup (one-time):** `cd apps/ui && npx playwright install chromium`. Run headed with `npm run test:e2e:watch`; filter with `-- --grep "Users"`.
- **Coverage:** `npm run api:coverage` (needs `cargo install cargo-llvm-cov`; 95% floor).
- **Parity vs. live Descope:** set `DESCOPE_PARITY_PROJECT_ID` + `DESCOPE_PARITY_MANAGEMENT_KEY`, then `make test-parity`.
- **Port already in use?** `lsof -ti:4600 | xargs kill -9`.

## Emulator Deviations

Rescope intentionally simplifies a few behaviors for local use:

| Behavior | Descope (production) | Rescope (emulator) |
| --- | --- | --- |
| OTP / magic link / reset delivery | Sends email or SMS | Returns the code/token directly in the API response |
| Management auth | Required (`401` without a key) | Optional (only invalid keys return `401`) |
| Test-token generation (`/v1/mgmt/tests/generate/*`) | Requires a designated test user | Accepts any existing user |
| Persistence | Cloud database | In-memory (use seed/snapshot to persist) |
| Rate limiting | Enforced | None |
| Email/phone verification | Real flow | Auto on OTP verify, or set via management API |
| Webhooks | Real HTTP delivery | Logged (or sent when connector mode is `invoke`) |
| SSO / IdPs | Real providers (Okta, Azure AD, …) | Local OIDC + SAML emulation with a user picker |

## Contributing & License

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Licensed under [Apache-2.0](LICENSE).
