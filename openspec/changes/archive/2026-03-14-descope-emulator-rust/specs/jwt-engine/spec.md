## ADDED Requirements

### Requirement: RSA key pair generation at startup

The emulator SHALL generate a 2048-bit RSA key pair on startup using the `rsa` crate. The generated private key SHALL be used to sign all JWTs. The corresponding public key SHALL be served via the JWKS endpoint.

#### Scenario: Fresh startup without key file

- **WHEN** the emulator starts without `DESCOPE_EMULATOR_KEY_FILE` set
- **THEN** a new 2048-bit RSA key pair is generated in memory

#### Scenario: Startup with key file

- **WHEN** `DESCOPE_EMULATOR_KEY_FILE` points to a valid PEM file
- **THEN** the emulator loads the private key from that file and derives the public key
- **THEN** JWTs signed in this run are identical in structure to previous runs using the same file

#### Scenario: Startup with invalid key file

- **WHEN** `DESCOPE_EMULATOR_KEY_FILE` points to a missing or malformed PEM file
- **THEN** the emulator exits with a clear error message including the file path

---

### Requirement: RS256 session JWT signing

The emulator SHALL sign all session JWTs using RS256 (RSA + SHA-256) with the startup-generated private key.

Session JWTs SHALL include the following claims:

| Claim            | Value                                                |
| ---------------- | ---------------------------------------------------- |
| `sub`            | user's `userId`                                      |
| `iss`            | `DESCOPE_PROJECT_ID`                                 |
| `exp`            | `now + DESCOPE_EMULATOR_SESSION_TTL` (default 3600s) |
| `iat`            | current Unix timestamp                               |
| `email`          | user's email                                         |
| `email_verified` | user's `verifiedEmail`                               |
| `phone`          | user's phone                                         |
| `phone_verified` | user's `verifiedPhone`                               |
| `name`           | user's display name                                  |
| `username`       | from user's `customAttributes.username`              |
| `company`        | from user's `customAttributes.company`               |
| `uid`            | from user's `customAttributes.uid`                   |
| `photo_url`      | user's picture                                       |
| `super`          | from user's `customAttributes.super` (bool)          |
| `support`        | from user's `customAttributes.support` (bool)        |
| `descopeId`      | user's `userId`                                      |

JWT header SHALL include `alg: "RS256"` and `kid: <key-id>`.

#### Scenario: Session JWT is signed with correct algorithm

- **WHEN** a session JWT is generated for any auth operation
- **THEN** the JWT header contains `"alg": "RS256"` and a `kid` matching the current key

#### Scenario: Session JWT contains all required claims

- **WHEN** a session JWT is generated for a user with email, customAttributes, and phone
- **THEN** all claims listed in the requirements table are present in the JWT payload

---

### Requirement: RS256 refresh JWT signing

The emulator SHALL sign all refresh JWTs using RS256 with the same private key as session JWTs. Refresh JWTs SHALL include `sub`, `iss`, `exp` (`now + DESCOPE_EMULATOR_REFRESH_TTL`, default 2592000s), and `iat` claims.

#### Scenario: Refresh JWT has longer TTL than session JWT

- **WHEN** both a session JWT and refresh JWT are generated simultaneously
- **THEN** the refresh JWT `exp` is greater than the session JWT `exp`

---

### Requirement: JWT verification

The emulator SHALL verify JWTs by checking the RS256 signature against the public key, validating the `exp` claim, and extracting claims. Verification SHALL use the same `jsonwebtoken` crate used for signing.

#### Scenario: Valid token passes verification

- **WHEN** a JWT is verified that was signed by the emulator's current key and is not expired
- **THEN** verification succeeds and all claims are returned

#### Scenario: Expired token is rejected

- **WHEN** a JWT is verified whose `exp` is in the past
- **THEN** verification returns an error

#### Scenario: Token signed by a different key is rejected

- **WHEN** a JWT is verified that was signed by a different RSA key
- **THEN** verification returns an error

---

### Requirement: JWKS endpoint

The emulator SHALL serve a JWK Set at `GET /.well-known/jwks.json` and `GET /v2/keys/{projectId}` containing the RSA public key in JWK format.

The JWK SHALL include: `kty: "RSA"`, `use: "sig"`, `alg: "RS256"`, `kid`, `n` (base64url-encoded modulus), `e` (base64url-encoded exponent).

#### Scenario: JWKS endpoint returns valid JWK Set

- **WHEN** `GET /.well-known/jwks.json` is called
- **THEN** the response is `{ "keys": [ { "kty": "RSA", "use": "sig", "alg": "RS256", "kid": "...", "n": "...", "e": "..." } ] }`

#### Scenario: JWKS public key matches signing key

- **WHEN** a JWT is signed and the JWKS endpoint is fetched
- **THEN** the public key in the JWKS set can verify the JWT's RS256 signature

#### Scenario: Alternate JWKS endpoint is equivalent

- **WHEN** `GET /v2/keys/{projectId}` is called with any projectId value
- **THEN** the response is identical to `/.well-known/jwks.json`
