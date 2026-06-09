## ADDED Requirements

### Requirement: Access keys can be created via Admin UI
The Access Keys page SHALL provide a form for creating a new access key with a name and optional expiry. Upon creation, the cleartext key value SHALL be shown once in the UI.

#### Scenario: Created key is displayed once
- **WHEN** user fills in a key name and clicks Create on the Access Keys page
- **THEN** the UI displays the full access key string (bearer token format) one time

#### Scenario: Created key appears in keys list
- **WHEN** an access key is created via the UI
- **THEN** `GET /v1/mgmt/accesskey/all` includes a key with the given name

---

### Requirement: Access key from UI can authorize mgmt API calls
The access key value shown in the UI SHALL be usable as a Bearer token in the `Authorization` header for management API calls, in lieu of the bootstrap management key.

#### Scenario: Key authorizes management API
- **WHEN** a key is created in the UI and its value is used as `Bearer <key>` on `GET /v1/mgmt/user/search`
- **THEN** the response is HTTP 200

---

### Requirement: Access keys can be deleted via Admin UI
The Access Keys page SHALL provide a delete control for each key. Deleted keys SHALL no longer authorize API calls.

#### Scenario: Deleted key removed from API list
- **WHEN** user deletes a key via the UI
- **THEN** `GET /v1/mgmt/accesskey/all` no longer contains that key
