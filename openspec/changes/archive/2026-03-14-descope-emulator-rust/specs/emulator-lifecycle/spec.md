## ADDED Requirements

### Requirement: Health check endpoint

`GET /health` SHALL return HTTP 200 with `{ "ok": true }` when the emulator is ready to accept requests. This endpoint MUST be available immediately after the server binds its port.

#### Scenario: Health check returns ok

- **WHEN** `GET /health` is called on a running emulator
- **THEN** the response is HTTP 200 with `{ "ok": true }`

---

### Requirement: State reset endpoint

`POST /emulator/reset` SHALL clear all in-memory state — user store, tenant store, token store, and revocation store — and then re-apply the seed file (if `DESCOPE_EMULATOR_SEED_FILE` is set). Returns `{ "ok": true }`.

#### Scenario: Reset clears all user state

- **WHEN** users are created then `POST /emulator/reset` is called
- **THEN** subsequent `GET /v1/mgmt/user?loginId=<any>` returns a not-found error for all previously created users

#### Scenario: Reset re-applies seed data

- **WHEN** `POST /emulator/reset` is called and a seed file is configured
- **THEN** seeded users and tenants are available again immediately

#### Scenario: Reset clears revocation store

- **WHEN** tokens are revoked then `POST /emulator/reset` is called
- **THEN** those tokens are no longer in the revocation set

---

### Requirement: Environment variable configuration

The emulator SHALL read its configuration from environment variables at startup with these defaults:

| Variable                       | Default            | Description                       |
| ------------------------------ | ------------------ | --------------------------------- |
| `DESCOPE_EMULATOR_PORT`        | `4600`             | HTTP listen port                  |
| `DESCOPE_PROJECT_ID`           | `emulator-project` | Project ID for JWTs and mgmt auth |
| `DESCOPE_MANAGEMENT_KEY`       | `emulator-key`     | Management key for mgmt auth      |
| `DESCOPE_EMULATOR_SEED_FILE`   | _(unset)_          | Path to JSON seed file            |
| `DESCOPE_EMULATOR_KEY_FILE`    | _(unset)_          | Path to RSA private key PEM file  |
| `DESCOPE_EMULATOR_SESSION_TTL` | `3600`             | Session JWT lifetime in seconds   |
| `DESCOPE_EMULATOR_REFRESH_TTL` | `2592000`          | Refresh JWT lifetime in seconds   |

#### Scenario: Emulator uses default port when env var is unset

- **WHEN** `DESCOPE_EMULATOR_PORT` is not set
- **THEN** the emulator binds to port 4600

#### Scenario: Emulator uses configured port when env var is set

- **WHEN** `DESCOPE_EMULATOR_PORT=9000` is set
- **THEN** the emulator binds to port 9000

---

### Requirement: CORS for all origins with credentials

The emulator SHALL include CORS headers on all responses that echo the incoming `Origin` header as `Access-Control-Allow-Origin` and set `Access-Control-Allow-Credentials: true`. Preflight `OPTIONS` requests SHALL return HTTP 200 with appropriate `Access-Control-Allow-Methods` and `Access-Control-Allow-Headers`.

#### Scenario: CORS headers are present on API responses

- **WHEN** any request includes an `Origin: http://localhost:4200` header
- **THEN** the response includes `Access-Control-Allow-Origin: http://localhost:4200` and `Access-Control-Allow-Credentials: true`

#### Scenario: OPTIONS preflight returns 200

- **WHEN** an `OPTIONS` preflight request is sent
- **THEN** the response is HTTP 200 with CORS method and header allow lists
