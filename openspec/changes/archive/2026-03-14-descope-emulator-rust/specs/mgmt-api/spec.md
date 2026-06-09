## ADDED Requirements

### Requirement: Management API authentication

All management endpoints SHALL require an `Authorization: Bearer {projectId}:{managementKey}` header. The emulator SHALL accept any non-empty value for both `projectId` and `managementKey` (no real validation). Requests missing the header SHALL return HTTP 401.

#### Scenario: Request with management auth header is accepted

- **WHEN** a management endpoint is called with `Authorization: Bearer emulator-project:emulator-key`
- **THEN** the request proceeds normally

#### Scenario: Request without auth header is rejected

- **WHEN** a management endpoint is called without an `Authorization` header
- **THEN** the response is HTTP 401

---

### Requirement: User create and createTestUser

`POST /v1/mgmt/user/create` and `POST /v1/mgmt/user/create/test` SHALL accept the `UserOptions` shape, delegate to the user store, and return `{ "ok": true, "user": UserResponse }`. The `/create/test` variant SHALL flag the user as a test user.

#### Scenario: User is created and returned

- **WHEN** `POST /v1/mgmt/user/create` is called with a unique loginId and options
- **THEN** the response is HTTP 200 with `{ "ok": true, "user": { "userId": "...", "loginIds": [...], ... } }`

#### Scenario: Duplicate loginId returns error

- **WHEN** `POST /v1/mgmt/user/create` is called with an existing loginId
- **THEN** the response is HTTP 400 with `{ "ok": false, "errorCode": "E062108" }`

---

### Requirement: User load

`GET /v1/mgmt/user?loginId={loginId}` SHALL return `{ "ok": true, "user": UserResponse }`. `GET /v1/mgmt/user/userid?userId={userId}` SHALL return the user by internal userId.

#### Scenario: User is loaded by loginId

- **WHEN** `GET /v1/mgmt/user?loginId=<loginId>` is called for an existing user
- **THEN** the response is HTTP 200 with `{ "ok": true, "user": { ... } }`

#### Scenario: User not found returns error

- **WHEN** `GET /v1/mgmt/user?loginId=<unknown>` is called
- **THEN** the response is HTTP 400 with `{ "ok": false, "errorCode": "E062108" }`

---

### Requirement: User search

`POST /v1/mgmt/user/search` SHALL accept `{ emails?, phones?, customAttributes?, withTestUser?, page?, limit? }` and return `{ "ok": true, "users": UserResponse[], "total": number }`. Default limit is 1000. Pagination is 0-indexed by `page`.

#### Scenario: Search by email returns matches

- **WHEN** `POST /v1/mgmt/user/search` is called with `{ "emails": ["a@b.com"] }`
- **THEN** only users with that email are returned

#### Scenario: Empty search returns all non-test users

- **WHEN** `POST /v1/mgmt/user/search` is called with `{}`
- **THEN** all non-test users are returned with correct `total`

---

### Requirement: User update and patch

`POST /v1/mgmt/user/update` SHALL fully replace mutable user fields. `PATCH /v1/mgmt/user/patch` SHALL partially update only the provided fields. Both return `{ "ok": true, "user": UserResponse }`.

#### Scenario: Update clears unprovided fields

- **WHEN** `POST /v1/mgmt/user/update` is called without `phone`
- **THEN** the stored user's `phone` is empty

#### Scenario: Patch preserves unprovided fields

- **WHEN** `PATCH /v1/mgmt/user/patch` is called without `phone`
- **THEN** the stored user's `phone` is unchanged

---

### Requirement: User email update

`POST /v1/mgmt/user/update/email` SHALL accept `{ loginId, email, verified }` and update the user's email and `verifiedEmail` fields. Returns `{ "ok": true, "user": UserResponse }`.

#### Scenario: Email and verification status are updated

- **WHEN** `POST /v1/mgmt/user/update/email` is called with `{ verified: true }`
- **THEN** the user's `email` and `verifiedEmail: true` are updated

---

### Requirement: User set active password

`POST /v1/mgmt/user/password/set/active` SHALL accept `{ loginId, password }`, hash the password, and store it for the user. The user SHALL then be authenticatable via `POST /v1/auth/password/signin`.

#### Scenario: Set password enables sign-in

- **WHEN** `POST /v1/mgmt/user/password/set/active` is called then `POST /v1/auth/password/signin` is called
- **THEN** sign-in returns a valid session JWT

---

### Requirement: User delete

`DELETE /v1/mgmt/user?loginId={loginId}` and `DELETE /v1/mgmt/user/userid?userId={userId}` SHALL remove users from the store. Deleting a non-existent user SHALL return `{ "ok": true }` (idempotent). `DELETE /v1/mgmt/user/test/delete/all` SHALL remove all test users.

#### Scenario: Delete is idempotent

- **WHEN** `DELETE /v1/mgmt/user?loginId=<unknown>` is called
- **THEN** the response is HTTP 200 with `{ "ok": true }`

#### Scenario: Delete all test users only removes test users

- **WHEN** `DELETE /v1/mgmt/user/test/delete/all` is called
- **THEN** test users are removed and regular users remain

---

### Requirement: User add tenant

`POST /v1/mgmt/user/tenant/add` SHALL accept `{ loginId, tenantId }` and add a `{ tenantId, roleNames: [], tenantName }` entry to the user's `userTenants`. The operation SHALL be idempotent.

#### Scenario: Tenant is added to user

- **WHEN** `POST /v1/mgmt/user/tenant/add` is called
- **THEN** the user's `userTenants` includes the new entry

#### Scenario: Adding same tenant twice is idempotent

- **WHEN** the same tenant is added twice
- **THEN** the user's `userTenants` contains only one entry for that tenant

---

### Requirement: Tenant load all

`GET /v1/mgmt/tenant/all` SHALL return `{ "ok": true, "tenants": Tenant[] }` with all tenants from the tenant store.

#### Scenario: All tenants are returned

- **WHEN** `GET /v1/mgmt/tenant/all` is called
- **THEN** all tenants in the store are returned with correct fields

---

### Requirement: Generate magic link for test user

`POST /v1/mgmt/tests/generate/magiclink` SHALL accept `{ deliveryMethod, loginId, uri }`, generate a magic link token, store it, and return `{ "ok": true, "link": "{uri}?t={token}" }`. Only test users are eligible.

#### Scenario: Magic link is generated for test user

- **WHEN** `POST /v1/mgmt/tests/generate/magiclink` is called for a test user
- **THEN** the response is HTTP 200 with `{ "ok": true, "link": "...?t=<hex64>" }`

#### Scenario: Magic link for non-test user is rejected

- **WHEN** `POST /v1/mgmt/tests/generate/magiclink` is called for a regular user
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: Generate embedded link

`POST /v1/mgmt/user/embeddedlink` SHALL accept `{ userId }`, generate a random token, and return `{ "ok": true, "token": "<hex64>" }`.

#### Scenario: Embedded link is generated for existing user

- **WHEN** `POST /v1/mgmt/user/embeddedlink` is called with a valid userId
- **THEN** the response is HTTP 200 with `{ "ok": true, "token": "<hex64>" }`
