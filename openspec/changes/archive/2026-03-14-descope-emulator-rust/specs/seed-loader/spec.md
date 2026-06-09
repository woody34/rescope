## ADDED Requirements

### Requirement: Seed file loading

When `DESCOPE_EMULATOR_SEED_FILE` is set, the emulator SHALL parse the JSON file at startup, create all tenants first, then all users. The seed file SHALL follow this schema:

```json
{
  "tenants": [
    {
      "id": "string",
      "name": "string",
      "domains": ["string"],
      "authType": "none|saml|oidc"
    }
  ],
  "users": [
    {
      "loginId": "string",
      "email": "string",
      "name": "string",
      "phone": "string",
      "password": "string",
      "isTestUser": true,
      "verifiedEmail": true,
      "verifiedPhone": false,
      "customAttributes": {
        "uid": "string",
        "username": "string",
        "company": "string"
      },
      "tenantIds": ["string"]
    }
  ]
}
```

Tenants are created before users so `tenantIds` references resolve correctly.

#### Scenario: Seeded users are available immediately after startup

- **WHEN** the emulator starts with a valid seed file
- **THEN** seeded users are retrievable via `GET /v1/mgmt/user?loginId=<seeded-loginId>` before any other API call

#### Scenario: Seeded users with password can authenticate

- **WHEN** a seeded user has a `password` field
- **THEN** `POST /v1/auth/password/signin` with those credentials returns a valid session JWT

#### Scenario: Seeded tenants are returned by loadAll

- **WHEN** the emulator starts with tenants in the seed file
- **THEN** `GET /v1/mgmt/tenant/all` returns those tenants

#### Scenario: Tenants are created before users

- **WHEN** a seeded user has `tenantIds` referencing a seeded tenant
- **THEN** the user's `userTenants` includes that tenant with correct `tenantName`

---

### Requirement: Seed file error handling

If `DESCOPE_EMULATOR_SEED_FILE` is set but the file does not exist, is not valid JSON, or does not match the expected schema, the emulator SHALL exit at startup with a clear error message including the file path and parse error details.

#### Scenario: Missing seed file causes startup failure

- **WHEN** `DESCOPE_EMULATOR_SEED_FILE` points to a non-existent file
- **THEN** the emulator exits with an error message containing the file path

#### Scenario: Invalid JSON seed file causes startup failure

- **WHEN** `DESCOPE_EMULATOR_SEED_FILE` points to a file with invalid JSON
- **THEN** the emulator exits with an error message containing the parse error
